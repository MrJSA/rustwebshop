use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub product_type: String, // 'physical' or 'digital'
    pub category: String,
    pub subcategory: String,
    pub base_price_cents: i32,
    pub digital_download_url: Option<String>,
    pub image_url: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub subtitle: String,
    pub variant_selector_label: String,
    pub short_description: String,
    pub long_description: String,
    pub images: JsonValue,
    pub has_multiple_variants: bool,
    pub tax_rate_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductVariant {
    pub id: Uuid,
    pub product_id: Uuid,
    pub sku: String,
    pub title: String,
    pub price_override_cents: Option<i32>,
    pub attributes: JsonValue,
    pub stock_quantity: i32,
    pub low_stock_threshold: i32,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub images: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductWithVariants {
    #[serde(flatten)]
    pub product: Product,
    pub variants: Vec<ProductVariant>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub title: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub product_type: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub base_price_cents: Option<i32>,
    pub digital_download_url: Option<String>,
    pub image_url: Option<String>,
    pub subtitle: Option<String>,
    pub variant_selector_label: Option<String>,
    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub images: Option<JsonValue>,
    pub has_multiple_variants: Option<bool>,
    pub tax_rate_percent: Option<f64>,
    pub variants: Option<Vec<CreateVariantRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub product_type: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub base_price_cents: Option<i32>,
    pub digital_download_url: Option<String>,
    pub image_url: Option<String>,
    pub is_active: Option<bool>,
    pub subtitle: Option<String>,
    pub variant_selector_label: Option<String>,
    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub images: Option<JsonValue>,
    pub has_multiple_variants: Option<bool>,
    pub tax_rate_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductPart {
    pub id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub part_name: String,
    pub part_sku: Option<String>,
    pub quantity: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePartRequest {
    pub variant_id: Option<Uuid>,
    pub part_name: String,
    pub part_sku: Option<String>,
    pub quantity: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVariantRequest {
    pub sku: Option<String>,
    pub title: Option<String>,
    pub price_override_cents: Option<i32>,
    pub attributes: Option<JsonValue>,
    pub stock_quantity: Option<i32>,
    pub low_stock_threshold: Option<i32>,
    pub image_url: Option<String>,
    pub images: Option<JsonValue>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVariantRequest {
    pub sku: String,
    pub title: String,
    pub price_override_cents: Option<i32>,
    pub attributes: JsonValue,
    pub stock_quantity: i32,
    pub low_stock_threshold: Option<i32>,
    pub image_url: Option<String>,
    pub images: Option<JsonValue>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStockRequest {
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductWithVariantsAndParts {
    #[serde(flatten)]
    pub product: Product,
    pub variants: Vec<ProductVariant>,
    pub parts: Vec<ProductPart>,
}

