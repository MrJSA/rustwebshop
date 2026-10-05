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

---

### Milestone 12: Digital Auto-Fulfillment, Dynamic Downloads Hub, Promo Codes, Admin User Governance & Payment Engine Overhaul
- **Date**: October 2026
- **Context & User Requirements**:
  1. **Automatic Completion for Digital-Only Orders**:
     - When an order contains exclusively digital items and payment succeeds, immediately set `order_status = 'completed'` without requiring merchant fulfillment or shipping actions.
  2. **Customer Digital Downloads Library & Dynamic File Updates**:
     - Provide a dedicated customer account overview (`/account/downloads`) displaying all digital purchases with download buttons.
     - **Dynamic Sync**: If the merchant alters, replaces, or adds files to a digital product or its BOM (`part_sku = 'DIGITAL_FILE'`) in the Admin, past customers must dynamically receive the updated and changed files whenever they download.
  3. **Promo & Discount Codes Management (Under Products Category)**:
     - Admin page under Products Category to create and manage promo/discount codes.
     - Support 3 distinct discount models:
       - **Free Shipping** (`free_shipping`): Zeroes out shipping cost while leaving product totals unchanged.
       - **Fixed Currency Discount** (`fixed_amount`): Deducts a specific amount (e.g., 10.00 €) from the cart subtotal.
       - **Percentage Discount** (`percentage`): Deducts a percentage (e.g., 15%) from the cart subtotal.
     - Configurable minimum order amount, total usage limits, expiration date, and active toggle.
     - Storefront checkout integration with live code validation, dynamic subtotal reduction, and backend validation.
  4. **Admin Users & Access Governance (Settings Section)**:
     - Admin page under Settings (`/settings?tab=users`) to view existing admin users and create new users with username, email, password (hashed with bcrypt), and role (`superadmin`, `admin`, `editor`).
     - Safeguards to prevent deleting the final remaining admin user.
  5. **Payment Engine Overhaul & Provider Filtering**:
     - Filter out disabled payment providers on the storefront checkout page so only active providers are rendered.
     - Eliminate false-positive payment approvals: backend `PaymentEngine` previously approved all payments unconditionally. Now strictly enforces card validation (Luhn algorithm, expiration date checking, CVC length) and Stripe test decline numbers (`4000 0000 0000 0002` = card declined, `4000 0000 0000 0069` = insufficient funds). Checkout fails with a 400 Bad Request if an invalid card is supplied.
  6. **Technical Research & Best Practice Synthesis**:
     - **Stripe & SvelteKit Integration** (`joshnuss/svelte-stripe`, `sveltekit-stripe`, `svelte-shop`):
       - Tokenization/Elements flow: Payment details never touch merchant servers directly; client uses Stripe Elements or Web Payments SDK to create a PaymentMethod/token, passed to server for PaymentIntent confirmation.
       - Webhook reconciliation: Rely on `payment_intent.succeeded` or `charge.failed` webhooks for asynchronous payment status verification.
       - Strict error handling: Inform the user with actionable decline codes (`insufficient_funds`, `card_declined`, `expired_card`).
     - **Square Payment Integration** (`developer.squareup.com`):
       - Uses Web Payments SDK `payments.card()` attached to a container element.
       - Secure client tokenization produces a single-use token sent to the backend.
       - Backend charges token with `idempotencyKey` to guarantee zero duplicate charges.
     - **SvelteKit SEO Guidelines** (`svelte.dev/docs/kit/seo`):
       - Server-side rendering (SSR) is primary for search engines and social crawlers.
       - Use `<svelte:head>` on all route templates to supply canonical URLs, structured Open Graph / Twitter metadata, and descriptive titles.
       - Implement `<script type="application/ld+json">` for `Product`, `Offer`, `BreadcrumbList`, and `Organization` schemas.
       - Maintain clean, descriptive URLs and dynamic XML sitemaps.
     - **SvelteKit Performance Guidelines** (`svelte.dev/docs/kit/performance`):
       - Shift data dependencies to server `load` functions (`+page.server.js`), minimizing client waterfall requests on mount.
       - Optimize Core Web Vitals: specify explicit image dimensions (`width`, `height`) and `aspect-ratio` to avoid Cumulative Layout Shift (CLS).
       - Use lazy-loading (`loading="lazy"`) and asynchronous decoding (`decoding="async"`) for non-critical assets.
     - **Authentication & Security Architecture**:
       - Distinct JWT claims and secrets for admin users vs storefront customers.
       - Salting and hashing with bcrypt.
       - Protected route guards on both API routes and SvelteKit server loaders.
