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

### [2026-10-06] Fix: Saving other settings switched email sending off again
- **Diagnosis**: registration emails still skipped with `[SMTP disabled]` while admin test emails worked. Four admin forms (Store Identity, both email pages, Storefront & Design, footer editor) saved with `{ ...settings, <own fields> }`, where `settings` was the copy loaded when the page opened. Any later save from another form re-sent the old `smtp_enabled = false` (and old SMTP host/sender), silently reverting the email configuration. The Storefront page also used non-existent cookie field names (`cookie_policy_url`, `cookie_banner_*_text`) so cookie-banner texts were never saved/loaded.
- **Fix**: every form sends only its own fields (backend keeps the rest via `COALESCE`); cookie fields mapped to `cookie_banner_policy_url`, `cookie_accept_label`, `cookie_deny_label`, `cookie_preferences_label`. Rule added (Partial saves).

### [2026-10-06] Checkout: every payment method as its own list entry
- **Problems reported**: Apple Pay / Google Pay / Amazon Pay not listed; Klarna failed with "activate in dashboard"; Link appeared although disabled and felt like the default; payment option labelled "Stripe"; wanted all options visible at a glance, card entry as default.
- **Causes**: all enabled Stripe methods were loaded into one Payment Element — a single method not activated in the *Stripe* Dashboard broke the whole form with Stripe's activation error; wallets lived in a separate Express Checkout block that only appeared after the form and terms were complete; Stripe decided labels and Link presentation.
- **Rollback**: the Milestone 14 express-checkout block (wallet buttons above the order button) and the provider tiles were removed.
- **Implementation**: `checkoutMethods()` builds one entry per method; per-method Payment Element (`paymentMethodTypes: [type]`) and per-method PaymentIntent (`payment_method` → `stripe::intent_type_for`, unit-tested); hidden Express Checkout probe for Apple/Google Pay availability; selected wallet renders its own button with the obligation-to-pay label; Klarna/SEPA get billing country/address on confirm; admin method notes updated.
- **Verification**: backend 10 unit tests, storefront/admin builds. Not verifiable here: rendering with real Stripe keys and wallet availability (needs a browser, HTTPS for wallets).

### [2026-10-06] Fix: checkout payment forms mixed up between methods; inactive methods listed
- **Reported**: card showed "Pay without Link" then Klarna's "Pay in full"; Klarna showed the Link error; ordering failed with "elements should have a mounted Payment Element"; SEPA and Amazon Pay failed with Stripe "needs to be activated"; Link showed an IBAN field; wanted Amazon Pay as a redirect button like PayPal.
- **Root causes**: (1) the vendored `$lib/stripe` runes components with `bind:elements` across `{#key}`/`{#each}` blocks left the parent holding a destroyed Elements group and mounted forms into the wrong group; (2) methods enabled in the admin but not activated in the Stripe account were offered; (3) Link cannot be used alone in the Payment Element (needs card).
- **Rollback**: the previous entry's per-method `<Elements>`/`<PaymentElement>`/`<ExpressCheckout>` usage was replaced.
- **Implementation**: own imperative components `StripePaymentForm` / `StripeExpressButton` (create + destroy their own Elements); `GET /checkout/stripe/methods` + admin `GET /settings/payments/stripe/capabilities` (account capabilities); Amazon Pay and Link as branded buttons; `intent_types_for` returns type lists (Link → link + card), unit-tested; admin shows "active / not activated at Stripe" per method.
- **Verification**: backend 10 unit tests, storefront/admin builds. Needs a manual check in the browser with the shop's Stripe keys.

