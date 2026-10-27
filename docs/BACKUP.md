# Automatic backups to GitHub

The shop can back itself up automatically — **daily, weekly or monthly** — to your own **private GitHub repository**.
If the server ever crashes or gets lost, you install the shop on a new server and load the latest backup. Nothing is lost
except what happened after that backup.

Setting it up takes about **5 minutes** and is done once.

---

## What is in a backup?

Each backup is a single ZIP file:

| File | Content |
|---|---|
| `database.sql` | The **complete database**: orders, invoices, customers, products, stock, coupons, pages, all settings, admin accounts |
| `uploads/` | All uploaded files: product images, logos, banners, digital downloads (can be switched off) |
| `RESTORE.txt` | Short restore instructions |
| `backup-info.json` | Date and shop name |

On GitHub, every backup is shown as a **release** (e.g. *Backup 2026-10-25 02:00 UTC*) with the ZIP file attached.
Old backups are deleted automatically. You choose how many to keep.

> **Keep the repository private.** A backup contains customer data, password hashes and your payment/email keys.
> The shop refuses to back up into a public repository. Turn on two-factor authentication for your GitHub account.

---

## Setup

You need a free GitHub account. If you don't have one yet, sign up at [github.com/signup](https://github.com/signup).

### Step 1 — Create a private repository