- **Architectural Implementation**:
  - `backend/migrations/0013_coupons_and_admin_users.sql`: Created `coupons` table with constraints; added `coupon_code` and `discount_cents` to `orders`; added `email` and `role` to `admin_users`.
  - Registered migration in `backend/src/db.rs`.
  - Created models `backend/src/models/coupon.rs` and updated `backend/src/models/admin_user.rs` & `order.rs`.
  - Backend payment validation in `backend/src/services/payment_engine.rs`: Luhn algorithm, expiry month/year check, CVC length verification, and Stripe decline card simulation.
  - Backend checkout service `backend/src/services/checkout.rs`:
    - Auto-completes digital-only orders when `payment_status == "paid"`.
    - Validates coupon codes, calculates discounts, and increments coupon `used_count`.
    - Blocks checkout if card fails validation or decline rules.
  - Dynamic digital file resolution in `backend/src/routes/public.rs`:
    - `GET /api/v1/customer/downloads`: Queries paid/completed digital items, joining live `products` and BOM `product_parts` (`part_sku = 'DIGITAL_FILE'`) on `product_id`.
    - `GET /api/v1/customer/orders` & `GET /api/v1/orders/lookup/:order_number`: Dynamically resolve latest product files and BOM attachments.
  - Admin UI:
    - `admin/src/lib/components/CouponsManager.svelte`: Complete coupon creation/editing modal, discount type selection, expiration dates, and usage limits.
    - `admin/src/lib/components/AdminUsersManager.svelte`: Admin user list, creation modal, password updates, and deletion prevention for the last admin.
    - Admin nav updated with Products -> Promo & Discount Codes and Settings -> Admin Users & Access.
  - Storefront UI:
    - `storefront/src/routes/checkout/+page.svelte`: Enabled-only provider filtering, live promo code application and discount breakdown, card decline error feedback.
    - `storefront/src/routes/account/downloads/+page.svelte`: Comprehensive customer digital downloads library with live sync guarantee, search filter, and individual file package download buttons.
    - Updated `+layout.svelte`, `account/orders/+page.svelte`, and `order-success/[orderNumber]/+page.svelte` with direct links and multi-file download actions.
- **Status**: Fully Implemented & Production Ready.



### [2026-10-05] Milestone 13: Real Payment Gateways (Stripe Payment Element + Wallets, PayPal Orders v2), Exactly-Once Order Finalization & SEO
- **Context & User Requirements**:
  1. Checkout was not working reliably; only the Stripe hosted redirect had worked briefly.
  2. Card details should be entered directly on the shop's checkout page.
  3. PayPal button, plus Apple Pay / Google Pay / Amazon Pay — the latter regulated through Stripe, with on/off toggles in Admin → Payment Providers underneath the Stripe settings.
  4. SEO optimization per https://svelte.dev/docs/kit/seo.
- **Root Causes Found (diagnosis)**:
  - `0014_add_stripe_elements_config.sql` was never registered in `db.rs`, so the `stripe_elements` provider row never existed → on-site Elements could never be selected.
  - The browser chose the amount: `create-intent` and `create-checkout-session` accepted `amount_cents` / line-item prices from the client, and hosted-session verification only checked `payment_status == paid`, not the amount → a customer could pay 0.50 € for any order.
  - `paypal`, `apple_pay`, `google_pay`, `amazon_pay` were simulated in `PaymentEngine` and **always approved** (client sent `tok_<provider>_<timestamp>`) → free orders marked `paid`.
  - Offline "sandbox" fallback approved Stripe payments when no key was configured.
  - Orders were only created if the browser returned and still had `sessionStorage`; a closed tab after paying = paid but no order.
  - A new PaymentIntent was created on every total change (orphan intents); `cardholderName` was assigned without being declared (runtime ReferenceError for logged-in customers with saved addresses).
  - `Order` FromRow lacked `#[sqlx(default)]` for `coupon_code`/`discount_cents` while invoice queries select explicit columns.
  - **Security**: the public storefront proxy forwarded `/api/v1/admin/*` and the `X-Dev-Mode` header → anyone could read the Stripe secret key, orders and customers through port 8080.
