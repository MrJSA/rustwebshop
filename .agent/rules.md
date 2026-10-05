# Agent Operating Rules & Architecture Standards

## 1. System Mission & Core Constraints
- **Domain**: High-performance, ACID-compliant, containerized E-Commerce platform.
- **Backend Core**: Written exclusively in Rust using Axum, Tokio, and SQLx.
- **Database**: PostgreSQL (relational schema, transactional integrity, JSONB with GIN indexing for variants and attributes).
- **Frontends**: SvelteKit with `@sveltejs/adapter-node` for both Customer Storefront (Host Port `8080` -> Container `3000`) and Admin Dashboard (Host Port `4000` -> Container `4000`).
- **Orchestration**: Docker Compose with 4 isolated services (`db`, `backend`, `storefront`, `admin`) on the `shop_net` bridge network.

---

## 2. Transaction Integrity & Concurrency Rules (CRITICAL)
- **Zero Race Conditions**: Under high concurrent checkout traffic, overselling is strictly forbidden.
- **Row-Level Locking**: Any stock deduction during checkout MUST be performed within a PostgreSQL transaction using `SELECT ... FOR UPDATE` on `product_variants`:
  ```sql
  SELECT stock_quantity FROM product_variants WHERE id = $1 FOR UPDATE;
  ```
- **Atomic Checkout**: Order creation, order item insertion, inventory deduction, and payment record creation must all occur within the same ACID transaction. If payment or stock verification fails, rollback immediately.

---

## 3. Architecture & Separation of Concerns
1. **Network Boundary**:
   - The database (`db`) is reachable only over internal Docker networking (`shop_net:5432`). It is never exposed directly to the public internet or host.
   - Storefront and Admin talk to the Backend API via internal network `http://backend:8000`.
   - Customer Storefront runs on host port `8080`.
   - Admin Dashboard runs on host port `4000` to allow dedicated firewall, VPN, or reverse-proxy protection.
   - **Public proxy hardening (CRITICAL)**: The storefront's `/api/v1/[...path]` proxy must return 404 for any `admin/*` path (including URL-encoded variants) and must strip the `X-Dev-Mode` header and `dev=true` query.
   - The backend port is published on `127.0.0.1` only (local debugging); the backend has no CORS layer because browsers never call it directly.

---

