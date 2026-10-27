use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub display_name: String,
    pub preferred_currency: String,
    pub phone: String,
    pub is_verified: bool,
    pub verification_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerProfileDTO {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub display_name: String,
    pub preferred_currency: String,
    pub phone: String,
    pub is_verified: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomerProfileRequest {
    pub first_name: String,
    pub last_name: String,
    pub display_name: String,
    pub email: String,
    pub preferred_currency: String,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CustomerChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomerAddress {
    pub id: Uuid,
    pub customer_email: String,
    pub address_type: String, // 'shipping' or 'billing'
    pub full_name: String,
    pub street_address: String,
    pub apartment_suite: Option<String>,
    pub city: String,
    pub state_province: String,
    pub postal_code: String,
    pub country_code: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SaveAddressRequest {
    pub address_type: String,
    pub full_name: String,
    pub street_address: String,
    pub apartment_suite: Option<String>,
    pub city: String,
    pub state_province: String,
    pub postal_code: String,
    pub country_code: String,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomerWishlistItem {
    pub id: Uuid,
    pub customer_email: String,
    pub product_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ToggleWishlistRequest {
    pub product_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CustomerRegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CustomerLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
/// Step 1: `{ email }` requests a reset link. Step 2: `{ token, new_password }` sets the new password.
pub struct ResetPasswordRequest {
    pub email: Option<String>,
    pub token: Option<String>,
    pub new_password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CustomerAuthResponse {
    pub token: String,
    pub email: String,
    pub full_name: String,
    pub first_name: String,
    pub last_name: String,
    pub preferred_currency: String,
    pub is_verified: bool,
}
