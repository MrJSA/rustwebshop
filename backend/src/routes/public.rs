use crate::models::{
    CartItemInput, Category, CheckoutRequest, Claims, CreateStockNotificationRequest, CustomerAddress,
    CustomerChangePasswordRequest, CustomerLoginRequest, CustomerRegisterRequest, EstimateShippingRequest,
    NavigationItem, PageContent, Product, ProductPart, ProductVariant, ProductWithVariants,
    PublicPaymentProviderInfo, ResetPasswordRequest, SaveAddressRequest, ShippingProvider,
    ShippingProviderWithZones, ShippingRate, ShippingZone, ShippingZoneWithRates, StoreSettings,
    StoreSettingsDTO, ToggleWishlistRequest, UpdateCustomerProfileRequest,
};
use crate::services::checkout::CheckoutService;
use axum::{
    extract::{Path, Query, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
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
        .route("/categories", get(public_list_categories))
        .route("/menu", get(public_get_menu))
        .route("/pages/:slug", get(public_get_page))
        .route("/shipping/providers", get(public_get_shipping_providers))
        .route("/products", get(list_products))
        .route("/products/carousels", get(public_get_carousels))
        .route("/products/:slug", get(get_product_by_slug))
        .route("/products/:id/related", get(public_get_related_products))
        .route("/products/:id/notify-stock", post(public_subscribe_stock_notification))
        .route("/products/:slug/parts", get(public_get_product_parts))
        .route("/cart/validate", post(validate_cart))
        .route("/shipping/rates", get(get_shipping_rates))
        .route("/checkout", post(execute_checkout))
        .route("/orders/lookup/:order_number", get(lookup_order))
        // Customer Authentication & Profile
        .route("/auth/customer/register", post(public_customer_register))
        .route("/auth/customer/login", post(public_customer_login))
        .route("/auth/customer/reset-password", post(public_customer_reset_password))
        .route("/customer/verify", get(public_verify_customer_email))
        .route("/customer/change-password", post(public_customer_change_password))
        .route("/customer/profile", get(public_get_customer_profile).put(public_update_customer_profile))
        .route("/customer/orders", get(public_get_customer_orders))
        .route("/customer/wishlist", get(public_get_customer_wishlist))
        .route("/customer/wishlist/toggle", post(public_toggle_wishlist))
        .route("/customer/addresses", get(public_get_customer_addresses).post(public_save_customer_address))
        .route("/customer/addresses/:id", delete(public_delete_customer_address))
}

async fn get_store_info(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let settings = sqlx::query_as::<_, StoreSettings>(
        "SELECT id, store_name, currency, currency_symbol, tax_rate_percent, deployment_mode, debug_mode, support_email, company_address, vat_id, logo_url, phone, hero_config, smtp_host, smtp_port, smtp_username, smtp_password, smtp_encryption, smtp_from_email, smtp_from_name, smtp_enabled, require_registered_checkout, require_email_verification, store_subtitle, show_store_title, show_store_subtitle, carousels_config, cookie_banner_enabled, cookie_banner_title, cookie_banner_description, cookie_banner_policy_url, cookie_accept_label, cookie_deny_label, cookie_preferences_label, updated_at FROM store_settings WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let store_dto = StoreSettingsDTO {
        store_name: settings.store_name,
        currency: settings.currency,
        currency_symbol: settings.currency_symbol,
        tax_rate_percent: settings.tax_rate_percent,
        deployment_mode: settings.deployment_mode,
        debug_mode: settings.debug_mode,
        support_email: settings.support_email,
        company_address: settings.company_address,
        vat_id: settings.vat_id,
        logo_url: settings.logo_url,
        phone: settings.phone,
        hero_config: settings.hero_config,
        smtp_host: settings.smtp_host,
        smtp_port: settings.smtp_port,
        smtp_username: settings.smtp_username,
        smtp_from_email: settings.smtp_from_email,
        smtp_from_name: settings.smtp_from_name,
        smtp_enabled: settings.smtp_enabled,
        require_registered_checkout: settings.require_registered_checkout,
        require_email_verification: settings.require_email_verification,
        store_subtitle: settings.store_subtitle,
        show_store_title: settings.show_store_title,
        show_store_subtitle: settings.show_store_subtitle,
        carousels_config: settings.carousels_config,
        cookie_banner_enabled: settings.cookie_banner_enabled,
        cookie_banner_title: settings.cookie_banner_title,
        cookie_banner_description: settings.cookie_banner_description,
        cookie_banner_policy_url: settings.cookie_banner_policy_url,
        cookie_accept_label: settings.cookie_accept_label,
        cookie_deny_label: settings.cookie_deny_label,
        cookie_preferences_label: settings.cookie_preferences_label,
    };

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
        "store": store_dto,
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
    let mut sql = "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images FROM products WHERE is_active = true".to_string();

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
            "SELECT id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, created_at, updated_at, images FROM product_variants WHERE product_id = $1 ORDER BY created_at ASC"
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
        "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images FROM products WHERE slug = $1 AND is_active = true"
    )
    .bind(slug)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Product not found".to_string()))?;

    let variants = sqlx::query_as::<_, ProductVariant>(
        "SELECT id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, created_at, updated_at, images FROM product_variants WHERE product_id = $1 ORDER BY created_at ASC"
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
        SELECT id, provider_id, zone_name, country_codes, is_default, created_at
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
    // 1. Check require_registered_checkout and require_email_verification
    if let Ok(settings_row) = sqlx::query("SELECT require_registered_checkout, require_email_verification FROM store_settings WHERE id = 1")
        .fetch_one(&pool)
        .await
    {
        let req_reg: bool = settings_row.get("require_registered_checkout");
        let req_verify: bool = settings_row.get("require_email_verification");

        if req_reg {
            let cust_opt = sqlx::query("SELECT is_verified FROM customers WHERE email = $1")
                .bind(&payload.customer_email)
                .fetch_optional(&pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            match cust_opt {
                None => {
                    return Err((
                        StatusCode::FORBIDDEN,
                        "Purchases are restricted to registered customers. Please log in or create an account to complete checkout.".to_string(),
                    ));
                }
                Some(cust) => {
                    let is_verified: bool = cust.get("is_verified");
                    if req_verify && !is_verified {
                        return Err((
                            StatusCode::FORBIDDEN,
                            "Your account email has not been verified yet. Please check your inbox and verify your email before placing an order.".to_string(),
                        ));
                    }
                }
            }
        }
    }

    match CheckoutService::execute_checkout(&pool, payload).await {
        Ok(res) => {
            // Trigger order created confirmation email
            let pool_clone = pool.clone();
            let order_num = res.order_number.clone();
            let is_paid = res.payment_status == "paid" || res.payment_status == "authorized";
            tokio::spawn(async move {
                crate::services::email::send_order_created_notification(&pool_clone, &order_num).await;
                if is_paid {
                    crate::services::email::send_payment_received_notification(&pool_clone, &order_num).await;
                }
            });
            Ok(Json(res))
        }
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

#[derive(Debug, Deserialize)]
pub struct MenuQuery {
    pub location: Option<String>,
}

// 7. Dynamic Navigation Menu Handler
async fn public_get_menu(
    State(pool): State<PgPool>,
    Query(query): Query<MenuQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let loc = query.location.unwrap_or_else(|| "header".to_string());
    let items = if loc == "all" {
        sqlx::query_as::<_, NavigationItem>(
            "SELECT id, location, label, url, sort_order, is_active, created_at FROM navigation_items WHERE is_active = true ORDER BY sort_order ASC, created_at ASC"
        )
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as::<_, NavigationItem>(
            "SELECT id, location, label, url, sort_order, is_active, created_at FROM navigation_items WHERE is_active = true AND location = $1 ORDER BY sort_order ASC, created_at ASC"
        )
        .bind(loc)
        .fetch_all(&pool)
        .await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(items))
}

// 8. Public CMS Policy Page Handler
async fn public_get_page(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let page = sqlx::query_as::<_, PageContent>(
        "SELECT slug, title, content_markdown, is_published, updated_at FROM pages WHERE slug = $1 AND is_published = true"
    )
    .bind(&slug)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Page not found".to_string()))?;

    // If shipment policy, also bundle active shipping providers with zones and rates!
    if slug == "shipment-policy" {
        let providers = fetch_all_active_providers_with_rates(&pool).await?;
        return Ok(Json(json!({
            "page": page,
            "providers": providers
        })));
    }

    Ok(Json(json!({ "page": page })))
}

// 9. Public Shipping Providers & Rates
async fn public_get_shipping_providers(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let providers = fetch_all_active_providers_with_rates(&pool).await?;
    Ok(Json(providers))
}

async fn fetch_all_active_providers_with_rates(pool: &PgPool) -> Result<Vec<ShippingProviderWithZones>, (StatusCode, String)> {
    let providers = sqlx::query_as::<_, ShippingProvider>(
        "SELECT id, name, code, tracking_url_template, is_active, sort_order, created_at FROM shipping_providers WHERE is_active = true ORDER BY sort_order ASC, name ASC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for p in providers {
        let zones = sqlx::query_as::<_, ShippingZone>(
            "SELECT id, provider_id, zone_name, country_codes, is_default, created_at FROM shipping_zones WHERE provider_id = $1 ORDER BY is_default DESC, zone_name ASC"
        )
        .bind(p.id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let mut zones_with_rates = Vec::new();
        for z in zones {
            let rates = sqlx::query_as::<_, ShippingRate>(
                "SELECT id, zone_id, name, package_type, min_weight_g, max_weight_g, price_cents, estimated_delivery_days FROM shipping_rates WHERE zone_id = $1 ORDER BY price_cents ASC"
            )
            .bind(z.id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            zones_with_rates.push(ShippingZoneWithRates { zone: z, rates });
        }

        result.push(ShippingProviderWithZones { provider: p, zones: zones_with_rates });
    }

    Ok(result)
}

// 10. Public Product Parts / BOM
async fn public_get_product_parts(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let product_id: Uuid = sqlx::query_scalar("SELECT id FROM products WHERE slug = $1")
        .bind(&slug)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Product not found".to_string()))?;

    let parts = sqlx::query_as::<_, ProductPart>(
        "SELECT id, product_id, variant_id, part_name, part_sku, quantity, notes, created_at FROM product_parts WHERE product_id = $1 ORDER BY created_at ASC"
    )
    .bind(product_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(parts))
}

// 11. Customer Authentication & Account Actions
fn extract_customer_email(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get(AUTHORIZATION)?.to_str().ok()?;
    if !auth_header.starts_with("Bearer ") {
        return None;
    }
    let token = &auth_header[7..];
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "super_secret_rustwebshop_jwt_token_2026".to_string());
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ).ok()?;
    Some(token_data.claims.sub)
}

async fn public_customer_register(
    State(pool): State<PgPool>,
    Json(payload): Json<CustomerRegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = Uuid::new_v4();
    let hashed = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let full_name = payload.full_name.clone().unwrap_or_else(|| {
        let first = payload.first_name.clone().unwrap_or_default();
        let last = payload.last_name.clone().unwrap_or_default();
        format!("{} {}", first, last).trim().to_string()
    });

    let first_name = payload.first_name.clone().unwrap_or_else(|| {
        full_name.split_whitespace().next().unwrap_or("").to_string()
    });
    let last_name = payload.last_name.clone().unwrap_or_else(|| {
        let parts: Vec<&str> = full_name.split_whitespace().collect();
        if parts.len() > 1 { parts[1..].join(" ") } else { "".to_string() }
    });
    let display_name = if !full_name.is_empty() { full_name.clone() } else { payload.email.split('@').next().unwrap_or("Customer").to_string() };

    // Check if email verification is required
    let require_verification: bool = sqlx::query_scalar("SELECT require_email_verification FROM store_settings WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap_or(false);

    let is_verified = !require_verification;
    let verification_token = if require_verification {
        Some(Uuid::new_v4().to_string())
    } else {
        None
    };

    // Insert into users
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role) VALUES ($1, $2, $3, $4, 'customer') ON CONFLICT (email) DO NOTHING"
    )
    .bind(user_id)
    .bind(&payload.email)
    .bind(&payload.email)
    .bind(&hashed)
    .execute(&pool)
    .await
    .ok();

    // Insert into customers
    sqlx::query(
        r#"
        INSERT INTO customers (id, email, password_hash, first_name, last_name, display_name, preferred_currency, is_verified, verification_token)
        VALUES ($1, $2, $3, $4, $5, $6, 'EUR', $7, $8)
        ON CONFLICT (email) DO UPDATE
        SET password_hash = $3, first_name = $4, last_name = $5, display_name = $6, is_verified = $7, verification_token = $8, updated_at = NOW()
        "#
    )
    .bind(user_id)
    .bind(&payload.email)
    .bind(&hashed)
    .bind(&first_name)
    .bind(&last_name)
    .bind(&display_name)
    .bind(is_verified)
    .bind(&verification_token)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Email already registered: {}", e)))?;

    // Send verification email if required
    if let Some(tok) = verification_token.clone() {
        let pool_clone = pool.clone();
        let email_copy = payload.email.clone();
        let name_copy = display_name.clone();
        tokio::spawn(async move {
            if let Ok(settings) = sqlx::query_as::<_, StoreSettings>("SELECT id, store_name, currency, currency_symbol, tax_rate_percent, deployment_mode, debug_mode, support_email, company_address, vat_id, logo_url, phone, hero_config, smtp_host, smtp_port, smtp_username, smtp_password, smtp_encryption, smtp_from_email, smtp_from_name, smtp_enabled, require_registered_checkout, require_email_verification, store_subtitle, show_store_title, show_store_subtitle, carousels_config, updated_at FROM store_settings WHERE id = 1").fetch_one(&pool_clone).await {
                crate::services::email::send_verification_email(&settings, &email_copy, &name_copy, &tok, "http://localhost:8080").await;
            }
        });
    }

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "super_secret_rustwebshop_jwt_token_2026".to_string());
    let claims = Claims {
        sub: payload.email.clone(),
        role: "customer".to_string(),
        exp: (Utc::now() + Duration::days(30)).timestamp() as usize,
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!({
        "token": token,
        "email": payload.email,
        "full_name": display_name,
        "first_name": first_name,
        "last_name": last_name,
        "preferred_currency": "EUR",
        "is_verified": is_verified,
        "verification_pending": require_verification
    }))))
}

