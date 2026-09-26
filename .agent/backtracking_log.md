# Agent Backtracking & Architecture Decision Log

This log tracks architectural decisions, state checkpoints, implementation milestones, and any rollbacks or problem resolutions during development.

---

## Log Entries

### [2026-09-23] Milestone 1: Architectural Alignment & Port Specification
- **Decision**: Decouple the system into 4 Docker Compose services (`db`, `backend`, `storefront`, `admin`) on the `shop_net` bridge network.
- **Port Strategy**:
  - Customer Storefront: Host Port `8080` (Container `3000`)
  - Admin Dashboard: Host Port `4000` (Container `4000`)
  - Rust Backend: Internal Container Port `8000` (`8081` optional host debug)
  - PostgreSQL Database: Internal `5432` only, service `db` with volume `pgdata`
- **Rationale**: Keeps database strictly private, exposes Customer Storefront on the primary web port `8080`, and isolates the Admin Dashboard on port `4000` for back-office security and separate firewalling.
- **Status**: Completed & Approved by User.

### [2026-09-23] Milestone 2: Core Domain Model & Schema Design
- **Decision**: Dual-tier product structure (`products` + `product_variants`).
  - Products store common metadata: `title`, `slug`, `description`, `product_type` (`physical` or `digital`), `category`, `subcategory`, `base_price_cents`, `digital_download_url`.
  - Variants store concrete sellable inventory units: `sku`, `title`, `price_override_cents`, `attributes` (`jsonb` with GIN indexing for color/size/material), `stock_quantity`, `low_stock_threshold`.
  - Orders store order items, delivery addresses, shipping rates, and payment statuses.
  - Settings tables store payment provider configs (Stripe, PayPal, Apple Pay, Google Pay, Amazon Pay), shipping zones/rates, and debug/deployment modes.
- **Status**: Implemented in database migrations.

### [2026-09-23] Milestone 3: Concurrency & Inventory Lock Safety
- **Decision**: Use PostgreSQL explicit row-level locking (`SELECT ... FOR UPDATE`) in the checkout transaction.
- **Rationale**: Prevents race conditions and overselling when multiple shoppers purchase the last available stock item simultaneously.
- **Status**: Completed.

### [2026-09-23] Milestone 4: Offline Compilation Robustness for Docker
- **Issue**: `sqlx::query!` compile-time macros fail offline without a live PostgreSQL server running during `docker build` or local `cargo check`.
- **Resolution**: Use `sqlx::query` and `sqlx::query_as::<_, Model>` with explicit `.bind(...)` and runtime mapping. This ensures the Rust binary builds self-contained in Docker without needing database connectivity at compile time.
- **Status**: Completed.

### [2026-09-23] Milestone 5: Docker Container Dependency Resolution
- **Issue**: In fresh container build environments without existing `package-lock.json` files, `npm ci` halts container builds.
- **Resolution**: Updated `storefront/Dockerfile` and `admin/Dockerfile` to use `npm install`, ensuring smooth automated package resolution in containerized builds.
- **Status**: Completed.

### [2026-09-23] Milestone 6: Rust Toolchain Edition 2024 Upgrades
- **Issue**: Upstream dependencies (e.g. `base64ct v1.8.3`) utilize Rust 2024 edition features requiring Rust 1.85+.
- **Resolution**: Upgraded builder image in `backend/Dockerfile` from `rust:1.84` to `rust:latest`.
- **Status**: Completed.

### [2026-09-23] Milestone 7: Client-Side SvelteKit Reverse Proxy & Full Integration Verification
- **Challenge**: Client-side `fetch()` calls in browser environments targeting `http://backend:8000` failed because `backend:8000` is an internal Docker Compose DNS name not resolvable directly by the user's host browser.
- **Resolution**: Implemented SvelteKit server-side reverse proxy route handlers at `/api/v1/[...path]/+server.js` in both `storefront` and `admin`. Client scripts now issue requests directly to `/api/v1/...` which the SvelteKit Node server proxies to `http://backend:8000` via internal Docker networking.
- **End-to-End Verification Results**:
  - Storefront catalog, dynamic SKU variant selection, and live warehouse inventory displayed at `http://localhost:8080/`.
  - Admin KPI analytics, warehouse inventory restocking, fulfillment workflows, and payment gateway toggles active at `http://localhost:4000/`.
  - ACID inventory decrement confirmed with `SELECT ... FOR UPDATE` (e.g., keyboard stock decremented 38 -> 36).
  - Printable Tax Invoice and Warehouse Packing Slip generation verified and functioning with authentication.
