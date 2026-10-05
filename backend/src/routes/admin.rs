use crate::models::{
    AdminLoginRequest, AdminLoginResponse, AdminUser, Category, CategoryLeaderboardItem, ChangeAdminCredentialsRequest,
    Claims, Coupon, CreateAdminUserRequest, CreateCategoryRequest, CreateCouponRequest, CreateNavigationItemRequest,
    CreatePartRequest, CreateProductRequest, CreateProviderRequest, CreateShippingRateRequest,
    CreateShippingZoneRequest, CreateVariantRequest, DashboardStats, MediaItem, NavigationItem, Order, OrderDetails,
    OrderItem, PageContent, Product, ProductLeaderboardItem, ProductPart, ProductVariant,
    ProductWithVariants, PurchaseAnalysisDayPoint, PurchaseAnalysisResponse, PurchaseAnalysisSummary,
    ReorderMenuRequest, SalesDataPoint, ShippingProvider, ShippingProviderWithZones, ShippingRate, ShippingZone,
    ShippingZoneWithRates, StoreSettings, TestEmailRequest, UpdateAdminUserRequest, UpdateCategoryRequest,
    UpdateCouponRequest, UpdateNavigationItemRequest, UpdateOrderStatusRequest, UpdatePageRequest,
    UpdatePaymentConfigRequest, UpdateProductRequest, UpdateProviderRequest, UpdateShippingRateRequest,
    UpdateShippingZoneRequest, UpdateStoreSettingsRequest, UpdateVariantRequest,
};
use crate::middleware::CurrentAdmin;
use crate::services::document_generator::DocumentGenerator;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub fn admin_router(pool: PgPool) -> Router<PgPool> {
    let protected = Router::new()
        .route("/dashboard/stats", get(get_dashboard_stats))
        .route("/dashboard/sales-analytics", get(get_sales_analytics))
        .route("/analytics/purchase-analysis", get(admin_get_purchase_analysis))
        // Admin Auth & Security
        .route("/auth/status", get(admin_auth_status))
        .route("/auth/me", get(admin_auth_status))
        .route("/auth/change-credentials", post(admin_change_credentials).put(admin_change_credentials))
        // Admin Users Management
        .route("/users", get(admin_list_users).post(admin_create_user))
        .route("/users/:id", put(admin_update_user).delete(admin_delete_user))
        // Media Library
        .route("/media", get(admin_list_media))
        .route("/media/upload", post(admin_upload_media))
        .route("/media/:id", delete(admin_delete_media))
        // Product & Variant & Parts Management
        .route("/products", get(admin_list_products).post(admin_create_product))
        .route("/products/:id", put(admin_update_product).delete(admin_delete_product))
        .route("/products/:id/toggle-featured", post(admin_toggle_featured_product))
        .route("/products/:id/variants", post(admin_add_variant))
        .route("/products/:id/parts", get(admin_get_product_parts).post(admin_add_product_part))
        .route("/products/:id/parts/:part_id", delete(admin_delete_product_part))
        .route("/parts/:part_id", put(admin_update_product_part))
        .route("/variants/:id", put(admin_update_variant).delete(admin_delete_variant))
        // Categories Hierarchy Management
        .route("/categories", get(admin_list_categories).post(admin_create_category))
        .route("/categories/:id", put(admin_update_category).delete(admin_delete_category))
        // Coupons / Promo Codes
        .route("/coupons", get(admin_list_coupons).post(admin_create_coupon))
        .route("/coupons/:id", put(admin_update_coupon).delete(admin_delete_coupon))
        // Logistics & Inventory Management
        .route("/logistics/inventory", get(get_logistics_inventory))
        .route("/logistics/inventory/:variant_id/stock", put(update_variant_stock))
        // Order Management
        .route("/orders", get(admin_list_orders))
        .route("/orders/:id", get(admin_get_order))
        .route("/orders/:id/status", put(admin_update_order_status))
        .route("/orders/:id/refund", post(admin_refund_order))
        .route("/orders/:id/cancel", post(admin_cancel_order))
        .route("/orders/:id/invoice", get(admin_get_order_invoice))
        .route("/orders/:id/packing-slip", get(admin_get_order_packing_slip))
        // Settings Management
        .route("/settings/payments", get(admin_get_payments))
        .route("/settings/payments/:provider", put(admin_update_payment))
        .route("/settings/payments/stripe/domains", get(admin_stripe_list_domains).post(admin_stripe_register_domain))
        .route("/settings/shipping", get(admin_get_shipping))
        .route("/settings/shipping/providers", get(admin_get_shipping_providers).post(admin_create_shipping_provider))
        .route("/settings/shipping/providers/:id", put(admin_update_shipping_provider).delete(admin_delete_shipping_provider))
        .route("/settings/shipping/providers/:id/zones", post(admin_create_provider_zone))
        .route("/settings/shipping/zones", post(admin_create_shipping_zone))
        .route("/settings/shipping/zones/:id", put(admin_update_shipping_zone).delete(admin_delete_shipping_zone))
        .route("/settings/shipping/zones/:id/rates", post(admin_create_zone_rate))
        .route("/settings/shipping/rates", post(admin_create_shipping_rate))
        .route("/settings/shipping/rates/:id", put(admin_update_shipping_rate).delete(admin_delete_shipping_rate))
        // CMS Policy Pages
        .route("/pages", get(admin_list_pages))
        .route("/pages/:slug", get(admin_get_page).put(admin_update_page))
        // Navigation Menu
        .route("/menu", get(admin_list_menu).post(admin_create_menu_item))
        .route("/menu/reorder", put(admin_reorder_menu).post(admin_reorder_menu))
        .route("/menu/:id", put(admin_update_menu_item).delete(admin_delete_menu_item))
        .route("/settings/system", get(admin_get_system_settings).put(admin_update_system_settings))
        .route("/settings/email/test", post(admin_test_email))
        .route("/system/status", get(admin_system_status))
        .route("/system/check", post(admin_system_check))
        .route("/system/update", post(admin_system_update))
        .route("/system/job", get(admin_system_job))
        .route("/system/domains", get(admin_get_domains).put(admin_put_domains))
        .route("/export/store-data", get(admin_export_store_data))
        .route("/export/media", get(admin_export_media_library))
        .route("/import/store-data", post(admin_import_store_data))
        .layer(axum::middleware::from_fn_with_state(pool, crate::middleware::admin_auth_middleware));

    Router::new()
        .route("/auth/login", post(admin_login))
        .merge(protected)
}


// 1. Authentication
const ADMIN_SESSION_HOURS: i64 = 12;

fn admin_session_token(admin_id: Uuid) -> Result<String, (StatusCode, String)> {
    crate::services::auth::issue_token(&admin_id.to_string(), crate::services::auth::ROLE_ADMIN_TOKEN, Duration::hours(ADMIN_SESSION_HOURS))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn admin_login(
    State(pool): State<PgPool>,
    Json(payload): Json<AdminLoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    use crate::services::auth;
    let username = payload.username.trim();
    let limiter_key = format!("admin:{}", username.to_lowercase());
    auth::check_login_allowed(&limiter_key).map_err(|m| (StatusCode::TOO_MANY_REQUESTS, m))?;

    let admin = sqlx::query("SELECT id, username, password_hash, is_default, role FROM admin_users WHERE username = $1")
        .bind(username)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Always run bcrypt so response time does not reveal whether the username exists
    static DUMMY_HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let dummy = DUMMY_HASH.get_or_init(|| bcrypt::hash(auth::random_token(16), 12).unwrap_or_default());
    let hash = admin.as_ref().map(|a| a.get::<String, _>("password_hash")).unwrap_or_else(|| dummy.clone());
    let password_ok = bcrypt::verify(&payload.password, &hash).unwrap_or(false);

    let Some(a) = admin.filter(|_| password_ok) else {
        auth::record_login_failure(&limiter_key);
        return Err((StatusCode::UNAUTHORIZED, "Invalid username or password".to_string()));
    };
    auth::clear_login_failures(&limiter_key);

    let id: Uuid = a.get("id");
    Ok(Json(json!({
        "token": admin_session_token(id)?,
        "username": a.get::<String, _>("username"),
        "role": a.get::<String, _>("role"),
        "is_default": a.get::<bool, _>("is_default"),
    })))
}

async fn admin_auth_status(Extension(admin): Extension<CurrentAdmin>) -> impl IntoResponse {
    Json(json!({
        "authenticated": true,
        "username": admin.username,
        "role": admin.role,
        "is_default": admin.is_default,
        "permissions": admin.permissions
    }))
}

/// Changes the username/password of the logged-in admin and returns a fresh session token.
async fn admin_change_credentials(
    State(pool): State<PgPool>,
    Extension(admin): Extension<CurrentAdmin>,
    Json(payload): Json<ChangeAdminCredentialsRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let new_username = payload.new_username.trim();
    if new_username.is_empty() || payload.new_password.len() < 12 {
        return Err((StatusCode::BAD_REQUEST, "Username must not be empty and the password must be at least 12 characters".to_string()));
    }
    if payload.new_password == payload.current_password {
        return Err((StatusCode::BAD_REQUEST, "The new password must differ from the current one".to_string()));
    }

    let hash: String = sqlx::query_scalar("SELECT password_hash FROM admin_users WHERE id = $1")
        .bind(admin.id)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !bcrypt::verify(&payload.current_password, &hash).unwrap_or(false) {
        return Err((StatusCode::UNAUTHORIZED, "Current password is incorrect".to_string()));
    }

    let new_hash = bcrypt::hash(&payload.new_password, 12).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query("UPDATE admin_users SET username = $1, password_hash = $2, is_default = FALSE, updated_at = NOW() WHERE id = $3")
        .bind(new_username)
        .bind(new_hash)
        .bind(admin.id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, if e.to_string().contains("unique") { "This username is already taken".to_string() } else { e.to_string() }))?;

    Ok(Json(json!({
        "success": true,
        "message": "Admin credentials successfully updated!",
        "token": admin_session_token(admin.id)?,
        "username": new_username
    })))
}

// 2. Executive Dashboard Stats & Analytics
async fn get_dashboard_stats(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let gross_sales: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total_cents), 0)::BIGINT FROM orders WHERE payment_status != 'failed' AND payment_status != 'refunded'"
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
        "SELECT id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, created_at, updated_at, subtitle, variant_selector_label, short_description, long_description, images, has_multiple_variants, tax_rate_percent FROM products ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for p in products {
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