async fn public_customer_login(
    State(pool): State<PgPool>,
    Json(payload): Json<CustomerLoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Check customers table first
    let cust_row = sqlx::query(
        "SELECT id, email, password_hash, first_name, last_name, display_name, preferred_currency, is_verified FROM customers WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (email, hash, first_name, last_name, display_name, preferred_currency, is_verified) = if let Some(r) = cust_row {
        (
            r.get::<String, _>("email"),
            r.get::<String, _>("password_hash"),
            r.get::<String, _>("first_name"),
            r.get::<String, _>("last_name"),
            r.get::<String, _>("display_name"),
            r.get::<String, _>("preferred_currency"),
            r.get::<bool, _>("is_verified"),
        )
    } else {
        let user_row = sqlx::query("SELECT id, username, email, password_hash FROM users WHERE email = $1 OR username = $1")
            .bind(&payload.email)
            .fetch_optional(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()))?;

        (
            user_row.get::<String, _>("email"),
            user_row.get::<String, _>("password_hash"),
            "".to_string(),
            "".to_string(),
            user_row.get::<String, _>("username"),
            "EUR".to_string(),
            true,
        )
    };

    let is_valid = bcrypt::verify(&payload.password, &hash).unwrap_or(false);
    if !is_valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()));
    }

    // Check verification status if required
    let require_verification: bool = sqlx::query_scalar("SELECT require_email_verification FROM store_settings WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap_or(false);

    if require_verification && !is_verified {
        return Err((
            StatusCode::FORBIDDEN,
            "Please verify your email address before logging in. Check your inbox for the activation link.".to_string(),
        ));
    }

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "super_secret_rustwebshop_jwt_token_2026".to_string());
    let claims = Claims {
        sub: email.clone(),
        role: "customer".to_string(),
        exp: (Utc::now() + Duration::days(30)).timestamp() as usize,
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({
        "token": token,
        "email": email,
        "full_name": if !display_name.is_empty() { display_name } else { email.split('@').next().unwrap_or("Customer").to_string() },
        "first_name": first_name,
        "last_name": last_name,
        "preferred_currency": preferred_currency
    })))
}

