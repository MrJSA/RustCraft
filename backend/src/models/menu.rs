use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NavigationItem {
    pub id: Uuid,
    pub label: String,
    pub url: String,
    pub sort_order: i32,
    pub is_active: bool,
    pub location: String, // 'header' or 'footer'
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNavigationItemRequest {
    pub label: String,
    pub url: String,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
    pub location: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNavigationItemRequest {
    pub label: Option<String>,
    pub url: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
    pub location: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderMenuItem {
    pub id: Uuid,
    pub sort_order: i32,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderMenuRequest {
    pub items: Vec<ReorderMenuItem>,
}