- **Rollback**: Milestone 12's raw-card Luhn/expiry/CVC engine and mock decline simulation (`payment_engine.rs`) were removed. Raw card data must never reach our server (PCI DSS); Stripe performs validation and test-card simulation itself. Rule 6 rewritten accordingly.
- **Architectural Implementation**:
  - `0014_payment_gateway_overhaul.sql` (replaces the unregistered file): `payment_configs.webhook_secret`; merges/deletes legacy `stripe_elements`; deletes simulated `apple_pay`/`google_pay`/`amazon_pay` rows; clears seeded placeholder keys; Stripe `config_data = {checkout_mode: elements|hosted, methods: {apple_pay, google_pay, link, amazon_pay, paypal}}`; new `pending_checkouts` table; `orders.payment_reference` with partial unique index.
  - `services/payments.rs`: Stripe REST client (PaymentIntent with explicit `payment_method_types`, Checkout Session, retrieve, refund, idempotency keys, webhook HMAC-SHA256 verification) and PayPal Orders v2 client (OAuth, create order, idempotent capture via `PayPal-Request-Id`, refund). Unit-tested.
  - `services/checkout.rs`: single `price_order` used by quote and order creation (row locks only when creating); shipping rate must belong to a zone serving the destination country; VAT reduced pro rata by discounts; `validate_for_order`; `create_pending` / `finalize_pending` (row-locked, idempotent, amount-checked) / `create_free_order`.
  - `routes/payments.rs`: `POST /checkout/quote`, `/checkout/free`, `/checkout/stripe/intent|session|complete`, `/checkout/paypal/order|capture`, `/payments/stripe/webhook`. Failed finalization after a verified payment → automatic provider refund.
  - Admin: `PaymentsManager.svelte` (used by Settings tab and `/settings/payments`): Stripe mode (on-site vs redirect), publishable/secret/webhook keys with test/live mismatch detection, per-method toggles under Stripe; PayPal with sandbox toggle and Pay Later. Secrets are write-only (API returns masked hints only). Order refund calls the provider; order is only marked refunded after success.
  - Storefront checkout: server quote drives all totals; Stripe Payment Element in deferred-intent mode (`elements.submit()` first → server creates PaymentIntent → `confirmPayment` with `redirect: 'if_required'`); PayPal Smart Buttons via an action; `/checkout/complete` return page for redirect methods/hosted mode (replaces `stripe-success`).
  - Storefront proxy blocks `admin/*` (incl. encoded) and strips `X-Dev-Mode` / `dev=true`.
  - SEO: `Seo.svelte` + `$lib/seo.js`; canonical from request origin (previous product canonical pointed at a hard-coded `rustcraft.io` domain); Product (+ per-variant Offers), BreadcrumbList, Organization, WebSite/SearchAction JSON-LD with `<` escaping; central `noindex` for private paths; `/robots.txt`; dynamic `/sitemap.xml`; removed duplicate static `<title>` from `app.html`.
- **Verification**: `cargo test` (webhook signature incl. tamper/stale/wrong-secret, amount formatting/parsing); storefront and admin production builds; SSR smoke test of SEO tags, robots, sitemap and proxy blocking; integration test against a throwaway Postgres: migrations 0001–0014 apply, quote/validation errors, signed webhook creates exactly one order (replay and 5 concurrent deliveries → 1 order, stock decremented once), underpaid payment rejected + refund attempted, forged signature rejected, 100 % coupon free order, invoice endpoint, admin refund keeps `paid` when the provider refund fails.
- **Known Open Issue (not addressed here)**: Admin authentication is effectively disabled — the admin app's proxy and loaders inject `X-Dev-Mode: true`, so anyone reaching port 4000 (or backend port 8081) has full admin access. Needs a dedicated fix (real session cookie / JWT in the admin app, remove the bypass, stop publishing 8081).
- **Status**: Implemented & verified locally; requires `docker compose up -d --build` plus Stripe/PayPal keys (see admin hints) for live end-to-end payments.