async fn public_customer_reset_password(
    State(pool): State<PgPool>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Some(new_pass) = payload.new_password {
        let hashed = bcrypt::hash(&new_pass, bcrypt::DEFAULT_COST)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        sqlx::query("UPDATE users SET password_hash = $1 WHERE email = $2")
            .bind(&hashed)
            .bind(&payload.email)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        sqlx::query("UPDATE customers SET password_hash = $1, updated_at = NOW() WHERE email = $2")
            .bind(&hashed)
            .bind(&payload.email)
            .execute(&pool)
            .await
            .ok();

        return Ok(Json(json!({ "message": "Password successfully reset. You may now log in." })));
    }

    Ok(Json(json!({ "message": "A password reset confirmation link has been sent to your email address." })))
}

async fn public_customer_change_password(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Json(payload): Json<CustomerChangePasswordRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    if payload.new_password != payload.confirm_password {
        return Err((StatusCode::BAD_REQUEST, "New passwords do not match".to_string()));
    }
    if payload.new_password.len() < 6 {
        return Err((StatusCode::BAD_REQUEST, "New password must be at least 6 characters".to_string()));
    }

    // Fetch existing password hash from customers or users
    let hash: Option<String> = sqlx::query_scalar("SELECT password_hash FROM customers WHERE email = $1")
        .bind(&email)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let existing_hash = match hash {
        Some(h) => h,
        None => {
            sqlx::query_scalar("SELECT password_hash FROM users WHERE email = $1")
                .bind(&email)
                .fetch_optional(&pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?
        }
    };

    let is_valid = bcrypt::verify(&payload.current_password, &existing_hash).unwrap_or(false);
    if !is_valid {
        return Err((StatusCode::BAD_REQUEST, "Current password is incorrect".to_string()));
    }

    let new_hash = bcrypt::hash(&payload.new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("UPDATE customers SET password_hash = $1, updated_at = NOW() WHERE email = $2")
        .bind(&new_hash)
        .bind(&email)
        .execute(&pool)
        .await
        .ok();

    sqlx::query("UPDATE users SET password_hash = $1 WHERE email = $2")
        .bind(&new_hash)
        .bind(&email)
        .execute(&pool)
        .await
        .ok();

    Ok(Json(json!({ "success": true, "message": "Password changed successfully" })))
}

async fn public_get_customer_profile(
    headers: HeaderMap,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let cust = sqlx::query(
        "SELECT id, email, first_name, last_name, display_name, preferred_currency, phone FROM customers WHERE email = $1"
    )
    .bind(&email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(r) = cust {
        return Ok(Json(json!({
            "id": r.get::<Uuid, _>("id"),
            "email": r.get::<String, _>("email"),
            "first_name": r.get::<String, _>("first_name"),
            "last_name": r.get::<String, _>("last_name"),
            "display_name": r.get::<String, _>("display_name"),
            "preferred_currency": r.get::<String, _>("preferred_currency"),
            "phone": r.get::<String, _>("phone"),
        })));
    }

    let user_row = sqlx::query("SELECT id, username, email FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    Ok(Json(json!({
        "id": user_row.get::<Uuid, _>("id"),
        "email": user_row.get::<String, _>("email"),
        "first_name": "",
        "last_name": "",
        "display_name": user_row.get::<String, _>("username"),
        "preferred_currency": "EUR",
        "phone": "",
    })))
}