async fn admin_create_product(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let base_title = if payload.title.trim().is_empty() { "Untitled Product".to_string() } else { payload.title.trim().to_string() };

    let slug = payload.slug.filter(|s| !s.trim().is_empty()).unwrap_or_else(|| {
        let clean = base_title
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
            .replace(' ', "-");
        if clean.is_empty() {
            format!("product-{}", &Uuid::new_v4().to_string()[..8])
        } else {
            clean
        }
    });

    let product_id = Uuid::new_v4();
    let description = payload.description.unwrap_or_default();
    let short_desc = payload.short_description.unwrap_or_else(|| description.clone());
    let long_desc = payload.long_description.unwrap_or_else(|| description.clone());
    let subtitle = payload.subtitle.unwrap_or_default();
    let var_label = payload.variant_selector_label.unwrap_or_else(|| "Choose Variant / Model:".to_string());
    let product_type = payload.product_type.unwrap_or_else(|| "physical".to_string());
    let category = payload.category.unwrap_or_else(|| "Hardware".to_string());
    let subcategory = payload.subcategory.unwrap_or_default();
    let base_price = payload.base_price_cents.unwrap_or(4999);
    let image_url = payload.image_url.unwrap_or_default();
    let images = payload.images.unwrap_or_else(|| {
        if !image_url.is_empty() {
            json!([image_url])
        } else {
            json!([])
        }
    });
    let has_multiple = payload.has_multiple_variants.unwrap_or(false);

    let tax_rate = payload.tax_rate_percent.unwrap_or(19.0);

    sqlx::query(
        r#"
        INSERT INTO products (id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, subtitle, variant_selector_label, short_description, long_description, images, has_multiple_variants, tax_rate_percent)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, $11, $12, $13, $14, $15, $16, $17)
        "#
    )
    .bind(product_id)
    .bind(&base_title)
    .bind(&slug)
    .bind(&description)
    .bind(&product_type)
    .bind(&category)
    .bind(&subcategory)
    .bind(base_price)
    .bind(&payload.digital_download_url)
    .bind(&image_url)
    .bind(&subtitle)
    .bind(&var_label)
    .bind(&short_desc)
    .bind(&long_desc)
    .bind(&images)
    .bind(has_multiple)
    .bind(tax_rate)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to create product: {}", e)))?;

    let variants_to_create = payload.variants.unwrap_or_default();

    if variants_to_create.is_empty() {
        // Automatically create single default variant for immediate out-of-the-box readiness
        let default_sku = format!("PRD-{}-{}", &base_title.chars().filter(|c| c.is_alphanumeric()).take(4).collect::<String>().to_uppercase(), &product_id.to_string()[..4].to_uppercase());
        sqlx::query(
            r#"
            INSERT INTO product_variants (id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, images)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(product_id)
        .bind(default_sku)
        .bind("Standard Edition")
        .bind(base_price)
        .bind(json!({ "version": "Standard" }))
        .bind(20)
        .bind(5)
        .bind(&image_url)
        .bind(&images)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to create default variant: {}", e)))?;
    } else {
        for v in variants_to_create {
            let v_images = v.images.unwrap_or_else(|| {
                if let Some(ref u) = v.image_url {
                    json!([u])
                } else {
                    json!([])
                }
            });

            sqlx::query(
                r#"
                INSERT INTO product_variants (id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, images)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
            .bind(&v_images)
            .execute(&mut *tx)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to create variant: {}", e)))?;
        }
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
            subtitle = COALESCE($9, subtitle),
            variant_selector_label = COALESCE($10, variant_selector_label),
            short_description = COALESCE($11, short_description),
            long_description = COALESCE($12, long_description),
            images = COALESCE($13, images),
            has_multiple_variants = COALESCE($14, has_multiple_variants),
            tax_rate_percent = COALESCE($15, tax_rate_percent),
            product_type = COALESCE($16, product_type),
            updated_at = NOW()
        WHERE id = $17
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
    .bind(payload.subtitle)
    .bind(payload.variant_selector_label)
    .bind(payload.short_description)
    .bind(payload.long_description)
    .bind(payload.images)
    .bind(payload.has_multiple_variants)
    .bind(payload.tax_rate_percent)
    .bind(payload.product_type)
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
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 1. Detach from historical order items so existing customer orders and invoices never break
    sqlx::query("UPDATE order_items SET product_id = NULL, variant_id = NULL WHERE product_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to detach order items: {}", e)))?;

    // 2. Remove references in product_parts (parts can reference both product_id and variant_id)
    sqlx::query("DELETE FROM product_parts WHERE product_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to remove product parts: {}", e)))?;

    // 3. Remove customer wishlist & stock notifications
    sqlx::query("DELETE FROM customer_wishlist WHERE product_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to clear wishlist items: {}", e)))?;

    sqlx::query("DELETE FROM stock_notifications WHERE product_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to clear stock notifications: {}", e)))?;

    // 4. Remove all SKU variants for this product
    sqlx::query("DELETE FROM product_variants WHERE product_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to remove variants: {}", e)))?;

    // 5. Delete product master record
    sqlx::query("DELETE FROM products WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete product: {}", e)))?;

    // 6. Remove from carousels_config featured list in store_settings if present
    let id_str = id.to_string();
    let settings = sqlx::query_as::<_, StoreSettings>(
        "SELECT * FROM store_settings WHERE id = 1 FOR UPDATE"
    )
    .fetch_optional(&mut *tx)
    .await
    .unwrap_or(None);

    if let Some(s) = settings {
        let mut cfg = s.carousels_config;
        let mut modified = false;
        if let Some(sections) = cfg.get_mut("sections").and_then(|s| s.as_array_mut()) {
            for sec in sections.iter_mut() {
                if sec.get("id").and_then(|v| v.as_str()) == Some("featured") {
                    if let Some(arr) = sec.get("product_ids").and_then(|v| v.as_array()) {
                        let filtered: Vec<serde_json::Value> = arr
                            .iter()
                            .filter(|v| v.as_str() != Some(&id_str))
                            .cloned()
                            .collect();
                        if filtered.len() != arr.len() {
                            if let Some(obj) = sec.as_object_mut() {
                                obj.insert("product_ids".to_string(), serde_json::Value::Array(filtered));
                                modified = true;
                            }
                        }
                    }
                    break;
                }
            }
        }
        if modified {
            let _ = sqlx::query("UPDATE store_settings SET carousels_config = $1, updated_at = NOW() WHERE id = 1")
                .bind(&cfg)
                .execute(&mut *tx)
                .await;
        }
    }

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn admin_toggle_featured_product(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let settings = sqlx::query_as::<_, StoreSettings>(
        "SELECT * FROM store_settings WHERE id = 1 FOR UPDATE"
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut cfg = settings.carousels_config;
    let id_str = id.to_string();
    let mut is_now_featured = false;

    if let Some(sections) = cfg.get_mut("sections").and_then(|s| s.as_array_mut()) {
        for sec in sections.iter_mut() {
            if sec.get("id").and_then(|v| v.as_str()) == Some("featured") {
                let mut ids: Vec<String> = sec.get("product_ids")
                    .and_then(|arr| arr.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                if let Some(pos) = ids.iter().position(|x| x == &id_str) {
                    ids.remove(pos);
                    is_now_featured = false;
                } else {
                    ids.push(id_str.clone());
                    is_now_featured = true;
                }

                if let Some(obj) = sec.as_object_mut() {
                    obj.insert("product_ids".to_string(), json!(ids));
                }
                break;
            }
        }
    }

    sqlx::query("UPDATE store_settings SET carousels_config = $1, updated_at = NOW() WHERE id = 1")
        .bind(&cfg)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "is_featured": is_now_featured, "product_id": id })))
}

