use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShippingZone {
    pub id: Uuid,
    pub zone_name: String,
    pub country_codes: JsonValue, // JSON array of country codes, e.g. ["DE", "FR"]
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShippingRate {
    pub id: Uuid,
    pub zone_id: Uuid,
    pub name: String,
    pub package_type: String, // 'standard', 'express', 'fragile', 'heavy'
    pub min_weight_g: i32,
    pub max_weight_g: i32,
    pub price_cents: i32,
    pub estimated_delivery_days: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingZoneWithRates {
    #[serde(flatten)]
    pub zone: ShippingZone,
    pub rates: Vec<ShippingRate>,
}

#[derive(Debug, Deserialize)]
pub struct EstimateShippingRequest {
    pub country_code: String,
    pub package_type: Option<String>,
    pub total_weight_g: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShippingZoneRequest {
    pub zone_name: String,
    pub country_codes: Vec<String>,
    pub is_default: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateShippingRateRequest {
    pub name: String,
    pub package_type: String,
    pub min_weight_g: Option<i32>,
    pub max_weight_g: Option<i32>,
    pub price_cents: i32,
    pub estimated_delivery_days: String,
}
