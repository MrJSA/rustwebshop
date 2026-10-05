use crate::models::{CheckoutQuote, CheckoutRequest, CheckoutResponse, DigitalDownloadItem};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{PgConnection, PgPool, Row};
use uuid::Uuid;

pub struct CheckoutService;

struct PricedLine {
    product_id: Uuid,
    variant_id: Uuid,
    product_title: String,
    variant_title: String,
    sku: String,
    unit_price_cents: i32,
    quantity: i32,
    total_price_cents: i32,
    is_digital: bool,
    download_url: Option<String>,
}

struct PricedOrder {
    lines: Vec<PricedLine>,
    digital_downloads: Vec<DigitalDownloadItem>,
    items_subtotal_cents: i32,
    discount_cents: i32,
    subtotal_cents: i32,
    shipping_cost_cents: i32,
    tax_cents: i32,
    total_cents: i32,
    is_digital_only: bool,
    coupon: Option<(Uuid, String)>,
    /// Physical order whose destination zone has rates but none was selected
    missing_shipping_rate: bool,
}

/// A payment that has been verified with the provider and may be turned into an order.
pub struct VerifiedPayment {
    pub provider: String,
    /// Refundable provider reference (Stripe PaymentIntent id, PayPal capture id).
    pub reference: String,
    pub amount_cents: i32,
    /// 'paid' or 'pending' (e.g. PayPal capture under review)
    pub status: String,
}

pub enum FinalizeError {
    /// The order could not be created although the payment went through — the caller must refund.
    OrderRejected(anyhow::Error),
    /// Unknown reference or a checkout that already failed — nothing to do.
    Invalid(anyhow::Error),
    /// Transient failure (database) — safe to retry, no refund.
    Internal(anyhow::Error),
}

impl std::fmt::Display for FinalizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinalizeError::OrderRejected(e) => write!(f, "{}", e),
            FinalizeError::Invalid(e) => write!(f, "{}", e),
            FinalizeError::Internal(e) => write!(f, "{}", e),
        }
    }
}

pub struct FinalizeOutcome {
    pub response: CheckoutResponse,
    pub newly_created: bool,
}

impl CheckoutService {
    /// Enforces the store's "registered customers only" / "verified email" checkout policy.
    pub async fn enforce_checkout_policy(pool: &PgPool, customer_email: &str) -> Result<()> {
        let Ok(settings_row) = sqlx::query("SELECT require_registered_checkout, require_email_verification FROM store_settings WHERE id = 1")
            .fetch_one(pool)
            .await
        else {
            return Ok(());
        };
        let req_reg: bool = settings_row.get("require_registered_checkout");
        let req_verify: bool = settings_row.get("require_email_verification");
        if !req_reg {
            return Ok(());
        }
        let cust = sqlx::query("SELECT is_verified FROM customers WHERE email = $1")
            .bind(customer_email)
            .fetch_optional(pool)
            .await?;
        match cust {
            None => Err(anyhow!(
                "Purchases are restricted to registered customers. Please log in or create an account to complete checkout."
            )),
            Some(c) if req_verify && !c.get::<bool, _>("is_verified") => Err(anyhow!(
                "Your account email has not been verified yet. Please check your inbox and verify your email before placing an order."
            )),
            _ => Ok(()),
        }
    }