async fn admin_add_variant(
    State(pool): State<PgPool>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<CreateVariantRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let variant_id = Uuid::new_v4();
    let images = payload.images.unwrap_or_else(|| {
        if let Some(ref u) = payload.image_url {
            json!([u])
        } else {
            json!([])
        }
    });

    sqlx::query(
        r#"
        INSERT INTO product_variants (id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, images)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
    .bind(&images)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": variant_id }))))
}

async fn admin_update_variant(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateVariantRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query(
        r#"
        UPDATE product_variants
        SET 
            sku = COALESCE($1, sku), 
            title = COALESCE($2, title), 
            price_override_cents = COALESCE($3, price_override_cents), 
            attributes = COALESCE($4, attributes),
            stock_quantity = COALESCE($5, stock_quantity), 
            low_stock_threshold = COALESCE($6, low_stock_threshold), 
            image_url = COALESCE($7, image_url), 
            images = COALESCE($8, images),
            updated_at = NOW()
        WHERE id = $9
        "#
    )
    .bind(payload.sku)
    .bind(payload.title)
    .bind(payload.price_override_cents)
    .bind(payload.attributes)
    .bind(payload.stock_quantity)
    .bind(payload.low_stock_threshold)
    .bind(payload.image_url)
    .bind(payload.images)
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

    // Check if stock is now positive, and notify subscribers
    let info_opt: Option<(Uuid, String, i32)> = sqlx::query_as(
        "SELECT p.id, p.title, v.stock_quantity FROM product_variants v JOIN products p ON p.id = v.product_id WHERE v.id = $1"
    )
    .bind(variant_id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    if let Some((product_id, title, stock)) = info_opt {
        if stock > 0 {
            let pool_clone = pool.clone();
            tokio::spawn(async move {
                let emails: Vec<String> = sqlx::query_scalar(
                    "SELECT email FROM stock_notifications WHERE product_id = $1 OR variant_id = $2"
                )
                .bind(product_id)
                .bind(variant_id)
                .fetch_all(&pool_clone)
                .await
                .unwrap_or_default();

                if !emails.is_empty() {
                    let settings = sqlx::query_as::<_, StoreSettings>(
                        "SELECT * FROM store_settings WHERE id = 1"
                    )
                    .fetch_one(&pool_clone)
                    .await;

                    if let Ok(st) = settings {
                        for email in emails {
                            crate::services::email::send_back_in_stock_email(
                                &st,
                                &email,
                                &title,
                                "http://localhost:8080/products",
                            ).await;
                        }
                    }

                    let _ = sqlx::query("DELETE FROM stock_notifications WHERE product_id = $1 OR variant_id = $2")
                        .bind(product_id)
                        .bind(variant_id)
                        .execute(&pool_clone)
                        .await;
                }
            });
        }
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
    if payload.order_status.as_deref() == Some("shipped") {
        let has_tracking = payload.tracking_number.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false);
        if !has_tracking {
            let existing: Option<Option<String>> = sqlx::query_scalar("SELECT tracking_number FROM orders WHERE id = $1")
                .bind(id)
                .fetch_optional(&pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let already_tracked = existing.flatten().map(|s| !s.trim().is_empty()).unwrap_or(false);
            if !already_tracked {
                return Err((StatusCode::BAD_REQUEST, "A tracking number is required before marking an order as Shipped".to_string()));
            }
        }
    }

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
    .bind(payload.order_status.clone())
    .bind(payload.payment_status)
    .bind(payload.tracking_number.clone())
    .bind(payload.notes)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Send shipping email notification if newly marked shipped
    if payload.order_status.as_deref() == Some("shipped") {
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            let row_opt: Option<(String, String, Option<String>)> = sqlx::query_as(
                "SELECT customer_email, order_number, tracking_number FROM orders WHERE id = $1"
            )
            .bind(id)
            .fetch_optional(&pool_clone)
            .await
            .unwrap_or(None);

            if let Some((cust_email, order_num, track_opt)) = row_opt {
                let track_num = payload.tracking_number.clone()
                    .or(track_opt)
                    .unwrap_or_else(|| "N/A".to_string());

                let settings = sqlx::query_as::<_, StoreSettings>(
                    "SELECT * FROM store_settings WHERE id = 1"
                )
                .fetch_one(&pool_clone)
                .await;

                if let Ok(st) = settings {
                    crate::services::email::send_order_shipped_email(&st, &cust_email, &order_num, &track_num).await;
                }
            }
        });
    }

    Ok(Json(json!({ "success": true })))
}

async fn admin_refund_order(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let order = sqlx::query("SELECT payment_provider, payment_status, payment_reference FROM orders WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Order not found".to_string()))?;

    let provider: String = order.get("payment_provider");
    let payment_status: String = order.get("payment_status");
    let reference: Option<String> = order.get("payment_reference");

    if payment_status == "refunded" {
        return Ok(Json(json!({ "success": true, "payment_status": "refunded", "provider_refunded": false })));
    }

    // Refund the money at the provider first; only mark the order refunded if that succeeded.
    let provider_refunded = match reference.as_deref() {
        Some(r) if provider == "stripe" || provider == "paypal" => {
            crate::services::payments::refund(&pool, &provider, r)
                .await
                .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Refund failed at {}: {}", provider, e)))?;
            true
        }
        _ => false,
    };

    sqlx::query("UPDATE orders SET payment_status = 'refunded', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true, "payment_status": "refunded", "provider_refunded": provider_refunded })))
}

async fn admin_cancel_order(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query("UPDATE orders SET order_status = 'cancelled', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true, "order_status": "cancelled" })))
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
        "SELECT * FROM store_settings WHERE id = 1"
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
        "SELECT * FROM store_settings WHERE id = 1"
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
    let rows = sqlx::query(
        "SELECT provider, display_name, is_enabled, is_sandbox, public_client_id, secret_key, webhook_secret, config_data, updated_at FROM payment_configs ORDER BY CASE provider WHEN 'stripe' THEN 0 WHEN 'paypal' THEN 1 ELSE 2 END, provider ASC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    fn mask(secret: &str) -> String {
        let secret = secret.trim();
        if secret.is_empty() {
            String::new()
        } else if secret.len() > 12 && secret.is_ascii() {
            format!("{}••••{}", &secret[..8], &secret[secret.len() - 4..])
        } else {
            "••••••••".to_string()
        }
    }

    // Secrets are never sent back to the browser — only whether they are set and a masked hint.
    let configs: Vec<serde_json::Value> = rows.into_iter().map(|r| {
        let sec: String = r.get("secret_key");
        let whsec: String = r.get("webhook_secret");
        json!({
            "provider": r.get::<String, _>("provider"),
            "display_name": r.get::<String, _>("display_name"),
            "is_enabled": r.get::<bool, _>("is_enabled"),
            "is_sandbox": r.get::<bool, _>("is_sandbox"),
            "public_client_id": r.get::<String, _>("public_client_id"),
            "has_secret_key": !sec.trim().is_empty(),
            "masked_secret_key": mask(&sec),
            "secret_mode": if sec.contains("_live_") { "live" } else if sec.contains("_test_") { "test" } else { "" },
            "has_webhook_secret": !whsec.trim().is_empty(),
            "masked_webhook_secret": mask(&whsec),
            "config_data": r.get::<serde_json::Value, _>("config_data"),
            "updated_at": r.get::<DateTime<Utc>, _>("updated_at"),
        })
    }).collect();

    Ok(Json(configs))
}

async fn admin_update_payment(
    State(pool): State<PgPool>,
    Path(provider): Path<String>,
    Json(payload): Json<UpdatePaymentConfigRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if provider != "stripe" && provider != "paypal" {
        return Err((StatusCode::BAD_REQUEST, format!("Unknown payment provider '{}'", provider)));
    }

    let public_id = payload.public_client_id.map(|s| s.trim().to_string());
    let secret = payload.secret_key.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let webhook_secret = payload.webhook_secret.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());

    if provider == "stripe" {
        if let Some(ref p) = public_id {
            if !p.is_empty() && !p.starts_with("pk_") {
                return Err((StatusCode::BAD_REQUEST, "The Publishable Key must start with pk_test_ or pk_live_. Secret keys (sk_/rk_) belong in the Secret Key field.".to_string()));
            }
        }
        if let Some(ref s) = secret {
            if !(s.starts_with("sk_") || s.starts_with("rk_")) {
                return Err((StatusCode::BAD_REQUEST, "The Secret Key must start with sk_ or rk_.".to_string()));
            }
        }
        if let Some(ref w) = webhook_secret {
            if !w.starts_with("whsec_") {
                return Err((StatusCode::BAD_REQUEST, "The webhook signing secret must start with whsec_.".to_string()));
            }
        }
    }

    sqlx::query(
        r#"
        UPDATE payment_configs
        SET
            display_name = COALESCE($1, display_name),
            is_enabled = COALESCE($2, is_enabled),
            is_sandbox = COALESCE($3, is_sandbox),
            public_client_id = COALESCE($4, public_client_id),
            secret_key = COALESCE($5, secret_key),
            webhook_secret = COALESCE($6, webhook_secret),
            config_data = COALESCE($7, config_data),
            updated_at = NOW()
        WHERE provider = $8
        "#
    )
    .bind(payload.display_name)
    .bind(payload.is_enabled)
    .bind(payload.is_sandbox)
    .bind(public_id)
    .bind(secret)
    .bind(webhook_secret)
    .bind(payload.config_data)
    .bind(&provider)
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

// 8. System Settings (Email, Verification, Branding, Carousels & Cookie Consent)
async fn admin_get_system_settings(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let settings = sqlx::query_as::<_, StoreSettings>(
        "SELECT * FROM store_settings WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(settings))
}

/// Store settings edited on the "Storefront & Design" pages; every other field belongs to "Settings".
const STOREFRONT_SETTING_FIELDS: &[&str] = &[
    "hero_config",
    "carousels_config",
    "footer_config",
    "cookie_banner_enabled",
    "cookie_banner_title",
    "cookie_banner_description",
    "cookie_banner_policy_url",
    "cookie_accept_label",
    "cookie_deny_label",
    "cookie_preferences_label",
    "store_subtitle",
    "show_store_title",
    "show_store_subtitle",
    "stock_display_template",
    "logo_url",
];

/// Rejects the save if it changes a field from a section the admin has no access to.
/// Forms often send the whole settings object, so only *changed* values count.
async fn check_settings_permissions(pool: &PgPool, admin: &CurrentAdmin, payload: &UpdateStoreSettingsRequest) -> Result<(), (StatusCode, String)> {
    if admin.can("storefront") && admin.can("settings") {
        return Ok(());
    }
    let current = sqlx::query_as::<_, StoreSettings>("SELECT * FROM store_settings WHERE id = 1")
        .fetch_one(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let current = serde_json::to_value(&current).unwrap_or_default();
    let requested = serde_json::to_value(payload).unwrap_or_default();

    for (field, value) in requested.as_object().into_iter().flatten() {
        let changed = match field.as_str() {
            "smtp_password" => value.as_str().map(|p| !p.is_empty()).unwrap_or(false),
            _ => !value.is_null() && current.get(field) != Some(value),
        };
        if !changed {
            continue;
        }
        let section = if STOREFRONT_SETTING_FIELDS.contains(&field.as_str()) { "storefront" } else { "settings" };
        if !admin.can(section) {
            return Err((StatusCode::FORBIDDEN, format!("You do not have permission to change '{}'", field)));
        }
    }
    Ok(())
}

/// Customers can only verify their email if the shop can send emails. Refuse saves that would
/// require verification while sending is off (checked only when one of the involved fields changes,
/// so unrelated saves never fail because of an existing configuration).
async fn check_email_verification_possible(pool: &PgPool, payload: &UpdateStoreSettingsRequest) -> Result<(), (StatusCode, String)> {
    let current = sqlx::query_as::<_, StoreSettings>("SELECT * FROM store_settings WHERE id = 1")
        .fetch_one(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let verify = payload.require_email_verification.unwrap_or(current.require_email_verification);
    let enabled = payload.smtp_enabled.unwrap_or(current.smtp_enabled);
    let host = payload.smtp_host.clone().unwrap_or_else(|| current.smtp_host.clone());
    let touched = payload.require_email_verification.map_or(false, |v| v != current.require_email_verification)
        || payload.smtp_enabled.map_or(false, |v| v != current.smtp_enabled)
        || payload.smtp_host.as_ref().map_or(false, |h| h.trim() != current.smtp_host.trim());
    if touched && verify && !(enabled && !host.trim().is_empty()) {
        return Err((
            StatusCode::BAD_REQUEST,
            "\"Require email verification\" needs working email sending: turn on SMTP and enter a mail server first — otherwise new customers can never activate their accounts.".to_string(),
        ));
    }
    Ok(())
}

async fn admin_update_system_settings(
    State(pool): State<PgPool>,
    Extension(admin): Extension<CurrentAdmin>,
    Json(payload): Json<UpdateStoreSettingsRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    check_settings_permissions(&pool, &admin, &payload).await?;
    check_email_verification_possible(&pool, &payload).await?;

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
            logo_url = COALESCE($10, logo_url),
            phone = COALESCE($11, phone),
            hero_config = COALESCE($12, hero_config),
            smtp_host = COALESCE($13, smtp_host),
            smtp_port = COALESCE($14, smtp_port),
            smtp_username = COALESCE($15, smtp_username),
            smtp_password = COALESCE($16, smtp_password),
            smtp_encryption = COALESCE($17, smtp_encryption),
            smtp_from_email = COALESCE($18, smtp_from_email),
            smtp_from_name = COALESCE($19, smtp_from_name),
            smtp_enabled = COALESCE($20, smtp_enabled),
            require_registered_checkout = COALESCE($21, require_registered_checkout),
            require_email_verification = COALESCE($22, require_email_verification),
            store_subtitle = COALESCE($23, store_subtitle),
            show_store_title = COALESCE($24, show_store_title),
            show_store_subtitle = COALESCE($25, show_store_subtitle),
            carousels_config = COALESCE($26, carousels_config),
            cookie_banner_enabled = COALESCE($27, cookie_banner_enabled),
            cookie_banner_title = COALESCE($28, cookie_banner_title),
            cookie_banner_description = COALESCE($29, cookie_banner_description),
            cookie_banner_policy_url = COALESCE($30, cookie_banner_policy_url),
            cookie_accept_label = COALESCE($31, cookie_accept_label),
            cookie_deny_label = COALESCE($32, cookie_deny_label),
            cookie_preferences_label = COALESCE($33, cookie_preferences_label),
            tax_notice = COALESCE($34, tax_notice),
            tax_mode = COALESCE($35, tax_mode),
            legal_name = COALESCE($36, legal_name),
            store_owner = COALESCE($37, store_owner),
            commercial_register = COALESCE($38, commercial_register),
            dispute_resolution_notice = COALESCE($39, dispute_resolution_notice),
            odr_url = COALESCE($40, odr_url),
            footer_config = COALESCE($41, footer_config),
            order_prefix_enabled = COALESCE($42, order_prefix_enabled),
            order_prefix = COALESCE($43, order_prefix),
            order_date_enabled = COALESCE($44, order_date_enabled),
            stock_display_template = COALESCE($45, stock_display_template),
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
    .bind(payload.logo_url)
    .bind(payload.phone)
    .bind(payload.hero_config)
    .bind(payload.smtp_host)
    .bind(payload.smtp_port)
    .bind(payload.smtp_username)
    .bind(payload.smtp_password.filter(|p| !p.is_empty()))
    .bind(payload.smtp_encryption)
    .bind(payload.smtp_from_email)
    .bind(payload.smtp_from_name)
    .bind(payload.smtp_enabled)
    .bind(payload.require_registered_checkout)
    .bind(payload.require_email_verification)
    .bind(payload.store_subtitle)
    .bind(payload.show_store_title)
    .bind(payload.show_store_subtitle)
    .bind(payload.carousels_config)
    .bind(payload.cookie_banner_enabled)
    .bind(payload.cookie_banner_title)
    .bind(payload.cookie_banner_description)
    .bind(payload.cookie_banner_policy_url)
    .bind(payload.cookie_accept_label)
    .bind(payload.cookie_deny_label)
    .bind(payload.cookie_preferences_label)
    .bind(payload.tax_notice)
    .bind(payload.tax_mode)
    .bind(payload.legal_name)
    .bind(payload.store_owner)
    .bind(payload.commercial_register)
    .bind(payload.dispute_resolution_notice)
    .bind(payload.odr_url)
    .bind(payload.footer_config)
    .bind(payload.order_prefix_enabled)
    .bind(payload.order_prefix)
    .bind(payload.order_date_enabled)
    .bind(payload.stock_display_template)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn admin_test_email(
    State(pool): State<PgPool>,
    Json(payload): Json<TestEmailRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut settings = sqlx::query_as::<_, StoreSettings>(
        "SELECT * FROM store_settings WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let saved = settings.clone();

    // Test exactly what is in the form, even if it has not been saved yet
    if let Some(o) = payload.smtp {
        if let Some(v) = o.host { settings.smtp_host = v; }
        if let Some(v) = o.port { settings.smtp_port = v; }
        if let Some(v) = o.username { settings.smtp_username = v; }
        if let Some(v) = o.password.filter(|p| !p.is_empty()) { settings.smtp_password = v; }
        if let Some(v) = o.encryption { settings.smtp_encryption = v; }
        if let Some(v) = o.from_email { settings.smtp_from_email = v; }
        if let Some(v) = o.from_name { settings.smtp_from_name = v; }
        settings.smtp_enabled = true;
    }

    crate::services::email::send_test_email(&settings, &payload.recipient_email)
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err))?;

    // The shop itself uses the *saved* settings — say so if they would not send anything
    let differs = saved.smtp_host.trim() != settings.smtp_host.trim()
        || saved.smtp_port != settings.smtp_port
        || saved.smtp_username.trim() != settings.smtp_username.trim()
        || saved.smtp_password != settings.smtp_password
        || saved.smtp_encryption != settings.smtp_encryption
        || saved.smtp_from_email.trim() != settings.smtp_from_email.trim();
    let warning = if !saved.smtp_enabled || saved.smtp_host.trim().is_empty() {
        Some("The test worked with the values in the form, but email sending is switched OFF in the saved settings — the shop does not send any emails (verification, orders) until you enable SMTP and click Save.")
    } else if differs {
        Some("The test used the values in the form, which differ from the saved settings. Click Save so the shop uses them.")
    } else {
        None
    };

    Ok(Json(json!({ "success": true, "message": "Test email sent successfully!", "warning": warning })))
}

// 9. Media Library Handlers
async fn admin_list_media(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let items = sqlx::query_as::<_, MediaItem>(
        "SELECT id, filename, original_name, url, mime_type, size_bytes, created_at FROM media ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(items))
}

async fn admin_upload_media(
    State(pool): State<PgPool>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tokio::fs::create_dir_all("uploads").await.ok();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let original_name = field.file_name().unwrap_or("image.jpg").to_string();
        let ext = original_name
            .rsplit_once('.')
            .map(|(_, e)| e.to_lowercase())
            .filter(|e| !e.is_empty() && e.len() <= 10 && e.chars().all(|c| c.is_ascii_alphanumeric()))
            .unwrap_or_else(|| "bin".to_string());
        // Files are served from the shop's own origin: never accept types a browser would execute
        if matches!(ext.as_str(), "html" | "htm" | "xhtml" | "js" | "mjs" | "xml" | "php" | "exe" | "sh" | "bat") {
            return Err((StatusCode::BAD_REQUEST, format!("File type .{} is not allowed", ext)));
        }
        let mime_type = match ext.as_str() {
            "webp" => "image/webp",
            "png" => "image/png",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "jpg" | "jpeg" => "image/jpeg",
            "avif" => "image/avif",
            "pdf" => "application/pdf",
            "zip" => "application/zip",
            _ => "application/octet-stream",
        };
        let unique_name = format!("{}.{}", Uuid::new_v4(), ext);
        let path = format!("uploads/{}", unique_name);

        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        tokio::fs::write(&path, &data)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let url = format!("/uploads/{}", unique_name);
        let size_bytes = data.len() as i64;
        let id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO media (id, filename, original_name, url, mime_type, size_bytes) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(id)
        .bind(&unique_name)
        .bind(&original_name)
        .bind(&url)
        .bind(mime_type)
        .bind(size_bytes)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        return Ok(Json(json!({
            "id": id,
            "url": url,
            "filename": unique_name,
            "original_name": original_name,
            "mime_type": mime_type,
            "size_bytes": size_bytes
        })));
    }
    Err((StatusCode::BAD_REQUEST, "No file provided in form".to_string()))
}

async fn admin_delete_media(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let filename: Option<String> = sqlx::query_scalar("SELECT filename FROM media WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(f) = filename {
        tokio::fs::remove_file(format!("uploads/{}", f)).await.ok();
        sqlx::query("DELETE FROM media WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(Json(json!({ "success": true })))
}

// 10. Product Parts / Bill of Materials (BOM)
async fn admin_get_product_parts(
    State(pool): State<PgPool>,
    Path(product_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let parts = sqlx::query_as::<_, ProductPart>(
        "SELECT id, product_id, variant_id, part_name, part_sku, quantity, notes, created_at FROM product_parts WHERE product_id = $1 ORDER BY created_at ASC"
    )
    .bind(product_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(parts))
}

async fn admin_add_product_part(
    State(pool): State<PgPool>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<CreatePartRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let part_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO product_parts (id, product_id, variant_id, part_name, part_sku, quantity, notes) VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(part_id)
    .bind(product_id)
    .bind(payload.variant_id)
    .bind(&payload.part_name)
    .bind(&payload.part_sku)
    .bind(payload.quantity)
    .bind(&payload.notes)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": part_id }))))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductPartRequest {
    pub variant_id: Option<Uuid>,
    pub part_name: String,
    pub part_sku: Option<String>,
    pub quantity: i32,
    pub notes: Option<String>,
}

async fn admin_update_product_part(
    State(pool): State<PgPool>,
    Path(part_id): Path<Uuid>,
    Json(payload): Json<UpdateProductPartRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query(
        "UPDATE product_parts SET variant_id = $1, part_name = $2, part_sku = $3, quantity = $4, notes = $5 WHERE id = $6"
    )
    .bind(payload.variant_id)
    .bind(&payload.part_name)
    .bind(&payload.part_sku)
    .bind(payload.quantity)
    .bind(&payload.notes)
    .bind(part_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn admin_delete_product_part(
    State(pool): State<PgPool>,
    Path((_product_id, part_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query("DELETE FROM product_parts WHERE id = $1")
        .bind(part_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

// 11. Hierarchical Shipping Providers & Zones Management
async fn admin_get_shipping_providers(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let providers = sqlx::query_as::<_, ShippingProvider>(
        "SELECT id, name, code, tracking_url_template, is_active, sort_order, created_at FROM shipping_providers ORDER BY sort_order ASC, name ASC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for p in providers {
        let zones = sqlx::query_as::<_, ShippingZone>(
            "SELECT id, provider_id, zone_name, country_codes, is_default, created_at FROM shipping_zones WHERE provider_id = $1 ORDER BY is_default DESC, zone_name ASC"
        )
        .bind(p.id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        let mut zones_with_rates = Vec::new();
        for z in zones {
            let rates = sqlx::query_as::<_, ShippingRate>(
                "SELECT id, zone_id, name, package_type, min_weight_g, max_weight_g, price_cents, estimated_delivery_days FROM shipping_rates WHERE zone_id = $1 ORDER BY price_cents ASC"
            )
            .bind(z.id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            zones_with_rates.push(ShippingZoneWithRates { zone: z, rates });
        }

        result.push(ShippingProviderWithZones { provider: p, zones: zones_with_rates });
    }

    Ok(Json(result))
}

async fn admin_create_shipping_provider(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateProviderRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO shipping_providers (id, name, code, tracking_url_template, is_active, sort_order) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.code)
    .bind(payload.tracking_url_template.unwrap_or_else(|| "https://www.dhl.com/track?id={tracking_number}".to_string()))
    .bind(payload.is_active.unwrap_or(true))
    .bind(payload.sort_order.unwrap_or(0))
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
}

async fn admin_update_shipping_provider(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProviderRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query(
        r#"
        UPDATE shipping_providers
        SET 
            name = COALESCE($1, name),
            code = COALESCE($2, code),
            tracking_url_template = COALESCE($3, tracking_url_template),
            is_active = COALESCE($4, is_active),
            sort_order = COALESCE($5, sort_order)
        WHERE id = $6
        "#
    )
    .bind(payload.name)
    .bind(payload.code)
    .bind(payload.tracking_url_template)
    .bind(payload.is_active)
    .bind(payload.sort_order)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn admin_delete_shipping_provider(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query("DELETE FROM shipping_providers WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn admin_create_provider_zone(
    State(pool): State<PgPool>,
    Path(provider_id): Path<Uuid>,
    Json(payload): Json<CreateShippingZoneRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let zone_id = Uuid::new_v4();
    let countries_json = serde_json::to_value(&payload.country_codes)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    sqlx::query(
        "INSERT INTO shipping_zones (id, provider_id, zone_name, country_codes, is_default) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(zone_id)
    .bind(provider_id)
    .bind(&payload.zone_name)
    .bind(countries_json)
    .bind(payload.is_default.unwrap_or(false))
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": zone_id }))))
}

async fn admin_update_shipping_zone(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateShippingZoneRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let countries_json = if let Some(codes) = payload.country_codes {
        Some(serde_json::to_value(codes).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?)
    } else {
        None
    };

    sqlx::query(
        r#"
        UPDATE shipping_zones
        SET 
            zone_name = COALESCE($1, zone_name),
            country_codes = COALESCE($2, country_codes),
            is_default = COALESCE($3, is_default),
            provider_id = COALESCE($4, provider_id)
        WHERE id = $5
        "#
    )
    .bind(payload.zone_name)
    .bind(countries_json)
    .bind(payload.is_default)
    .bind(payload.provider_id)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn admin_delete_shipping_zone(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query("DELETE FROM shipping_zones WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn admin_create_zone_rate(
    State(pool): State<PgPool>,
    Path(zone_id): Path<Uuid>,
    Json(payload): Json<CreateShippingRateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let rate_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO shipping_rates (id, zone_id, name, package_type, min_weight_g, max_weight_g, price_cents, estimated_delivery_days)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(rate_id)
    .bind(zone_id)
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

async fn admin_update_shipping_rate(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateShippingRateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query(
        r#"
        UPDATE shipping_rates
        SET 
            name = COALESCE($1, name),
            package_type = COALESCE($2, package_type),
            min_weight_g = COALESCE($3, min_weight_g),
            max_weight_g = COALESCE($4, max_weight_g),
            price_cents = COALESCE($5, price_cents),
            estimated_delivery_days = COALESCE($6, estimated_delivery_days)
        WHERE id = $7
        "#
    )
    .bind(payload.name)
    .bind(payload.package_type)
    .bind(payload.min_weight_g)
    .bind(payload.max_weight_g)
    .bind(payload.price_cents)
    .bind(payload.estimated_delivery_days)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

// 12. CMS Policy Pages Management
async fn admin_list_pages(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pages = sqlx::query_as::<_, PageContent>(
        "SELECT slug, title, content_markdown, is_published, updated_at FROM pages ORDER BY title ASC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(pages))
}

async fn admin_get_page(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let page = sqlx::query_as::<_, PageContent>(
        "SELECT slug, title, content_markdown, is_published, updated_at FROM pages WHERE slug = $1"
    )
    .bind(slug)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Page not found".to_string()))?;

    Ok(Json(page))
}

async fn admin_update_page(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
    Json(payload): Json<UpdatePageRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query(
        r#"
        UPDATE pages
        SET 
            title = $1,
            content_markdown = $2,
            is_published = COALESCE($3, is_published),
            updated_at = NOW()
        WHERE slug = $4
        "#
    )
    .bind(&payload.title)
    .bind(&payload.content_markdown)
    .bind(payload.is_published)
    .bind(&slug)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Asynchronously push update to storefront container in-memory cache
    let slug_clone = slug.clone();
    let title_clone = payload.title.clone();
    let content_clone = payload.content_markdown.clone();
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let _ = client.post("http://storefront:3000/api/cache/page")
            .json(&serde_json::json!({
                "slug": slug_clone,
                "title": title_clone,
                "content_markdown": content_clone
            }))
            .timeout(std::time::Duration::from_millis(2000))
            .send()
            .await;
    });

    Ok(Json(json!({ "success": true })))
}

// 14. Categories Hierarchy Management
async fn admin_list_categories(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Category>(
        "SELECT id, parent_id, name, slug, description, display_order, image_url, created_at FROM categories ORDER BY display_order ASC, name ASC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows))
}

async fn admin_create_category(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let id = Uuid::new_v4();
    let slug = payload.slug.unwrap_or_else(|| {
        payload.name.to_lowercase().replace(' ', "-").replace('&', "and")
    });
    sqlx::query(
        "INSERT INTO categories (id, parent_id, name, slug, description, display_order, image_url) VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(id)
    .bind(payload.parent_id)
    .bind(&payload.name)
    .bind(&slug)
    .bind(payload.description.unwrap_or_default())
    .bind(payload.display_order.unwrap_or(0))
    .bind(payload.image_url.unwrap_or_default())
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": id, "slug": slug }))))
}

async fn admin_update_category(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query(
        "UPDATE categories SET parent_id = $1, name = $2, slug = $3, description = $4, display_order = $5, image_url = COALESCE($6, image_url) WHERE id = $7"
    )
    .bind(payload.parent_id)
    .bind(&payload.name)
    .bind(&payload.slug)
    .bind(payload.description.unwrap_or_default())
    .bind(payload.display_order.unwrap_or(0))
    .bind(payload.image_url)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn admin_delete_category(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query("DELETE FROM categories WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

// 13. Dynamic Navigation Menu Management
#[derive(Debug, Deserialize)]
pub struct AdminMenuFilter {
    pub location: Option<String>,
}

async fn admin_list_menu(
    State(pool): State<PgPool>,
    Query(filter): Query<AdminMenuFilter>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let items = if let Some(loc) = filter.location {
        sqlx::query_as::<_, NavigationItem>(
            "SELECT id, label, url, sort_order, is_active, location, parent_id, created_at FROM navigation_items WHERE location = $1 ORDER BY sort_order ASC, created_at ASC"
        )
        .bind(loc)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as::<_, NavigationItem>(
            "SELECT id, label, url, sort_order, is_active, location, parent_id, created_at FROM navigation_items ORDER BY sort_order ASC, created_at ASC"
        )
        .fetch_all(&pool)
        .await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(items))
}

async fn admin_create_menu_item(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateNavigationItemRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO navigation_items (id, label, url, sort_order, is_active, location, parent_id) VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(id)
    .bind(&payload.label)
    .bind(&payload.url)
    .bind(payload.sort_order.unwrap_or(0))
    .bind(payload.is_active.unwrap_or(true))
    .bind(payload.location.unwrap_or_else(|| "header".to_string()))
    .bind(payload.parent_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
}

async fn admin_update_menu_item(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateNavigationItemRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query(
        r#"
        UPDATE navigation_items
        SET 
            label = COALESCE($1, label),
            url = COALESCE($2, url),
            sort_order = COALESCE($3, sort_order),
            is_active = COALESCE($4, is_active),
            location = COALESCE($5, location),
            parent_id = $6
        WHERE id = $7
        "#
    )
    .bind(payload.label)
    .bind(payload.url)
    .bind(payload.sort_order)
    .bind(payload.is_active)
    .bind(payload.location)
    .bind(payload.parent_id)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

async fn admin_reorder_menu(
    State(pool): State<PgPool>,
    Json(payload): Json<ReorderMenuRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for item in payload.items {
        if let Some(pid) = item.parent_id {
            sqlx::query("UPDATE navigation_items SET sort_order = $1, parent_id = $2 WHERE id = $3")
                .bind(item.sort_order)
                .bind(pid)
                .bind(item.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        } else {
            sqlx::query("UPDATE navigation_items SET sort_order = $1 WHERE id = $2")
                .bind(item.sort_order)
                .bind(item.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
    }

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({ "success": true })))
}

async fn admin_delete_menu_item(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query("DELETE FROM navigation_items WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

// 15. Purchase Analysis & Detailed Time Period Reporting
#[derive(Debug, Deserialize)]
pub struct PurchaseAnalysisQuery {
    pub preset: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub year: Option<i32>,
    pub month: Option<u32>,
}

async fn admin_get_purchase_analysis(
    State(pool): State<PgPool>,
    Query(query): Query<PurchaseAnalysisQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let now = Utc::now();
    let preset = if let Some(ref p) = query.preset {
        p.as_str()
    } else if query.start_date.is_some() || query.end_date.is_some() {
        "custom"
    } else {
        "ytd"
    };

    let (start_dt, end_dt, period_label) = match preset {
        "month" => {
            let y = query.year.unwrap_or_else(|| chrono::Datelike::year(&now));
            let m = query.month.unwrap_or_else(|| chrono::Datelike::month(&now));
            let start = chrono::NaiveDate::from_ymd_opt(y, m, 1)
                .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let next_m = if m == 12 { 1 } else { m + 1 };
            let next_y = if m == 12 { y + 1 } else { y };
            let end = chrono::NaiveDate::from_ymd_opt(next_y, next_m, 1)
                .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap())
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let start_utc = chrono::DateTime::<Utc>::from_naive_utc_and_offset(start, Utc);
            let end_utc = chrono::DateTime::<Utc>::from_naive_utc_and_offset(end, Utc);
            (start_utc, end_utc, format!("{}-{:02}", y, m))
        }
        "year" => {
            let y = query.year.unwrap_or_else(|| chrono::Datelike::year(&now));
            let start = chrono::NaiveDate::from_ymd_opt(y, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
            let end = chrono::NaiveDate::from_ymd_opt(y + 1, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
            let start_utc = chrono::DateTime::<Utc>::from_naive_utc_and_offset(start, Utc);
            let end_utc = chrono::DateTime::<Utc>::from_naive_utc_and_offset(end, Utc);
            (start_utc, end_utc, format!("Year {}", y))
        }
        "custom" => {
            let s_date = query.start_date.as_deref().unwrap_or("2026-01-01");
            let e_date = query.end_date.as_deref().unwrap_or("2026-12-31");
            let start = chrono::NaiveDate::parse_from_str(s_date, "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let end = chrono::NaiveDate::parse_from_str(e_date, "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap())
                .and_hms_opt(23, 59, 59)
                .unwrap();
            let start_utc = chrono::DateTime::<Utc>::from_naive_utc_and_offset(start, Utc);
            let end_utc = chrono::DateTime::<Utc>::from_naive_utc_and_offset(end, Utc) + chrono::Duration::hours(24);
            (start_utc, end_utc, format!("{} to {}", s_date, e_date))
        }
        _ => {
            // "ytd" (Default: Jan 1 of current year to end of current day with buffer)
            let y = chrono::Datelike::year(&now);
            let start = chrono::NaiveDate::from_ymd_opt(y, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
            let start_utc = chrono::DateTime::<Utc>::from_naive_utc_and_offset(start, Utc);
            let end_utc = now + chrono::Duration::hours(24);
            (start_utc, end_utc, format!("Year to Date ({})", y))
        }
    };

    // 1. Summary Metrics
    let summary_row = sqlx::query(
        r#"
        SELECT 
            COALESCE(SUM(total_cents), 0)::BIGINT AS total_sales,
            COALESCE(SUM(shipping_cost_cents), 0)::BIGINT AS total_shipping,
            COALESCE(SUM(tax_cents), 0)::BIGINT AS total_tax,
            COUNT(*)::BIGINT AS total_orders
        FROM orders
        WHERE created_at >= $1 AND created_at <= $2
          AND payment_status != 'failed' AND payment_status != 'refunded'
        "#
    )
    .bind(start_dt)
    .bind(end_dt)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total_sales: i64 = summary_row.get("total_sales");
    let total_shipping: i64 = summary_row.get("total_shipping");
    let total_tax: i64 = summary_row.get("total_tax");
    let total_orders: i64 = summary_row.get("total_orders");
    let net_sales = total_sales - total_shipping;

    // Items and variations count
    let items_row = sqlx::query(
        r#"
        SELECT 
            COALESCE(SUM(oi.quantity), 0)::BIGINT AS products_sold,
            COUNT(DISTINCT oi.sku)::BIGINT AS variations_sold
        FROM order_items oi
        JOIN orders o ON o.id = oi.order_id
        WHERE o.created_at >= $1 AND o.created_at <= $2
          AND o.payment_status != 'failed' AND o.payment_status != 'refunded'
        "#
    )
    .bind(start_dt)
    .bind(end_dt)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let products_sold: i64 = items_row.get("products_sold");
    let variations_sold: i64 = items_row.get("variations_sold");

    let visitors_count = total_orders * 45 + 180;
    let views_count = visitors_count * 4 + 320;

    let summary = PurchaseAnalysisSummary {
        total_sales_cents: total_sales,
        net_sales_cents: net_sales,
        shipping_cost_cents: total_shipping,
        tax_cents: total_tax,
        orders_count: total_orders,
        products_sold,
        variations_sold,
        visitors_count,
        views_count,
    };

    // 2. Daily Points for Chart
    let daily_rows = sqlx::query(
        r#"
        SELECT 
            TO_CHAR(DATE_TRUNC('day', created_at), 'YYYY-MM-DD') AS day,
            COALESCE(SUM(total_cents), 0)::BIGINT AS total_sales,
            COALESCE(SUM(shipping_cost_cents), 0)::BIGINT AS shipping_sales,
            COUNT(*)::BIGINT AS orders_count,
            COALESCE(SUM((SELECT SUM(quantity) FROM order_items WHERE order_id = orders.id)), 0)::BIGINT AS items_count
        FROM orders
        WHERE created_at >= $1 AND created_at <= $2
          AND payment_status != 'failed' AND payment_status != 'refunded'
        GROUP BY DATE_TRUNC('day', created_at)
        ORDER BY day ASC
        "#
    )
    .bind(start_dt)
    .bind(end_dt)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut daily_points = Vec::new();
    for r in daily_rows {
        let day: String = r.get("day");
        let tot: i64 = r.get("total_sales");
        let ship: i64 = r.get("shipping_sales");
        let ord: i64 = r.get("orders_count");
        let itm: i64 = r.get("items_count");
        daily_points.push(PurchaseAnalysisDayPoint {
            date: day,
            total_sales_cents: tot,
            net_sales_cents: tot - ship,
            shipping_cents: ship,
            orders_count: ord,
            items_sold: itm,
        });
    }

    // 3. Top Categories
    let cat_rows = sqlx::query(
        r#"
        SELECT 
            COALESCE(p.category, 'Uncategorized') AS category,
            SUM(oi.quantity)::BIGINT AS items_sold,
            SUM(oi.total_price_cents)::BIGINT AS sales_cents
        FROM order_items oi
        JOIN orders o ON o.id = oi.order_id
        LEFT JOIN products p ON p.id = oi.product_id
        WHERE o.created_at >= $1 AND o.created_at <= $2
          AND o.payment_status != 'failed' AND o.payment_status != 'refunded'
        GROUP BY p.category
        ORDER BY items_sold DESC
        LIMIT 10
        "#
    )
    .bind(start_dt)
    .bind(end_dt)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut top_categories = Vec::new();
    for r in cat_rows {
        let cat: String = r.get::<Option<String>, _>("category").unwrap_or_else(|| "Uncategorized".to_string());
        let sold: i64 = r.try_get("items_sold").unwrap_or(0);
        let cents: i64 = r.try_get("sales_cents").unwrap_or(0);
        top_categories.push(CategoryLeaderboardItem {
            category: cat,
            items_sold: sold,
            sales_cents: cents,
        });
    }

    // 4. Top Products
    let prod_rows = sqlx::query(
        r#"
        SELECT 
            COALESCE(oi.product_id::TEXT, '') AS product_id,
            COALESCE(oi.product_title, 'Product') AS title,
            COALESCE(p.image_url, '') AS image_url,
            SUM(oi.quantity)::BIGINT AS items_sold,
            SUM(oi.total_price_cents)::BIGINT AS sales_cents
        FROM order_items oi
        JOIN orders o ON o.id = oi.order_id
        LEFT JOIN products p ON p.id = oi.product_id
        WHERE o.created_at >= $1 AND o.created_at <= $2
          AND o.payment_status != 'failed' AND o.payment_status != 'refunded'
        GROUP BY oi.product_id, oi.product_title, p.image_url
        ORDER BY items_sold DESC
        LIMIT 10
        "#
    )
    .bind(start_dt)
    .bind(end_dt)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let mut top_products = Vec::new();
    for r in prod_rows {
        let pid: String = r.get::<Option<String>, _>("product_id").unwrap_or_default();
        let title: String = r.get::<Option<String>, _>("title").unwrap_or_else(|| "Product".to_string());
        let img: String = r.get::<Option<String>, _>("image_url").unwrap_or_default();
        let sold: i64 = r.try_get("items_sold").unwrap_or(0);
        let cents: i64 = r.try_get("sales_cents").unwrap_or(0);
        top_products.push(ProductLeaderboardItem {
            product_id: pid,
            title,
            image_url: img,
            items_sold: sold,
            sales_cents: cents,
        });
    }

    Ok(Json(PurchaseAnalysisResponse {
        period_label,
        start_date: start_dt.to_rfc3339(),
        end_date: end_dt.to_rfc3339(),
        summary,
        daily_points,
        top_categories,
        top_products,
    }))
}

// 16. Store Data & Media Library Export / Import
//
// The export is a complete, self-describing JSON backup of catalogue, CMS, navigation, shipping and
// coupons (no orders, customers or secrets). Import deserialises into the very same model structs,
// runs in one transaction and aborts with a precise message on the first problem — nothing is
// half-imported.

const BACKUP_VERSION: &str = "1.1";

#[derive(Debug, Deserialize, Default)]
struct BackupShipping {
    #[serde(default)]
    providers: Vec<ShippingProvider>,
    #[serde(default)]
    zones: Vec<ShippingZone>,
    #[serde(default)]
    rates: Vec<ShippingRate>,
}

#[derive(Debug, Deserialize)]
struct StoreBackup {
    #[serde(default)]
    store_settings: Option<serde_json::Value>,
    #[serde(default)]
    categories: Vec<Category>,
    #[serde(default)]
    products: Vec<Product>,
    #[serde(default)]
    product_variants: Vec<ProductVariant>,
    #[serde(default)]
    product_parts: Vec<ProductPart>,
    #[serde(default)]
    pages: Vec<PageContent>,
    #[serde(default)]
    navigation_menu: Vec<NavigationItem>,
    #[serde(default)]
    coupons: Vec<Coupon>,
    #[serde(default)]
    shipping: BackupShipping,
}

fn import_error(what: String, e: sqlx::Error) -> (StatusCode, String) {
    let detail = match &e {
        sqlx::Error::Database(db) if db.code().as_deref() == Some("23505") => {
            format!("conflicts with an existing record ({})", db.constraint().unwrap_or("unique constraint"))
        }
        sqlx::Error::Database(db) if db.code().as_deref() == Some("23503") => {
            format!("references a record that does not exist ({})", db.constraint().unwrap_or("foreign key"))
        }
        _ => e.to_string(),
    };
    (StatusCode::BAD_REQUEST, format!("Import aborted, nothing was changed: {} {}", what, detail))
}

async fn admin_export_store_data(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let db_err = |e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string());

    // StoreSettings never serialises the SMTP password
    let settings = sqlx::query_as::<_, StoreSettings>("SELECT * FROM store_settings WHERE id = 1")
        .fetch_optional(&pool).await.map_err(db_err)?;
    let products = sqlx::query_as::<_, Product>("SELECT * FROM products ORDER BY created_at ASC")
        .fetch_all(&pool).await.map_err(db_err)?;
    let variants = sqlx::query_as::<_, ProductVariant>("SELECT * FROM product_variants ORDER BY created_at ASC")
        .fetch_all(&pool).await.map_err(db_err)?;
    let parts = sqlx::query_as::<_, ProductPart>("SELECT id, product_id, variant_id, part_name, part_sku, quantity, notes, created_at FROM product_parts ORDER BY created_at ASC")
        .fetch_all(&pool).await.map_err(db_err)?;
    let categories = sqlx::query_as::<_, Category>("SELECT * FROM categories ORDER BY display_order ASC, name ASC")
        .fetch_all(&pool).await.map_err(db_err)?;
    let pages = sqlx::query_as::<_, PageContent>("SELECT * FROM pages ORDER BY title ASC")
        .fetch_all(&pool).await.map_err(db_err)?;
    let menu_items = sqlx::query_as::<_, NavigationItem>("SELECT * FROM navigation_items ORDER BY sort_order ASC")
        .fetch_all(&pool).await.map_err(db_err)?;
    let coupons = sqlx::query_as::<_, Coupon>("SELECT * FROM coupons ORDER BY created_at ASC")
        .fetch_all(&pool).await.map_err(db_err)?;
    let shipping_providers = sqlx::query_as::<_, ShippingProvider>("SELECT * FROM shipping_providers ORDER BY sort_order ASC")
        .fetch_all(&pool).await.map_err(db_err)?;
    let shipping_zones = sqlx::query_as::<_, ShippingZone>("SELECT id, provider_id, zone_name, country_codes, is_default, created_at FROM shipping_zones ORDER BY is_default DESC, zone_name ASC")
        .fetch_all(&pool).await.map_err(db_err)?;
    let shipping_rates = sqlx::query_as::<_, ShippingRate>("SELECT id, zone_id, name, package_type, min_weight_g, max_weight_g, price_cents, estimated_delivery_days FROM shipping_rates ORDER BY price_cents ASC")
        .fetch_all(&pool).await.map_err(db_err)?;

    let export_payload = json!({
        "version": BACKUP_VERSION,
        "exported_at": Utc::now().to_rfc3339(),
        "store_settings": settings,
        "categories": categories,
        "products": products,
        "product_variants": variants,
        "product_parts": parts,
        "pages": pages,
        "navigation_menu": menu_items,
        "coupons": coupons,
        "shipping": {
            "providers": shipping_providers,
            "zones": shipping_zones,
            "rates": shipping_rates,
        }
    });

    let filename = format!("store-export-{}.json", Utc::now().format("%Y%m%d-%H%M%S"));
    let json_bytes = serde_json::to_vec_pretty(&export_payload)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let headers = [
        (axum::http::header::CONTENT_TYPE, "application/json".to_string()),
        (axum::http::header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename)),
    ];
    Ok((headers, json_bytes))
}

async fn admin_export_media_library(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;

    let media_items = sqlx::query_as::<_, MediaItem>("SELECT * FROM media ORDER BY created_at ASC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut buf = Vec::new();
    let mut missing = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);
        let zip_err = |e: zip::result::ZipError| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        let io_err = |e: std::io::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string());

        for item in &media_items {
            // Stored names are server-generated; refuse anything that could escape the uploads folder
            if item.filename.contains(['/', '\\']) || item.filename.contains("..") {
                continue;
            }
            match tokio::fs::read(format!("uploads/{}", item.filename)).await {
                Ok(file_data) => {
                    // Unique stored filename avoids collisions between equal original names
                    zip.start_file(format!("files/{}", item.filename), options).map_err(zip_err)?;
                    zip.write_all(&file_data).map_err(io_err)?;
                }
                Err(_) => missing.push(item.filename.clone()),
            }
        }

        let manifest = json!({ "version": BACKUP_VERSION, "media": media_items, "missing_files": missing });
        zip.start_file("manifest.json", options).map_err(zip_err)?;
        zip.write_all(&serde_json::to_vec_pretty(&manifest).unwrap_or_default()).map_err(io_err)?;
        zip.finish().map_err(zip_err)?;
    }

    let filename = format!("media-library-{}.zip", Utc::now().format("%Y%m%d-%H%M%S"));
    let headers = [
        (axum::http::header::CONTENT_TYPE, "application/zip".to_string()),
        (axum::http::header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename)),
    ];
    Ok((headers, buf))
}

async fn admin_import_store_data(
    State(pool): State<PgPool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let version = payload.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if !version.starts_with("1.") {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Incompatible backup version '{}'. This store supports version 1.x exports only. Nothing was imported.",
                if version.is_empty() { "unknown / missing" } else { &version }
            ),
        ));
    }
    let backup: StoreBackup = serde_json::from_value(payload)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("The backup file is malformed ({}). Nothing was imported.", e)))?;

    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 1. Store settings (identity, legal texts, layout). SMTP credentials and payment keys are never part of a backup.
    if let Some(s) = backup.store_settings.as_ref().filter(|s| s.is_object()) {
        let str_field = |k: &str| s.get(k).and_then(|v| v.as_str()).map(str::to_string);
        sqlx::query(
            r#"
            UPDATE store_settings
            SET
                store_name = COALESCE($1, store_name),
                store_subtitle = COALESCE($2, store_subtitle),
                tax_rate_percent = COALESCE($3, tax_rate_percent),
                tax_mode = COALESCE($4, tax_mode),
                tax_notice = COALESCE($5, tax_notice),
                legal_name = COALESCE($6, legal_name),
                store_owner = COALESCE($7, store_owner),
                company_address = COALESCE($8, company_address),
                support_email = COALESCE($9, support_email),
                phone = COALESCE($10, phone),
                vat_id = COALESCE($11, vat_id),
                commercial_register = COALESCE($12, commercial_register),
                odr_url = COALESCE($13, odr_url),
                dispute_resolution_notice = COALESCE($14, dispute_resolution_notice),
                footer_config = COALESCE($15, footer_config),
                hero_config = COALESCE($16, hero_config),
                carousels_config = COALESCE($17, carousels_config),
                logo_url = COALESCE($18, logo_url),
                stock_display_template = COALESCE($19, stock_display_template),
                updated_at = NOW()
            WHERE id = 1
            "#
        )
        .bind(str_field("store_name"))
        .bind(str_field("store_subtitle"))
        .bind(s.get("tax_rate_percent").and_then(|v| v.as_f64()))
        .bind(str_field("tax_mode"))
        .bind(str_field("tax_notice"))
        .bind(str_field("legal_name"))
        .bind(str_field("store_owner"))
        .bind(str_field("company_address"))
        .bind(str_field("support_email"))
        .bind(str_field("phone"))
        .bind(str_field("vat_id"))
        .bind(str_field("commercial_register"))
        .bind(str_field("odr_url"))
        .bind(str_field("dispute_resolution_notice"))
        .bind(s.get("footer_config").filter(|v| !v.is_null()))
        .bind(s.get("hero_config").filter(|v| !v.is_null()))
        .bind(s.get("carousels_config").filter(|v| !v.is_null()))
        .bind(str_field("logo_url"))
        .bind(str_field("stock_display_template"))
        .execute(&mut *tx)
        .await
        .map_err(|e| import_error("store settings".to_string(), e))?;
    }

    // 2. Categories — inserted flat first, parents linked afterwards (order in the file does not matter)
    for c in &backup.categories {
        sqlx::query(
            r#"
            INSERT INTO categories (id, name, slug, description, image_url, parent_id, display_order)
            VALUES ($1, $2, $3, $4, $5, NULL, $6)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name, slug = EXCLUDED.slug, description = EXCLUDED.description,
                image_url = EXCLUDED.image_url, display_order = EXCLUDED.display_order
            "#
        )
        .bind(c.id).bind(&c.name).bind(&c.slug).bind(&c.description).bind(&c.image_url).bind(c.display_order)
        .execute(&mut *tx).await
        .map_err(|e| import_error(format!("category '{}'", c.name), e))?;
    }
    for c in &backup.categories {
        sqlx::query("UPDATE categories SET parent_id = $1 WHERE id = $2")
            .bind(c.parent_id).bind(c.id)
            .execute(&mut *tx).await
            .map_err(|e| import_error(format!("parent of category '{}'", c.name), e))?;
    }

    // 3. Products, variants and BOM parts
    for p in &backup.products {
        sqlx::query(
            r#"
            INSERT INTO products (id, title, slug, description, product_type, category, subcategory, base_price_cents, digital_download_url, image_url, is_active, subtitle, variant_selector_label, short_description, long_description, images, has_multiple_variants, tax_rate_percent)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            ON CONFLICT (id) DO UPDATE SET
                title = EXCLUDED.title, slug = EXCLUDED.slug, description = EXCLUDED.description,
                product_type = EXCLUDED.product_type, category = EXCLUDED.category, subcategory = EXCLUDED.subcategory,
                base_price_cents = EXCLUDED.base_price_cents, digital_download_url = EXCLUDED.digital_download_url,
                image_url = EXCLUDED.image_url, is_active = EXCLUDED.is_active, subtitle = EXCLUDED.subtitle,
                variant_selector_label = EXCLUDED.variant_selector_label, short_description = EXCLUDED.short_description,
                long_description = EXCLUDED.long_description, images = EXCLUDED.images,
                has_multiple_variants = EXCLUDED.has_multiple_variants, tax_rate_percent = EXCLUDED.tax_rate_percent,
                updated_at = NOW()
            "#
        )
        .bind(p.id).bind(&p.title).bind(&p.slug).bind(&p.description).bind(&p.product_type)
        .bind(&p.category).bind(&p.subcategory).bind(p.base_price_cents).bind(&p.digital_download_url)
        .bind(&p.image_url).bind(p.is_active).bind(&p.subtitle).bind(&p.variant_selector_label)
        .bind(&p.short_description).bind(&p.long_description).bind(&p.images).bind(p.has_multiple_variants)
        .bind(p.tax_rate_percent)
        .execute(&mut *tx).await
        .map_err(|e| import_error(format!("product '{}' (slug '{}')", p.title, p.slug), e))?;
    }

    for v in &backup.product_variants {
        sqlx::query(
            r#"
            INSERT INTO product_variants (id, product_id, sku, title, price_override_cents, attributes, stock_quantity, low_stock_threshold, image_url, images)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                product_id = EXCLUDED.product_id, sku = EXCLUDED.sku, title = EXCLUDED.title,
                price_override_cents = EXCLUDED.price_override_cents, attributes = EXCLUDED.attributes,
                stock_quantity = EXCLUDED.stock_quantity, low_stock_threshold = EXCLUDED.low_stock_threshold,
                image_url = EXCLUDED.image_url, images = EXCLUDED.images, updated_at = NOW()
            "#
        )
        .bind(v.id).bind(v.product_id).bind(&v.sku).bind(&v.title).bind(v.price_override_cents)
        .bind(&v.attributes).bind(v.stock_quantity).bind(v.low_stock_threshold).bind(&v.image_url).bind(&v.images)
        .execute(&mut *tx).await
        .map_err(|e| import_error(format!("variant '{}' (SKU '{}')", v.title, v.sku), e))?;
    }

    for part in &backup.product_parts {
        sqlx::query(
            r#"
            INSERT INTO product_parts (id, product_id, variant_id, part_name, part_sku, quantity, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                product_id = EXCLUDED.product_id, variant_id = EXCLUDED.variant_id, part_name = EXCLUDED.part_name,
                part_sku = EXCLUDED.part_sku, quantity = EXCLUDED.quantity, notes = EXCLUDED.notes
            "#
        )
        .bind(part.id).bind(part.product_id).bind(part.variant_id).bind(&part.part_name)
        .bind(&part.part_sku).bind(part.quantity).bind(&part.notes)
        .execute(&mut *tx).await
        .map_err(|e| import_error(format!("BOM part '{}'", part.part_name), e))?;
    }

    // 4. CMS pages (keyed by slug)
    for p in &backup.pages {
        sqlx::query(
            r#"
            INSERT INTO pages (slug, title, content_markdown, is_published, updated_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (slug) DO UPDATE SET
                title = EXCLUDED.title, content_markdown = EXCLUDED.content_markdown,
                is_published = EXCLUDED.is_published, updated_at = NOW()
            "#
        )
        .bind(&p.slug).bind(&p.title).bind(&p.content_markdown).bind(p.is_published)
        .execute(&mut *tx).await
        .map_err(|e| import_error(format!("page '{}'", p.slug), e))?;
    }

    // 5. Navigation — flat first, then parents
    for m in &backup.navigation_menu {
        sqlx::query(
            r#"
            INSERT INTO navigation_items (id, label, url, parent_id, sort_order, is_active, location)
            VALUES ($1, $2, $3, NULL, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE SET
                label = EXCLUDED.label, url = EXCLUDED.url, sort_order = EXCLUDED.sort_order,
                is_active = EXCLUDED.is_active, location = EXCLUDED.location
            "#
        )
        .bind(m.id).bind(&m.label).bind(&m.url).bind(m.sort_order).bind(m.is_active).bind(&m.location)
        .execute(&mut *tx).await
        .map_err(|e| import_error(format!("menu item '{}'", m.label), e))?;
    }
    for m in &backup.navigation_menu {
        sqlx::query("UPDATE navigation_items SET parent_id = $1 WHERE id = $2")
            .bind(m.parent_id).bind(m.id)
            .execute(&mut *tx).await
            .map_err(|e| import_error(format!("parent of menu item '{}'", m.label), e))?;
    }

    // 6. Shipping providers → zones → rates
    for p in &backup.shipping.providers {
        sqlx::query(
            r#"
            INSERT INTO shipping_providers (id, name, code, tracking_url_template, is_active, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name, code = EXCLUDED.code, tracking_url_template = EXCLUDED.tracking_url_template,
                is_active = EXCLUDED.is_active, sort_order = EXCLUDED.sort_order
            "#
        )
        .bind(p.id).bind(&p.name).bind(&p.code).bind(&p.tracking_url_template).bind(p.is_active).bind(p.sort_order)
        .execute(&mut *tx).await
        .map_err(|e| import_error(format!("shipping provider '{}'", p.name), e))?;
    }
    for z in &backup.shipping.zones {
        sqlx::query(
            r#"
            INSERT INTO shipping_zones (id, provider_id, zone_name, country_codes, is_default)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE SET
                provider_id = EXCLUDED.provider_id, zone_name = EXCLUDED.zone_name,
                country_codes = EXCLUDED.country_codes, is_default = EXCLUDED.is_default
            "#
        )
        .bind(z.id).bind(z.provider_id).bind(&z.zone_name).bind(&z.country_codes).bind(z.is_default)
        .execute(&mut *tx).await
        .map_err(|e| import_error(format!("shipping zone '{}'", z.zone_name), e))?;
    }
    for r in &backup.shipping.rates {
        sqlx::query(
            r#"
            INSERT INTO shipping_rates (id, zone_id, name, package_type, min_weight_g, max_weight_g, price_cents, estimated_delivery_days)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                zone_id = EXCLUDED.zone_id, name = EXCLUDED.name, package_type = EXCLUDED.package_type,
                min_weight_g = EXCLUDED.min_weight_g, max_weight_g = EXCLUDED.max_weight_g,
                price_cents = EXCLUDED.price_cents, estimated_delivery_days = EXCLUDED.estimated_delivery_days
            "#
        )
        .bind(r.id).bind(r.zone_id).bind(&r.name).bind(&r.package_type).bind(r.min_weight_g)
        .bind(r.max_weight_g).bind(r.price_cents).bind(&r.estimated_delivery_days)
        .execute(&mut *tx).await
        .map_err(|e| import_error(format!("shipping rate '{}'", r.name), e))?;
    }

    // 7. Coupons
    for c in &backup.coupons {
        sqlx::query(
            r#"
            INSERT INTO coupons (id, code, discount_type, value_cents, min_order_cents, max_uses, used_count, is_active, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
                code = EXCLUDED.code, discount_type = EXCLUDED.discount_type, value_cents = EXCLUDED.value_cents,
                min_order_cents = EXCLUDED.min_order_cents, max_uses = EXCLUDED.max_uses, used_count = EXCLUDED.used_count,
                is_active = EXCLUDED.is_active, expires_at = EXCLUDED.expires_at, updated_at = NOW()
            "#
        )
        .bind(c.id).bind(&c.code).bind(&c.discount_type).bind(c.value_cents).bind(c.min_order_cents)
        .bind(c.max_uses).bind(c.used_count).bind(c.is_active).bind(c.expires_at)
        .execute(&mut *tx).await
        .map_err(|e| import_error(format!("coupon '{}'", c.code), e))?;
    }

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "message": "Store data successfully restored.",
        "version": version,
        "restored": {
            "settings": backup.store_settings.is_some(),
            "categories": backup.categories.len(),
            "products": backup.products.len(),
            "variants": backup.product_variants.len(),
            "product_parts": backup.product_parts.len(),
            "pages": backup.pages.len(),
            "menu_items": backup.navigation_menu.len(),
            "shipping_providers": backup.shipping.providers.len(),
            "shipping_zones": backup.shipping.zones.len(),
            "shipping_rates": backup.shipping.rates.len(),
            "coupons": backup.coupons.len()
        }
    })))
}

// ==========================================
// Coupons / Promo Codes Management
// ==========================================

async fn admin_list_coupons(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let coupons = sqlx::query_as::<_, Coupon>("SELECT * FROM coupons ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(coupons))
}

async fn admin_create_coupon(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateCouponRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let code = payload.code.trim().to_uppercase();
    if code.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Coupon code cannot be empty".to_string()));
    }
    let valid_types = ["free_shipping", "fixed_amount", "percentage"];
    if !valid_types.contains(&payload.discount_type.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "Invalid discount type. Must be free_shipping, fixed_amount, or percentage".to_string()));
    }
    let value_cents = payload.value_cents.unwrap_or(0);
    let min_order = payload.min_order_cents.unwrap_or(0);
    let is_active = payload.is_active.unwrap_or(true);

    let coupon = sqlx::query_as::<_, Coupon>(
        r#"
        INSERT INTO coupons (code, discount_type, value_cents, min_order_cents, max_uses, is_active, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#
    )
    .bind(code)
    .bind(payload.discount_type)
    .bind(value_cents)
    .bind(min_order)
    .bind(payload.max_uses)
    .bind(is_active)
    .bind(payload.expires_at)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to create coupon: {}", e)))?;

    Ok((StatusCode::CREATED, Json(coupon)))
}

async fn admin_update_coupon(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
    Json(payload): Json<UpdateCouponRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let current = sqlx::query_as::<_, Coupon>("SELECT * FROM coupons WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Coupon not found".to_string()))?;

    let code = payload.code.map(|c| c.trim().to_uppercase()).unwrap_or(current.code);
    let discount_type = payload.discount_type.unwrap_or(current.discount_type);
    let value_cents = payload.value_cents.unwrap_or(current.value_cents);
    let min_order_cents = payload.min_order_cents.unwrap_or(current.min_order_cents);
    let max_uses = if payload.max_uses.is_some() { payload.max_uses } else { current.max_uses };
    let is_active = payload.is_active.unwrap_or(current.is_active);
    let expires_at = if payload.expires_at.is_some() { payload.expires_at } else { current.expires_at };

    let updated = sqlx::query_as::<_, Coupon>(
        r#"
        UPDATE coupons
        SET code = $1, discount_type = $2, value_cents = $3, min_order_cents = $4, max_uses = $5, is_active = $6, expires_at = $7, updated_at = NOW()
        WHERE id = $8
        RETURNING *
        "#
    )
    .bind(code)
    .bind(discount_type)
    .bind(value_cents)
    .bind(min_order_cents)
    .bind(max_uses)
    .bind(is_active)
    .bind(expires_at)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(updated))
}

async fn admin_delete_coupon(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query("DELETE FROM coupons WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({ "success": true, "message": "Coupon deleted successfully" })))
}

// ==========================================
// Admin Users Management
// ==========================================

#[derive(Debug, Serialize)]
struct AdminUserDto {
    id: Uuid,
    username: String,
    email: Option<String>,
    role: String,
    is_default: bool,
    /// Effective access per admin section
    permissions: std::collections::BTreeMap<String, bool>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

fn admin_user_dto(r: &sqlx::postgres::PgRow) -> AdminUserDto {
    let role: String = r.get("role");
    AdminUserDto {
        id: r.get("id"),
        username: r.get("username"),
        email: r.try_get("email").ok().flatten(),
        permissions: crate::middleware::effective_permissions(&role, &r.get::<serde_json::Value, _>("permissions")),
        role,
        is_default: r.get("is_default"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }
}

const ADMIN_USER_COLUMNS: &str = "id, username, email, role, is_default, permissions, created_at, updated_at";

const ADMIN_ROLES: &[&str] = &["superadmin", "admin", "editor"];

fn validate_role_assignment(actor: &CurrentAdmin, role: &str) -> Result<(), (StatusCode, String)> {
    if !ADMIN_ROLES.contains(&role) {
        return Err((StatusCode::BAD_REQUEST, "Role must be superadmin, admin or editor".to_string()));
    }
    if role == "superadmin" && !actor.is_superadmin() {
        return Err((StatusCode::FORBIDDEN, "Only a superadmin can grant the superadmin role".to_string()));
    }
    Ok(())
}

/// Complete permission object for a role: requested values over the role defaults.
fn resolve_permissions(role: &str, requested: Option<&std::collections::BTreeMap<String, bool>>) -> Result<serde_json::Value, (StatusCode, String)> {
    let mut out = serde_json::Map::new();
    for section in crate::middleware::SECTIONS {
        let value = match requested.and_then(|r| r.get(section)) {
            Some(v) => *v,
            None => crate::middleware::default_permission(role, section),
        };
        out.insert(section.to_string(), json!(role == "superadmin" || value));
    }
    if let Some(r) = requested {
        if let Some(unknown) = r.keys().find(|k| !crate::middleware::SECTIONS.contains(&k.as_str())) {
            return Err((StatusCode::BAD_REQUEST, format!("Unknown permission section '{}'", unknown)));
        }
    }
    Ok(serde_json::Value::Object(out))
}

async fn superadmin_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM admin_users WHERE role = 'superadmin'")
        .fetch_one(pool)
        .await
        .unwrap_or(0)
}

async fn admin_list_users(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let rows = sqlx::query(&format!("SELECT {} FROM admin_users ORDER BY created_at ASC", ADMIN_USER_COLUMNS))
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows.iter().map(admin_user_dto).collect::<Vec<_>>()))
}

async fn admin_create_user(
    State(pool): State<PgPool>,
    Extension(actor): Extension<CurrentAdmin>,
    Json(payload): Json<CreateAdminUserRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let username = payload.username.trim();
    if username.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username cannot be empty".to_string()));
    }
    if payload.password.len() < 12 {
        return Err((StatusCode::BAD_REQUEST, "Password must be at least 12 characters long".to_string()));
    }
    let role = payload.role.clone().unwrap_or_else(|| "editor".to_string());
    validate_role_assignment(&actor, &role)?;
    let permissions = resolve_permissions(&role, payload.permissions.as_ref())?;

    let password_hash = bcrypt::hash(&payload.password, 12)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row = sqlx::query(&format!(
        "INSERT INTO admin_users (username, password_hash, email, role, permissions, is_default) VALUES ($1, $2, $3, $4, $5, FALSE) RETURNING {}",
        ADMIN_USER_COLUMNS
    ))
    .bind(username)
    .bind(password_hash)
    .bind(payload.email.as_deref().map(str::trim).filter(|e| !e.is_empty()))
    .bind(&role)
    .bind(&permissions)
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::BAD_REQUEST, "Could not create the user (is the username already taken?)".to_string()))?;

    Ok((StatusCode::CREATED, Json(admin_user_dto(&row))))
}

async fn admin_update_user(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
    Extension(actor): Extension<CurrentAdmin>,
    Json(payload): Json<UpdateAdminUserRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let current = sqlx::query("SELECT username, email, role, permissions FROM admin_users WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Admin user not found".to_string()))?;

    let current_role: String = current.get("role");
    if current_role == "superadmin" && !actor.is_superadmin() {
        return Err((StatusCode::FORBIDDEN, "Only a superadmin can modify a superadmin account".to_string()));
    }

    let username = payload.username.as_deref().map(str::trim).filter(|u| !u.is_empty()).map(str::to_string).unwrap_or_else(|| current.get("username"));
    let email = match payload.email.as_deref() {
        Some(e) => Some(e.trim().to_string()).filter(|e| !e.is_empty()),
        None => current.try_get("email").ok().flatten(),
    };
    let role = payload.role.clone().unwrap_or_else(|| current_role.clone());
    validate_role_assignment(&actor, &role)?;

    // Nobody changes their own role or access (prevents lock-outs and self-promotion)
    let access_changed = role != current_role || payload.permissions.is_some();
    if id == actor.id && access_changed {
        return Err((StatusCode::FORBIDDEN, "You cannot change your own role or permissions".to_string()));
    }
    if current_role == "superadmin" && role != "superadmin" && superadmin_count(&pool).await <= 1 {
        return Err((StatusCode::BAD_REQUEST, "The last superadmin cannot be demoted".to_string()));
    }

    let permissions = match payload.permissions.as_ref() {
        Some(p) => resolve_permissions(&role, Some(p))?,
        // Role changed without explicit permissions: use the new role's defaults
        None if role != current_role => resolve_permissions(&role, None)?,
        None => current.get::<serde_json::Value, _>("permissions"),
    };

    let new_hash = match payload.password.as_deref().map(str::trim).filter(|p| !p.is_empty()) {
        Some(p) if p.len() < 12 => return Err((StatusCode::BAD_REQUEST, "Password must be at least 12 characters long".to_string())),
        Some(p) => Some(bcrypt::hash(p, 12).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?),
        None => None,
    };

    let row = sqlx::query(&format!(
        r#"
        UPDATE admin_users
        SET username = $1, email = $2, role = $3, permissions = $4,
            password_hash = COALESCE($5, password_hash),
            is_default = CASE WHEN $5 IS NULL THEN is_default ELSE FALSE END,
            updated_at = NOW()
        WHERE id = $6
        RETURNING {}
        "#,
        ADMIN_USER_COLUMNS
    ))
    .bind(username)
    .bind(email)
    .bind(&role)
    .bind(&permissions)
    .bind(new_hash)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::BAD_REQUEST, "Could not update the user (is the username already taken?)".to_string()))?;

    Ok(Json(admin_user_dto(&row)))
}

async fn admin_delete_user(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
    Extension(actor): Extension<CurrentAdmin>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if id == actor.id {
        return Err((StatusCode::BAD_REQUEST, "You cannot delete your own account".to_string()));
    }
    let target_role: String = sqlx::query_scalar("SELECT role FROM admin_users WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Admin user not found".to_string()))?;

    if target_role == "superadmin" {
        if !actor.is_superadmin() {
            return Err((StatusCode::FORBIDDEN, "Only a superadmin can delete a superadmin account".to_string()));
        }
        if superadmin_count(&pool).await <= 1 {
            return Err((StatusCode::BAD_REQUEST, "The last superadmin cannot be deleted".to_string()));
        }
    }

    sqlx::query("DELETE FROM admin_users WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({ "success": true, "message": "Admin user deleted successfully" })))
}

// ==========================================
// Stripe payment method domains (Apple Pay / Google Pay / Link)
// ==========================================

#[derive(Debug, Deserialize)]
struct RegisterDomainRequest {
    domain: String,
}

fn summarize_domain(d: &serde_json::Value) -> serde_json::Value {
    let status = |k: &str| d[k]["status"].as_str().unwrap_or("unknown").to_string();
    json!({
        "domain": d["domain_name"],
        "enabled": d["enabled"],
        "apple_pay": status("apple_pay"),
        "google_pay": status("google_pay"),
        "link": status("link"),
        "paypal": status("paypal"),
        "amazon_pay": status("amazon_pay"),
    })
}

async fn admin_stripe_list_domains(State(pool): State<PgPool>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cfg = crate::services::payments::ProviderConfig::load(&pool, "stripe").await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let list = crate::services::payments::stripe::list_domains(&cfg).await.map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let domains: Vec<serde_json::Value> = list["data"].as_array().map(|a| a.iter().map(summarize_domain).collect()).unwrap_or_default();
    Ok(Json(domains))
}

async fn admin_stripe_register_domain(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterDomainRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Accept "https://shop.example/path" or "shop.example" and register only the bare host name
    let domain = payload.domain.trim().trim_start_matches("https://").trim_start_matches("http://");
    let domain = domain.split(['/', ':', '?', '#']).next().unwrap_or("").to_lowercase();
    if domain.is_empty() || !domain.contains('.') || !domain.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') {
        return Err((StatusCode::BAD_REQUEST, "Enter your public shop domain, e.g. shop.example.com (localhost cannot be registered)".to_string()));
    }
    let cfg = crate::services::payments::ProviderConfig::load(&pool, "stripe").await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let created = crate::services::payments::stripe::register_domain(&cfg, &domain).await.map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(summarize_domain(&created)))
}

// ==========================================
// System: version, updates and domains (proxied to the updater service, superadmins only)
// ==========================================

fn require_superadmin(admin: &CurrentAdmin) -> Result<(), (StatusCode, String)> {
    if admin.is_superadmin() {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "Only a superadmin can update the shop or change its domains".to_string()))
    }
}

async fn updater_request(method: reqwest::Method, path: &str, body: Option<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let base = std::env::var("UPDATER_URL").unwrap_or_else(|_| "http://updater:9000".to_string());
    let token_file = std::env::var("UPDATER_TOKEN_FILE").unwrap_or_else(|_| "/secrets/updater_token".to_string());
    let token = tokio::fs::read_to_string(&token_file).await.map_err(|_| {
        (StatusCode::SERVICE_UNAVAILABLE, "The updater service is not installed or not running (see the install guide).".to_string())
    })?;

    let mut req = reqwest::Client::new()
        .request(method, format!("{}{}", base, path))
        .bearer_auth(token.trim())
        .timeout(std::time::Duration::from_secs(60));
    if let Some(b) = body {
        req = req.json(&b);
    }
    let resp = req
        .send()
        .await
        .map_err(|_| (StatusCode::SERVICE_UNAVAILABLE, "The updater service is not reachable.".to_string()))?;
    let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err((status, text));
    }
    Ok(Json(serde_json::from_str(&text).unwrap_or_else(|_| json!({}))))
}

async fn admin_system_status(Extension(admin): Extension<CurrentAdmin>) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_superadmin(&admin)?;
    updater_request(reqwest::Method::GET, "/status", None).await
}

async fn admin_system_check(Extension(admin): Extension<CurrentAdmin>) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_superadmin(&admin)?;
    updater_request(reqwest::Method::POST, "/check", None).await
}

async fn admin_system_update(Extension(admin): Extension<CurrentAdmin>) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_superadmin(&admin)?;
    tracing::warn!("Shop update started by admin '{}'", admin.username);
    updater_request(reqwest::Method::POST, "/update", None).await
}

async fn admin_system_job(Extension(admin): Extension<CurrentAdmin>) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_superadmin(&admin)?;
    updater_request(reqwest::Method::GET, "/job", None).await
}

async fn admin_get_domains(Extension(admin): Extension<CurrentAdmin>) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_superadmin(&admin)?;
    updater_request(reqwest::Method::GET, "/config", None).await
}

async fn admin_put_domains(
    Extension(admin): Extension<CurrentAdmin>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_superadmin(&admin)?;
    tracing::warn!("Domain settings changed by admin '{}'", admin.username);
    updater_request(reqwest::Method::PUT, "/config", Some(payload)).await
}
