# RustCraft E-Commerce — High-Performance Rust Webshop

A fast, memory-safe, and ACID-compliant webshop built with a **Rust (Axum + SQLx + PostgreSQL)** backend and modern **SvelteKit** user storefront and administrative dashboard, containerized with Docker.

> **Installing on a server?** Follow **[INSTALL.md](INSTALL.md)** — domains/subdomains, automatic HTTPS, backups and one-click updates.

### Architecture

| Part | Technology |
| :--- | :--- |
| Backend API, business logic, payments, emails | **Rust** (Axum, SQLx, PostgreSQL) |
| Updater (releases, self-update, domain setup) | **Rust** |
| Customer storefront & admin dashboard | **Svelte** (SvelteKit; a thin Node runtime only renders pages and forwards API calls) |
| HTTPS proxy (optional) | Caddy |

---

## 🚀 Quick Start (local)

Start the entire stack using Docker Compose:

```bash
docker compose up -d --build
```

### 🌐 Access URLs & Ports

| Service | Host URL | Description |
| :--- | :--- | :--- |
| **Customer Storefront** | [http://localhost:8080](http://localhost:8080) | Customer-facing shop, product showcase, shopping cart, and checkout. |
| **Admin Dashboard** | [http://localhost:4000](http://localhost:4000) | Management back-office for inventory, orders, media, categories, and settings. |
| **Backend REST API** | [http://localhost:8081](http://localhost:8081) | Rust Axum API, reachable from this machine only (container port `8000`). |
| **PostgreSQL Database** | internal only | Not published; reachable by the backend over the Docker network. |

---

## 🔐 Initial Admin Login

There is no built-in default password. On the very first start the backend creates the user `admin` with

- the password from the `ADMIN_INITIAL_PASSWORD` environment variable (min. 12 characters), **or**
- a random password printed once in the backend log: `docker compose logs backend | grep "Initial admin"`

Open the Admin Dashboard at [http://localhost:4000](http://localhost:4000), log in, and you will be required to choose your own password (min. 12 characters) before the admin area unlocks.

Admin roles: `superadmin` (everything), `admin` (everything except granting superadmin), `editor` (catalogue, orders and content — no users, payments, email, system settings or import/export).

---

## 🛠 Features & Architecture

### 1. Email Addon & SMTP Notification Engine
- Connect your shop to any SMTP mail server (SendGrid, Mailgun, AWS SES, Gmail, Postmark, etc.) under **Settings &rarr; Email & Auth Policies**.
- Automatic customer notifications:
  - **Order Confirmation**: Dispatched immediately upon order placement with itemized breakdown.
  - **Payment Confirmation Receipt**: Dispatched once Stripe, PayPal, or simulated sandbox marks order as authorized or paid.
  - **Dispatched Shipment Notification**: Triggered when marking an order as *Shipped*, including carrier tracking link (DHL, ParcelsApp, etc.).
  - **Back-in-Stock Notification**: Dispatched to waitlist subscribers the moment an out-of-stock product's inventory is replenished.
  - **Account Verification Email**: Sent to newly registered customers with an activation link.
- Integrated **"Send Test Email"** button in admin settings for instantaneous connection testing.

### 2. Customer Registration & Checkout Policies
- **Require Registered Customers for Checkout**: Configurable toggle in admin settings. When enabled, only customers who have registered an account can complete a purchase.
- **Mandatory Email Verification**: Configurable toggle in admin settings requiring newly registered users to verify their email address before logging in or completing checkouts.

### 3. Arbitrary Depth Category Hierarchy
- Support for unlimited category nesting: `Category` &rarr; `Subcategory` &rarr; `Sub-Subcategory` &rarr; `...`.
- Recursive tree navigation view with inline collapsible branches, add-child actions at any level, and a hierarchical parent category selector dropdown.

### 4. Multiple Navigation Menus & Page Selector
- Separate customizable navigation menus for:
  - **Header Menu**: Centered or left-aligned navigation bar.
  - **Footer Menu**: Customer service and policy links.
- **Predefined Target Pages Overview**: View all available core store routes, product categories, and policy CMS markdown pages.
- **Quick Preset Dropdown**: Select any page to automatically populate link title and URL.

### 5. Media Library & Automatic WebP Compression
- Dedicated **Media Library** tab in the admin console.
- Persistent image storage across container restarts via the `uploads_data` Docker volume.
- Automatic image compression to high-efficiency **WebP** on upload.
- Integrated modal image picker for shop logos, hero banners, product images, and featured buttons.

### 6. Storefront Hero Showcase & Auto-Rotating Looping Carousels
- **8BitDo Full-Width Carousel**: Widescreen slider showcasing selected highlight products.
- **8BitMods 4 Featured Buttons**: Split layout with 60% slider and 40% grid of 4 product buttons featuring floating transparent images with depth layering, customizable subtitles, toggleable price badges, and custom button background colors.
- **5 Products Per Row Dynamic Carousels**:
  - Automatically centers products if &le; 5 items.
  - Automatically turns into a smooth horizontal scrollable carousel with left/right scroll arrows if &gt; 5 items.
  - **Endless Looping**: Seamlessly restarts from the beginning when reaching the end of the item row.
  - **Auto-Rotation**: Smoothly scrolls every 7 seconds, automatically pausing when the cursor hovers over the carousel.
  - Configurable sections: Featured Products, New Arrivals (configurable days threshold), Best Sellers (configurable limit), and In-Stock Product Catalog.

### 7. Brand Identity & Header Transparency
- Customizable subtitle (replace default `"Rust Powered • ACID Fast"` or toggle off/on).
- Toggle shop title text off to display an enlarged logo reaching into the header with a centered navigation menu.
- Glassmorphism navigation bar with frosted translucent backdrop blur.
- **Custom Physical Stock Message Template**: Configure custom live stock text in admin settings (e.g. `In Stock ({stock} units available in central warehouse)`). Live inventory dynamically replaces the `{stock}` placeholder on storefront product pages.

### 8. Back-in-Stock Notifications
- Physical products with 0 stock display an out-of-stock badge and an automated waitlist email subscription input.
- Replenishing variant inventory in the admin dashboard triggers automatic emails to waitlisted customers and clears the list.

### 9. Multi-Provider Checkout & Invoicing
- Support for **Stripe**, **PayPal**, Apple Pay, Google Pay, and Amazon Pay.
- Automated generation of tax-compliant **PDF Invoices** and warehouse **Packing Slips** with live in-browser preview and direct PDF download.

### 10. EU & German E-Commerce Law Compliance
- **Button-Lösung (§ 312j Abs. 3 BGB)**: Checkout submission button strictly states *"Order with Obligation to Pay"* (*Zahlungspflichtig bestellen*).
- **Small Business Regulation (§ 19 UStG / Kleingewerbe)**:
  - Toggle between § 19 UStG exempt, B2C included VAT, or B2B excluded VAT.
  - Default statutory tax notice in English: *"According to § 19 UStG, no value-added tax is charged (small business regulation)."*
  - In § 19 UStG mode, manual product VAT inputs are automatically locked to 0% to prevent unlawful VAT charging.
  - Disclosed automatically on checkout, order confirmations, and PDF invoices (§ 14 UStG).
- **Digital Goods Right of Withdrawal Waiver (§ 356 Abs. 5 BGB)**:
  - Mandatory checkout consent checkbox for immediate execution of digital contracts, acknowledging the statutory loss of the right of withdrawal upon instant download delivery.
- **GoBD-Compliant Sequential Order Numbers**:
  - Ascending sequential counter starting at 10000.
  - Configurable alphanumeric brand prefix (up to 7 characters, e.g. `ORD-10001`) and daily date codes (`YYYYMMDD`).
- **Statutory Legal CMS Pages & Cookie Consent**:
  - Pre-installed Legal Notice (*Impressum* pursuant to § 5 DDG), General Terms & Conditions (*AGB*), Right of Revocation (*Widerrufsbelehrung*), and Privacy Policy (*DSGVO*).
  - TTDSG § 25 compliant granular Cookie Banner with preferences management.

### 11. Digital Products Architecture & Multi-File Downloads
- **Physical vs. Digital Delivery Toggle**: Switch products between physical goods and digital downloads with a single click.
- **Zero-Cost Digital Shipping**: Orders containing exclusively digital items automatically waive shipping fees (`0.00 €`), disable physical carrier selection, and bill only the digital product.
- **Virtual Inventory**: Physical warehouse stock tracking is disabled for digital products, reflecting unlimited virtual stock (`∞`).
- **Multi-File Assets Manager**: Upload multiple download files (firmware, manuals, ZIP archives, 3D STL models) or assign external URLs per product.
- **Digital Bill of Materials (BOM)**: Attach digital asset files directly within the product BOM alongside physical parts, tagged with `⚡ DIGITAL FILE` badges.
- **Instant & Account Downloads**: Instant download buttons on order confirmation and persistent downloadable access in the customer account drawer.

### 12. Customer Account Orders Detailed View (`/account/orders`)
- Customers can click any completed order card to open a full slide-over drawer:
  - 4-step fulfillment status tracker (*Order Placed* &rarr; *Processing* &rarr; *Shipped* &rarr; *Delivered*).
  - Itemized product list with variant names, SKUs, quantities, and line item prices.
  - One-click digital download buttons for purchased digital files.
  - Shipping address and financial summary breakdown.