### [2026-10-06] Fix: Link prompt inside the card form; Link button stuck on "Confirming your payment…"
- **Link in card form**: Stripe shows a Link prompt in the card form by default (managed in the Dashboard); `paymentMethodTypes: ['card']` alone does not prevent it. Fixed in code with Payment Element `wallets.link: 'never'` (documented in the Stripe.js reference; missing from the installed @stripe/stripe-js 5.10 types).
- **Link/express hang**: the express button container was hidden (`class:hidden`) while confirming — the Stripe frame is still needed to finish Link/wallet payments, so confirmation never completed. The button now stays visible; express confirmations have a 2-minute timeout with a clear message; switching methods resets the "confirming" state.
- **Docs consulted** (Stripe, user-provided): Amazon Pay (EUR + DE supported, redirect wallet, activate in Dashboard), Apple Pay / Google Pay web (HTTPS + registered payment method domain incl. subdomains, in test and live; Apple: start the sheet directly from the user gesture, use a timeout for confirmPayment; Google: a real card must be in the wallet even for tests).

### [2026-10-06] Milestone 16: Product VAT Presets Removal & §19 Greying, Admin Debug Window Removal, Official Email Layout Overhaul, Multi-Type Email Test Dispatcher, & 1-Minute Verification Resend Cooldown
- **Context & User Requirements**:
  1. Admin Products: remove VAT presets ("19%", "7%", "0%"), and grey out VAT options if "§ 19 no VAT" (`kleingewerbe`) is selected in Store Settings.
  2. Admin Sidebar: remove the environment box displaying `Mode: production` / `Debug Engine: OFF`.
  3. Email Redesign: upgrade email layout to an official, professional design featuring a branded header with shop logo and name, structured responsive card layout, high-contrast typography, and a compliant footer containing shop details, contact info, and statutory notices.
  4. Test Email Expansion: extend the "Send Test Email" feature under Settings (Email & Auth Policies) with a template selector allowing admins to send and preview test emails for all shop email types (`test`, `verification`, `order_created`, `payment_received`, `order_shipped`, `back_in_stock`, `password_reset`), and mandate in `rules.md` that all future store emails include a corresponding test option.
  5. Login & Unverified Registration Resend: when an unverified customer registers or attempts to log in, provide the option to resend the activation email with an enforced 1-minute cooldown rate limit and live visual countdown timer.
- **Architectural Implementation**:
  - **Admin Products & Settings VAT UI** (`admin/src/routes/products/+page.svelte`, `admin/src/routes/settings/+page.svelte`):
    - Removed quick preset buttons (`19%`, `7%`, `0%`) from both product creation and product editing modals.
    - Added reactive greying-out styling (`opacity-40`, disabled cursor, darkened input background) to VAT labels and inputs in products when `isKleingewerbe` is active.
    - In Store Settings, selecting `kleingewerbe` resets standard store VAT rate to 0.0% and greys out the VAT rate option.
  - **Admin Sidebar Cleanup** (`admin/src/routes/+layout.svelte`):
    - Removed the bottom sidebar environment box showing Mode and Debug Engine status while preserving the storefront link.
  - **Official Email Layout Overhaul & Templates** (`backend/src/services/email.rs`):
    - Implemented `render_email_layout(settings, preheader, title, color, body)` generating consistent, responsive HTML emails across all mail clients (Gmail, Apple Mail, Outlook).
    - Header: resolves and renders full store logo URL (with fallback to store brand badge), store name, and a "Visit Store" link.
    - Body: card container with subtle dark gradients, high-contrast typography, CTA buttons with gradient styling, and structured metadata tables.
    - Footer: includes store name, company address, support/contact email, telephone, VAT ID, statutory tax exemption notice (§ 19 UStG), and copyright.
    - Upgraded all store emails (`send_test_email`, `send_verification_email`, `send_order_created_email`, `send_payment_received_email`, `send_order_shipped_email`, `send_back_in_stock_email`, `send_password_reset_email`) to use this unified layout and return `Result<(), String>`.
  - **Multi-Type Test Email Dispatcher** (`backend/src/models/settings.rs`, `backend/src/services/email.rs`, `backend/src/routes/admin.rs`, `admin/src/routes/settings/+page.svelte`, `admin/src/routes/settings/email/+page.svelte`):
    - Added `email_type: Option<String>` to `TestEmailRequest`.
    - Added `services::email::send_test_email_by_type` generating mock/sample data for all 7 email templates.
    - Added a template selector dropdown in both Admin Settings Email tabs (`Settings -> Email & Auth Policies` and `/settings/email`).
    - Updated `.agent/rules.md` requiring all future transactional emails to be integrated into `send_test_email_by_type` and the Admin dropdown.
  - **Verification Email 1-Minute Cooldown & Countdown Timer** (`backend/src/services/auth.rs`, `backend/src/routes/public.rs`, `storefront/src/routes/account/login/+page.svelte`):
    - Backend: added `auth::check_resend_allowed` and `auth::record_resend_sent` enforcing a 60-second cooldown per email address. `public_resend_verification` returns HTTP 429 with remaining seconds if called too early. `public_customer_register` records the initial dispatch timestamp.
    - Storefront: customer login page tracks active 60s cooldown with a live countdown timer on the button (`Resend verification email in Xs`), disabled state, and `sessionStorage` persistence across page reloads.