- **Status**: Completed & Fully Operational.

### [2026-09-24] Milestone 8: Storefront Overhaul, Policy CMS, Hierarchical Shipping, BOM, & Account System
- **Context & User Requirements**:
  1. Remove admin link from storefront header.
  2. Remove technical hero banner paragraph from storefront.
  3. Relocate search bar to top header bar positioned cleanly above the navigation menu.
  4. Dynamic customizable navigation menu editable via Admin Dashboard (`/settings/menu`).
  5. Remove packing slip admin link from post-checkout page.
  6. Remove dev port display from storefront footer.
  7. Add 7 Markdown policy pages: Shipment Policy, Contact Information, Legal Notice (Impressum), Terms & Conditions (AGB), Privacy Policy (Datenschutz), Cookie Policy, and Return Policy (Widerrufsbelehrung).
  8. Live carrier schedules & rates dynamic table embedded on Shipment Policy page.
  9. Dual-state customer account dropdown in header (Logged out: Login/Register, Lost Password; Logged in: Orders, Wishlist, Addresses, Account details, Logout).
  10. Admin image upload to server with client-side WebP compression option.
  11. Product variants with subcategory versioning under the same product selectable by shop user.
  12. Bill of Materials (BOM) / composite product parts per variant.
  13. Product edit capabilities in admin product management.
  14. 3-tier hierarchical shipping management: Providers (DHL, Hermes, UPS) -> Zones (with Country Codes) -> Price Categories / Rates.
- **Architectural Implementation**:
  - `0002_enhanced_features.sql`: Added `shipping_providers`, `shipping_zones.provider_id`, `product_parts`, `pages`, `navigation_items`, `customer_addresses`, `customer_wishlist`.
  - Backend media server mounted at `/uploads` via `tower_http::services::ServeDir`.
  - Markdown CMS editor with live preview in Admin (`/settings/pages`).
  - Zero-dependency client-side WebP canvas compression in browser before uploading to server.
- **Status**: Completed & Verified.

### [2026-09-24] Milestone 9: Policy Push-Caching, Hero Customizer (8BitDo & 8BitMods), Category Tree, Variant Media & BOM, Order Logistics Modals
- **Context & User Requirements**:
  1. Policy Push-Caching: When policies are edited in admin, push updates asynchronously to the customer storefront container in-memory cache for instant 0ms load.
  2. Hero Section Overhaul: Clean technical text from hero; allow admin to switch between Option A (Full-width item carousel like 8bitdo.com) and Option B (Split hero with 60% carousel + 40% 4 featured product buttons like 8bitmods.com) with customizable button colors and product selection.
  3. Shop Identity: Admin upload shop logo (with WebP compression), set store name, phone, address, tax rate.
  4. Category Hierarchy Tree: Dedicated Admin tab (`/categories`) for managing nested categories and interactive category tree selector in product modals.
  5. Variant Images, Versioning & BOM: Upload dedicated image per variant (WebP), versioning under products, and full editing of BOM parts (`PUT /api/v1/admin/parts/:id`).
  6. Orders & Fulfillment Slips: Clean fulfillment badges in table; order details modal displaying customer phone, shipping & billing addresses, line items, totals; order cancellation and refunding; tracking number enforcement on status change to `shipped`; downloadable Tax Invoice and Packing Slip PDFs.
  7. Customer Addresses & Profile: Fixed address persistence (`created_at` column sync); added standard Google Chrome `autocomplete` tokens (`shipping street-address`, `billing address-line1`, etc.); customer profile editing with currency selector (`EUR`, `USD`, `GBP`, `CHF`, `JPY`) and password change verification.
