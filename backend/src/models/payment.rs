use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentConfig {
    pub provider: String, // 'stripe', 'paypal'
    pub display_name: String,
    pub is_enabled: bool,
    pub is_sandbox: bool,
    pub public_client_id: String,
    #[serde(skip_serializing)]
    pub secret_key: String,
    pub config_data: JsonValue,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicPaymentProviderInfo {
    pub provider: String,
    pub display_name: String,
    pub is_enabled: bool,
    pub is_sandbox: bool,
    pub public_client_id: String,
    pub config_data: JsonValue,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePaymentConfigRequest {
    pub display_name: Option<String>,
    pub is_enabled: Option<bool>,
    pub is_sandbox: Option<bool>,
    pub public_client_id: Option<String>,
    pub secret_key: Option<String>,
    pub webhook_secret: Option<String>,
    pub config_data: Option<JsonValue>,
}