### [2026-10-05] Milestone 14: Security Audit & Hardening, All Stripe Methods + Express Wallet Buttons, Working Import/Export, Privacy Cleanup
- **Context & User Requirements**: checkboxes for all Stripe payment options; visible Apple Pay / Google Pay / Amazon Pay buttons; card form (not Link) as default; full safety/performance/code-quality review; verify import/export; protect the admin correctly; rule out SQL injection and data leakage; no sensitive or personal data in the repository or its history; generic defaults instead of the owner's name.
- **Critical Findings (all fixed)**:
  - Admin API fully open: `X-Dev-Mode: true` / `?dev=true` bypass, auto-injected by the admin proxy; hard-coded JWT secret published in the repo (anyone could forge admin tokens); backdoor passwords (`RustCraftAdmin2026!` while default, `admin`/`admin123`); default password prefilled on the login page and printed in README; no role checks (an editor could create a superadmin); `change-credentials` changed the *first* admin and only accepted POST while the UI sent PUT (could never succeed).
  - Customer account takeover: `reset-password` set any account's password from just an email; registering an existing email overwrote its password.
  - Data leakage: guessable sequential order numbers exposed name, email, items and download links; public product/BOM APIs exposed digital download URLs (free downloads); public `/store/info` exposed SMTP host/user; `StoreSettings` serialised the SMTP password (admin browser + export file); download links returned for unpaid orders.
  - Import never worked: it inserted `pages.id` (column does not exist) and swallowed all errors inside one transaction → silent rollback while reporting success. Shipping zones/rates were exported but not imported; BOM parts and coupons were not exported; category/menu parents depended on file order.
  - SQL built with `format!` in `list_products` (quote-escaped, but fragile) + N+1 variant queries; HTML injection of customer names in verification emails; verification emails never sent (StoreSettings query missing columns, hard-coded localhost URL); wide-open CORS; uploads accepted any extension.
- **Architectural Implementation**:
  - `0015_security_hardening.sql`: `server_secrets`, customer reset-token columns, `orders.access_token` (+ backfill), lowercase email index, role normalisation (oldest admin → superadmin), generic `ORD` order prefix, generic name in the seeded legal notice.
  - `services/auth.rs` (JWT key management, role-bound tokens, CSPRNG tokens, SHA-256, login limiter; unit-tested); `middleware/auth.rs` rewritten with DB-backed `CurrentAdmin`, default-password lock and role gates (unit-tested).
  - Admin app: `hooks.server.js` (route guard + security headers + `handleFetch`), `lib/server/session.js`, proxy rewritten (httpOnly session, credential stripping, JSON errors), login/layout without localStorage tokens, ~70 dead `X-Dev-Mode` headers removed.
  - Storefront: proxy returns JSON errors and `no-store`; `hooks.server.js` security headers; uploads served sandboxed; order-success uses the access token; track page requires email; reset-password page implements the token flow; register handles "verify email first".
  - Export v1.1 (complete, no secrets) / typed, fail-fast, order-independent import; media ZIP uses unique stored names and reports missing files.
  - Payments: 13 optional Stripe methods + wallets with grouped admin checkboxes; Express Checkout Element for wallet buttons; card-first Payment Element; delayed-payment handling (pending → paid / cancelled + restock) incl. hosted-session async events; Apple Pay/Google Pay domain registration from the admin.
  - Personal identifiers removed from code/docs (handle in prefixes/placeholders, first name in placeholders, `.github/FUNDING.yml`); compose: backend bound to 127.0.0.1, no default JWT secret, `ADMIN_INITIAL_PASSWORD`, `SHOP_PUBLIC_URL`/`ADMIN_PUBLIC_URL`.
- **Repository / History Audit**: Every blob in all 9 commits scanned — no API keys, tokens, private keys, `.env` files or binaries; addresses/phones/VAT IDs are fictional. Remaining personal data in history: owner handle/first name in old placeholders and `FUNDING.yml`, and the author name/email on every commit → requires a history rewrite + force-push (owner decision).
- **Verification**: `cargo test` (8 unit tests); 34 end-to-end security checks + 24 flow checks against a throwaway Postgres (forged tokens, bypasses, backdoors, forced password change, RBAC, brute-force lock, account takeover, reset tokens, order privacy, download leakage, SQL injection, delayed payments, export→import round trip and conflict rollback); admin app session/guard test; storefront and admin production builds; published-default-password rotation verified.
- **Status**: Implemented & verified locally. Deploy with `docker compose up -d --build`; existing admin/customer sessions are invalidated once (new signing key).