- **Architectural Implementation**:
  - `0003_extended_features.sql`: Added `categories` table hierarchy, `customers` profile table, `store_settings` columns (`logo_url`, `phone`, `hero_config`), and `orders.customer_phone`.
  - Push-cache endpoint `POST /api/cache/page` in Storefront; `policies/[slug]/+page.server.js` checks in-memory cache with stale-while-revalidate headers.
  - SvelteKit Admin proxy handles flattened and nested order structures.
  - All 4 Docker containers running healthy on host ports 8080 (Storefront) and 4000 (Admin).
- **Status**: Completed, Fully Verified & Production Ready.

### [2026-09-24] Milestone 10: Email Engine, Media Library, Dynamic Carousels, Category Recursion, Admin Security & UI Polish
- **Context & User Requirements**:
  1. **Email Addon**: Connect shop to SMTP server. Send automated emails for order creation, payment confirmation, shipping with tracking link, back-in-stock alerts, customer registration email verification, and a test email button in admin settings.
  2. **Customer Checkout Policy**: Admin settings toggle to require registered accounts for checkout and mandatory email verification before purchase.
  3. **Arbitrary Depth Category Hierarchy**: Unlimited category nesting (Category -> Subcategory -> Sub-Subcategory -> ...) in database, admin tree management, and product category picker.
  4. **Multiple Navigation Menus & Route Inspector**: Header Menu and Footer Menu tabs; live overview panel of all system routes/pages/policies/categories with smart dropdown link creator.
  5. **Fix Invoice PDF Authorization**: Resolved `{"error":"Missing or invalid Authorization header"}` error when downloading Invoices and Packing Slips from the Admin orders table.
  6. **Admin Authentication & Default Password Security**: Dedicated `/login` page with default credentials (`admin` / `RustCraftAdmin2026!`); persistent security warning banner and modal prompt to change credentials on first login; documented in `README.md`.
  7. **Storefront Product Carousels & Dynamic Ordering**: 5 products per row (centered horizontally when <= 5, smooth carousel scroll when > 5) for Featured Products, New Products (configurable days limit), Best Sellers (by sales count), and Product Catalog (in-stock only); admin reordering and toggling.
  8. **8BitMods 4 Featured Buttons Overhaul**: Transparent floating product image on top with enhanced depth/layering; custom subtitle; toggleable price badge; removed "Featured" badge text.
  9. **Media Library & Image Upload Proxying**: Added persistent volume `uploads_data:/app/uploads` to Docker Compose; frontend reverse proxies for `/uploads/[...file]` in Storefront and Admin; dedicated Media Library (`/settings/media`) with automatic WebP compression, thumbnail grid, and modal picker.
  10. **Shop Branding & Header Transparency**: Customizable subtitle (replace hardcoded text or toggle off/on); toggle shop title off (enlarged logo reaching into header, shifting navigation to center); glassmorphism header menu blur.
  11. **Back-in-Stock Waitlist**: Email subscription form on out-of-stock items; automatic email notification dispatched to waitlisted customers upon inventory restocking.
- **Architectural Implementation**:
  - `0004_media_email_auth_extended.sql`: Tables `admin_users`, `media`, `stock_notifications`; columns for SMTP credentials, checkout policies, branding flags, carousel configs in `store_settings`; `is_verified` and `verification_token` in `customers`; `location` in `navigation_items`.
  - Backend `backend/src/services/email.rs` utilizing `lettre` with async TLS and responsive HTML email templates.
  - SvelteKit `[...path]/+server.js` updated to proxy binary array buffers (fixing multipart uploads > 512KB and resolving PDF streaming).
  - Configured `BODY_SIZE_LIMIT: 50M` in `docker-compose.yml`.
  - Recursive Svelte component `CategoryTreeNode.svelte` for arbitrary depth category nesting.
  - Reusable `MediaPickerModal.svelte` across admin settings and product management.
  - End-to-end browser subagent verified all flows on ports 8080 and 4000.
- **Status**: Completed, Fully Verified & Production Ready.

