use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub order_number: String,
    pub customer_name: String,
    pub customer_email: String,
    pub shipping_address: JsonValue,
    pub billing_address: JsonValue,
    pub shipping_rate_id: Option<Uuid>,
    pub shipping_cost_cents: i32,
    pub subtotal_cents: i32,
    pub tax_cents: i32,
    pub total_cents: i32,
    pub payment_provider: String,
    pub payment_status: String,
    pub order_status: String,
    pub tracking_number: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Uuid,
    pub product_title: String,
    pub variant_title: String,
    pub sku: String,
    pub unit_price_cents: i32,
    pub quantity: i32,
    pub total_price_cents: i32,
    pub is_digital: bool,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDetails {
    #[serde(flatten)]
    pub order: Order,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Deserialize)]
pub struct CartItemInput {
    pub variant_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutRequest {
    pub customer_name: String,
    pub customer_email: String,
    pub shipping_address: AddressInput,
    pub billing_address: Option<AddressInput>,
    pub shipping_rate_id: Option<Uuid>,
    pub payment_provider: String, // 'stripe', 'paypal', 'apple_pay', 'google_pay', 'amazon_pay'
    pub payment_token: Option<String>,
    pub items: Vec<CartItemInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInput {
    pub full_name: String,
    pub street_address: String,
    pub apartment_suite: Option<String>,
    pub city: String,
    pub state_province: Option<String>,
    pub postal_code: String,
    pub country_code: String,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub order_number: String,
    pub total_cents: i32,
    pub payment_status: String,
    pub order_status: String,
    pub payment_redirect_url: Option<String>,
    pub digital_items: Vec<DigitalDownloadItem>,
}

#[derive(Debug, Serialize)]
pub struct DigitalDownloadItem {
    pub title: String,
    pub download_url: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub order_status: Option<String>,
    pub payment_status: Option<String>,
    pub tracking_number: Option<String>,
    pub notes: Option<String>,
}