async fn public_update_customer_profile(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Json(payload): Json<UpdateCustomerProfileRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO customers (id, email, password_hash, first_name, last_name, display_name, preferred_currency, phone)
        VALUES ($1, $2, 'placeholder', $3, $4, $5, $6, $7)
        ON CONFLICT (email) DO UPDATE
        SET first_name = $3, last_name = $4, display_name = $5, preferred_currency = $6, phone = $7, updated_at = NOW()
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&email)
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.display_name)
    .bind(&payload.preferred_currency)
    .bind(payload.phone.unwrap_or_default())
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn public_get_customer_orders(
    headers: HeaderMap,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let orders = sqlx::query(
        "SELECT id, order_number, customer_name, customer_email, shipping_cost_cents, subtotal_cents, tax_cents, total_cents, payment_provider, payment_status, order_status, tracking_number, created_at FROM orders WHERE customer_email = $1 ORDER BY created_at DESC"
    )
    .bind(&email)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let list = orders.into_iter().map(|r| {
        json!({
            "id": r.get::<Uuid, _>("id"),
            "order_number": r.get::<String, _>("order_number"),
            "customer_name": r.get::<String, _>("customer_name"),
            "customer_email": r.get::<String, _>("customer_email"),
            "total_cents": r.get::<i32, _>("total_cents"),
            "payment_provider": r.get::<String, _>("payment_provider"),
            "payment_status": r.get::<String, _>("payment_status"),
            "order_status": r.get::<String, _>("order_status"),
            "tracking_number": r.get::<Option<String>, _>("tracking_number"),
            "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        })
    }).collect::<Vec<_>>();

    Ok(Json(list))
}

async fn public_get_customer_wishlist(
    headers: HeaderMap,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers).unwrap_or_else(|| "guest@rustwebshop.local".to_string());

    let items = sqlx::query(
        r#"
        SELECT p.id, p.title, p.slug, p.category, p.subcategory, p.base_price_cents, p.image_url, p.product_type
        FROM customer_wishlist w
        JOIN products p ON w.product_id = p.id
        WHERE w.customer_email = $1
        ORDER BY w.created_at DESC
        "#
    )
    .bind(&email)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let list = items.into_iter().map(|r| {
        json!({
            "id": r.get::<Uuid, _>("id"),
            "title": r.get::<String, _>("title"),
            "slug": r.get::<String, _>("slug"),
            "category": r.get::<String, _>("category"),
            "subcategory": r.get::<String, _>("subcategory"),
            "base_price_cents": r.get::<i32, _>("base_price_cents"),
            "image_url": r.get::<String, _>("image_url"),
            "product_type": r.get::<String, _>("product_type"),
        })
    }).collect::<Vec<_>>();

    Ok(Json(list))
}

async fn public_toggle_wishlist(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Json(payload): Json<ToggleWishlistRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers).unwrap_or_else(|| "guest@rustwebshop.local".to_string());

    let existing: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM customer_wishlist WHERE customer_email = $1 AND product_id = $2"
    )
    .bind(&email)
    .bind(payload.product_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(id) = existing {
        sqlx::query("DELETE FROM customer_wishlist WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(json!({ "in_wishlist": false })))
    } else {
        sqlx::query("INSERT INTO customer_wishlist (id, customer_email, product_id) VALUES ($1, $2, $3)")
            .bind(Uuid::new_v4())
            .bind(&email)
            .bind(payload.product_id)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(json!({ "in_wishlist": true })))
    }
}