### [2026-09-26] Milestone 11: EU/German Legal Compliance, Digital Product Architecture, Multi-File Downloads, GoBD Numbering & Storefront Polish
- **Context & User Requirements**:
  1. **Digital-Only Shipping Exemption**: If an order contains only digital products, disable physical shipping selection, strictly waive shipping costs to `0.00 €`, and only bill product price.
  2. **English Default Tax Exemption Text**: Default notice for § 19 UStG small business regulation must be English: *"According to § 19 UStG, no value-added tax is charged (small business regulation)."*
  3. **Products & BOM Digital Toggle & Multi-File Upload**:
     - Toggle between Physical Product (OFF) and Digital Product (ON).
     - When Digital is ON, disable stock tracking function and show virtual unlimited stock (`∞`).
     - Multi-file upload and management for digital assets.
     - BOM (Bill of Materials) support for attaching digital asset files alongside physical components.
  4. **Conditional Manual VAT Percentage on Products**:
     - Manual product VAT rate input conditionally disabled when the shop is in § 19 UStG (Kleingewerbe) mode, displaying a `§ 19 UStG (0% Exempt)` badge.
  5. **Overview & Analytics Orders Crash Fix**:
     - Resolve issue where Overview and Analytics showed 0 orders while Orders & Slips showed active orders.
  6. **Customer Account Orders Detailed View (`/account/orders`)**:
     - Allow customers to click any order card to view full details in an interactive drawer: fulfillment timeline, itemized products, quantities, prices, digital download links, and delivery addresses.
  7. **EU & German E-Commerce Law Compliance**:
     - Fulfill German Button-Lösung (§ 312j Abs. 3 BGB: *"Order with Obligation to Pay"*).
     - Digital goods waiver checkbox (§ 356 Abs. 5 BGB) for immediate execution of digital downloads.
     - GoBD-compliant sequential ascending order numbering.
     - Statutory disclosures: Impressum (§ 5 DDG), AGB, Widerrufsbelehrung, DSGVO, and TTDSG Cookie Banner.
  8. **Custom Stock Text in Storefront & Design Settings**:
     - Configurable template `"In Stock ({stock} units available in central warehouse)"` dynamically displaying live inventory.
  9. **Storefront Product Carousels Auto-Rotate & Endlessly Looping**:
     - Seamless infinite wrapping when reaching the end of the carousel.
     - Auto-advance every 7 seconds, pausing on hover.
- **Architectural Implementation**:
  - `0010_tax_modes_and_product_vat.sql`: Added `tax_mode`, `tax_notice`, and product-level `tax_rate_percent`.
  - `0011_custom_order_numbers.sql`: Added sequence `order_number_seq` starting at 10000 and custom format settings (`order_prefix`, `order_prefix_enabled`, `order_date_enabled`).
  - `0012_stock_template_and_english_tax.sql`: Added `stock_display_template` and updated default English notice.
  - Backend `backend/src/services/checkout.rs`:
    - Evaluates `is_digital_only` flag across cart items, forcing `shipping_cost_cents = 0`.
    - Multi-file download links formatted from JSON array in `products.digital_download_url` and BOM items (`part_sku = 'DIGITAL_FILE'`).
  - Backend `backend/src/routes/admin.rs`:
    - Fixed `ColumnDecode` null panic in `admin_get_purchase_analysis` by using `LEFT JOIN` and nullable extraction (`Option<String>`), restoring Overview and Analytics.
  - Storefront `storefront/src/routes/account/orders/+page.svelte`: Built interactive order detail drawer with status step tracker, line items, and digital download buttons.
  - Storefront `storefront/src/routes/checkout/+page.svelte`: Added digital-only banner, legal checkboxes, and Button-Lösung button.
  - Storefront `storefront/src/routes/+page.svelte`: Added endless looping logic, 7-second auto-scroll interval, and hover detection.
  - Admin `admin/src/routes/products/+page.svelte`: Added Physical/Digital delivery toggle, multi-file uploader, stock disabled state, conditional VAT input, and BOM digital asset support.
  - End-to-end browser subagent verified all flows, order placement `#ORD-10001`, and instant downloads.
- **Status**: Completed, Fully Verified & Production Ready.