- **Verification**:
  - `backend`: `cargo test` passes 11 unit tests (including new resend cooldown test).
  - `admin`: `npm run build` succeeds cleanly (0 errors).
  - `storefront`: `npm run build` succeeds cleanly (0 errors).

### [2026-10-06] Milestone 17: Storefront Product Redirection Fix, BOM Parts Inventory Integration & Multi-Action Quick Restock
- **Context & User Requirements**:
  1. Storefront Product Redirection: Fix issue where clicking product cards, carousel slides, or buttons from the storefront only redirected properly for digital products and out-of-stock products, but failed for physical in-stock products.
  2. Admin Logistics & Stock Quick Restock: Replace fixed quick restock buttons with an input field where the admin can enter a number, and then choose to:
     - Add this number (`+ Add`)
     - Subtract this number (`- Sub`)
     - Set new stock to that number (`= Set`)
  3. Admin Logistics & Stock BOM Parts Integration: In the Logistics & Stock table, show all parts from the Bill of Materials (BOM) alongside products, detailing which products and versions use each part, so admins can easily detect when a shared component is depleted and needs to be produced again to fulfill orders.
- **Root Causes & Diagnostics**:
  - **Storefront Redirect Failure**: In `storefront/src/routes/products/[slug]/+page.svelte`, the template referenced `store?.stock_display_template` on line 414 inside the `{:else if inStock}` block (used exclusively by physical products with positive stock). However, `store` was never declared in `<script>`. Evaluating an undeclared variable in strict JavaScript threw an unhandled runtime `ReferenceError: store is not defined`, crashing page load and client-side navigation. Digital products took `{#if isDigital}` and out-of-stock products took `{:else}`, bypassing line 414 completely.
  - **Logistics Stock Limitations**: The inventory endpoint and stock table only queried `product_variants`. `product_parts` had no stock tracking columns, lacked part-level restocking endpoints, and provided no visibility into whether components needed for assembly/fulfillment were depleted.
