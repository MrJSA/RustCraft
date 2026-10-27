# Installing the shop on a server

This guide installs the shop on a Linux server (Ubuntu 22.04/24.04 or Debian 12) with Docker. At the end you have:

- the **shop** at `https://shop.example.com`
- the **admin dashboard** at `https://admin.example.com`
- automatic HTTPS certificates, database backups before every update, and one-click updates from the admin

Replace `example.com` with your own domain throughout.

---

## 1. Requirements

| | Minimum |
|---|---|
| Server | Linux VPS/root server, 2 vCPU, **2 GB RAM** (building the Rust backend needs memory; add swap on smaller servers) |
| Disk | 10 GB free |
| Domain | Two (sub)domains, e.g. `shop.example.com` and `admin.example.com` |
| Ports | 22 (SSH), 80 and 443 (HTTPS) |

Small server? Add 2 GB swap:

```bash
sudo fallocate -l 2G /swapfile && sudo chmod 600 /swapfile && sudo mkswap /swapfile && sudo swapon /swapfile
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```

## 2. DNS

At your domain provider create two records pointing to the server's public IP:

```
shop.example.com    A    203.0.113.10
admin.example.com   A    203.0.113.10
```

(Add `AAAA` records too if the server has IPv6.) Wait until `ping shop.example.com` shows your server IP.

## 3. Install Docker and git

```bash
sudo apt update && sudo apt install -y git curl
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker $USER   # log out and back in afterwards
docker compose version          # must print v2.x
```

## 4. Download the shop

```bash
sudo mkdir -p /opt/rustwebshop && sudo chown $USER: /opt/rustwebshop
git clone https://github.com/MrJSA/RustCraft.git /opt/rustwebshop
cd /opt/rustwebshop
git checkout "$(git describe --tags --abbrev=0 2>/dev/null || echo master)"   # newest release
```

## 5. Configure

```bash
cp .env.example .env
nano .env
```

Set at least:

```ini
DB_PASSWORD=<output of: openssl rand -hex 24>

SHOP_PUBLIC_URL=https://shop.example.com
ADMIN_PUBLIC_URL=https://admin.example.com

# Built-in HTTPS proxy (recommended unless the server already runs nginx/Apache/Traefik)
COMPOSE_PROFILES=proxy
SHOP_DOMAIN=shop.example.com
ADMIN_DOMAIN=admin.example.com
PUBLISH_ADDR=127.0.0.1
```

> Any host names work, as long as shop and admin differ: the shop can live on the main domain (`example.com`)
> or a subdomain (`shop.example.com`), and the admin on any other (sub)domain, including one below the shop
> (`admin.shop.example.com`). Each needs its own DNS A/AAAA record pointing to this server.

