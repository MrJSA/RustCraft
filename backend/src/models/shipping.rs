use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShippingProvider {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub tracking_url_template: String,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShippingZone {
    pub id: Uuid,
    pub provider_id: Option<Uuid>,
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
    /// Maximum package size in mm (0 = no limit)
    #[serde(default)]
    #[sqlx(default)]
    pub max_length_mm: i32,
    #[serde(default)]
    #[sqlx(default)]
    pub max_width_mm: i32,
    #[serde(default)]
    #[sqlx(default)]
    pub max_height_mm: i32,
}

impl ShippingRate {
    pub fn limits(&self) -> crate::services::packing::RateLimits {
        crate::services::packing::RateLimits {
            min_weight_g: self.min_weight_g,
            max_weight_g: self.max_weight_g,
            max_dims: crate::services::packing::sorted_dims(Some(self.max_length_mm), Some(self.max_width_mm), Some(self.max_height_mm)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingZoneWithRates {
    #[serde(flatten)]
    pub zone: ShippingZone,
    pub rates: Vec<ShippingRate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingProviderWithZones {
    #[serde(flatten)]
    pub provider: ShippingProvider,
    pub zones: Vec<ShippingZoneWithRates>,
}

#[derive(Debug, Deserialize)]
pub struct EstimateShippingRequest {
    pub country_code: String,
    pub package_type: Option<String>,
    pub total_weight_g: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub code: String,
    pub tracking_url_template: Option<String>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub tracking_url_template: Option<String>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShippingZoneRequest {
    pub provider_id: Option<Uuid>,
    pub zone_name: String,
    pub country_codes: Vec<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShippingZoneRequest {
    pub zone_name: Option<String>,
    pub country_codes: Option<Vec<String>>,
    pub is_default: Option<bool>,
    pub provider_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShippingRateRequest {
    pub zone_id: Option<Uuid>,
    pub name: String,
    pub package_type: String,
    pub min_weight_g: Option<i32>,
    pub max_weight_g: Option<i32>,
    pub price_cents: i32,
    pub estimated_delivery_days: String,
    pub max_length_mm: Option<i32>,
    pub max_width_mm: Option<i32>,
    pub max_height_mm: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShippingRateRequest {
    pub name: Option<String>,
    pub package_type: Option<String>,
    pub min_weight_g: Option<i32>,
    pub max_weight_g: Option<i32>,
    pub price_cents: Option<i32>,
    pub estimated_delivery_days: Option<String>,
    pub max_length_mm: Option<i32>,
    pub max_width_mm: Option<i32>,
    pub max_height_mm: Option<i32>,
}