1. Open **[github.com/new](https://github.com/new)**.
2. **Repository name:** e.g. `shop-backups`
3. Select **Private**.
4. Leave everything else as it is and click **Create repository**.

That's it. The repository may stay empty; the shop sets it up on the first backup.

### Step 2 — Create an access token

The token is like a password that only lets the shop write into this one repository and nothing else.
Keep it in your password manager: you need it again to restore a backup on a new server.

1. Open **[github.com/settings/personal-access-tokens/new](https://github.com/settings/personal-access-tokens/new)**
   (GitHub → your picture → *Settings* → *Developer settings* → *Personal access tokens* → *Fine-grained tokens* → *Generate new token*).
2. **Token name:** e.g. `Shop backup`
3. **Expiration:** pick *No expiration* if offered, otherwise the longest possible duration (see [Renewing the token](#renewing-the-token)).
4. **Repository access:** choose **Only select repositories** and select your backup repository (`shop-backups`).
5. **Permissions** → **Repository permissions** → **Contents** → set to **Read and write**.
   (*Metadata: Read-only* is selected automatically. No other permissions are needed.)
6. Click **Generate token** and **copy** the token (it starts with `github_pat_…`). GitHub shows it only once.

### Step 3 — Connect the shop

1. Log in to the admin as a **superadmin**.
2. Go to **Settings → Export & Backups**. The first box is **Automatic Backups to GitHub**.
3. **GitHub repository:** `your-account/shop-backups` — or just paste the link to the repository.
4. **Access token:** paste the token.
5. Click **Test connection**. You should see *"Connected: … is private and the token can store backups there."*
6. Choose **how often**:
   - **Daily** — every night (recommended for shops with daily orders)
   - **Weekly** — every Sunday
   - **Monthly** — on the 1st of each month
7. Choose **at what time** (your local time; a night hour is best) and **how many backups to keep**
   (e.g. 30 daily backups = one month of history, 12 monthly backups = one year).
8. Tick **Automatic backups switched on** and click **Save backup settings**.

The first backup starts right away. After a minute or two the box shows **Last backup** with a green tick, and
**View backups** opens the list on GitHub.

You can click **Back up now** at any time, e.g. before making big changes to the shop.

---

## Is it working?

- The backup box in the admin shows the **last backup**, its size and the **next scheduled backup**.
- On GitHub, open your repository → **Releases** (right side) to see all backups.
- **If a backup fails**, the shop retries every hour and sends one email to all superadmins that have an email address
  (or to the shop's support address). This needs working email settings (*Settings → Email & Auth Policies*).
  You can preview this email there with the test email type **Automatic Backup Failed**.

Common error messages:

| Message | What to do |
|---|---|
| *GitHub rejected the access token* | The token is wrong, expired or deleted. Create a new one (Step 2), paste it and save. |
| *Repository … was not found* | Check the repository name. The token must have access to exactly this repository (Step 2, point 4). |
| *The access token may not … in …* | The token is missing **Contents: Read and write** (Step 2, point 5). Edit the token on GitHub and change it. |
| *… is a public repository* | Make the repository private: repository → *Settings* → *Danger Zone* → *Change visibility*. |
| *pg_dump is not installed* | The backend image is older than this feature. Rebuild: `docker compose up -d --build backend` |
| *The backup is … GitHub accepts files up to 2 GB* | Too many/large uploads. Untick *Include uploaded files* and back up the media library separately (see below). |

---

## Renewing the token

If GitHub did not offer *No expiration*, the token stops working on its expiry date. The admin shows a warning
14 days before, and the last backup message mentions it too. To renew:

1. Create a new token exactly as in [Step 2](#step-2--create-an-access-token).
2. In the admin, paste it into **Access token** and click **Save backup settings**.

The old token can then be deleted on GitHub.

---

## Restoring a backup

Use this when the server crashed, you moved to a new server, or you want to go back to an earlier state.

### 1. Install the shop

On a new server, install the shop as described in [INSTALL.md](INSTALL.md) (steps 1–6), with the same domains as before.
Use the **same or a newer shop version** than the one the backup was made with. You do not need to set up anything
in the admin — all settings come back with the backup.

(Restoring on the existing server: skip this step.)

### 2. Restore in the admin

1. Log in to the admin as a **superadmin** (on a new server: with the initial admin password from the install).
2. Go to **Settings → Export & Backups** → **Restore a Backup**.
3. Enter your backup repository and its access token (on the existing server the saved ones are used — leave the fields empty)
   and click **Show backups**.
4. Select the backup you want (normally the newest, at the top).
5. Tick **I understand that all current data of this shop is replaced by the backup** and click **Restore selected backup**.

The shop downloads the backup, checks it and loads it. This takes from a few seconds to a few minutes, depending on the
size. The database restore is all-or-nothing: if anything goes wrong, the shop's data stays exactly as it was and the
error is shown. Afterwards the shop restarts by itself and the admin takes you to the login page.

Instead of picking a backup on GitHub you can also upload a downloaded `shop-backup-….zip` file there (up to 50 MB).

**Restoring into a test or local copy?** Tick **Switch automatic backups off after restoring**, so the copy does not
write its own backups into the repository of your live shop.

### 3. Done

- Log in to the admin with your **old** admin username and password (the accounts come back with the backup).
- Orders, customers, products, settings, payment and email settings are all back.
- The backup settings come back too, so **automatic backups continue** on the new server without setting them up again.

A backup made with a **newer** shop version is refused with a message — update the shop first
(*Settings → System & Updates*), then restore.

### Restoring on the command line

Only needed if the admin cannot be reached. Download the `shop-backup-….zip` file from your backup repository
(→ **Releases** → *Assets*), copy it into the shop folder on the server (e.g. `scp shop-backup-20261025-020000.zip user@your-server:/opt/rustwebshop/`)
and run there:

```bash
cd /opt/rustwebshop
sudo apt install -y unzip
unzip shop-backup-20261025-020000.zip -d restore

# Stop the shop, load the database, start the shop again
docker compose stop backend storefront admin
docker compose exec -T db psql -v ON_ERROR_STOP=1 --single-transaction -U shop_user -d shop_db < restore/database.sql
docker compose start backend storefront admin

# Copy the uploaded images and files back
docker compose cp restore/uploads/. backend:/app/uploads/

# Clean up
rm -rf restore shop-backup-20261025-020000.zip
```

---

## Good to know

- **Where are the backups stored?** As release files in your GitHub repository. They do not make the repository
  itself bigger, and GitHub accepts files up to 2 GB per backup.
- **Who can see them?** Only you and people you invite to the private repository. The token in the shop can only access
  this one repository.
- **Data protection (GDPR):** the backups contain personal data of your customers and are stored at GitHub.
  Mention GitHub as a storage provider in your privacy policy.
- **Disconnect:** *Disconnect* in the admin stops the automatic backups and deletes the token from the shop.
  Backups already on GitHub stay there until you delete them (or the whole repository).
- **Manual backups** are still possible on the server (see [INSTALL.md](INSTALL.md#12-backups--restore)), and the admin's
  *Export & Backups* tab can additionally export the catalogue as JSON and the media library as ZIP.
