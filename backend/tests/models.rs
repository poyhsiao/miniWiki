use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Space {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSpaceRequest {
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSpaceRequest {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpaceMembership {
    pub id: String,
    pub space_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
    pub invited_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMemberRequest {
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub space_id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub icon: Option<String>,
    pub content: serde_json::Value,
    pub content_size: i32,
    pub is_archived: bool,
    pub created_by: String,
    pub last_edited_by: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub icon: Option<String>,
    pub parent_id: Option<String>,
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub icon: Option<String>,
    pub parent_id: Option<String>,
    pub content: Option<serde_json::Value>,
}