> Using your own web server instead? Leave `COMPOSE_PROFILES` empty, keep `PUBLISH_ADDR=127.0.0.1` and see [section 9](#9-using-your-own-reverse-proxy-nginx-apache-caddy-traefik).

## 6. Start

```bash
docker compose up -d --build        # first build takes 5–15 minutes
docker compose ps                   # all services should be "running"/"healthy"
```

Get the initial admin password (shown once):

```bash
docker compose logs backend | grep "Initial admin"
```

Open `https://admin.example.com`, log in as `admin` with that password and choose your own password (min. 12 characters).

## 7. Firewall

```bash
sudo ufw allow OpenSSH
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow 443/udp
sudo ufw enable
```

Do **not** open 8080/4000/8081 — with `PUBLISH_ADDR=127.0.0.1` they only listen locally anyway.

## 8. First steps in the admin

1. **Settings → Store Identity & Legal** — shop name, address, legal texts (Impressum, AGB …).
2. **Settings → Email & Auth Policies** — SMTP server of your mail provider, then *Send test email*.
   Port 587 = STARTTLS, port 465 = SSL/TLS. Leave *From address* empty to send from the SMTP login.
3. **Settings → Payment Providers**
   - Stripe: publishable + secret key, choose payment methods, register `shop.example.com` for Apple Pay / Google Pay.
   - Webhook in the Stripe Dashboard → `https://shop.example.com/api/v1/payments/stripe/webhook`
     (events listed on the settings page), paste the `whsec_…` secret.
   - PayPal: Client ID + Secret of a REST app; turn *Sandbox* off for live credentials.
4. **Settings → Admin Users & Access** — create editors/admins and tick the sections each person may use.

## 9. Using your own reverse proxy (nginx, Apache, Caddy, Traefik)

Keep `COMPOSE_PROFILES` empty and `PUBLISH_ADDR=127.0.0.1`; the shop listens on `127.0.0.1:8080`, the admin on `127.0.0.1:4000`.
`SHOP_PUBLIC_URL` / `ADMIN_PUBLIC_URL` must be the exact public URLs.

**nginx** (`/etc/nginx/sites-available/shop`, then `ln -s` to `sites-enabled` and `sudo certbot --nginx -d shop.example.com -d admin.example.com`):

```nginx
server {
    server_name shop.example.com;
    client_max_body_size 50M;
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

server {
    server_name admin.example.com;
    client_max_body_size 50M;
    location / {
        proxy_pass http://127.0.0.1:4000;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

**Caddy** (`/etc/caddy/Caddyfile`):

```
shop.example.com {
    reverse_proxy 127.0.0.1:8080
}
admin.example.com {
    reverse_proxy 127.0.0.1:4000
}
```

Tip: restrict the admin domain further (VPN, IP allow-list or HTTP basic auth) if only a few people need it.

## 10. Changing domains later

Either edit `.env` and run `docker compose up -d`, or use **Admin → Settings → System & Updates → Domains**
(superadmins only), which writes `.env` and restarts the services for you.

## 11. Updates

**From the admin (recommended):** *Settings → System & Updates → Check for updates → Update now*.
The updater backs up the database to `backups/`, switches the code to the newest release tag, rebuilds and restarts the shop.
If the build fails, the previous code stays active.

**Manually:**

```bash
cd /opt/rustwebshop
docker compose exec -T db pg_dump -U shop_user shop_db > backups/manual-$(date +%F).sql
git fetch --tags && git checkout "$(git tag -l 'v*' --sort=-v:refname | head -1)"
docker compose up -d --build
```

Database migrations run automatically when the backend starts.

## 12. Backups & restore

**Recommended:** switch on the automatic daily/weekly/monthly backups to a private GitHub repository in
*Settings → Export & Backups*. Setup and restore are described step by step in [BACKUP.md](BACKUP.md).

Manual backups on the server:

```bash
# Database
docker compose exec -T db pg_dump -U shop_user shop_db | gzip > backups/shop-$(date +%F).sql.gz
# Uploaded images and files
docker run --rm -v rustwebshop_uploads_data:/data -v "$PWD/backups":/out alpine tar czf /out/uploads-$(date +%F).tar.gz -C /data .
```

Restore a database dump:

```bash
docker compose stop backend
gunzip -c backups/shop-YYYY-MM-DD.sql.gz | docker compose exec -T db psql -U shop_user -d shop_db
docker compose start backend
```

Copy the `backups/` folder to another machine regularly. The admin's *Export & Backups* tab additionally exports the store settings and catalogue as JSON (importable into another installation; passwords and API keys are not included and must be entered again) and the media library as ZIP.

## 13. Troubleshooting

| Problem | Check |
|---|---|
| Something does not start | `docker compose ps`, `docker compose logs -f backend` (or `storefront`, `admin`, `proxy`, `updater`) |
| No HTTPS certificate | DNS points to the server? Ports 80/443 open? `docker compose logs proxy` |
| Emails are not sent | *Send test email* shows the exact server answer; check host, port (587/465), login and that the sender address belongs to your mail account |
| Apple Pay / Google Pay buttons missing | Only on HTTPS with the domain registered in the Stripe settings, and only on supported devices |
| "Updater not reachable" | `docker compose ps updater`; the updater needs `/var/run/docker.sock` |

## Publishing a new release (for the maintainer)

1. Update `VERSION` (e.g. `1.1.0`) and commit.
2. Create and push a tag with release notes:
   ```bash
   git tag -a v1.1.0 -m "What changed in this release"
   git push origin master --follow-tags
   ```
3. Every installation now offers *Update now* in *Settings → System & Updates*.

## Security notes

- The **updater** service can control Docker on the host (that is how it rebuilds the shop). It has no published port, is only reachable by the backend over a private network and requires a token generated on first start. Remove the `updater` service from `docker-compose.yml` if you prefer updating manually.
- Files the updater changes in the checkout are owned by `root`; use `sudo` for manual git commands in `/opt/rustwebshop` afterwards.
- Keep `.env` private (`chmod 600 .env`); it is never committed.
