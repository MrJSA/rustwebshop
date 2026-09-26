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
6. **Payment Providers**:
   - Multi-gateway architecture: Stripe, PayPal (primary priority), Apple Pay, Google Pay, Amazon Pay.
   - Each provider has configurable sandbox/production modes and toggle switches managed in the Admin Settings.
7. **Shipping Provider Matrix**:
   - Zones configured by country codes and package tiers (`standard`, `express`, `fragile`, `heavy`).
   - Shipping rates automatically determined during customer checkout based on country and basket items.
8. **Environment Toggles**:
   - **Debug Mode**: Displays live sandbox payment helpers, verbose logging, and mock triggers.
   - **Deployment Mode**: `development`, `staging`, `production`, `demo`.

---

## 4. Code Quality & Agent Etiquette
- Maintain clean, idiomatic Rust code with structured error handling (`Result<T, AppError>`).
- Maintain clean TypeScript and Svelte components with reactive stores and SSR loaders.
- Ensure Dockerfiles are multi-stage and optimized for layer caching and minimal image size.
- Log every major architectural decision and rollback in `.agent/backtracking_log.md`.

