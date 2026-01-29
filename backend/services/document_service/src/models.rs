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

// ============================================
// Comment Request Types
// ============================================

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateCommentRequest {
    #[validate(length(min = 1, max = 5000))]
    pub content: String,

    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateCommentRequest {
    #[validate(length(min = 1, max = 5000))]
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListCommentsQuery {
    pub parent_id: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

// ============================================
// Comment Response Types
// ============================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentResponse {
    pub id: String,
    pub document_id: String,
    pub parent_id: Option<String>,
    pub author_id: String,
    pub author_name: String,
    pub author_avatar: Option<String>,
    pub content: String,
    pub is_resolved: bool,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentListResponse {
    pub comments: Vec<CommentResponse>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCommentResponse {
    pub id: String,
    pub message: String,
    pub comment: CommentResponse,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_document_request_valid() {
        let request = CreateDocumentRequest {
            title: "Test Document".to_string(),
            icon: Some("📝".to_string()),
            parent_id: None,
            content: Some(serde_json::json!({"type": "Y.Doc"})),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_document_request_empty_title() {
        let request = CreateDocumentRequest {
            title: "".to_string(),
            icon: None,
            parent_id: None,
            content: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_document_request_title_too_long() {
        let request = CreateDocumentRequest {
            title: "a".repeat(201),
            icon: None,
            parent_id: None,
            content: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_update_document_request_partial() {
        let request = UpdateDocumentRequest {
            title: Some("Updated Title".to_string()),
            icon: None,
            content: None,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_list_documents_query_defaults() {
        let query = ListDocumentsQuery {
            parent_id: None,
            limit: None,
            offset: None,
        };
        assert!(query.parent_id.is_none());
        assert!(query.limit.is_none());
        assert!(query.offset.is_none());
    }

    #[test]
    fn test_create_version_request_valid() {
        let request = CreateVersionRequest {
            content: serde_json::json!({"text": "version content"}),
            title: "Version 1".to_string(),
            change_summary: Some("Initial version".to_string()),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_version_request_empty_title() {
        let request = CreateVersionRequest {
            content: serde_json::json!({"text": "content"}),
            title: "".to_string(),
            change_summary: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_document_response_creation() {
        let response = DocumentResponse {
            id: "doc-123".to_string(),
            space_id: "space-456".to_string(),
            parent_id: None,
            title: "My Document".to_string(),
            icon: Some("📄".to_string()),
            content: serde_json::json!({"text": "content"}),
            content_size: 100,
            is_archived: false,
            created_by: "user-789".to_string(),
            last_edited_by: "user-789".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(response.id, "doc-123");
        assert!(response.icon.is_some());
        assert!(!response.is_archived);
    }

    #[test]
    fn test_document_list_response() {
        let response = DocumentListResponse {
            documents: vec![],
            total: 0,
            limit: 50,
            offset: 0,
        };
        assert!(response.documents.is_empty());
        assert_eq!(response.total, 0);
    }

    #[test]
    fn test_version_response_creation() {
        let response = VersionResponse {
            id: "ver-123".to_string(),
            document_id: "doc-456".to_string(),
            version_number: 1,
            title: "Initial Version".to_string(),
            content: serde_json::json!({"text": "content"}),
            created_by: "user-789".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            change_summary: Some("First version".to_string()),
        };
        assert_eq!(response.version_number, 1);
        assert!(response.change_summary.is_some());
    }

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<()> = ApiResponse::error("NOT_FOUND", "Document not found");
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().error, "NOT_FOUND");
    }

    #[test]
    fn test_create_space_request_valid() {
        let request = CreateSpaceRequest {
            name: "My Space".to_string(),
            icon: Some("📁".to_string()),
            description: Some("A test space".to_string()),
            is_public: false,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_add_member_request_valid() {
        let request = AddMemberRequest {
            user_id: "user-123".to_string(),
            role: "editor".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_add_member_request_invalid_role() {
        let request = AddMemberRequest {
            user_id: "user-123".to_string(),
            role: "a".repeat(25), // Max is 20
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_comment_request_valid() {
        let request = CreateCommentRequest {
            content: "This is a test comment".to_string(),
            parent_id: None,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_comment_request_empty_content() {
        let request = CreateCommentRequest {
            content: "".to_string(),
            parent_id: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_comment_request_content_too_long() {
        let request = CreateCommentRequest {
            content: "a".repeat(5001), // Max is 5000
            parent_id: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_comment_response_creation() {
        let response = CommentResponse {
            id: "comment-123".to_string(),
            document_id: "doc-456".to_string(),
            parent_id: None,
            author_id: "user-789".to_string(),
            author_name: "Test User".to_string(),
            author_avatar: None,
            content: "Test comment".to_string(),
            is_resolved: false,
            resolved_by: None,
            resolved_at: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: Some("2024-01-01T00:00:00Z".to_string()),
        };
        assert_eq!(response.id, "comment-123");
        assert!(!response.is_resolved);
    }

    #[test]
    fn test_space_response_creation() {
        let response = SpaceResponse {
            id: "space-123".to_string(),
            owner_id: "user-456".to_string(),
            name: "Test Space".to_string(),
            icon: Some("🚀".to_string()),
            description: Some("Description".to_string()),
            is_public: true,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            user_role: Some("owner".to_string()),
        };
        assert_eq!(response.name, "Test Space");
        assert!(response.is_public);
        assert!(response.user_role.is_some());
    }

    #[test]
    fn test_member_response_creation() {
        let response = MemberResponse {
            id: "member-123".to_string(),
            space_id: "space-456".to_string(),
            user_id: "user-789".to_string(),
            role: "editor".to_string(),
            joined_at: "2024-01-01T00:00:00Z".to_string(),
            invited_by: "user-abc".to_string(),
        };
        assert_eq!(response.role, "editor");
    }

    #[test]
    fn test_document_path_response() {
        let response = DocumentPathResponse {
            path: vec![
                DocumentPathItem {
                    id: "doc-1".to_string(),
                    title: "Root".to_string(),
                    level: 0,
                },
                DocumentPathItem {
                    id: "doc-2".to_string(),
                    title: "Child".to_string(),
                    level: 1,
                },
            ],
        };
        assert_eq!(response.path.len(), 2);
        assert_eq!(response.path[0].level, 0);
        assert_eq!(response.path[1].level, 1);
    }

    #[test]
    fn test_version_diff_response() {
        let response = VersionDiffResponse {
            from_version: 1,
            to_version: 2,
            from_content: serde_json::json!({"text": "old"}),
            to_content: serde_json::json!({"text": "new"}),
        };
        assert_eq!(response.from_version, 1);
        assert_eq!(response.to_version, 2);
    }

    #[test]
    fn test_children_response() {
        let response = ChildrenResponse {
            documents: vec![],
            total: 0,
        };
        assert!(response.documents.is_empty());
        assert_eq!(response.total, 0);
    }

    #[test]
    fn test_list_comments_query_with_params() {
        let query = ListCommentsQuery {
            parent_id: Some("comment-123".to_string()),
            limit: Some(10),
            offset: Some(20),
        };
        assert!(query.parent_id.is_some());
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(20));
    }

    #[test]
    fn test_restore_version_request() {
        let request = RestoreVersionRequest { version_number: 5 };
        assert_eq!(request.version_number, 5);
    }

    #[test]
    fn test_export_query() {
        let query = ExportQuery {
            format: Some("markdown".to_string()),
        };
        assert_eq!(query.format, Some("markdown".to_string()));
    }
}
