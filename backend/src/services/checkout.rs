use crate::models::{
    CheckoutRequest, CheckoutResponse, DigitalDownloadItem,
};
use crate::services::payment_engine::PaymentEngine;
use anyhow::{anyhow, Result};
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct CheckoutService;

impl CheckoutService {
    pub async fn execute_checkout(
        pool: &PgPool,
        req: CheckoutRequest,
    ) -> Result<CheckoutResponse> {
        if req.items.is_empty() {
            return Err(anyhow!("Cart cannot be empty for checkout"));
        }

        // Start strict ACID database transaction
        let mut tx = pool.begin().await?;

        // 1. Fetch store settings for tax calculation
        let tax_rate: f64 = sqlx::query_scalar("SELECT tax_rate_percent FROM store_settings WHERE id = 1")
            .fetch_one(&mut *tx)
            .await
            .unwrap_or(19.0);

        // 2. Fetch shipping rate if provided
        let mut shipping_cost_cents = 0;
        if let Some(shipping_rate_id) = req.shipping_rate_id {
            if let Ok(rate_row) = sqlx::query("SELECT price_cents FROM shipping_rates WHERE id = $1")
                .bind(shipping_rate_id)
                .fetch_one(&mut *tx)
                .await
            {
                shipping_cost_cents = rate_row.get::<i32, _>("price_cents");
            }
        }

        // 3. Process line items with ROW-LEVEL LOCKING (SELECT ... FOR UPDATE)
        let mut subtotal_cents = 0;
        let mut order_items_to_insert = Vec::new();
        let mut digital_downloads = Vec::new();

        for item in &req.items {
            if item.quantity <= 0 {
                return Err(anyhow!("Invalid quantity for item"));
            }

            // ACQUIRE ROW-LEVEL LOCK ON THE VARIANT RECORD
            let variant_row = sqlx::query(
                r#"
                SELECT 
                    pv.id, pv.product_id, pv.sku, pv.title as variant_title, 
                    pv.price_override_cents, pv.stock_quantity,
                    p.title as product_title, p.base_price_cents, p.product_type,
                    p.digital_download_url
                FROM product_variants pv
                JOIN products p ON p.id = pv.product_id
                WHERE pv.id = $1
                FOR UPDATE OF pv
                "#,
            )
            .bind(item.variant_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| anyhow!("Variant with ID {} not found", item.variant_id))?;

            let variant_id: Uuid = variant_row.get("id");
            let product_id: Uuid = variant_row.get("product_id");
            let sku: String = variant_row.get("sku");
            let variant_title: String = variant_row.get("variant_title");
            let price_override_cents: Option<i32> = variant_row.get("price_override_cents");
            let stock_quantity: i32 = variant_row.get("stock_quantity");
            let product_title: String = variant_row.get("product_title");
            let base_price_cents: i32 = variant_row.get("base_price_cents");
            let product_type: String = variant_row.get("product_type");
            let digital_download_url: Option<String> = variant_row.get("digital_download_url");

            let is_digital = product_type == "digital";

            // If physical, verify available stock
            if !is_digital && stock_quantity < item.quantity {
                return Err(anyhow!(
                    "Insufficient stock for '{}' (SKU: {}). Available: {}, Requested: {}",
                    variant_title,
                    sku,
                    stock_quantity,
                    item.quantity
                ));
            }

            // Determine unit price (variant override or base product price)
            let unit_price = price_override_cents.unwrap_or(base_price_cents);
            let total_item_price = unit_price * item.quantity;
            subtotal_cents += total_item_price;

            // Deduct stock if physical product
            if !is_digital {
                sqlx::query(
                    r#"
                    UPDATE product_variants
                    SET stock_quantity = stock_quantity - $1,
                        updated_at = NOW()
                    WHERE id = $2
                    "#,
                )
                .bind(item.quantity)
                .bind(variant_id)
                .execute(&mut *tx)
                .await?;
            }

            let download_url = if is_digital {
                digital_download_url.clone()
            } else {
                None
            };

            if let Some(ref url) = download_url {
                digital_downloads.push(DigitalDownloadItem {
                    title: product_title.clone(),
                    download_url: url.clone(),
                });
            }

            order_items_to_insert.push((
                product_id,
                variant_id,
                product_title,
                variant_title,
                sku,
                unit_price,
                item.quantity,
                total_item_price,
                is_digital,
                download_url,
            ));
        }

        // 4. Calculate Tax and Total
        let tax_cents = ((subtotal_cents as f64) * (tax_rate / 100.0)).round() as i32;
        let total_cents = subtotal_cents + shipping_cost_cents + tax_cents;

        // 5. Verify & Process Payment
        let payment_row = sqlx::query(
            "SELECT is_enabled, is_sandbox, config_data FROM payment_configs WHERE provider = $1"
        )
        .bind(&req.payment_provider)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow!("Payment provider '{}' is not configured", req.payment_provider))?;

        let is_enabled: bool = payment_row.get("is_enabled");
        let is_sandbox: bool = payment_row.get("is_sandbox");
        let config_data: serde_json::Value = payment_row.get("config_data");

        if !is_enabled {
            return Err(anyhow!("Payment provider '{}' is currently disabled", req.payment_provider));
        }

        let payment_result = PaymentEngine::process_payment(
            &req.payment_provider,
            total_cents,
            "EUR",
            req.payment_token.as_deref(),
            &config_data,
            is_sandbox,
        )
        .await?;

        if !payment_result.success {
            return Err(anyhow!("Payment failed: {}", payment_result.message));
        }

        // 6. Generate Human-Readable Order Number (e.g. ORD-20260923-XXXX)
        let order_number = format!(
            "ORD-{}-{}",
            Utc::now().format("%Y%m%d"),
            Uuid::new_v4().simple().to_string()[..6].to_uppercase()
        );

        let shipping_addr_json = serde_json::to_value(&req.shipping_address)?;
        let billing_addr_json = match req.billing_address {
            Some(ref b) => serde_json::to_value(b)?,
            None => shipping_addr_json.clone(),
        };

        let order_id = Uuid::new_v4();

        // 7. Insert Order Record
        sqlx::query(
            r#"
            INSERT INTO orders (
                id, order_number, customer_name, customer_email,
                shipping_address, billing_address, shipping_rate_id,
                shipping_cost_cents, subtotal_cents, tax_cents, total_cents,
                payment_provider, payment_status, order_status
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'processing'
            )
            "#,
        )
        .bind(order_id)
        .bind(&order_number)
        .bind(&req.customer_name)
        .bind(&req.customer_email)
        .bind(shipping_addr_json)
        .bind(billing_addr_json)
        .bind(req.shipping_rate_id)
        .bind(shipping_cost_cents)
        .bind(subtotal_cents)
        .bind(tax_cents)
        .bind(total_cents)
        .bind(&req.payment_provider)
        .bind(&payment_result.status)
        .execute(&mut *tx)
        .await?;

        // 8. Insert Order Items
        for item in order_items_to_insert {
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
            .bind(item.0)
            .bind(item.1)
            .bind(item.2)
            .bind(item.3)
            .bind(item.4)
            .bind(item.5)
            .bind(item.6)
            .bind(item.7)
            .bind(item.8)
            .bind(item.9)
            .execute(&mut *tx)
            .await?;
        }

        // Commit transaction atomically
        tx.commit().await?;

        Ok(CheckoutResponse {
            order_number,
            total_cents,
            payment_status: payment_result.status,
            order_status: "processing".to_string(),
            payment_redirect_url: payment_result.redirect_url,
            digital_items: digital_downloads,
        })
    }
}
