use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MediaItem {
    pub id: Uuid,
    pub filename: String,
    pub original_name: String,
    pub url: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
}
