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
    // Write-only: never sent to browsers or included in exports
    #[serde(skip_serializing, default)]
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
    pub tax_mode: String,
    pub legal_name: String,
    pub store_owner: String,
    pub commercial_register: String,
    pub dispute_resolution_notice: String,
    pub odr_url: String,
    pub footer_config: serde_json::Value,
    pub order_prefix_enabled: bool,
    pub order_prefix: String,
    pub order_date_enabled: bool,
    pub stock_display_template: String,
    #[serde(default)]
    pub address_street: String,
    #[serde(default)]
    pub address_house_number: String,
    #[serde(default)]
    pub address_extra: String,
    #[serde(default)]
    pub address_postal_code: String,
    #[serde(default)]
    pub address_city: String,
    #[serde(default)]
    pub address_country: String,
    #[serde(default = "default_true")]
    pub low_stock_alerts_enabled: bool,
    #[serde(default = "default_stock_managers")]
    pub low_stock_alert_recipients_mode: String,
    #[serde(default)]
    pub low_stock_alert_custom_emails: String,
    #[serde(default = "default_json_array")]
    pub low_stock_alert_selected_user_ids: serde_json::Value,
    /// Steuernummer — required on invoices without a VAT ID (e.g. small businesses)
    #[serde(default)]
    #[sqlx(default)]
    pub tax_number: String,
    /// Attach the invoice as PDF to the payment confirmation email
    #[serde(default = "default_true")]
    #[sqlx(default)]
    pub email_invoice_pdf: bool,
    /// Make that PDF a ZUGFeRD / Factur-X e-invoice (EN 16931 XML embedded)
    #[serde(default)]
    #[sqlx(default)]
    pub einvoice_enabled: bool,
    #[serde(default)]
    #[sqlx(default)]
    pub reviews_enabled: bool,
    #[serde(default)]
    #[sqlx(default)]
    pub newsletter_enabled: bool,
    #[serde(default)]
    #[sqlx(default)]
    pub sale_prices_enabled: bool,
    /// VAT-free sales to EU businesses with a VIES-checked VAT ID
    #[serde(default)]
    #[sqlx(default)]
    pub reverse_charge_enabled: bool,
    /// Downloads per file and days after payment (0 = unlimited)
    #[serde(default = "default_download_limit")]
    #[sqlx(default)]
    pub download_limit: i32,
    #[serde(default)]
    #[sqlx(default)]
    pub download_days: i32,
    #[serde(default)]
    #[sqlx(default)]
    pub default_delivery_time: String,
    /// GPSR manufacturer details used when a product has none of its own
    #[serde(default)]
    #[sqlx(default)]
    pub gpsr_default_manufacturer: String,
    pub updated_at: DateTime<Utc>,
}

fn default_true() -> bool {
    true
}
fn default_download_limit() -> i32 {
    10
}
fn default_stock_managers() -> String {
    "stock_managers".to_string()
}
fn default_json_array() -> serde_json::Value {
    serde_json::json!([])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreSettingsDTO {
    pub store_name: String,
    pub currency: String,
    pub currency_symbol: String,
    pub tax_rate_percent: f64,
    pub tax_mode: String,
    pub deployment_mode: String,
    pub debug_mode: bool,
    pub support_email: String,
    pub company_address: String,
    pub vat_id: String,
    pub logo_url: String,
    pub phone: String,
    pub hero_config: serde_json::Value,
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
    pub legal_name: String,
    pub store_owner: String,
    pub commercial_register: String,
    pub dispute_resolution_notice: String,
    pub odr_url: String,
    pub footer_config: serde_json::Value,
    pub order_prefix_enabled: bool,
    pub order_prefix: String,
    pub order_date_enabled: bool,
    pub stock_display_template: String,
    pub address_street: String,
    pub address_house_number: String,
    pub address_extra: String,
    pub address_postal_code: String,
    pub address_city: String,
    pub address_country: String,
    pub tax_number: String,
    pub reviews_enabled: bool,
    pub newsletter_enabled: bool,
    pub sale_prices_enabled: bool,
    /// EU businesses can order VAT-free with a VIES-checked VAT ID
    pub eu_business_vat_free: bool,
    pub default_delivery_time: String,
    pub download_limit: i32,
    pub download_days: i32,
    pub gpsr_default_manufacturer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateStoreSettingsRequest {
    pub store_name: Option<String>,
    pub currency: Option<String>,
    pub currency_symbol: Option<String>,
    pub tax_rate_percent: Option<f64>,
    pub tax_mode: Option<String>,
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
    pub legal_name: Option<String>,
    pub store_owner: Option<String>,
    pub commercial_register: Option<String>,
    pub dispute_resolution_notice: Option<String>,
    pub odr_url: Option<String>,
    pub footer_config: Option<serde_json::Value>,
    pub order_prefix_enabled: Option<bool>,
    pub order_prefix: Option<String>,
    pub order_date_enabled: Option<bool>,
    pub stock_display_template: Option<String>,
    pub address_street: Option<String>,
    pub address_house_number: Option<String>,
    pub address_extra: Option<String>,
    pub address_postal_code: Option<String>,
    pub address_city: Option<String>,
    pub address_country: Option<String>,
    pub low_stock_alerts_enabled: Option<bool>,
    pub low_stock_alert_recipients_mode: Option<String>,
    pub low_stock_alert_custom_emails: Option<String>,
    pub low_stock_alert_selected_user_ids: Option<serde_json::Value>,
    pub tax_number: Option<String>,
    pub email_invoice_pdf: Option<bool>,
    pub einvoice_enabled: Option<bool>,
    pub reviews_enabled: Option<bool>,
    pub newsletter_enabled: Option<bool>,
    pub sale_prices_enabled: Option<bool>,
    pub reverse_charge_enabled: Option<bool>,
    pub download_limit: Option<i32>,
    pub download_days: Option<i32>,
    pub default_delivery_time: Option<String>,
    pub gpsr_default_manufacturer: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TestEmailRequest {
    #[serde(alias = "recipient", alias = "to_email")]
    pub recipient_email: String,
    #[serde(default)]
    pub email_type: Option<String>,
    /// Unsaved values from the settings form; an empty password keeps the stored one
    #[serde(default)]
    pub smtp: Option<SmtpOverride>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SmtpOverride {
    pub host: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub encryption: Option<String>,
    pub from_email: Option<String>,
    pub from_name: Option<String>,
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
