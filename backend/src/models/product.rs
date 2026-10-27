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
    #[serde(default)]
    #[sqlx(default)]
    pub weight_grams: Option<i32>,
    /// Overrides the shop's default delivery time
    #[serde(default)]
    #[sqlx(default)]
    pub delivery_time: Option<String>,
    #[serde(default)]
    #[sqlx(default)]
    pub sale_starts_at: Option<DateTime<Utc>>,
    #[serde(default)]
    #[sqlx(default)]
    pub sale_ends_at: Option<DateTime<Utc>>,
    /// GPSR: manufacturer name, postal and electronic address
    #[serde(default)]
    #[sqlx(default)]
    pub gpsr_manufacturer: Option<String>,
    /// GPSR: responsible person in the EU (if the manufacturer is outside the EU)
    #[serde(default)]
    #[sqlx(default)]
    pub gpsr_responsible_person: Option<String>,
    /// GPSR: warnings and safety information
    #[serde(default)]
    #[sqlx(default)]
    pub gpsr_safety_info: Option<String>,
    /// Net content for the unit price (§ 4 PAngV), e.g. 0.25 with unit "kg"
    #[serde(default)]
    #[sqlx(default)]
    pub unit_content: Option<f64>,
    #[serde(default)]
    #[sqlx(default)]
    pub unit_content_unit: Option<String>,
    /// Package size in mm (for choosing the shipping package)
    #[serde(default)]
    #[sqlx(default)]
    pub length_mm: Option<i32>,
    #[serde(default)]
    #[sqlx(default)]
    pub width_mm: Option<i32>,
    #[serde(default)]
    #[sqlx(default)]
    pub height_mm: Option<i32>,
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
    #[serde(default)]
    #[sqlx(default)]
    pub weight_grams: Option<i32>,
    #[serde(default)]
    #[sqlx(default)]
    pub sale_price_cents: Option<i32>,
    #[serde(default)]
    #[sqlx(default)]
    pub length_mm: Option<i32>,
    #[serde(default)]
    #[sqlx(default)]
    pub width_mm: Option<i32>,
    #[serde(default)]
    #[sqlx(default)]
    pub height_mm: Option<i32>,
    /// Effective price for the storefront (sale, regular and lowest price of the last 30 days)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[sqlx(skip)]
    pub pricing: Option<crate::services::pricing::VariantPricing>,
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
    #[serde(default)]
    pub extras: Option<ProductExtras>,
}

/// Shipping weight, delivery time, sale window, product safety (GPSR) and unit price data.
/// When sent, every field is saved as given (null clears it).
#[derive(Debug, Deserialize, Default, Clone)]
pub struct ProductExtras {
    pub weight_grams: Option<i32>,
    pub length_mm: Option<i32>,
    pub width_mm: Option<i32>,
    pub height_mm: Option<i32>,
    pub delivery_time: Option<String>,
    pub sale_starts_at: Option<DateTime<Utc>>,
    pub sale_ends_at: Option<DateTime<Utc>>,
    pub gpsr_manufacturer: Option<String>,
    pub gpsr_responsible_person: Option<String>,
    pub gpsr_safety_info: Option<String>,
    pub unit_content: Option<f64>,
    pub unit_content_unit: Option<String>,
}

/// Variant weight and sale price; when sent, both are saved as given (null clears).
#[derive(Debug, Deserialize, Default, Clone)]
pub struct VariantExtras {
    pub weight_grams: Option<i32>,
    #[serde(default)]
    pub length_mm: Option<i32>,
    #[serde(default)]
    pub width_mm: Option<i32>,
    #[serde(default)]
    pub height_mm: Option<i32>,
    pub sale_price_cents: Option<i32>,
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
    #[serde(default)]
    pub extras: Option<ProductExtras>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductPart {
    pub id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub part_id: Option<Uuid>,
    pub part_name: String,
    pub part_sku: Option<String>,
    pub quantity: i32,
    pub notes: Option<String>,
    pub storage_location: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub stock_quantity: i32,
    #[serde(default)]
    pub low_stock_threshold: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreatePartRequest {
    pub variant_id: Option<Uuid>,
    pub part_id: Option<Uuid>,
    pub part_name: Option<String>,
    pub part_sku: Option<String>,
    pub quantity: i32,
    pub notes: Option<String>,
    pub storage_location: Option<String>,
    pub stock_quantity: Option<i32>,
    pub low_stock_threshold: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BomPart {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub storage_location: Option<String>,
    pub stock_quantity: i32,
    pub low_stock_threshold: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBomPartRequest {
    pub sku: String,
    pub name: String,
    pub storage_location: Option<String>,
    pub stock_quantity: Option<i32>,
    pub low_stock_threshold: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBomPartRequest {
    pub sku: Option<String>,
    pub name: Option<String>,
    pub storage_location: Option<String>,
    pub stock_quantity: Option<i32>,
    pub low_stock_threshold: Option<i32>,
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
    #[serde(default)]
    pub extras: Option<VariantExtras>,
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
    #[serde(default)]
    pub extras: Option<VariantExtras>,
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