    /// Single source of truth for pricing. With `lock_rows` the variant and coupon rows are locked
    /// (`SELECT ... FOR UPDATE`) so the caller can safely decrement stock / increment coupon usage.
    async fn price_order(conn: &mut PgConnection, req: &CheckoutRequest, lock_rows: bool) -> Result<PricedOrder> {
        if req.items.is_empty() {
            return Err(anyhow!("Cart cannot be empty for checkout"));
        }
        let (store_tax_rate, tax_mode): (f64, String) = sqlx::query_as(
            "SELECT tax_rate_percent, COALESCE(tax_mode, 'kleingewerbe') FROM store_settings WHERE id = 1",
        )
        .fetch_one(&mut *conn)
        .await
        .unwrap_or((19.0, "kleingewerbe".to_string()));

        let lock_clause = if lock_rows { "FOR UPDATE OF pv" } else { "" };
        let variant_sql = format!(
            r#"
            SELECT
                pv.id, pv.product_id, pv.sku, pv.title as variant_title,
                pv.price_override_cents, pv.stock_quantity,
                p.title as product_title, p.base_price_cents, p.product_type,
                p.digital_download_url, p.tax_rate_percent, p.is_active
            FROM product_variants pv
            JOIN products p ON p.id = pv.product_id
            WHERE pv.id = $1
            {}
            "#,
            lock_clause
        );

        let mut lines = Vec::new();
        let mut digital_downloads = Vec::new();
        let mut items_subtotal_cents = 0;
        let mut vat_before_discount = 0.0_f64;

        for item in &req.items {
            if item.quantity <= 0 {
                return Err(anyhow!("Invalid quantity for item"));
            }

            let row = sqlx::query(&variant_sql)
                .bind(item.variant_id)
                .fetch_optional(&mut *conn)
                .await?
                .ok_or_else(|| anyhow!("A product in your cart is no longer available"))?;

            let product_id: Uuid = row.get("product_id");
            let sku: String = row.get("sku");
            let variant_title: String = row.get("variant_title");
            let stock_quantity: i32 = row.get("stock_quantity");
            let product_title: String = row.get("product_title");
            let is_active: bool = row.get("is_active");
            let is_digital = row.get::<String, _>("product_type") == "digital";
            let item_tax_rate = row.get::<Option<f64>, _>("tax_rate_percent").unwrap_or(store_tax_rate);

            if !is_active {
                return Err(anyhow!("'{}' is no longer available", product_title));
            }
            if !is_digital && stock_quantity < item.quantity {
                return Err(anyhow!(
                    "Insufficient stock for '{}' (SKU: {}). Available: {}, Requested: {}",
                    variant_title, sku, stock_quantity, item.quantity
                ));
            }

            let unit_price = row
                .get::<Option<i32>, _>("price_override_cents")
                .unwrap_or_else(|| row.get::<i32, _>("base_price_cents"));
            let line_total = unit_price * item.quantity;
            items_subtotal_cents += line_total;

            let rate = item_tax_rate / 100.0;
            match tax_mode.as_str() {
                "included" if rate > 0.0 => vat_before_discount += line_total as f64 - line_total as f64 / (1.0 + rate),
                "excluded" => vat_before_discount += line_total as f64 * rate,
                _ => {}
            }

            let download_url: Option<String> = if is_digital { row.get("digital_download_url") } else { None };
            if is_digital {
                if let Some(ref url_str) = download_url {
                    if let Ok(files) = serde_json::from_str::<Vec<serde_json::Value>>(url_str) {
                        for f in files {
                            let name = f.get("name").and_then(|v| v.as_str()).unwrap_or("File");
                            let u = f.get("url").and_then(|v| v.as_str()).unwrap_or("");
                            if !u.is_empty() {
                                digital_downloads.push(DigitalDownloadItem {
                                    title: format!("{} ({})", product_title, name),
                                    download_url: u.to_string(),
                                });
                            }
                        }
                    } else if !url_str.trim().is_empty() {
                        digital_downloads.push(DigitalDownloadItem {
                            title: product_title.clone(),
                            download_url: url_str.clone(),
                        });
                    }
                }

                // BOM product_parts can carry additional digital files
                let parts = sqlx::query("SELECT part_name, notes FROM product_parts WHERE product_id = $1 AND part_sku = 'DIGITAL_FILE'")
                    .bind(product_id)
                    .fetch_all(&mut *conn)
                    .await
                    .unwrap_or_default();
                for p in parts {
                    let pname: String = p.get("part_name");
                    if let Some(furl) = p.get::<Option<String>, _>("notes") {
                        if !furl.trim().is_empty() {
                            digital_downloads.push(DigitalDownloadItem {
                                title: format!("{} ({})", product_title, pname),
                                download_url: furl,
                            });
                        }
                    }
                }
            }

            lines.push(PricedLine {
                product_id,
                variant_id: row.get("id"),
                product_title,
                variant_title,
                sku,
                unit_price_cents: unit_price,
                quantity: item.quantity,
                total_price_cents: line_total,
                is_digital,
                download_url,
            });
        }

        let is_digital_only = lines.iter().all(|l| l.is_digital);

        // Shipping: the selected rate must belong to the zone that serves the destination country
        let mut shipping_cost_cents = 0;
        let mut missing_shipping_rate = false;
        if !is_digital_only {
            let country = req.shipping_address.country_code.trim().to_uppercase();
            let country_json = json!([country]).to_string();
            // Zones listing the country explicitly; the default zone only if none does
            const SERVING_ZONES: &str = r#"
                WITH matched AS (SELECT id FROM shipping_zones WHERE country_codes @> $1::jsonb)
                SELECT id FROM matched
                UNION ALL
                SELECT id FROM shipping_zones WHERE is_default = true AND NOT EXISTS (SELECT 1 FROM matched)
            "#;

            match req.shipping_rate_id {
                Some(rate_id) => {
                    shipping_cost_cents = sqlx::query_scalar(&format!(
                        "SELECT price_cents FROM shipping_rates WHERE id = $2 AND zone_id IN ({})",
                        SERVING_ZONES
                    ))
                    .bind(&country_json)
                    .bind(rate_id)
                    .fetch_optional(&mut *conn)
                    .await?
                    .ok_or_else(|| anyhow!("The selected shipping option is not available for {}", country))?;
                }
                None => {
                    let available: i64 = sqlx::query_scalar(&format!(
                        "SELECT COUNT(*) FROM shipping_rates WHERE zone_id IN ({})",
                        SERVING_ZONES
                    ))
                    .bind(&country_json)
                    .fetch_one(&mut *conn)
                    .await?;
                    missing_shipping_rate = available > 0;
                }
            }
        }

        // Coupon
        let mut coupon = None;
        let mut discount_cents = 0;
        let mut subtotal_cents = items_subtotal_cents;
        if let Some(code) = req.coupon_code.as_deref().map(str::trim).filter(|c| !c.is_empty()) {
            let coupon_sql = format!(
                "SELECT id, code, discount_type, value_cents, min_order_cents, max_uses, used_count, expires_at FROM coupons WHERE UPPER(code) = UPPER($1) AND is_active = true {}",
                if lock_rows { "FOR UPDATE" } else { "" }
            );
            let c = sqlx::query(&coupon_sql)
                .bind(code)
                .fetch_optional(&mut *conn)
                .await?
                .ok_or_else(|| anyhow!("Coupon '{}' is invalid or inactive", code))?;

            if let Some(exp) = c.get::<Option<DateTime<Utc>>, _>("expires_at") {
                if exp < Utc::now() {
                    return Err(anyhow!("Coupon '{}' has expired", code));
                }
            }
            if let Some(max_u) = c.get::<Option<i32>, _>("max_uses") {
                if c.get::<i32, _>("used_count") >= max_u {
                    return Err(anyhow!("Coupon '{}' usage limit has been reached", code));
                }
            }
            let min_order_cents: i32 = c.get("min_order_cents");
            if items_subtotal_cents < min_order_cents {
                return Err(anyhow!(
                    "Order subtotal of {:.2} € is below the minimum {:.2} € required for coupon '{}'",
                    items_subtotal_cents as f64 / 100.0,
                    min_order_cents as f64 / 100.0,
                    code
                ));
            }

            let value_cents: i32 = c.get("value_cents");
            match c.get::<String, _>("discount_type").as_str() {
                "free_shipping" => {
                    discount_cents = shipping_cost_cents;
                    shipping_cost_cents = 0;
                }
                "fixed_amount" => {
                    discount_cents = value_cents.clamp(0, items_subtotal_cents);
                    subtotal_cents = items_subtotal_cents - discount_cents;
                }
                "percentage" => {
                    let pct = value_cents.clamp(0, 100);
                    discount_cents = ((items_subtotal_cents as f64) * (pct as f64 / 100.0)).round() as i32;
                    subtotal_cents = (items_subtotal_cents - discount_cents).max(0);
                }
                _ => {}
            }
            coupon = Some((c.get::<Uuid, _>("id"), c.get::<String, _>("code")));
        }

        // VAT is reduced pro rata by product discounts
        let discount_factor = if items_subtotal_cents > 0 {
            subtotal_cents as f64 / items_subtotal_cents as f64
        } else {
            0.0
        };
        let (tax_cents, total_cents) = match tax_mode.as_str() {
            "included" => ((vat_before_discount * discount_factor).round() as i32, subtotal_cents + shipping_cost_cents),
            "excluded" => {
                let vat = (vat_before_discount * discount_factor).round() as i32;
                (vat, subtotal_cents + shipping_cost_cents + vat)
            }
            _ => (0, subtotal_cents + shipping_cost_cents),
        };

        Ok(PricedOrder {
            lines,
            digital_downloads,
            items_subtotal_cents,
            discount_cents,
            subtotal_cents,
            shipping_cost_cents,
            tax_cents,
            total_cents,
            is_digital_only,
            coupon,
            missing_shipping_rate,
        })
    }

