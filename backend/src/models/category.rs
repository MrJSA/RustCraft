use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub display_order: i32,
    pub image_url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTreeItem {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub display_order: i32,
    pub image_url: String,
    pub children: Vec<CategoryTreeItem>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub image_url: Option<String>,
}