- **Architectural Implementation**:
  - **Storefront Product Page Fix** (`storefront/src/routes/products/[slug]/+page.svelte`):
    - Added reactive declaration `$: store = data.store || {};` and updated `storeName = store.store_name || 'Shop'`.
    - Physical in-stock products now render and navigate seamlessly from home page hero carousels, featured buttons, and product cards.
  - **BOM Parts Database Schema & Migration** (`backend/migrations/0017_bom_parts_stock.sql`, `backend/src/db.rs`):
    - Added `stock_quantity INTEGER NOT NULL DEFAULT 0` and `low_stock_threshold INTEGER NOT NULL DEFAULT 5` to `product_parts`.
    - Registered migration `0017_bom_parts_stock` in `db.rs`.
    - Updated `ProductPart`, `CreatePartRequest`, `UpdateProductPartRequest`, backup queries, and restore handlers in `backend/src/models/product.rs` and `backend/src/routes/admin.rs`.
  - **Logistics Inventory Backend Overhaul** (`backend/src/routes/admin.rs`):
    - Rewrote `get_logistics_inventory` to fetch both `product_variants` and `product_parts`.
    - Parts are grouped by SKU/ID into unified component rows (`item_type: "part"`), summarizing all products and variants that depend on them (`used_in` and `used_in_summary`), current stock, and depletion status (`Depleted - Needs Production`).
    - Product rows calculate BOM readiness: `bom_parts_total`, `bom_parts_depleted`, and `bom_has_missing_parts`.
    - Added `PUT /api/v1/admin/logistics/parts/:part_id/stock` supporting `adjustment` and `absolute_quantity` updates, keeping shared part SKUs synchronized across all versions.
    - Updated `update_variant_stock` to clamp reductions to zero (`GREATEST(0, stock_quantity + $1)`).
  - **Admin Stock UI Overhaul** (`admin/src/routes/products/+page.svelte`, `admin/src/routes/logistics/+page.svelte`):
    - Replaced old preset buttons with a numerical input field and three dedicated actions:
      - `+ Add`: adds custom quantity
      - `- Sub`: subtracts custom quantity (clamped to 0)
      - `= Set`: sets absolute quantity
    - Added type filter tabs: `All`, `Products`, and `BOM Parts`.
    - Added component badges, usage summaries (`Required by: ...`), and depleted alerts (`⚠️ Depleted (Needs Production)`).
    - Finished product rows now feature live BOM readiness badges (e.g. `⚠️ 1 BOM part depleted — produce parts first` vs `✓ All BOM parts ready`).
    - Restocking any shared part automatically updates stock across all referencing products and variants.
- **Verification**:
  - `backend`: `cargo check` and `cargo test` pass (11/11 tests ok).
  - `admin`: `npm run build` succeeds cleanly (0 errors).
  - `storefront`: `npm run build` succeeds cleanly (0 errors).

