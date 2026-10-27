use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PageContent {
    pub slug: String,
    pub title: String,
    pub content_markdown: String,
    pub is_published: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePageRequest {
    pub title: String,
    pub content_markdown: String,
    pub is_published: Option<bool>,
}
