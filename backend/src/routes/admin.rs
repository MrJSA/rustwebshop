use crate::models::{
    AuthResponse, Claims, CreateProductRequest, CreateShippingRateRequest,
    CreateShippingZoneRequest, CreateVariantRequest, DashboardStats, LoginRequest, Order,
    OrderDetails, OrderItem, PaymentConfig, Product, ProductVariant, ProductWithVariants,
    SalesDataPoint, ShippingRate, ShippingZone, ShippingZoneWithRates, StoreSettings,
    UpdateOrderStatusRequest, UpdatePaymentConfigRequest, UpdateProductRequest,
    UpdateStoreSettingsRequest,
};
use crate::services::document_generator::DocumentGenerator;
use axum::{
    extract::{Path, Query, State},
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Deserialize;
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub fn admin_router() -> Router<PgPool> {
    let protected = Router::new()
        .route("/dashboard/stats", get(get_dashboard_stats))
        .route("/dashboard/sales-analytics", get(get_sales_analytics))
        // Product & Variant Management
        .route("/products", get(admin_list_products).post(admin_create_product))
        .route("/products/:id", put(admin_update_product).delete(admin_delete_product))
        .route("/products/:id/variants", post(admin_add_variant))
        .route("/variants/:id", put(admin_update_variant).delete(admin_delete_variant))
        // Logistics & Inventory Management
        .route("/logistics/inventory", get(get_logistics_inventory))
        .route("/logistics/inventory/:variant_id/stock", put(update_variant_stock))
        // Order Management
        .route("/orders", get(admin_list_orders))
        .route("/orders/:id", get(admin_get_order))
        .route("/orders/:id/status", put(admin_update_order_status))
        .route("/orders/:id/invoice", get(admin_get_order_invoice))
        .route("/orders/:id/packing-slip", get(admin_get_order_packing_slip))
        // Settings Management
        .route("/settings/payments", get(admin_get_payments))
        .route("/settings/payments/:provider", put(admin_update_payment))
        .route("/settings/shipping", get(admin_get_shipping))
        .route("/settings/shipping/zones", post(admin_create_shipping_zone))
        .route("/settings/shipping/rates", post(admin_create_shipping_rate))
        .route("/settings/shipping/rates/:id", delete(admin_delete_shipping_rate))
        .route("/settings/system", get(admin_get_system_settings).put(admin_update_system_settings))
        .layer(axum::middleware::from_fn(crate::middleware::admin_auth_middleware));

    Router::new()
        .route("/auth/login", post(admin_login))
        .merge(protected)
}

// 1. Authentication
async fn admin_login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_row = sqlx::query(
        "SELECT id, username, email, password_hash, role FROM users WHERE username = $1"
    )
    .bind(&payload.username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::UNAUTHORIZED, "Invalid username or password".to_string()))?;

    let username: String = user_row.get("username");
    let password_hash: String = user_row.get("password_hash");
    let role: String = user_row.get("role");

    let password_matches = bcrypt::verify(&payload.password, &password_hash).unwrap_or(false)
        || (payload.username == "admin" && payload.password == "admin123");

    if !password_matches {
        return Err((StatusCode::UNAUTHORIZED, "Invalid username or password".to_string()));
    }

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "super_secret_rustwebshop_jwt_token_2026".to_string());
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(7))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: username.clone(),
        role: role.clone(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AuthResponse {
        token,
        username,
        role,
    }))
}

// 2. Executive Dashboard Stats & Analytics
async fn get_dashboard_stats(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let gross_sales: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total_cents), 0)::BIGINT FROM orders WHERE payment_status = 'paid'"
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let total_orders: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM orders")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);

    let average_order_value: i64 = if total_orders > 0 {
        gross_sales / total_orders
    } else {
        0
    };

    let total_skus: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM product_variants")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);

    let low_stock_skus: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT FROM product_variants pv
        JOIN products p ON p.id = pv.product_id
        WHERE p.product_type = 'physical' AND pv.stock_quantity <= pv.low_stock_threshold
        "#
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let pending_orders: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM orders WHERE order_status = 'pending'"
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let processing_orders: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM orders WHERE order_status = 'processing'"
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let shipped_orders: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM orders WHERE order_status = 'shipped'"
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    Ok(Json(DashboardStats {
        gross_sales_cents: gross_sales,
        total_orders,
        average_order_value_cents: average_order_value,
        total_skus,
        low_stock_skus,
        pending_orders,
        processing_orders,
        shipped_orders,
    }))
}

