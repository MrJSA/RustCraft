use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminUser {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AdminLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AdminLoginResponse {
    pub token: String,
    pub username: String,
    pub is_default: bool,
}

#[derive(Debug, Deserialize)]
pub struct ChangeAdminCredentialsRequest {
    pub current_password: String,
    pub new_username: Option<String>,
    pub new_password: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAdminUserRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub permissions: Option<std::collections::BTreeMap<String, bool>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAdminUserRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub permissions: Option<std::collections::BTreeMap<String, bool>>,
}

