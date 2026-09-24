# RustCraft E-Commerce — High-Performance Rust Webshop

A fast, memory-safe, and ACID-compliant webshop built with a **Rust (Axum + SQLx + PostgreSQL)** backend and modern **SvelteKit** user storefront and administrative dashboard, containerized with Docker.

---

## 🚀 Quick Start

Start the entire stack using Docker Compose:

```bash
docker compose up -d --build
```

### 🌐 Access URLs & Ports

| Service | Host URL | Description |
| :--- | :--- | :--- |
| **Customer Storefront** | [http://localhost:8080](http://localhost:8080) | Customer-facing shop, product showcase, shopping cart, and checkout. |
| **Admin Dashboard** | [http://localhost:4000](http://localhost:4000) | Management back-office for inventory, orders, media, categories, and settings. |
| **Backend REST API** | [http://localhost:8081](http://localhost:8081) | Rust Axum API (Internal container port: `8000`). |
| **PostgreSQL Database** | `localhost:5432` | ACID-compliant relational storage. |

---

## 🔐 Default Admin Credentials

When accessing the Admin Dashboard at [http://localhost:4000](http://localhost:4000), use the default administrative credentials:

- **Username**: `admin`
- **Password**: `RustCraftAdmin2026!`

> **Security Notice**: Upon first login, a persistent security banner and credential prompt will appear, allowing you to customize your admin username and password. You can also update credentials at any time via the user badge in the admin header.

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

### 6. Storefront Hero Showcase & 5-per-Row Carousels
- **8BitDo Full-Width Carousel**: Widescreen slider showcasing selected highlight products.
- **8BitMods 4 Featured Buttons**: Split layout with 60% slider and 40% grid of 4 product buttons featuring floating transparent images with depth layering, customizable subtitles, toggleable price badges, and custom button background colors.
- **5 Products Per Row Carousels**:
  - Automatically centers products if &le; 5 items.
  - Automatically turns into a smooth horizontal scrollable carousel with left/right scroll arrows if &gt; 5 items.
  - Configurable sections: Featured Products, New Arrivals (configurable days threshold), Best Sellers (configurable limit), and In-Stock Product Catalog.

### 7. Brand Identity & Header Transparency
- Customizable subtitle (replace default `"Rust Powered • ACID Fast"` or toggle off/on).
- Toggle shop title text off to display an enlarged logo reaching into the header with a centered navigation menu.
- Glassmorphism navigation bar with frosted translucent backdrop blur.

### 8. Back-in-Stock Notifications
- Physical products with 0 stock display an out-of-stock badge and an automated waitlist email subscription input.
- Replenishing variant inventory in the admin dashboard triggers automatic emails to waitlisted customers and clears the list.

### 9. Multi-Provider Checkout & Invoicing
- Support for **Stripe**, **PayPal**, Apple Pay, Google Pay, and Amazon Pay.
- Automated generation of tax-compliant **PDF Invoices** and warehouse **Packing Slips**.