async fn public_get_customer_addresses(
    headers: HeaderMap,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let addrs = sqlx::query_as::<_, CustomerAddress>(
        "SELECT id, customer_email, address_type, full_name, street_address, apartment_suite, city, state_province, postal_code, country_code, is_default, created_at FROM customer_addresses WHERE customer_email = $1 ORDER BY is_default DESC, created_at DESC"
    )
    .bind(&email)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(addrs))
}

async fn public_save_customer_address(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Json(payload): Json<SaveAddressRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO customer_addresses (id, customer_email, address_type, full_name, street_address, apartment_suite, city, state_province, postal_code, country_code, is_default)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#
    )
    .bind(id)
    .bind(&email)
    .bind(&payload.address_type)
    .bind(&payload.full_name)
    .bind(&payload.street_address)
    .bind(payload.apartment_suite)
    .bind(&payload.city)
    .bind(&payload.state_province)
    .bind(&payload.postal_code)
    .bind(&payload.country_code)
    .bind(payload.is_default.unwrap_or(true))
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
}

async fn public_delete_customer_address(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let email = extract_customer_email(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    sqlx::query("DELETE FROM customer_addresses WHERE id = $1 AND customer_email = $2")
        .bind(id)
        .bind(&email)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn public_list_categories(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Category>(
        "SELECT id, parent_id, name, slug, description, display_order, image_url, created_at FROM categories ORDER BY display_order ASC, name ASC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows))
}

#[derive(Debug, Deserialize)]
pub struct VerifyQuery {
    pub token: String,
}

async fn public_verify_customer_email(
    State(pool): State<PgPool>,
    Query(query): Query<VerifyQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let result = sqlx::query(
        "UPDATE customers SET is_verified = true, verification_token = NULL, updated_at = NOW() WHERE verification_token = $1"
    )
    .bind(&query.token)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::BAD_REQUEST, "Invalid or expired verification token".to_string()));
    }

    Ok(Json(json!({ "success": true, "message": "Email verified successfully! You can now log in and place orders." })))
}

