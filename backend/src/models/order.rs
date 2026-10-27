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
    #[serde(default)]
    #[sqlx(default)]
    pub coupon_code: Option<String>,
    #[serde(default)]
    #[sqlx(default)]
    pub discount_cents: i32,
    #[serde(default)]
    #[sqlx(default)]
    pub payment_reference: Option<String>,
    /// Paid in test mode (Stripe test keys / PayPal sandbox) — excluded from analytics
    #[serde(default)]
    #[sqlx(default)]
    pub is_test: bool,
    /// Tax mode at the time of purchase (kleingewerbe / included / excluded)
    #[serde(default)]
    #[sqlx(default)]
    pub tax_mode: Option<String>,
    #[serde(default)]
    #[sqlx(default)]
    pub buyer_company: Option<String>,
    #[serde(default)]
    #[sqlx(default)]
    pub buyer_vat_id: Option<String>,
    /// VAT-free EU business order (intra-Community supply / reverse charge)
    #[serde(default)]
    #[sqlx(default)]
    pub reverse_charge: bool,
    /// VIES answer kept as proof
    #[serde(default)]
    #[sqlx(default)]
    pub vat_check: Option<JsonValue>,
    #[serde(default)]
    #[sqlx(default)]
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Option<Uuid>,
    pub variant_id: Option<Uuid>,
    pub product_title: String,
    pub variant_title: String,
    pub sku: String,
    pub unit_price_cents: i32,
    pub quantity: i32,
    pub total_price_cents: i32,
    pub is_digital: bool,
    pub download_url: Option<String>,
    /// VAT rate of this line at the time of purchase
    #[serde(default)]
    #[sqlx(default)]
    pub tax_rate_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDetails {
    #[serde(flatten)]
    pub order: Order,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItemInput {
    pub variant_id: Uuid,
    pub quantity: i32,
}

/// Everything needed to price and create an order. Card data never reaches this server:
/// payments are authorized client-side (Stripe Elements / PayPal) and verified server-side by reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutRequest {
    pub customer_name: String,
    pub customer_email: String,
    pub shipping_address: AddressInput,
    pub billing_address: Option<AddressInput>,
    pub shipping_rate_id: Option<Uuid>,
    pub coupon_code: Option<String>,
    pub items: Vec<CartItemInput>,
    /// Business customers: company name and EU VAT ID (VAT-free when valid and from another EU country)
    #[serde(default)]
    pub buyer_company: Option<String>,
    #[serde(default)]
    pub vat_id: Option<String>,
}

/// Server-computed price breakdown shown on the checkout page and charged by the payment provider.
#[derive(Debug, Serialize)]
pub struct CheckoutQuote {
    pub items_subtotal_cents: i32,
    pub discount_cents: i32,
    pub subtotal_cents: i32,
    pub shipping_cost_cents: i32,
    pub tax_cents: i32,
    pub total_cents: i32,
    pub is_digital_only: bool,
    pub coupon_code: Option<String>,
    pub currency: String,
    /// Physical goods weight for weight-based shipping rates
    pub total_weight_g: i32,
    /// Shipping rates the cart fits into for the destination (cheapest first)
    pub shipping_options: Vec<serde_json::Value>,
    /// VAT-free EU business order
    pub reverse_charge: bool,
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
    /// Secret that lets the buyer view this order without an account (never guessable)
    pub access_token: String,
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
