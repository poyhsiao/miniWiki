use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub timezone: String,
    pub language: String,
    pub is_active: bool,
    pub is_email_verified: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Space {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SpaceMembership {
    pub id: Uuid,
    pub space_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: Uuid,
    pub space_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub icon: Option<String>,
    pub content: serde_json::Value,
    pub content_size: i32,
    pub is_archived: bool,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub last_edited_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentVersion {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version_number: i32,
    pub content: serde_json::Value,
    pub title: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub change_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShareLink {
    pub id: Uuid,
    pub document_id: Uuid,
    pub created_by: Uuid,
    /// Share token - sensitive, should not be serialized in general responses
    #[serde(skip_serializing)]
    pub token: String,
    /// Access code hash - sensitive, should not be serialized
    #[serde(skip_serializing)]
    pub access_code: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub permission: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub click_count: i32,
    pub max_access_count: Option<i32>,
}
