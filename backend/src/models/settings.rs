use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StoreSettings {
    pub id: i32,
    pub store_name: String,
    pub currency: String,
    pub currency_symbol: String,
    pub tax_rate_percent: f64,
    pub deployment_mode: String, // 'development', 'staging', 'production', 'demo'
    pub debug_mode: bool,
    pub support_email: String,
    pub company_address: String,
    pub vat_id: String,
    pub logo_url: String,
    pub phone: String,
    pub hero_config: serde_json::Value,
    pub smtp_host: String,
    pub smtp_port: i32,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_encryption: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub smtp_enabled: bool,
    pub require_registered_checkout: bool,
    pub require_email_verification: bool,
    pub store_subtitle: String,
    pub show_store_title: bool,
    pub show_store_subtitle: bool,
    pub carousels_config: serde_json::Value,
    pub cookie_banner_enabled: bool,
    pub cookie_banner_title: String,
    pub cookie_banner_description: String,
    pub cookie_banner_policy_url: String,
    pub cookie_accept_label: String,
    pub cookie_deny_label: String,
    pub cookie_preferences_label: String,
    pub tax_notice: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreSettingsDTO {
    pub store_name: String,
    pub currency: String,
    pub currency_symbol: String,
    pub tax_rate_percent: f64,
    pub deployment_mode: String,
    pub debug_mode: bool,
    pub support_email: String,
    pub company_address: String,
    pub vat_id: String,
    pub logo_url: String,
    pub phone: String,
    pub hero_config: serde_json::Value,
    pub smtp_host: String,
    pub smtp_port: i32,
    pub smtp_username: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub smtp_enabled: bool,
    pub require_registered_checkout: bool,
    pub require_email_verification: bool,
    pub store_subtitle: String,
    pub show_store_title: bool,
    pub show_store_subtitle: bool,
    pub carousels_config: serde_json::Value,
    pub cookie_banner_enabled: bool,
    pub cookie_banner_title: String,
    pub cookie_banner_description: String,
    pub cookie_banner_policy_url: String,
    pub cookie_accept_label: String,
    pub cookie_deny_label: String,
    pub cookie_preferences_label: String,
    pub tax_notice: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStoreSettingsRequest {
    pub store_name: Option<String>,
    pub currency: Option<String>,
    pub currency_symbol: Option<String>,
    pub tax_rate_percent: Option<f64>,
    pub deployment_mode: Option<String>,
    pub debug_mode: Option<bool>,
    pub support_email: Option<String>,
    pub company_address: Option<String>,
    pub vat_id: Option<String>,
    pub logo_url: Option<String>,
    pub phone: Option<String>,
    pub hero_config: Option<serde_json::Value>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<i32>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_encryption: Option<String>,
    pub smtp_from_email: Option<String>,
    pub smtp_from_name: Option<String>,
    pub smtp_enabled: Option<bool>,
    pub require_registered_checkout: Option<bool>,
    pub require_email_verification: Option<bool>,
    pub store_subtitle: Option<String>,
    pub show_store_title: Option<bool>,
    pub show_store_subtitle: Option<bool>,
    pub carousels_config: Option<serde_json::Value>,
    pub cookie_banner_enabled: Option<bool>,
    pub cookie_banner_title: Option<String>,
    pub cookie_banner_description: Option<String>,
    pub cookie_banner_policy_url: Option<String>,
    pub cookie_accept_label: Option<String>,
    pub cookie_deny_label: Option<String>,
    pub cookie_preferences_label: Option<String>,
    pub tax_notice: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TestEmailRequest {
    pub recipient_email: String,
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub gross_sales_cents: i64,
    pub total_orders: i64,
    pub average_order_value_cents: i64,
    pub total_skus: i64,
    pub low_stock_skus: i64,
    pub pending_orders: i64,
    pub processing_orders: i64,
    pub shipped_orders: i64,
}

#[derive(Debug, Serialize)]
pub struct SalesDataPoint {
    pub date: String,
    pub orders_count: i64,
    pub sales_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct PurchaseAnalysisSummary {
    pub total_sales_cents: i64,
    pub net_sales_cents: i64,
    pub shipping_cost_cents: i64,
    pub tax_cents: i64,
    pub orders_count: i64,
    pub products_sold: i64,
    pub variations_sold: i64,
    pub visitors_count: i64,
    pub views_count: i64,
}

#[derive(Debug, Serialize)]
pub struct PurchaseAnalysisDayPoint {
    pub date: String,
    pub total_sales_cents: i64,
    pub net_sales_cents: i64,
    pub shipping_cents: i64,
    pub orders_count: i64,
    pub items_sold: i64,
}

#[derive(Debug, Serialize)]
pub struct CategoryLeaderboardItem {
    pub category: String,
    pub items_sold: i64,
    pub sales_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct ProductLeaderboardItem {
    pub product_id: String,
    pub title: String,
    pub image_url: String,
    pub items_sold: i64,
    pub sales_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct PurchaseAnalysisResponse {
    pub period_label: String,
    pub start_date: String,
    pub end_date: String,
    pub summary: PurchaseAnalysisSummary,
    pub daily_points: Vec<PurchaseAnalysisDayPoint>,
    pub top_categories: Vec<CategoryLeaderboardItem>,
    pub top_products: Vec<ProductLeaderboardItem>,
}
