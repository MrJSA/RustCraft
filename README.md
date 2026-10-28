# RustCraft — Rust Webshop

A self-hosted webshop with a **Rust** backend (Axum, SQLx, PostgreSQL) and **SvelteKit** storefront and admin dashboard, shipped as one Docker Compose stack with one-click updates.

**See it in use:** [shop.mrjsa.com](https://shop.mrjsa.com) runs on RustCraft.

> **Installing on a server?** Follow the **[Installation guide](https://github.com/MrJSA/RustCraft/wiki/Installation)** in the [wiki](https://github.com/MrJSA/RustCraft/wiki): domains, automatic HTTPS, first login, backups and updates.

## Architecture

| Service | Technology | Purpose |
| :--- | :--- | :--- |
| `backend` | Rust · Axum · SQLx | REST API, checkout, payments, emails, PDF documents, migrations |
| `storefront` | SvelteKit | Customer shop (server-rendered, forwards API calls) |
| `admin` | SvelteKit | Admin dashboard |
| `db` | PostgreSQL 16 | Data (internal network only) |
| `updater` | Rust | Checks release tags, backs up the database, rebuilds the stack |
| `proxy` (optional) | Caddy | Automatic HTTPS for the shop and admin domains |

## Local development

```bash
docker compose up -d --build
docker compose logs backend | grep "Initial admin"   # one-time admin password
```

| | URL |
| :--- | :--- |
| Storefront | http://localhost:8080 |
| Admin dashboard | http://localhost:4000 (user `admin`) |
| Backend API | http://localhost:8081 (this machine only) |

Database migrations run automatically when the backend starts.

## Features

### Catalogue & inventory
- Physical and digital products with variants, per-variant images, prices and stock.
- Unlimited category nesting.
- **Bill of materials:** parts are shared by SKU. Every product that uses the same SKU shows the same part name, storage location and stock. A product's stock is calculated from its parts (how many units can be built), and sales and cancelled payments update part stock automatically.
- **Logistics & Stock:** one list of products and parts with storage locations, stock adjustments and low-stock filters.
- Low-stock email alerts for parts, sent to stock managers, selected admins or extra addresses.
- Back-in-stock waitlist: customers are emailed as soon as a sold-out variant is available again.
- Digital products with multiple files or external links, downloadable from the order confirmation and the customer account.

### Checkout & payments
- **Stripe:** cards, Apple Pay, Google Pay, Link, Klarna, SEPA, iDEAL and more, each listed separately. Only methods active in your Stripe account are offered.
- **PayPal.**
- Coupons (percentage, fixed amount, free shipping) with limits and expiry dates.
- Shipping providers, zones and weight-based rates with tracking links. Orders with only digital items skip shipping.
- Optional: checkout only for registered customers, and mandatory email verification.

### Orders & documents
- Order management with status workflow and tracking numbers.
- PDF invoices and packing slips.
- Customer accounts with order history, status tracker and downloads.

### Storefront
- Choose between two homepage hero layouts: a full-width carousel, or a split hero with four featured product buttons.
- Product carousels (featured, new arrivals, best sellers, in stock) with auto-rotation.
- Custom logo, store title and subtitle, header and footer menus, and Markdown content pages.
- Configurable stock message, e.g. `In stock ({stock} available)`.

### Emails
- Sent through any SMTP server.
- Order confirmation, payment received, shipment, back-in-stock, account verification, password reset and low-stock alerts.
- New-order emails to the team: choose superadmins only, admins and superadmins, or everyone incl. editors. Each admin can switch them off for themselves under their account.
- All emails share one branded layout. You can send a test of each template from the admin.

### German / EU law
- "Zahlungspflichtig bestellen" order button (§ 312j BGB) and a waiver checkbox for the right of withdrawal on digital goods (§ 356 BGB).
- Tax modes: small business (§ 19 UStG), VAT included (B2C) or VAT excluded (B2B). The tax notice appears on checkout, emails and invoices.
- Sequential order numbers with an optional prefix and date code.
- Templates for legal notice, terms, withdrawal policy and privacy policy. Granular cookie consent banner.

### Administration
- Roles: `superadmin`, `admin` and `editor`, with per-section permissions. Each account can have an email address, used for alerts.
- Export and import of store data as JSON: settings, catalogue incl. parts and stock, pages, menus, shipping, coupons and payment settings. The import also works on a different installation. Passwords and API keys are never exported.
- Media library with download of all files as ZIP. Images are converted to WebP on upload.
- Invoices as PDF, attached automatically to the payment confirmation email. Optional EN 16931 e-invoices (ZUGFeRD / Factur-X, PDF/A-3 with embedded XML); the XML can also be downloaded per order.
- Optional sale prices that show the lowest price of the last 30 days (§ 11 PAngV), unit prices and the VAT note next to every price.
- Optional product reviews from verified buyers (with approval) and a double opt-in newsletter with campaigns.
- Protected digital downloads (personal links with download limit and validity).
- Product weights and weight-based shipping rates; delivery times; product safety information (GPSR).
- VAT-free orders for EU businesses with a live VIES check (intra-Community supply / reverse charge).
- Withdrawal button with confirmation emails, invoices stored unchangeable and credit notes on refunds.
- Automatic daily, weekly or monthly backups of the complete database and all uploads to a private GitHub repository (see [Backups](https://github.com/MrJSA/RustCraft/wiki/Backups)).
- Updates from the admin with an automatic database backup (see [Updates](https://github.com/MrJSA/RustCraft/wiki/Updates)), on the stable or the beta channel.

## Bugs and feature requests

Please read the [rules for issues](https://github.com/MrJSA/RustCraft/wiki/Reporting-Issues) and search the existing issues before opening a new one.

## Branches and releases

| Branch | Tags | Installations on |
| :--- | :--- | :--- |
| `master` | `v1.6.0` | **Stable** channel (default, for live shops) |
| `beta` | `v1.6.0-beta.1` | **Beta** channel (testing, *Settings → System & Updates → Update channel*) |

Development and pull requests go to `beta`. See [Release Channels](https://github.com/MrJSA/RustCraft/wiki/Release-Channels) for details.

1. **Beta:** on `beta`, set `VERSION` (e.g. `1.6.0-beta.1`), commit, then
   ```bash
   git tag -a v1.6.0-beta.1 -m "What is new in this beta"
   git push origin beta --follow-tags
   ```
2. **Stable:** merge `beta` into `master`, set `VERSION` (e.g. `1.6.0`), commit, then
   ```bash
   git tag -a v1.6.0 -m "What changed in this release"
   git push origin master --follow-tags
   ```
   and merge `master` back into `beta`.
3. Installations on the matching channel now offer *Update now* in *Settings → System & Updates*.
