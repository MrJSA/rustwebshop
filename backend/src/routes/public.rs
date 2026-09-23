use crate::models::{
    CartItemInput, CheckoutRequest, EstimateShippingRequest, Product, ProductVariant,
    ProductWithVariants, PublicPaymentProviderInfo, ShippingRate, ShippingZone, StoreSettings,
};
use crate::services::checkout::CheckoutService;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ProductQuery {
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub product_type: Option<String>,
    pub search: Option<String>,
}

pub fn public_router() -> Router<PgPool> {
    Router::new()
        .route("/store/info", get(get_store_info))
        .route("/products", get(list_products))
        .route("/products/:slug", get(get_product_by_slug))
        .route("/cart/validate", post(validate_cart))
        .route("/shipping/rates", get(get_shipping_rates))
        .route("/checkout", post(execute_checkout))
        .route("/orders/lookup/:order_number", get(lookup_order))
}

async fn get_store_info(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let settings = sqlx::query_as::<_, StoreSettings>(
        "SELECT id, store_name, currency, currency_symbol, tax_rate_percent, deployment_mode, debug_mode, support_email, company_address, vat_id, updated_at FROM store_settings WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let payment_providers = sqlx::query_as::<_, PaymentProviderRow>(
        "SELECT provider, display_name, is_enabled, is_sandbox, public_client_id, config_data FROM payment_configs WHERE is_enabled = true"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|r| PublicPaymentProviderInfo {
        provider: r.provider,
        display_name: r.display_name,
        is_enabled: r.is_enabled,
        is_sandbox: r.is_sandbox,
        public_client_id: r.public_client_id,
        config_data: r.config_data,
    })
    .collect::<Vec<_>>();

    Ok(Json(json!({
        "store": settings,
        "payment_providers": payment_providers
    })))
}

#[derive(sqlx::FromRow)]
struct PaymentProviderRow {
    provider: String,
    display_name: String,
    is_enabled: bool,
    is_sandbox: bool,
    public_client_id: String,
    config_data: serde_json::Value,
}

