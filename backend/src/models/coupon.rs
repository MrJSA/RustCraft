use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Coupon {
    pub id: Uuid,
    pub code: String,
    pub discount_type: String, // 'free_shipping', 'fixed_amount', 'percentage'
    pub value_cents: i32,
    pub min_order_cents: i32,
    pub max_uses: Option<i32>,
    pub used_count: i32,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCouponRequest {
    pub code: String,
    pub discount_type: String,
    pub value_cents: Option<i32>,
    pub min_order_cents: Option<i32>,
    pub max_uses: Option<i32>,
    pub is_active: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCouponRequest {
    pub code: Option<String>,
    pub discount_type: Option<String>,
    pub value_cents: Option<i32>,
    pub min_order_cents: Option<i32>,
    pub max_uses: Option<i32>,
    pub is_active: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateCouponRequest {
    pub code: String,
    pub subtotal_cents: i32,
    pub shipping_cost_cents: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ValidateCouponResponse {
    pub valid: bool,
    pub code: String,
    pub discount_type: String,
    pub discount_cents: i32,
    pub message: String,
}
