use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Space {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSpaceRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(max = 50))]
    pub icon: Option<String>,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSpaceRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,
    #[validate(length(max = 50))]
    pub icon: Option<String>,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SpaceMembership {
    pub id: Uuid,
    pub space_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: chrono::NaiveDateTime,
    pub invited_by: Uuid,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddMemberRequest {
    pub user_id: String,
    #[validate(length(min = 1, max = 50))]
    pub role: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMemberRequest {
    #[validate(length(min = 1, max = 50))]
    pub role: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SpaceError {
    #[error("Space not found")]
    NotFound,
    #[error("Access denied")]
    Forbidden,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