async fn list_products(
    State(pool): State<PgPool>,
    Query(query): Query<ProductQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut sql = "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at FROM products WHERE is_active = true".to_string();

    if let Some(cat) = &query.category {
        sql.push_str(&format!(" AND category = '{}'", cat.replace('\'', "''")));
    }
    if let Some(subcat) = &query.subcategory {
        sql.push_str(&format!(" AND subcategory = '{}'", subcat.replace('\'', "''")));
    }
    if let Some(ptype) = &query.product_type {
        sql.push_str(&format!(" AND product_type = '{}'", ptype.replace('\'', "''")));
    }
    if let Some(s) = &query.search {
        let clean_search = s.replace('\'', "''");
        sql.push_str(&format!(" AND (title ILIKE '%{}%' OR description ILIKE '%{}%')", clean_search, clean_search));
    }
    sql.push_str(" ORDER BY created_at DESC");

    let products = sqlx::query_as::<_, Product>(&sql)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for product in products {
        let variants = sqlx::query_as::<_, ProductVariant>(
            "SELECT id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, created_at, updated_at FROM product_variants WHERE product_id = $1 ORDER BY created_at ASC"
        )
        .bind(product.id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        result.push(ProductWithVariants { product, variants });
    }

    Ok(Json(result))
}

async fn get_product_by_slug(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let product = sqlx::query_as::<_, Product>(
        "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at FROM products WHERE slug = $1 AND is_active = true"
    )
    .bind(slug)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Product not found".to_string()))?;

    let variants = sqlx::query_as::<_, ProductVariant>(
        "SELECT id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, created_at, updated_at FROM product_variants WHERE product_id = $1 ORDER BY created_at ASC"
    )
    .bind(product.id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Ok(Json(ProductWithVariants { product, variants }))
}

async fn validate_cart(
    State(pool): State<PgPool>,
    Json(items): Json<Vec<CartItemInput>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut validated_items = Vec::new();
    let mut total_cents = 0;

    for item in items {
        let variant_row = sqlx::query(
            r#"
            SELECT pv.id, pv.sku, pv.title as variant_title, pv.price_override_cents, pv.stock_quantity,
                   p.title as product_title, p.base_price_cents, p.product_type
            FROM product_variants pv
            JOIN products p ON p.id = pv.product_id
            WHERE pv.id = $1
            "#,
        )
        .bind(item.variant_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if let Some(v) = variant_row {
            let id: Uuid = v.get("id");
            let sku: String = v.get("sku");
            let variant_title: String = v.get("variant_title");
            let price_override_cents: Option<i32> = v.get("price_override_cents");
            let stock_quantity: i32 = v.get("stock_quantity");
            let product_title: String = v.get("product_title");
            let base_price_cents: i32 = v.get("base_price_cents");
            let product_type: String = v.get("product_type");

            let unit_price = price_override_cents.unwrap_or(base_price_cents);
            let item_total = unit_price * item.quantity;
            total_cents += item_total;

            let in_stock = product_type == "digital" || stock_quantity >= item.quantity;

            validated_items.push(json!({
                "variant_id": id,
                "sku": sku,
                "product_title": product_title,
                "variant_title": variant_title,
                "unit_price_cents": unit_price,
                "quantity": item.quantity,
                "item_total_cents": item_total,
                "available_stock": stock_quantity,
                "in_stock": in_stock,
                "is_digital": product_type == "digital"
            }));
        }
    }

    Ok(Json(json!({
        "items": validated_items,
        "subtotal_cents": total_cents
    })))
}

async fn get_shipping_rates(
    State(pool): State<PgPool>,
    Query(query): Query<EstimateShippingRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let country = query.country_code.to_uppercase();

    let zone = sqlx::query_as::<_, ShippingZone>(
        r#"
        SELECT id, zone_name, country_codes, is_default, created_at
        FROM shipping_zones
        WHERE country_codes @> $1::jsonb OR is_default = true
        ORDER BY (country_codes @> $1::jsonb) DESC
        LIMIT 1
        "#,
    )
    .bind(json!([country]).to_string())
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(z) = zone {
        let rates = sqlx::query_as::<_, ShippingRate>(
            "SELECT id, zone_id, name, package_type, min_weight_g, max_weight_g, price_cents, estimated_delivery_days FROM shipping_rates WHERE zone_id = $1 ORDER BY price_cents ASC"
        )
        .bind(z.id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        Ok(Json(json!({
            "zone": z,
            "rates": rates
        })))
    } else {
        Ok(Json(json!({
            "zone": null,
            "rates": []
        })))
    }
}

async fn execute_checkout(
    State(pool): State<PgPool>,
    Json(payload): Json<CheckoutRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match CheckoutService::execute_checkout(&pool, payload).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err((StatusCode::BAD_REQUEST, err.to_string())),
    }
}

async fn lookup_order(
    State(pool): State<PgPool>,
    Path(order_number): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let order_row = sqlx::query(
        r#"
        SELECT id, order_number, customer_name, customer_email, total_cents,
               payment_provider, payment_status, order_status, tracking_number, created_at
        FROM orders
        WHERE order_number = $1
        "#,
    )
    .bind(&order_number)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Order not found".to_string()))?;

    let order_id: Uuid = order_row.get("id");
    let num: String = order_row.get("order_number");
    let cust_name: String = order_row.get("customer_name");
    let cust_email: String = order_row.get("customer_email");
    let total: i32 = order_row.get("total_cents");
    let provider: String = order_row.get("payment_provider");
    let pay_status: String = order_row.get("payment_status");
    let ord_status: String = order_row.get("order_status");
    let tracking: Option<String> = order_row.get("tracking_number");
    let created: chrono::DateTime<chrono::Utc> = order_row.get("created_at");

    let items_rows = sqlx::query(
        r#"
        SELECT product_title, variant_title, sku, quantity, unit_price_cents, is_digital, download_url
        FROM order_items
        WHERE order_id = $1
        "#,
    )
    .bind(order_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let items = items_rows.into_iter().map(|r| {
        json!({
            "product_title": r.get::<String, _>("product_title"),
            "variant_title": r.get::<String, _>("variant_title"),
            "sku": r.get::<String, _>("sku"),
            "quantity": r.get::<i32, _>("quantity"),
            "unit_price_cents": r.get::<i32, _>("unit_price_cents"),
            "is_digital": r.get::<bool, _>("is_digital"),
            "download_url": r.get::<Option<String>, _>("download_url"),
        })
    }).collect::<Vec<_>>();

    Ok(Json(json!({
        "order": {
            "order_number": num,
            "customer_name": cust_name,
            "customer_email": cust_email,
            "total_cents": total,
            "payment_provider": provider,
            "payment_status": pay_status,
            "order_status": ord_status,
            "tracking_number": tracking,
            "created_at": created
        },
        "items": items
    })))
}