## 2a. Authentication, Authorization & Data Protection (CRITICAL)
- **No bypasses, no backdoors**: There is no development auth bypass (`X-Dev-Mode`, `?dev=true`, `?token=` are gone) and no hard-coded password. Never reintroduce one.
- **Secrets**: The JWT signing key comes from `JWT_SECRET` (≥ 32 chars, not a published value) or is generated and persisted in `server_secrets`. Payment secrets, webhook secrets and the SMTP password are write-only (`#[serde(skip_serializing)]` / masked) and never appear in API responses, exports or logs.
- **Tokens are role-bound**: `services::auth::verify_token(token, role)`; admin tokens (`sub` = admin id, 12 h) and customer tokens (`sub` = email, 30 d) are not interchangeable. The admin middleware reloads the admin from the DB on every request (deleted users lose access immediately).
- **Admin sessions**: The admin app keeps the token only in the httpOnly, `SameSite=Strict` cookie `admin_session`. `hooks.server.js` guards every page/API/upload, `handleFetch` attaches the session to server-side loads, and the proxy strips client-supplied `Authorization`/`Cookie`/`X-Dev-Mode`. Page JavaScript never sees a token.
- **Initial admin**: Created from `ADMIN_INITIAL_PASSWORD` or a random password logged once; `is_default` accounts can only call `auth/status|me|change-credentials` until they set a new password (≥ 12 chars). Accounts still using the formerly published password are rotated at startup.
- **Roles & section permissions**: Access is granted per user and per admin section — `overview` (Overview & Analytics), `products`, `orders` (Orders & Slips), `storefront` (Storefront & Design), `settings` — stored in `admin_users.permissions`. Role defaults: superadmin everything (cannot be restricted); admin all but settings; editor products + orders only. Section mapping lives in `middleware::required_access` (backend, deny-by-default → settings) and `sectionForPath` (admin `hooks.server.js`); keep both in sync when adding pages/APIs. The shared `PUT /settings/system` checks each *changed* field (`STOREFRONT_SETTING_FIELDS` vs. settings). Nobody changes their own role/permissions; only superadmins grant/modify/delete superadmins; the last superadmin cannot be removed or demoted; nobody deletes themselves. Updates and domain changes are superadmin-only.
- **Brute force**: Admin login, customer login and reset emails are limited per account (8 attempts / 15 min); unknown accounts take the same bcrypt time.
- **Customer accounts**: Registration never overwrites an existing account (409). Password reset is a two-step emailed token flow (32-byte random token, only its SHA-256 stored, 1 h, single use). Emails are normalised to lowercase.
- **Guest order privacy**: Order numbers are sequential (GoBD) and therefore guessable — `GET /orders/lookup/:number` requires the order's 64-hex `access_token` (returned at checkout, used on the order-success URL) or the matching email. Download links are only returned for paid orders.
- **Digital goods**: Public catalogue APIs never return `digital_download_url` or `DIGITAL_FILE` BOM rows (`public_product()`); files are only listed for paid orders.
- **SQL**: All user input is bound (`$n` parameters). Never build SQL with `format!` from request data; escape `%`, `_`, `\` for `ILIKE` searches.
- **Uploads**: Extensions are validated (no `html/js/xml/...`); both upload proxies serve files with `Content-Security-Policy: sandbox` and `nosniff`.
- **Security headers**: Storefront and admin set `X-Frame-Options`, `nosniff`, `Referrer-Policy`, `Permissions-Policy` (storefront keeps `payment` allowed for wallets), HSTS on HTTPS; admin responses are `Cache-Control: no-store`.
- **Privacy in the repository**: No personal names, handles, addresses, keys or real credentials in code, seeds, placeholders or docs — use generic values (`Max Mustermann`, `Example Store`, `ORD`). `.env` stays git-ignored.
2. **Product Modeling & Digital Goods**:
   - Products are either `physical` or `digital`.
   - When a product is marked as `digital`, physical stock tracking is deactivated (virtual unlimited inventory `∞`).
   - Digital products support multiple downloadable asset files (PDFs, ZIPs, STL models, firmware) stored as JSON or BOM entries (`part_sku = 'DIGITAL_FILE'`).
   - If an order contains only digital products, physical shipping is waived to `0.00 €` and only the digital product is billed.
3. **Taxation & Small Business Mode (§ 19 UStG)**:
   - Supports 3 tax modes: `kleingewerbe` (German Small Business Regulation), `included` (B2C gross pricing), and `excluded` (B2B net pricing).
   - When `tax_mode == 'kleingewerbe'`, manual VAT percentage inputs on products must be disabled and evaluated as `0.0%` to comply with statutory German tax law.
   - Invoices and Packing Slips must automatically display the statutory exemption notice: *"According to § 19 UStG, no value-added tax is charged (small business regulation)."*
4. **EU & German E-Commerce Legal Compliance**:
   - **Button-Lösung (§ 312j Abs. 3 BGB)**: Final checkout submission button must explicitly state *"Order with Obligation to Pay"* (*Zahlungspflichtig bestellen*).
   - **Digital Goods Withdrawal Waiver (§ 356 Abs. 5 BGB)**: Digital purchases must require affirmative customer consent to waive the 14-day statutory right of withdrawal upon instant download delivery.
   - **GoBD Sequential Order Numbers**: Order numbers must strictly follow an ascending sequence counter (e.g. starting at 10000) with optional store prefix and date code.
   - **Statutory Disclosures**: Impressum (§ 5 DDG), AGB, Widerrufsbelehrung, DSGVO, and Cookie Consent (§ 25 TTDSG) must be accessible across the storefront.
5. **Logistics & Warehousing**:
   - Inventory tracking by SKU for physical products.
   - Configurable stock availability display template with `{stock}` placeholder.
   - Stock thresholds trigger low-stock alerts on the Admin logistics dashboard.
   - Packing slips must render warehouse-ready information (SKU, variant breakdown, quantity, shipping address).
    - Invoices must render legally compliant tax invoice breakdowns (order number, subtotal, tax rate, shipping cost, payment method, customer billing address).
6. **Payment Providers & Validation Rules (CRITICAL)**:
   - **Gateways**: Exactly two real gateways exist in `payment_configs`: `stripe` and `paypal`. All other methods are payment *methods of Stripe*, toggled in `config_data.methods` (keys = `stripe::OPTIONAL_METHODS` + `apple_pay`/`google_pay`; mirrored in `storefront/src/lib/stripeMethods.js` and the admin `PaymentsManager`). They are never separate providers. Never add a provider that approves payments without verifying them with the real provider API.
   - **Checkout UI**: The Payment Element always lists `card` first (default tab); wallets/one-click methods (Apple Pay, Google Pay, Link, Amazon Pay, PayPal via Stripe, Klarna) render as Express Checkout buttons that only appear once the form is complete and the legal checkboxes are ticked. Apple Pay/Google Pay require HTTPS and a domain registered via Admin → Payment Providers.
   - **Delayed methods** (e.g. SEPA Direct Debit): a `processing` PaymentIntent creates the order with `payment_status = pending`; `payment_intent.succeeded` marks it paid (digital-only → completed), `payment_intent.payment_failed` cancels it and returns stock and coupon usage. A paid order is never cancelled by a late event.
   - **PCI Scope**: Card numbers, CVCs and wallet tokens must never reach our servers. Card data is entered only in Stripe Elements (Payment Element) or on Stripe's hosted page; PayPal is authorized in PayPal's own buttons/popup.
   - **Server-Side Amounts**: The amount charged is always computed by the backend (`CheckoutService::quote` / `price_order`). The browser never sends an amount. The storefront displays the server quote (`POST /checkout/quote`).
   - **Pending Checkout → Order (exactly once)**: Before redirecting to/authorizing with a provider, the backend stores the checkout in `pending_checkouts` keyed by the provider reference (PaymentIntent / Checkout Session / PayPal order id). Orders are created only by `CheckoutService::finalize_pending`, which locks the pending row, verifies the paid amount equals the stored amount, and is idempotent (customer return page and webhook may race).
   - **Verification**: Stripe PaymentIntents must be `succeeded` with `amount_received` == expected and currency `eur`; Checkout Sessions must be `payment_status = paid`; PayPal captures must be `COMPLETED` (or `PENDING` → order `payment_status = pending`). Stripe webhooks must pass HMAC-SHA256 signature verification (`Stripe-Signature`, 5-minute tolerance).
   - **Automatic Refunds**: If an order cannot be created after a verified payment (e.g. stock sold out meanwhile, amount mismatch), refund it automatically at the provider and tell the customer. Admin refunds call the provider refund API via `orders.payment_reference`; an order is only marked `refunded` after the provider confirmed the refund.
   - **Provider Visibility**: Storefront checkout MUST ONLY display gateways with `is_enabled == true` and a configured public key (`pk_...` / PayPal client id).
   - **Secrets**: `secret_key` and `webhook_secret` are write-only — the admin API returns only `has_*` flags and masked hints; `config_data` is public (sent to the storefront) and must never contain secrets.
   - **Testing**: Use Stripe test keys (`pk_test_`/`sk_test_`) with Stripe's test cards (`4242 4242 4242 4242` success, `4000 0000 0000 0002` declined, `4000 0000 0000 9995` insufficient funds, `4000 0025 0000 3155` 3-D Secure) and PayPal sandbox credentials. There is no offline mock approval path.
   - **Zero False Approvals**: Never create an order or transition to `order-success` unless the provider confirmed the payment (or the server-computed total is exactly 0 € → `free` order). Otherwise return an explicit HTTP 4xx with descriptive messaging.
   - **Button-Lösung with wallets/PayPal**: Wallet sheets open from our own "Order with Obligation to Pay" button (Payment Element). PayPal buttons are rendered directly beneath the label "Order with Obligation to Pay (… €) via PayPal" and refuse to open until terms are accepted.
7. **Digital Fulfillment & Dynamic Updates**:
   - **Instant Auto-Completion**: If an order contains only digital products, set `order_status = 'completed'` immediately upon successful payment. Do not queue for physical warehouse shipping.
   - **Dynamic Asset Resolution**: When customers download digital products via `/account/downloads` or order details, dynamically query the current `products` table and `product_parts` (`part_sku = 'DIGITAL_FILE'`). If files are updated or revised by the merchant, customers must automatically receive the latest files.
8. **Promo & Discount Code Engine**:
   - Supported discount types: `free_shipping`, `fixed_amount`, `percentage`.
   - Constraints: enforce `is_active`, `min_order_amount_cents`, `max_uses`, and `expires_at`.
   - Atomic usage increment: increment `used_count` within the checkout transaction.
9. **Admin User Governance**:
   - Passwords must be hashed using bcrypt before storage.
   - Protect against privilege escalation and ensure the final remaining `superadmin` user cannot be deleted.
10. **SvelteKit SEO & Performance Standards**:
    - **SEO**: Every public route renders `$lib/components/Seo.svelte` (title, description, canonical, OpenGraph/Twitter, JSON-LD such as `Product`, `BreadcrumbList`, `Organization`, `WebSite`). Canonical and OG URLs are built from the request origin (`$page.url.origin`) — never hard-code a domain; production must set the `ORIGIN` env var to the real shop URL.
    - **Indexing**: Private paths (`/account`, `/checkout`, `/order-success`, `/track`) are `noindex` (set centrally in `+layout.svelte` via `PRIVATE_PATH`) and disallowed in `/robots.txt`; search result pages are `noindex, follow`. `/sitemap.xml` is generated dynamically from products, categories and menu-linked policy pages.
    - **JSON-LD safety**: Always serialize via `jsonLdScript()` (escapes `<`) — never interpolate raw JSON into `{@html}`.
    - **Performance**: Preload critical assets, enforce explicit dimensions on images to prevent CLS, use `loading="lazy"` on below-the-fold content, and leverage server `load` functions (`+page.server.js`) to avoid client waterfall latency.
11. **Shipping Provider Matrix**:
    - Zones configured by country codes and package tiers (`standard`, `express`, `fragile`, `heavy`).
    - Shipping rates automatically determined during customer checkout based on country and basket items.
12. **Environment Toggles**:
    - **Debug Mode**: Displays live sandbox payment helpers, verbose logging, and mock triggers.
    - **Deployment Mode**: `development`, `staging`, `production`, `demo`.

---

## 4. Code Quality & Agent Etiquette
- Maintain clean, idiomatic Rust code with structured error handling (`Result<T, AppError>`).
- Maintain clean TypeScript and Svelte components with reactive stores and SSR loaders.
- Ensure Dockerfiles are multi-stage and optimized for layer caching and minimal image size.
- Log every major architectural decision and rollback in `.agent/backtracking_log.md`.
- **Language policy**: Implement logic in **Rust** wherever possible (backend, updater, future services); the frontends are **Svelte/SvelteKit**, whose Node runtime only renders pages, guards sessions and forwards API calls — no business logic there.
- **Commits**: plain messages without any Claude/AI attribution or Co-Authored-By trailers; author identity `MrJSA <126828137+MrJSA@users.noreply.github.com>` (never the owner's real name).
- **Releases & updates**: `VERSION` (semver) is the installed version; releases are annotated tags `vX.Y.Z` (tag message = release notes). The Rust `updater` service (docker socket, private `updater_net`, token in the `updater_secrets` volume) checks tags, backs up the DB to `backups/`, checks out the tag, rebuilds `backend storefront admin` and rolls the code back if the build fails. It never restarts itself and refuses to overwrite local code changes. Compose has no host bind mounts besides the updater's own (so it can rebuild from inside its container) and a pinned `name: rustwebshop`.
- **Deployment**: `INSTALL.md` is the server guide. Domains via `SHOP_DOMAIN`/`ADMIN_DOMAIN` + `COMPOSE_PROFILES=proxy` (Caddy, automatic HTTPS) or an own reverse proxy with `PUBLISH_ADDR=127.0.0.1`; `SHOP_PUBLIC_URL`/`ADMIN_PUBLIC_URL` must match the public URLs (SvelteKit `ORIGIN`).
- **Email**: `send_email_raw` returns an error when SMTP is off (no fake success); ports 465/587 force TLS/STARTTLS; empty sender address = SMTP login. Always load settings with `SELECT * FROM store_settings` (column lists drift and silently break `query_as`).


