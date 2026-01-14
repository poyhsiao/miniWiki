use serde::{Deserialize, Serialize};
use validator::Validate;

// ============================================
// Request Types
// ============================================

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateDocumentRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,

    #[validate(length(max = 50))]
    pub icon: Option<String>,

    pub parent_id: Option<String>,

    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateDocumentRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,

    #[validate(length(max = 50))]
    pub icon: Option<String>,

    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListDocumentsQuery {
    pub parent_id: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateVersionRequest {
    pub content: serde_json::Value,

    #[validate(length(min = 1, max = 200))]
    pub title: String,

    #[validate(length(max = 500))]
    pub change_summary: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListVersionsQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreVersionRequest {
    pub version_number: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
}

// ============================================
// Response Types
// ============================================

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentResponse {
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
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentListResponse {
    pub documents: Vec<DocumentResponse>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentResponse {
    pub id: String,
    pub message: String,
    pub document: DocumentResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionResponse {
    pub id: String,
    pub document_id: String,
    pub version_number: i32,
    pub title: String,
    pub content: serde_json::Value,
    pub created_by: String,
    pub created_at: String,
    pub change_summary: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionListResponse {
    pub versions: Vec<VersionResponse>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVersionResponse {
    pub id: String,
    pub version_number: i32,
    pub message: String,
    pub version: VersionResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreVersionResponse {
    pub document: DocumentResponse,
    pub message: String,
    pub restored_from_version: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChildrenResponse {
    pub documents: Vec<DocumentResponse>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentPathItem {
    pub id: String,
    pub title: String,
    pub level: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentPathResponse {
    pub path: Vec<DocumentPathItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionDiffResponse {
    pub from_version: i32,
    pub to_version: i32,
    pub from_content: serde_json::Value,
    pub to_content: serde_json::Value,
}

// ============================================
// Error Response Types
// ============================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
}

// ============================================
// API Response Wrapper
// ============================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiErrorResponse>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error_code: &str, message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiErrorResponse {
                error: error_code.to_string(),
                message: message.to_string(),
            }),
        }
    }
}

// ============================================
// Space Request Types
// ============================================

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateSpaceRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: String,

    #[validate(length(max = 50))]
    pub icon: Option<String>,

    pub description: Option<String>,

    pub is_public: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateSpaceRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,

    #[validate(length(max = 50))]
    pub icon: Option<String>,

    pub description: Option<String>,

    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AddMemberRequest {
    pub user_id: String,

    #[validate(length(min = 1, max = 20))]
    #[serde(rename = "role")]
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateMemberRequest {
    #[validate(length(min = 1, max = 20))]
    #[serde(rename = "role")]
    pub role: String,
}

// ============================================
// Space Response Types
// ============================================

#[derive(Debug, Serialize, Deserialize)]
pub struct SpaceResponse {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: String,
    pub updated_at: String,
    pub user_role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpaceListResponse {
    pub spaces: Vec<SpaceResponse>,
    pub total: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberResponse {
    pub id: String,
    pub space_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
    pub invited_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberListResponse {
    pub members: Vec<MemberResponse>,
    pub total: i32,
}