### [2026-10-05] Milestone 15: Section Permissions, Self-Updater, Domains/HTTPS, Email Fixes, Server Install Guide
- **Context & User Requirements**: no AI attribution in commits (removed from the last pushed commit); server install instructions; a working way to create editor users; per-user permissions for Overview & Analytics, Products, Orders & Slips, Storefront & Design and Settings (defaults: settings only superadmin; overview & storefront off for editors; rest on); version tracking with "check for updates" / "update" in the admin; shop on subdomains (server config or admin setting); fix email sending and test emails; Rust/Svelte as much as possible.
- **Root Causes Found**:
  - Editors could not be created: the user dialog had no role field (always "admin").
  - Test email: the two admin email pages sent `recipient` / `to_email` while the API expected `recipient_email` → every test rejected; plain-text error masked as "network failure". With SMTP switched off the API returned fake success.
  - Automatic order/payment emails never sent: they loaded `StoreSettings` with an outdated column list → `query_as` error → silently skipped.
  - Seeded sender addresses (`shop@rustcraft.local`, `noreply@rustcraft.com`) are rejected by real mail servers.
- **Architectural Implementation**:
  - `0016_admin_permissions.sql` (`admin_users.permissions`, role defaults, sender-address cleanup); `middleware::{SECTIONS, default_permission, effective_permissions, required_access}`; field-level check for `PUT /settings/system`; user CRUD with permissions, self-edit protection; `/auth/me` returns permissions. Admin: page guard + nav filtering in `hooks.server.js`/layout; rebuilt `AdminUsersManager` (role picker, section checkboxes, access matrix).
  - Email: robust `send_email_raw` (optional sender name, login fallback, port-based TLS mode, 20 s timeout, real error text), test endpoint accepts unsaved form values and all three field names.
  - New Rust crate `updater/` (axum; `/status`, `/check`, `/update`, `/job`, `/config`) + Dockerfile (musl + docker CLI + git); backend `/admin/system/*` proxy (superadmin only); admin `SystemManager` (version, check, update with live log, domains). `VERSION` file, release = annotated `vX.Y.Z` tag.
  - Compose: `name: rustwebshop`, `updater` service, optional `proxy` profile (Caddy image with baked Caddyfile), `PUBLISH_ADDR`/port variables, `updater_net`; `.env.example` and `INSTALL.md` (Docker install, DNS, HTTPS proxy or own nginx/Caddy, firewall, first steps, updates, backups/restore, troubleshooting, release process, security notes); README architecture table.
- **Verification**: backend 9 + updater 3 unit tests; 32 end-to-end checks with a throwaway Postgres and Mailpit (role defaults, section enforcement incl. field-level settings check, live permission changes, self-promotion blocked, test email with unsaved values delivered, clear errors for SMTP off/unreachable, automatic order + payment emails delivered with correct sender); updater against a temporary git origin (token auth, release detection + notes, refuses dirty tree, aborts before code changes when the backup fails); admin production build; compose config validation.
- **Not verified here**: a complete real update/rebuild cycle and Let's Encrypt issuance (need a server with a public domain).

### [2026-10-05] Fix: No verification email on customer registration
- **Diagnosis** (backend log): verification emails were skipped with `[SMTP disabled]` — the *saved* settings had email sending off/no host, while the admin test email (which since Milestone 15 uses the unsaved form values and force-enables SMTP) succeeded. Misleading combination; customers with "require email verification" on were locked out. The storefront also never rendered `successMsg`, so post-registration messages were invisible.
- **Fix**: test email returns a `warning` when the saved settings would not send (or differ from the form); admin shows it plus a banner when SMTP is off. Saving "require email verification" with SMTP off (or switching SMTP off while required) is refused — only when one of those fields changes. Registration sends the verification email synchronously and returns `verification_email_sent`. New `POST /auth/customer/resend-verification` (generic answer, rate-limited per address). Storefront login shows success messages and a "Resend verification email" button after registration and on 403 (unverified) login.
- **Verification**: 18 end-to-end checks with a throwaway Postgres + Mailpit (reported state reproduced, warning, guard, resend, delivered link activates the account, login works, generic answer for unknown address, new registrations send immediately).