    /// Customer / address checks required before money is taken or an order is written.
    fn validate_for_order(req: &CheckoutRequest, priced: &PricedOrder) -> Result<()> {
        if req.customer_name.trim().is_empty() {
            return Err(anyhow!("Please enter your full name"));
        }
        let email = req.customer_email.trim();
        if email.len() < 3 || !email.contains('@') || email.contains(char::is_whitespace) {
            return Err(anyhow!("Please enter a valid email address"));
        }
        if !priced.is_digital_only {
            let addr = &req.shipping_address;
            if addr.street_address.trim().is_empty() || addr.city.trim().is_empty() || addr.postal_code.trim().is_empty() {
                return Err(anyhow!("Please enter a complete shipping address"));
            }
            if priced.missing_shipping_rate {
                return Err(anyhow!("Please select a shipping option"));
            }
        }
        Ok(())
    }

    /// Prices the cart without locking. With `strict` the customer details are validated as well.
    pub async fn quote(pool: &PgPool, req: &CheckoutRequest, strict: bool) -> Result<CheckoutQuote> {
        let mut conn = pool.acquire().await?;
        let priced = Self::price_order(&mut *conn, req, false).await?;
        if strict {
            Self::validate_for_order(req, &priced)?;
        }
        Ok(CheckoutQuote {
            items_subtotal_cents: priced.items_subtotal_cents,
            discount_cents: priced.discount_cents,
            subtotal_cents: priced.subtotal_cents,
            shipping_cost_cents: priced.shipping_cost_cents,
            tax_cents: priced.tax_cents,
            total_cents: priced.total_cents,
            is_digital_only: priced.is_digital_only,
            coupon_code: priced.coupon.map(|(_, code)| code),
            currency: "EUR".to_string(),
        })
    }