async fn public_subscribe_stock_notification(
    State(pool): State<PgPool>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<CreateStockNotificationRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if payload.email.trim().is_empty() || !payload.email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "Valid email address is required".to_string()));
    }
    sqlx::query(
        r#"
        INSERT INTO stock_notifications (id, product_id, variant_id, customer_email)
        VALUES ($1, $2, $3, $4)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(product_id)
    .bind(payload.variant_id)
    .bind(&payload.email.trim().to_lowercase())
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true, "message": "You will be notified when this item is back in stock!" })))
}

async fn public_get_related_products(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let current_product = sqlx::query_as::<_, Product>(
        "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images FROM products WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Product not found".to_string()))?;

    let related = sqlx::query_as::<_, Product>(
        r#"
        SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images
        FROM products 
        WHERE is_active = true AND id != $1
        ORDER BY (category = $2) DESC, (subcategory = $3) DESC, created_at DESC
        LIMIT 5
        "#
    )
    .bind(id)
    .bind(&current_product.category)
    .bind(&current_product.subcategory)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut result = Vec::new();
    for p in related {
        let variants = sqlx::query_as::<_, ProductVariant>(
            "SELECT id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, created_at, updated_at, images FROM product_variants WHERE product_id = $1 ORDER BY created_at ASC"
        )
        .bind(p.id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        result.push(ProductWithVariants { product: p, variants });
    }

    Ok(Json(result))
}

