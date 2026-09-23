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