    /// Creates the order inside the caller's transaction for a payment that was already verified.
    /// Re-prices with row locks and refuses if the verified amount differs from the order total.
    pub async fn create_order(conn: &mut PgConnection, req: &CheckoutRequest, payment: &VerifiedPayment) -> Result<CheckoutResponse> {
        let priced = Self::price_order(conn, req, true).await?;
        Self::validate_for_order(req, &priced)?;

        if priced.total_cents != payment.amount_cents {
            return Err(anyhow!(
                "Paid amount ({:.2} €) does not match the order total ({:.2} €)",
                payment.amount_cents as f64 / 100.0,
                priced.total_cents as f64 / 100.0
            ));
        }

        for line in priced.lines.iter().filter(|l| !l.is_digital) {
            sqlx::query("UPDATE product_variants SET stock_quantity = stock_quantity - $1, updated_at = NOW() WHERE id = $2")
                .bind(line.quantity)
                .bind(line.variant_id)
                .execute(&mut *conn)
                .await?;
        }

        if let Some((coupon_id, _)) = &priced.coupon {
            sqlx::query("UPDATE coupons SET used_count = used_count + 1, updated_at = NOW() WHERE id = $1")
                .bind(coupon_id)
                .execute(&mut *conn)
                .await?;
        }

        // GoBD-compliant consecutive order number
        let (prefix_enabled, order_prefix, date_enabled): (bool, String, bool) = sqlx::query_as(
            "SELECT COALESCE(order_prefix_enabled, true), COALESCE(order_prefix, 'ORD'), COALESCE(order_date_enabled, true) FROM store_settings WHERE id = 1",
        )
        .fetch_one(&mut *conn)
        .await
        .unwrap_or((true, "ORD".to_string(), true));
        let seq_num: i64 = sqlx::query_scalar("SELECT nextval('order_number_seq')").fetch_one(&mut *conn).await?;

        let mut parts = Vec::new();
        let clean_prefix = order_prefix.trim().to_uppercase();
        if prefix_enabled && !clean_prefix.is_empty() {
            parts.push(clean_prefix);
        }
        if date_enabled {
            parts.push(Utc::now().format("%Y%m%d").to_string());
        }
        parts.push(seq_num.to_string());
        let order_number = parts.join("-");

        let shipping_addr_json = serde_json::to_value(&req.shipping_address)?;
        let billing_addr_json = match req.billing_address {
            Some(ref b) => serde_json::to_value(b)?,
            None => shipping_addr_json.clone(),
        };

        // Digital-only orders auto-complete once paid (no shipping needed)
        let order_status = if priced.is_digital_only && payment.status == "paid" { "completed" } else { "processing" };
        let shipping_rate_id = if priced.is_digital_only { None } else { req.shipping_rate_id };
        let order_id = Uuid::new_v4();
        let access_token = crate::services::auth::random_token(32);

        sqlx::query(
            r#"
            INSERT INTO orders (
                id, order_number, customer_name, customer_email,
                shipping_address, billing_address, shipping_rate_id,
                shipping_cost_cents, subtotal_cents, tax_cents, total_cents,
                payment_provider, payment_status, order_status, coupon_code, discount_cents,
                payment_reference, access_token
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            "#,
        )
        .bind(order_id)
        .bind(&order_number)
        .bind(req.customer_name.trim())
        .bind(req.customer_email.trim())
        .bind(shipping_addr_json)
        .bind(billing_addr_json)
        .bind(shipping_rate_id)
        .bind(priced.shipping_cost_cents)
        .bind(priced.subtotal_cents)
        .bind(priced.tax_cents)
        .bind(priced.total_cents)
        .bind(&payment.provider)
        .bind(&payment.status)
        .bind(order_status)
        .bind(priced.coupon.as_ref().map(|(_, code)| code.clone()))
        .bind(priced.discount_cents)
        .bind(&payment.reference)
        .bind(&access_token)
        .execute(&mut *conn)
        .await?;

        for line in &priced.lines {
            sqlx::query(
                r#"
                INSERT INTO order_items (
                    order_id, product_id, variant_id, product_title,
                    variant_title, sku, unit_price_cents, quantity,
                    total_price_cents, is_digital, download_url
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                "#,
            )
            .bind(order_id)
            .bind(line.product_id)
            .bind(line.variant_id)
            .bind(&line.product_title)
            .bind(&line.variant_title)
            .bind(&line.sku)
            .bind(line.unit_price_cents)
            .bind(line.quantity)
            .bind(line.total_price_cents)
            .bind(line.is_digital)
            .bind(&line.download_url)
            .execute(&mut *conn)
            .await?;
        }

        let paid = payment.status == "paid";
        Ok(CheckoutResponse {
            order_number,
            access_token,
            total_cents: priced.total_cents,
            payment_status: payment.status.clone(),
            order_status: order_status.to_string(),
            payment_redirect_url: None,
            digital_items: if paid { priced.digital_downloads } else { Vec::new() },
        })
    }

    /// Stores the checkout payload server-side before the customer is sent to the payment provider,
    /// so the order can be created from trusted data by the return page or by a webhook.
    pub async fn create_pending(pool: &PgPool, provider: &str, req: &CheckoutRequest, amount_cents: i32) -> Result<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO pending_checkouts (id, provider, checkout_payload, amount_cents) VALUES ($1, $2, $3, $4)")
            .bind(id)
            .bind(provider)
            .bind(serde_json::to_value(req)?)
            .bind(amount_cents)
            .execute(pool)
            .await?;
        Ok(id)
    }

    pub async fn attach_reference(pool: &PgPool, pending_id: Uuid, reference: &str) -> Result<()> {
        sqlx::query("UPDATE pending_checkouts SET provider_reference = $1, updated_at = NOW() WHERE id = $2")
            .bind(reference)
            .bind(pending_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn pending_exists(pool: &PgPool, provider: &str, reference: &str) -> Result<Option<Uuid>> {
        Ok(sqlx::query_scalar("SELECT id FROM pending_checkouts WHERE provider = $1 AND provider_reference = $2")
            .bind(provider)
            .bind(reference)
            .fetch_optional(pool)
            .await?)
    }

    /// Turns a pending checkout into an order exactly once. Safe to call concurrently from the
    /// customer's return page and the provider webhook: the pending row is locked for the duration.
    pub async fn finalize_pending(pool: &PgPool, provider_reference: &str, payment: VerifiedPayment) -> Result<FinalizeOutcome, FinalizeError> {
        let mut tx = pool.begin().await.map_err(|e| FinalizeError::Internal(e.into()))?;

        let row = sqlx::query(
            "SELECT id, provider, checkout_payload, amount_cents, status, order_number FROM pending_checkouts WHERE provider_reference = $1 FOR UPDATE",
        )
        .bind(provider_reference)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| FinalizeError::Internal(e.into()))?
        .ok_or_else(|| FinalizeError::Invalid(anyhow!("Unknown payment reference")))?;

        let pending_id: Uuid = row.get("id");
        let status: String = row.get("status");

        if row.get::<String, _>("provider") != payment.provider {
            return Err(FinalizeError::Invalid(anyhow!("Payment provider mismatch")));
        }

        if status == "completed" {
            let order_number: String = row.get::<Option<String>, _>("order_number").unwrap_or_default();
            let existing = sqlx::query("SELECT total_cents, payment_status, order_status, COALESCE(access_token, '') AS access_token FROM orders WHERE order_number = $1")
                .bind(&order_number)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| FinalizeError::Internal(e.into()))?;
            let (total_cents, payment_status, order_status, access_token) = match existing {
                Some(o) => (o.get("total_cents"), o.get("payment_status"), o.get("order_status"), o.get("access_token")),
                None => (payment.amount_cents, payment.status.clone(), "processing".to_string(), String::new()),
            };
            return Ok(FinalizeOutcome {
                response: CheckoutResponse {
                    order_number,
                    access_token,
                    total_cents,
                    payment_status,
                    order_status,
                    payment_redirect_url: None,
                    digital_items: Vec::new(),
                },
                newly_created: false,
            });
        }
        if status == "failed" {
            return Err(FinalizeError::Invalid(anyhow!("This checkout could not be completed and its payment was refunded.")));
        }

        let expected: i32 = row.get("amount_cents");
        let result = match serde_json::from_value::<CheckoutRequest>(row.get("checkout_payload")) {
            Err(e) => Err(anyhow!("Stored checkout is unreadable: {}", e)),
            Ok(_) if payment.amount_cents != expected => Err(anyhow!(
                "Paid amount ({:.2} €) does not match the checkout total ({:.2} €)",
                payment.amount_cents as f64 / 100.0,
                expected as f64 / 100.0
            )),
            Ok(req) => Self::create_order(&mut *tx, &req, &payment).await,
        };

        match result {
            Ok(response) => {
                sqlx::query("UPDATE pending_checkouts SET status = 'completed', order_number = $1, updated_at = NOW() WHERE id = $2")
                    .bind(&response.order_number)
                    .bind(pending_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| FinalizeError::Internal(e.into()))?;
                tx.commit().await.map_err(|e| FinalizeError::Internal(e.into()))?;
                Ok(FinalizeOutcome { response, newly_created: true })
            }
            Err(err) => {
                let _ = tx.rollback().await;
                let _ = sqlx::query("UPDATE pending_checkouts SET status = 'failed', failure_reason = $1, updated_at = NOW() WHERE id = $2")
                    .bind(err.to_string())
                    .bind(pending_id)
                    .execute(pool)
                    .await;
                Err(FinalizeError::OrderRejected(err))
            }
        }
    }

    /// A delayed payment (e.g. SEPA Direct Debit) was confirmed: mark the pending order paid and
    /// auto-complete digital-only orders. Returns the order number if an order changed.
    pub async fn mark_payment_succeeded(pool: &PgPool, payment_reference: &str) -> Result<Option<String>> {
        Ok(sqlx::query_scalar(
            r#"
            UPDATE orders o
            SET payment_status = 'paid',
                order_status = CASE
                    WHEN NOT EXISTS (SELECT 1 FROM order_items oi WHERE oi.order_id = o.id AND NOT oi.is_digital) THEN 'completed'
                    ELSE o.order_status
                END,
                updated_at = NOW()
            WHERE o.payment_reference = $1 AND o.payment_status = 'pending'
            RETURNING o.order_number
            "#,
        )
        .bind(payment_reference)
        .fetch_optional(pool)
        .await?)
    }

    /// A delayed payment failed: cancel the pending order, return its stock and coupon usage.
    pub async fn mark_payment_failed(pool: &PgPool, payment_reference: &str) -> Result<Option<String>> {
        let mut tx = pool.begin().await?;
        let order = sqlx::query(
            "SELECT id, order_number, coupon_code FROM orders WHERE payment_reference = $1 AND payment_status = 'pending' FOR UPDATE",
        )
        .bind(payment_reference)
        .fetch_optional(&mut *tx)
        .await?;
        let Some(order) = order else {
            return Ok(None);
        };
        let order_id: Uuid = order.get("id");

        sqlx::query(
            r#"
            UPDATE product_variants pv
            SET stock_quantity = pv.stock_quantity + oi.quantity, updated_at = NOW()
            FROM order_items oi
            WHERE oi.order_id = $1 AND oi.variant_id = pv.id AND NOT oi.is_digital
            "#,
        )
        .bind(order_id)
        .execute(&mut *tx)
        .await?;
        if let Some(code) = order.get::<Option<String>, _>("coupon_code") {
            sqlx::query("UPDATE coupons SET used_count = GREATEST(used_count - 1, 0), updated_at = NOW() WHERE code = $1")
                .bind(code)
                .execute(&mut *tx)
                .await?;
        }
        sqlx::query("UPDATE orders SET payment_status = 'failed', order_status = 'cancelled', updated_at = NOW() WHERE id = $1")
            .bind(order_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(Some(order.get("order_number")))
    }

    /// Orders whose total is 0 € (e.g. 100 % coupon) need no payment provider.
    pub async fn create_free_order(pool: &PgPool, req: &CheckoutRequest) -> Result<CheckoutResponse> {
        let mut tx = pool.begin().await?;
        let payment = VerifiedPayment {
            provider: "free".to_string(),
            reference: format!("free-{}", Uuid::new_v4()),
            amount_cents: 0,
            status: "paid".to_string(),
        };
        let response = Self::create_order(&mut *tx, req, &payment).await?;
        tx.commit().await?;
        Ok(response)
    }
}