async fn public_get_carousels(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let config_val: serde_json::Value = sqlx::query_scalar("SELECT carousels_config FROM store_settings WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|_| json!({
            "sections": [
                { "id": "featured", "title": "Featured Gear", "enabled": true, "product_ids": [] },
                { "id": "new", "title": "New Arrivals", "enabled": true, "days": 30 },
                { "id": "bestsellers", "title": "Best Sellers", "enabled": true, "limit": 10 },
                { "id": "catalog", "title": "In Stock Hardware & Gear", "enabled": true }
            ]
        }));

    let sections = config_val.get("sections").and_then(|s| s.as_array()).cloned().unwrap_or_default();
    let mut carousels = Vec::new();

    for sec in sections {
        let enabled = sec.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
        if !enabled {
            continue;
        }
        let sec_id = sec.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let title = sec.get("title").and_then(|v| v.as_str()).unwrap_or("Products");

        let mut products: Vec<Product> = Vec::new();

        match sec_id {
            "featured" => {
                let ids: Vec<Uuid> = sec.get("product_ids")
                    .and_then(|arr| arr.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).filter_map(|s| Uuid::parse_str(s).ok()).collect())
                    .unwrap_or_default();
                if !ids.is_empty() {
                    products = sqlx::query_as::<_, Product>(
                        "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images FROM products WHERE is_active = true AND id = ANY($1)"
                    )
                    .bind(&ids)
                    .fetch_all(&pool)
                    .await
                    .unwrap_or_default();
                } else {
                    products = sqlx::query_as::<_, Product>(
                        "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images FROM products WHERE is_active = true ORDER BY created_at DESC LIMIT 10"
                    )
                    .fetch_all(&pool)
                    .await
                    .unwrap_or_default();
                }
            }
            "new" => {
                let days = sec.get("days").and_then(|v| v.as_i64()).unwrap_or(30);
                products = sqlx::query_as::<_, Product>(
                    "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images FROM products WHERE is_active = true AND created_at >= NOW() - ($1 || ' days')::INTERVAL ORDER BY created_at DESC LIMIT 15"
                )
                .bind(days.to_string())
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
            }
            "bestsellers" => {
                let limit = sec.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);
                products = sqlx::query_as::<_, Product>(
                    r#"
                    SELECT p.id, p.title, p.slug, p.description, p.product_type, p.category, p.subcategory,
                           p.base_price_cents, p.digital_download_url, p.image_url, p.is_active, p.created_at, p.updated_at,
                           p.subtitle, p.variant_selector_label, p.short_description, p.long_description, p.images
                    FROM products p
                    LEFT JOIN order_items oi ON oi.product_id = p.id
                    WHERE p.is_active = true
                    GROUP BY p.id
                    ORDER BY COALESCE(SUM(oi.quantity), 0) DESC, p.created_at DESC
                    LIMIT $1
                    "#
                )
                .bind(limit)
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
            }
            "catalog" => {
                products = sqlx::query_as::<_, Product>(
                    r#"
                    SELECT DISTINCT p.id, p.title, p.slug, p.description, p.product_type, p.category, p.subcategory,
                           p.base_price_cents, p.digital_download_url, p.image_url, p.is_active, p.created_at, p.updated_at,
                           p.subtitle, p.variant_selector_label, p.short_description, p.long_description, p.images
                    FROM products p
                    JOIN product_variants pv ON pv.product_id = p.id
                    WHERE p.is_active = true AND (p.product_type = 'digital' OR pv.stock_quantity > 0)
                    ORDER BY p.created_at DESC
                    LIMIT 20
                    "#
                )
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
            }
            _ => {}
        }

        let mut items_with_variants = Vec::new();
        for product in products {
            let variants = sqlx::query_as::<_, ProductVariant>(
                "SELECT id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, created_at, updated_at, images FROM product_variants WHERE product_id = $1 ORDER BY created_at ASC"
            )
            .bind(product.id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            items_with_variants.push(ProductWithVariants { product, variants });
        }

        carousels.push(json!({
            "id": sec_id,
            "title": title,
            "items": items_with_variants
        }));
    }

    Ok(Json(carousels))
}