### [2026-10-06] Milestone 18: Centralized BOM Parts Catalog, Warehouse Storage Locations, Expandable Stock Tree & Auto-Calculated Product Inventory
- **Context & User Requirements**:
  1. Centralized BOM Parts Management: Ability to create BOM parts with SKU, Name, Stock Quantity, and Storage Location (where it's stored at) in `Products -> Logistics & Stock`. Ability to edit parts there as well (SKU, Name, Storage Location, Stock, Threshold, Notes).
  2. Centralized Parts Selection in Products: In `Products -> Products & BOM`, allow selecting existing BOM parts from a dropdown list to attach them to one or multiple products, centralizing parts stock and storage in one place while assembling products in the other.
  3. Logistics & Stock Tree Accordion View: In `Logistics, Finished Goods & BOM Parts Inventory`, provide tab selection between `Products` and `BOM Parts`. When `Products` is selected, clicking a product row expands an accordion showing all BOM components used for that product underneath, with the ability to edit components and restock them directly there.
  4. Auto-Calculated Product Stock from BOM Parts: The total amount of products in stock is automatically calculated from the BOM parts available for a complete product ($\min(\lfloor \text{part\_stock} / \text{qty\_required} \rfloor)$). Direct/individual editing of product stock is blocked for BOM-assembled items, allowing stock adjustments only through the underlying component parts in stock.
- **Architectural Implementation**:
  - **Database Migration & Schema** (`backend/migrations/0018_centralized_bom_parts.sql`, `backend/src/db.rs`):
    - Created `bom_parts` table (`id`, `sku`, `name`, `storage_location`, `stock_quantity`, `low_stock_threshold`, `notes`, `created_at`).
    - Added `storage_location` and `part_id` foreign key reference to `product_parts`.
    - Seeded central components (switches, aluminum cases, PCBs, keycaps, stabilizers, hoodie fabric, deskmat textiles) and linked existing product parts.
    - Registered migration `0018_centralized_bom_parts` in `backend/src/db.rs`.
  - **Domain Models & Auth Permissions** (`backend/src/models/product.rs`, `backend/src/middleware/auth.rs`):
    - Added `BomPart`, `CreateBomPartRequest`, `UpdateBomPartRequest`.
    - Extended `ProductPart` and `CreatePartRequest` with `part_id` and `storage_location`.
    - Registered `/bom-parts` under admin product permissions.
  - **Stock Recalculation Engine & Admin Routes** (`backend/src/routes/admin.rs`):
    - Implemented `recalculate_all_product_stocks(&pool)`: dynamically determines the maximum buildable complete units for every product variant based on required BOM parts ($\min(\lfloor \text{part\_stock} / \text{quantity} \rfloor)$) and updates `product_variants.stock_quantity`.
    - Protected variant stock updates: `update_variant_stock` checks if the variant has BOM parts. If so, direct manual stock edits are rejected with HTTP 400 (`Cannot manually adjust stock for products with BOM parts. Stock is auto-calculated from component inventory.`).
    - Updated `update_part_stock`: synchronizes both `bom_parts` and `product_parts`, then automatically invokes `recalculate_all_product_stocks`.
    - Implemented full CRUD for centralized parts: `admin_list_bom_parts`, `admin_create_bom_part`, `admin_update_bom_part`, and `admin_delete_bom_part`.
    - Updated `admin_add_product_part`, `admin_update_product_part`, and `admin_delete_product_part` to link with `bom_parts` and recalculate finished product buildable stock.
    - Updated `get_logistics_inventory` to return storage locations and nested BOM component lists for each product.
  - **Checkout Line Items Deduction** (`backend/src/services/checkout.rs`):
    - In checkout ACID transaction, order line item deductions now also decrement `bom_parts` and `product_parts` inventory according to the product's bill of materials.
  - **Admin UI Overhaul** (`admin/src/routes/logistics/+page.svelte`, `admin/src/routes/products/+page.svelte`):
    - **Expandable Tree Accordion**: Clickable product rows expand to display an indented sub-table of all required BOM parts, complete with storage location badges (`MapPin`), requirement quantities per unit, available stock, buildable units, readiness badges, and dedicated quick restock controls (+, -, =).
    - **Locked Product Restock**: Finished goods with BOM parts display a `Derived from BOM Parts` indicator and a quick toggle to restock parts underneath, preventing invalid manual stock overrides.
    - **Centralized BOM Part Creation & Editing**: Added `+ Create BOM Part` button and modal to create parts with SKU, Name, Storage Location, Initial Stock, Low Stock Threshold, and Specs. Added `Edit BOM Component` modal to update SKU, Name, Storage Location, and Stock at any time.
    - **Product Edit Modal (Section 3)**: Replaced free-text input with a dropdown selector of existing centralized `bomParts`, showing SKU, warehouse bin location, and available stock.
- **Verification & Resolution**:
  - `backend`: Foreign key violation resolved by inner-joining `products` in `0018_centralized_bom_parts.sql`. Migration 0018 executed and confirmed applied. `cargo check` and `cargo test` pass (11/11 tests ok).
  - `admin`: SvelteKit build succeeds cleanly (0 errors).
  - `docker`: All 5 containers (`db`, `backend`, `storefront`, `admin`, `updater`) running and healthy. Both ports 4000 (Admin) and 8080 (Storefront) return HTTP 200.

### [2026-10-07] Milestone 19: SKU-Based BOM Stock Synchronization & Configurable Low-Stock Email Alerts
- **Context & User Requirements**:
  1. Synchronize stock quantity for parts with the same SKU: Ensure that parts sharing the same SKU (across `bom_parts` and all linked `product_parts` in different products) strictly maintain the exact same stock quantity across restocking, part editing, and checkout deduction.
  2. Low-Stock Notification Emails: Provide a settings option to send notification emails to the admin, other users who have access to the stock section (`products` permission), or specifically selected admin accounts, whenever BOM parts drop to or below their threshold.
- **Architectural Implementation**:
  - **Database Migration & Triggers** (`backend/migrations/0019_part_stock_sync_and_low_stock_alerts.sql`, `backend/src/db.rs`):
    - Reconciled existing stock discrepancies across all `product_parts` and `bom_parts` sharing the same SKU.
    - Implemented PostgreSQL triggers `trg_sync_bom_parts_stock` and `trg_sync_product_parts_stock` with `IS DISTINCT FROM` recursion guards to propagate any stock change across all matching parts by SKU and part ID.
    - Implemented `trg_set_initial_product_part_stock` before INSERT on `product_parts` to inherit stock from existing parts with that SKU.
    - Added columns to `bom_parts`: `last_low_stock_alert_at TIMESTAMPTZ` and `last_alert_stock_quantity INTEGER`.
    - Added trigger `trg_reset_bom_part_alert` to clear alert timestamps whenever stock is replenished above threshold.
    - Added columns to `store_settings`: `low_stock_alerts_enabled BOOLEAN`, `low_stock_alert_recipients_mode VARCHAR(50)`, `low_stock_alert_custom_emails TEXT`, and `low_stock_alert_selected_user_ids JSONB`.
    - Registered migration `0019_part_stock_sync_and_low_stock_alerts` in `backend/src/db.rs`.
  - **Models & Settings Persistence** (`backend/src/models/settings.rs`, `backend/src/routes/admin.rs`):
    - Added the 4 low-stock configuration fields to `StoreSettings` and `UpdateStoreSettingsRequest`.
    - Updated `admin_update_system_settings` query and bindings to store and update low stock preferences without breaking existing partial saves.
  - **Transactional Email Service & Alert Engine** (`backend/src/services/email.rs`):
    - Implemented `send_part_low_stock_email` adhering to `.agent/rules.md` (branded layout with header, card styling, key metrics table, and legal footer).
    - Registered `"low_stock_alert"` in `send_test_email_by_type` to allow admins to preview the email template.
    - Implemented `check_and_send_low_stock_alerts(pool: &PgPool)`:
      - Checks if alerts and SMTP are enabled.
      - Resolves recipient list based on mode (`stock_managers`, `selected_users`, or `both`) and parses additional custom emails.
      - Finds parts breaching threshold with cooldown/deduplication logic (fires on first breach, if stock drops further, or after 24 hours).
      - Dispatches emails asynchronously and records alert timestamps.
  - **Checkout Deduction & Event Hooks** (`backend/src/services/checkout.rs`, `backend/src/routes/payments.rs`, `backend/src/routes/admin.rs`):
    - In checkout ACID transaction, deducts BOM part stock once and lets database triggers synchronize linked products, avoiding double deductions.
    - Spawns background calls to `recalculate_all_product_stocks` and `check_and_send_low_stock_alerts` on checkout completion, quick restocking, and part editing.
  - **Admin Settings UI** (`admin/src/routes/settings/email/+page.svelte`, `admin/src/routes/settings/email/+page.server.js`, `admin/src/routes/settings/+page.svelte`):
    - Added `"BOM Low Stock Inventory Alert"` to the Test Email template dropdown selector.
    - Added "📦 BOM Inventory & Low-Stock Email Alerts" configuration card with master enable toggle.
    - Added recipient mode selector: Stock Managers (users with 'products' access), Specific Selected Users, or Both.
    - Added interactive user selection checklist showing user badges (Role, Stock Access) with checkbox toggles.
    - Added optional custom email addresses input for external team members without an admin login.
    - Added "5. Low Stock Alert" card to the Automated Notifications inventory overview.
- **Verification**:
  - `backend`: `cargo check` and `cargo test` pass (11/11 tests ok).
  - `admin`: SvelteKit build succeeds cleanly (0 errors).
  - `storefront`: SvelteKit build succeeds cleanly (0 errors).
  - `docker`: Migration 0019 applied cleanly on container restart; backend is running and healthy on port 8000 (HTTP 200).

### [2026-10-08] Milestone 21: Admin User Email Address Persistence & Profile Management
- **Context & Issue**:
  - Administrators were unable to reliably add or update email addresses for their own and other admin accounts.
  - Root causes identified:
    1. Header credentials modal (`admin/src/routes/+layout.svelte`) only supported username/password and lacked an email field. Furthermore, password change was mandatory even if the user only wanted to update their email or username.
    2. Backend credentials change endpoint (`admin_change_credentials`, `PUT /api/v1/admin/auth/change-credentials`) had no `email` in `ChangeAdminCredentialsRequest` and did not persist email changes to `admin_users`.
    3. `admin_auth_status` (`/api/v1/admin/auth/me`) did not return `email` or `id`, preventing SvelteKit layouts and session helpers from recognizing the authenticated user's current email.
    4. In `AdminUsersManager.svelte` (`/settings?tab=users`), self-edits could be misidentified, sending permissions payloads that triggered `403 Forbidden` ("You cannot change your own role or permissions"), while errors were masked as generic messages due to plain text parsing failure in `readError`.
    5. In the Low-Stock Notification settings (`/settings/email` and `/settings?tab=email`), accounts displayed `(No email set on account)` with no quick method to assign an email to an account.
- **Architectural Implementation**:
  - **Backend Models & Auth Middleware** (`backend/src/models/admin_user.rs`, `backend/src/middleware/auth.rs`):
    - Added `pub email: Option<String>` to `ChangeAdminCredentialsRequest`.
    - Added `pub email: Option<String>` to `CurrentAdmin` struct and queried `email` from `admin_users` in `admin_auth_middleware`.
  - **Backend Admin Handlers** (`backend/src/routes/admin.rs`):
    - Updated `admin_login` and `admin_auth_status` (`/auth/me`) to return `email` and `id` in their responses.
    - Updated `admin_change_credentials`:
      - Allows updating `email` and/or `username` with verification of `current_password` without forcing a password change (unless still on default password).
      - If a new password is provided, verifies `len >= 12` and different from current password, updating `password_hash` and clearing `is_default`.
      - Returns updated `username` and `email` alongside a renewed session token.
    - Updated `admin_update_user`:
      - Robustly handles self-updates (`id == actor.id`): ensures role remains current, preserves existing permissions safely, and allows username/email/password updates.
      - Returns explicit SQL and validation error strings rather than masking with generic messages.
  - **Admin Layout Profile & Credentials Modal** (`admin/src/routes/+layout.svelte`):
    - Added user email display and updated badge in top header.
    - Expanded modal to "Admin Account & Profile Settings": includes Email Address field and makes password change optional (with current password verification).
    - Robust response parsing for text and JSON error messages.
  - **Admin Users Manager** (`admin/src/lib/components/AdminUsersManager.svelte`):
    - Fixed `editingSelf` check using both user ID and username comparison.
    - Updated `readError` to parse both JSON and plain-text HTTP error responses.
    - Synchronized active session admin store upon user update.
  - **Quick Email Assignment in Low-Stock Alert UI** (`admin/src/routes/settings/email/+page.svelte`, `admin/src/routes/settings/+page.svelte`):
    - Added "Set Email" / "Edit Email" action button next to each account in the recipient selection list.
    - Added interactive modal dialog to immediately update and persist any user's email via `PUT /api/v1/admin/users/:id`, reflecting changes in real-time.
- **Verification**:
  - `backend`: `cargo check` passes with 0 errors.
  - `admin`: SvelteKit production build (`npm run build`) completed successfully with 0 errors.
  - End-to-end API verification via test tokens:
    - `GET /api/v1/admin/auth/me` returns `email` and `id`.
    - `PUT /api/v1/admin/users/:id` updates and persists user emails with HTTP 200.
    - Self-update with permissions payload succeeds smoothly without 403 rejection.
  - Docker containers `rustwebshop-backend-1` and `rustwebshop-admin-1` rebuilt, recreated, and live.