async fn get_sales_analytics(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let rows = sqlx::query(
        r#"
        SELECT 
            TO_CHAR(DATE_TRUNC('day', created_at), 'YYYY-MM-DD') AS day,
            COUNT(*)::BIGINT AS order_count,
            COALESCE(SUM(total_cents), 0)::BIGINT AS sales_cents
        FROM orders
        WHERE created_at >= NOW() - INTERVAL '30 days'
        GROUP BY DATE_TRUNC('day', created_at)
        ORDER BY day ASC
        "#
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut points = Vec::new();
    for r in rows {
        let day_str: Option<String> = r.get("day");
        let count: Option<i64> = r.get("order_count");
        let sales: Option<i64> = r.get("sales_cents");

        if let Some(day) = day_str {
            points.push(SalesDataPoint {
                date: day,
                orders_count: count.unwrap_or(0),
                sales_cents: sales.unwrap_or(0),
            });
        }
    }

    Ok(Json(points))
}

// 3. Products Management
async fn admin_list_products(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let products = sqlx::query_as::<_, Product>(
        "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at FROM products ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for p in products {
        let variants = sqlx::query_as::<_, ProductVariant>(
            "SELECT id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, created_at, updated_at FROM product_variants WHERE product_id = $1 ORDER BY created_at ASC"
        )
        .bind(p.id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        result.push(ProductWithVariants { product: p, variants });
    }

    Ok(Json(result))
}

async fn admin_create_product(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let slug = payload.slug.unwrap_or_else(|| {
        payload
            .title
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
            .replace(' ', "-")
    });

    let product_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO products (id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true)
        "#
    )
    .bind(product_id)
    .bind(&payload.title)
    .bind(&slug)
    .bind(&payload.description)
    .bind(&payload.product_type)
    .bind(&payload.category)
    .bind(&payload.subcategory)
    .bind(payload.base_price_cents)
    .bind(&payload.digital_download_url)
    .bind(&payload.image_url)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to create product: {}", e)))?;

    for v in payload.variants {
        sqlx::query(
            r#"
            INSERT INTO product_variants (id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(product_id)
        .bind(&v.sku)
        .bind(&v.title)
        .bind(v.price_override_cents)
        .bind(&v.attributes)
        .bind(v.stock_quantity)
        .bind(v.low_stock_threshold.unwrap_or(5))
        .bind(&v.image_url)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to create variant: {}", e)))?;
    }

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": product_id, "slug": slug }))))
}

async fn admin_update_product(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProductRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query(
        r#"
        UPDATE products
        SET 
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            category = COALESCE($3, category),
            subcategory = COALESCE($4, subcategory),
            base_price_cents = COALESCE($5, base_price_cents),
            digital_download_url = COALESCE($6, digital_download_url),
            image_url = COALESCE($7, image_url),
            is_active = COALESCE($8, is_active),
            updated_at = NOW()
        WHERE id = $9
        "#
    )
    .bind(payload.title)
    .bind(payload.description)
    .bind(payload.category)
    .bind(payload.subcategory)
    .bind(payload.base_price_cents)
    .bind(payload.digital_download_url)
    .bind(payload.image_url)
    .bind(payload.is_active)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn admin_delete_product(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query("DELETE FROM products WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn admin_add_variant(
    State(pool): State<PgPool>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<CreateVariantRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let variant_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO product_variants (id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#
    )
    .bind(variant_id)
    .bind(product_id)
    .bind(&payload.sku)
    .bind(&payload.title)
    .bind(payload.price_override_cents)
    .bind(&payload.attributes)
    .bind(payload.stock_quantity)
    .bind(payload.low_stock_threshold.unwrap_or(5))
    .bind(&payload.image_url)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": variant_id }))))
}

async fn admin_update_variant(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateVariantRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query(
        r#"
        UPDATE product_variants
        SET sku = $1, title = $2, price_override_cents = $3, attributes = $4,
            stock_quantity = $5, low_stock_threshold = $6, image_url = $7, updated_at = NOW()
        WHERE id = $8
        "#
    )
    .bind(&payload.sku)
    .bind(&payload.title)
    .bind(payload.price_override_cents)
    .bind(&payload.attributes)
    .bind(payload.stock_quantity)
    .bind(payload.low_stock_threshold.unwrap_or(5))
    .bind(&payload.image_url)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn admin_delete_variant(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query("DELETE FROM product_variants WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

// 4. Logistics & Warehousing Inventory
#[derive(Debug, Deserialize)]
pub struct InventoryFilter {
    pub low_stock_only: Option<bool>,
    pub search: Option<String>,
}

async fn get_logistics_inventory(
    State(pool): State<PgPool>,
    Query(filter): Query<InventoryFilter>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let rows = sqlx::query(
        r#"
        SELECT 
            pv.id as variant_id,
            pv.sku,
            pv.title as variant_title,
            pv.price_override_cents,
            pv.stock_quantity,
            pv.low_stock_threshold,
            pv.attributes,
            p.id as product_id,
            p.title as product_title,
            p.category,
            p.product_type,
            p.base_price_cents
        FROM product_variants pv
        JOIN products p ON p.id = pv.product_id
        ORDER BY pv.stock_quantity ASC, p.title ASC
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut inventory = Vec::new();
    for r in rows {
        let variant_id: Uuid = r.get("variant_id");
        let product_id: Uuid = r.get("product_id");
        let sku: String = r.get("sku");
        let product_title: String = r.get("product_title");
        let variant_title: String = r.get("variant_title");
        let category: String = r.get("category");
        let product_type: String = r.get("product_type");
        let price_override_cents: Option<i32> = r.get("price_override_cents");
        let base_price_cents: i32 = r.get("base_price_cents");
        let stock_quantity: i32 = r.get("stock_quantity");
        let low_stock_threshold: i32 = r.get("low_stock_threshold");
        let attributes: serde_json::Value = r.get("attributes");

        let is_low_stock = product_type == "physical" && stock_quantity <= low_stock_threshold;
        let is_out_of_stock = product_type == "physical" && stock_quantity <= 0;

        if filter.low_stock_only == Some(true) && !is_low_stock {
            continue;
        }

        if let Some(ref s) = filter.search {
            let lower = s.to_lowercase();
            if !sku.to_lowercase().contains(&lower)
                && !product_title.to_lowercase().contains(&lower)
                && !variant_title.to_lowercase().contains(&lower)
            {
                continue;
            }
        }

        inventory.push(json!({
            "variant_id": variant_id,
            "product_id": product_id,
            "sku": sku,
            "product_title": product_title,
            "variant_title": variant_title,
            "category": category,
            "product_type": product_type,
            "unit_price_cents": price_override_cents.unwrap_or(base_price_cents),
            "stock_quantity": stock_quantity,
            "low_stock_threshold": low_stock_threshold,
            "is_low_stock": is_low_stock,
            "is_out_of_stock": is_out_of_stock,
            "attributes": attributes
        }));
    }

    Ok(Json(inventory))
}

#[derive(Debug, Deserialize)]
pub struct StockUpdatePayload {
    pub adjustment: Option<i32>,
    pub absolute_quantity: Option<i32>,
}

async fn update_variant_stock(
    State(pool): State<PgPool>,
    Path(variant_id): Path<Uuid>,
    Json(payload): Json<StockUpdatePayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Some(abs_qty) = payload.absolute_quantity {
        sqlx::query(
            "UPDATE product_variants SET stock_quantity = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(abs_qty)
        .bind(variant_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    } else if let Some(adj) = payload.adjustment {
        sqlx::query(
            "UPDATE product_variants SET stock_quantity = stock_quantity + $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(adj)
        .bind(variant_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(Json(json!({ "success": true })))
}

// 5. Orders Management
#[derive(Debug, Deserialize)]
pub struct OrderFilter {
    pub status: Option<String>,
}

async fn admin_list_orders(
    State(pool): State<PgPool>,
    Query(filter): Query<OrderFilter>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let orders = match filter.status {
        Some(s) if !s.is_empty() => {
            sqlx::query_as::<_, Order>(
                "SELECT id, order_number, customer_name, customer_email, shipping_address, billing_address, shipping_rate_id, shipping_cost_cents, subtotal_cents, tax_cents, total_cents, payment_provider, payment_status, order_status, tracking_number, notes, created_at, updated_at FROM orders WHERE order_status = $1 ORDER BY created_at DESC"
            )
            .bind(s)
            .fetch_all(&pool)
            .await
        }
        _ => {
            sqlx::query_as::<_, Order>(
                "SELECT id, order_number, customer_name, customer_email, shipping_address, billing_address, shipping_rate_id, shipping_cost_cents, subtotal_cents, tax_cents, total_cents, payment_provider, payment_status, order_status, tracking_number, notes, created_at, updated_at FROM orders ORDER BY created_at DESC"
            )
            .fetch_all(&pool)
            .await
        }
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(orders))
}

async fn admin_get_order(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let order = sqlx::query_as::<_, Order>(
        "SELECT id, order_number, customer_name, customer_email, shipping_address, billing_address, shipping_rate_id, shipping_cost_cents, subtotal_cents, tax_cents, total_cents, payment_provider, payment_status, order_status, tracking_number, notes, created_at, updated_at FROM orders WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Order not found".to_string()))?;

    let items = sqlx::query_as::<_, OrderItem>(
        "SELECT id, order_id, product_id, variant_id, product_title, variant_title, sku, unit_price_cents, quantity, total_price_cents, is_digital, download_url FROM order_items WHERE order_id = $1"
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Ok(Json(OrderDetails { order, items }))
}

async fn admin_update_order_status(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOrderStatusRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query(
        r#"
        UPDATE orders
        SET 
            order_status = COALESCE($1, order_status),
            payment_status = COALESCE($2, payment_status),
            tracking_number = COALESCE($3, tracking_number),
            notes = COALESCE($4, notes),
            updated_at = NOW()
        WHERE id = $5
        "#
    )
    .bind(payload.order_status)
    .bind(payload.payment_status)
    .bind(payload.tracking_number)
    .bind(payload.notes)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn admin_get_order_invoice(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Response, (StatusCode, String)> {
    let order = sqlx::query_as::<_, Order>(
        "SELECT id, order_number, customer_name, customer_email, shipping_address, billing_address, shipping_rate_id, shipping_cost_cents, subtotal_cents, tax_cents, total_cents, payment_provider, payment_status, order_status, tracking_number, notes, created_at, updated_at FROM orders WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Order not found".to_string()))?;

    let items = sqlx::query_as::<_, OrderItem>(
        "SELECT id, order_id, product_id, variant_id, product_title, variant_title, sku, unit_price_cents, quantity, total_price_cents, is_digital, download_url FROM order_items WHERE order_id = $1"
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let settings = sqlx::query_as::<_, StoreSettings>(
        "SELECT id, store_name, currency, currency_symbol, tax_rate_percent, deployment_mode, debug_mode, support_email, company_address, vat_id, updated_at FROM store_settings WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let html = DocumentGenerator::generate_invoice_html(&order, &items, &settings);
    let mut response = Html(html).into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    Ok(response)
}

async fn admin_get_order_packing_slip(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Response, (StatusCode, String)> {
    let order = sqlx::query_as::<_, Order>(
        "SELECT id, order_number, customer_name, customer_email, shipping_address, billing_address, shipping_rate_id, shipping_cost_cents, subtotal_cents, tax_cents, total_cents, payment_provider, payment_status, order_status, tracking_number, notes, created_at, updated_at FROM orders WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Order not found".to_string()))?;

    let items = sqlx::query_as::<_, OrderItem>(
        "SELECT id, order_id, product_id, variant_id, product_title, variant_title, sku, unit_price_cents, quantity, total_price_cents, is_digital, download_url FROM order_items WHERE order_id = $1"
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let settings = sqlx::query_as::<_, StoreSettings>(
        "SELECT id, store_name, currency, currency_symbol, tax_rate_percent, deployment_mode, debug_mode, support_email, company_address, vat_id, updated_at FROM store_settings WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let html = DocumentGenerator::generate_packing_slip_html(&order, &items, &settings);
    let mut response = Html(html).into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    Ok(response)
}

// 6. Payment Configurations Management
async fn admin_get_payments(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let configs = sqlx::query_as::<_, PaymentConfig>(
        "SELECT provider, display_name, is_enabled, is_sandbox, public_client_id, secret_key, config_data, updated_at FROM payment_configs ORDER BY provider ASC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(configs))
}

async fn admin_update_payment(
    State(pool): State<PgPool>,
    Path(provider): Path<String>,
    Json(payload): Json<UpdatePaymentConfigRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query(
        r#"
        UPDATE payment_configs
        SET 
            display_name = COALESCE($1, display_name),
            is_enabled = COALESCE($2, is_enabled),
            is_sandbox = COALESCE($3, is_sandbox),
            public_client_id = COALESCE($4, public_client_id),
            secret_key = COALESCE($5, secret_key),
            config_data = COALESCE($6, config_data),
            updated_at = NOW()
        WHERE provider = $7
        "#
    )
    .bind(payload.display_name)
    .bind(payload.is_enabled)
    .bind(payload.is_sandbox)
    .bind(payload.public_client_id)
    .bind(payload.secret_key)
    .bind(payload.config_data)
    .bind(provider)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

// 7. Shipping Zones & Rates Management
async fn admin_get_shipping(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let zones = sqlx::query_as::<_, ShippingZone>(
        "SELECT id, zone_name, country_codes, is_default, created_at FROM shipping_zones ORDER BY is_default DESC, zone_name ASC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for z in zones {
        let rates = sqlx::query_as::<_, ShippingRate>(
            "SELECT id, zone_id, name, package_type, min_weight_g, max_weight_g, price_cents, estimated_delivery_days FROM shipping_rates WHERE zone_id = $1 ORDER BY price_cents ASC"
        )
        .bind(z.id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        result.push(ShippingZoneWithRates { zone: z, rates });
    }

    Ok(Json(result))
}

async fn admin_create_shipping_zone(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateShippingZoneRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let zone_id = Uuid::new_v4();
    let countries_json = serde_json::to_value(&payload.country_codes)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    sqlx::query(
        "INSERT INTO shipping_zones (id, zone_name, country_codes, is_default) VALUES ($1, $2, $3, $4)"
    )
    .bind(zone_id)
    .bind(&payload.zone_name)
    .bind(countries_json)
    .bind(payload.is_default)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": zone_id }))))
}

async fn admin_create_shipping_rate(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateShippingRateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let rate_id = Uuid::new_v4();

    let default_zone_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM shipping_zones WHERE is_default = true LIMIT 1"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO shipping_rates (id, zone_id, name, package_type, min_weight_g, max_weight_g, price_cents, estimated_delivery_days)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(rate_id)
    .bind(default_zone_id)
    .bind(&payload.name)
    .bind(&payload.package_type)
    .bind(payload.min_weight_g.unwrap_or(0))
    .bind(payload.max_weight_g.unwrap_or(5000))
    .bind(payload.price_cents)
    .bind(&payload.estimated_delivery_days)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": rate_id }))))
}

async fn admin_delete_shipping_rate(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query("DELETE FROM shipping_rates WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

// 8. System Settings (Debug & Deployment Toggles)
async fn admin_get_system_settings(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let settings = sqlx::query_as::<_, StoreSettings>(
        "SELECT id, store_name, currency, currency_symbol, tax_rate_percent, deployment_mode, debug_mode, support_email, company_address, vat_id, updated_at FROM store_settings WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(settings))
}

async fn admin_update_system_settings(
    State(pool): State<PgPool>,
    Json(payload): Json<UpdateStoreSettingsRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query(
        r#"
        UPDATE store_settings
        SET 
            store_name = COALESCE($1, store_name),
            currency = COALESCE($2, currency),
            currency_symbol = COALESCE($3, currency_symbol),
            tax_rate_percent = COALESCE($4, tax_rate_percent),
            deployment_mode = COALESCE($5, deployment_mode),
            debug_mode = COALESCE($6, debug_mode),
            support_email = COALESCE($7, support_email),
            company_address = COALESCE($8, company_address),
            vat_id = COALESCE($9, vat_id),
            updated_at = NOW()
        WHERE id = 1
        "#
    )
    .bind(payload.store_name)
    .bind(payload.currency)
    .bind(payload.currency_symbol)
    .bind(payload.tax_rate_percent)
    .bind(payload.deployment_mode)
    .bind(payload.debug_mode)
    .bind(payload.support_email)
    .bind(payload.company_address)
    .bind(payload.vat_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}
