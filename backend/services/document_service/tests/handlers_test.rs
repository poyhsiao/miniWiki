//! Unit tests for document_service models and validation
//!
//! This module contains tests for:
//! - Document request/response models
//! - Version models
//! - Space models
//! - Validation functions

use document_service::models::*;
use document_service::validation::*;

// Test DocumentResponse
#[test]
fn test_document_response_structure() {
    let response = DocumentResponse {
        id: "doc-123".to_string(),
        space_id: "space-456".to_string(),
        parent_id: Some("parent-789".to_string()),
        title: "Test Document".to_string(),
        icon: Some("📄".to_string()),
        content: DocumentContent("Test content".to_string()),
        content_size: 12,
        is_archived: false,
        created_by: "user-abc".to_string(),
        last_edited_by: "user-abc".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };

    assert_eq!(response.id, "doc-123");
    assert_eq!(response.title, "Test Document");
    assert!(response.parent_id.is_some());
    assert_eq!(response.content.0, "Test content");
}

// Test DocumentContent
#[test]
fn test_document_content_new() {
    let content = DocumentContent("Hello World".to_string());
    assert_eq!(content.0, "Hello World");
}

#[test]
fn test_document_content_empty() {
    let content = DocumentContent("".to_string());
    assert!(content.0.is_empty());
}

// Test CreateDocumentRequest
#[test]
fn test_create_document_request_valid() {
    let req = CreateDocumentRequest {
        title: "New Document".to_string(),
        parent_id: Some("parent-id".to_string()),
        icon: Some("📝".to_string()),
        content: Some(DocumentContent("Initial content".to_string())),
    };

    assert_eq!(req.title, "New Document");
    assert!(req.parent_id.is_some());
    assert!(req.icon.is_some());
    assert!(req.content.is_some());
}

// Test UpdateDocumentRequest
#[test]
fn test_update_document_request_partial() {
    let req = UpdateDocumentRequest {
        title: Some("Updated Title".to_string()),
        icon: None,
        content: Some(DocumentContent("Updated content".to_string())),
    };

    assert!(req.title.is_some());
    assert!(req.icon.is_none());
    assert!(req.content.is_some());
}

// Test DocumentVersionResponse
#[test]
fn test_version_response_structure() {
    let version = VersionResponse {
        id: "version-123".to_string(),
        document_id: "doc-456".to_string(),
        version_number: 1,
        title: "Document Title".to_string(),
        content: serde_json::json!({"text": "Version content"}),
        created_by: "user-789".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        change_summary: Some("Initial version".to_string()),
    };

    assert_eq!(version.id, "version-123");
    assert_eq!(version.version_number, 1);
    assert!(version.change_summary.is_some());
}

// Test CreateVersionRequest
#[test]
fn test_create_version_request_valid() {
    let req = CreateVersionRequest {
        content: DocumentContent("Version content".to_string()),
        title: "Version Title".to_string(),
        change_summary: Some("Changes made".to_string()),
    };

    assert_eq!(req.title, "Version Title");
    assert!(req.change_summary.is_some());
}

// Test DocumentListResponse
#[test]
fn test_document_list_response_structure() {
    let response = DocumentListResponse {
        documents: vec![
            DocumentResponse {
                id: "doc-1".to_string(),
                space_id: "space-1".to_string(),
                parent_id: None,
                title: "Doc 1".to_string(),
                icon: None,
                content: DocumentContent("".to_string()),
                content_size: 0,
                is_archived: false,
                created_by: "user-1".to_string(),
                last_edited_by: "user-1".to_string(),
                created_at: "".to_string(),
                updated_at: "".to_string(),
            },
            DocumentResponse {
                id: "doc-2".to_string(),
                space_id: "space-1".to_string(),
                parent_id: None,
                title: "Doc 2".to_string(),
                icon: None,
                content: DocumentContent("".to_string()),
                content_size: 0,
                is_archived: false,
                created_by: "user-1".to_string(),
                last_edited_by: "user-1".to_string(),
                created_at: "".to_string(),
                updated_at: "".to_string(),
            },
        ],
        total: 2,
        limit: 20,
        offset: 0,
    };

    assert_eq!(response.documents.len(), 2);
    assert_eq!(response.total, 2);
}

// Test SpaceResponse
#[test]
fn test_space_response_structure() {
    let response = SpaceResponse {
        id: "space-123".to_string(),
        name: "My Space".to_string(),
        icon: Some("📁".to_string()),
        description: Some("A test space".to_string()),
        is_public: false,
        owner_id: "user-456".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        user_role: Some("owner".to_string()),
    };

    assert_eq!(response.id, "space-123");
    assert_eq!(response.name, "My Space");
    assert!(!response.is_public);
    assert_eq!(response.user_role, Some("owner".to_string()));
}

// Test CreateSpaceRequest
#[test]
fn test_create_space_request_valid() {
    let req = CreateSpaceRequest {
        name: "New Space".to_string(),
        icon: Some("🚀".to_string()),
        description: Some("Description".to_string()),
        is_public: true,
    };

    assert_eq!(req.name, "New Space");
    assert!(req.is_public);
}

// Test MemberResponse
#[test]
fn test_member_response_structure() {
    let response = MemberResponse {
        id: "member-123".to_string(),
        space_id: "space-456".to_string(),
        user_id: "user-123".to_string(),
        role: "editor".to_string(),
        joined_at: "2024-01-01T00:00:00Z".to_string(),
        invited_by: "user-abc".to_string(),
    };

    assert_eq!(response.id, "member-123");
    assert_eq!(response.space_id, "space-456");
    assert_eq!(response.user_id, "user-123");
    assert_eq!(response.role, "editor");
}

// Test AddMemberRequest
#[test]
fn test_add_member_request_valid() {
    let req = AddMemberRequest {
        user_id: "user-456".to_string(),
        role: "viewer".to_string(),
    };

    assert_eq!(req.user_id, "user-456");
    assert_eq!(req.role, "viewer");
}

// Test UpdateMemberRequest
#[test]
fn test_update_member_request_valid() {
    let req = UpdateMemberRequest {
        role: "editor".to_string(),
    };

    assert_eq!(req.role, "editor");
}

// Test validation functions
#[test]
fn test_validate_title_empty() {
    let result = validate_title("");
    assert!(result.is_err());
}

#[test]
fn test_validate_title_valid() {
    let result = validate_title("Valid Title");
    assert!(result.is_ok());
}

#[test]
fn test_validate_title_too_long() {
    let long_title = "a".repeat(1001);
    let result = validate_title(&long_title);
    assert!(result.is_err());
}

#[test]
fn test_validate_content_size_valid() {
    let result = validate_content_size(1000);
    assert!(result.is_ok());
}

#[test]
fn test_validate_content_size_too_large() {
    let result = validate_content_size(10_000_000);
    assert!(result.is_err());
}

// Test content operations
#[test]
fn test_content_truncate() {
    let content = DocumentContent("Short content".to_string());
    let truncated = content.truncate(5);
    assert_eq!(truncated.0, "Short");
}

#[test]
fn test_content_truncate_longer_than_content() {
    let content = DocumentContent("Short".to_string());
    let truncated = content.truncate(100);
    assert_eq!(truncated.0, "Short");
}

// Test ApiResponse wrapper
#[test]
fn test_api_response_success() {
    let response = ApiResponse::<DocumentResponse>::success(DocumentResponse {
        id: "doc-123".to_string(),
        space_id: "space-456".to_string(),
        parent_id: None,
        title: "Test".to_string(),
        icon: None,
        content: DocumentContent("".to_string()),
        content_size: 0,
        is_archived: false,
        created_by: "user-1".to_string(),
        last_edited_by: "user-1".to_string(),
        created_at: "".to_string(),
        updated_at: "".to_string(),
    });

    assert!(response.success);
    assert!(response.error.is_none());
    assert!(response.data.is_some());
}

#[test]
fn test_api_response_error() {
    let response = ApiResponse::<()>::error("NOT_FOUND", "Document not found");

    assert!(!response.success);
    assert!(response.data.is_none());
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().error, "NOT_FOUND");
}

// Test ChildrenResponse
#[test]
fn test_children_response_structure() {
    let response = ChildrenResponse {
        documents: vec![],
        total: 0,
    };

    assert!(response.documents.is_empty());
    assert_eq!(response.total, 0);
}

// Test DocumentPathResponse
#[test]
fn test_path_response_structure() {
    let response = DocumentPathResponse {
        path: vec![
            DocumentPathItem {
                id: "root".to_string(),
                title: "Root".to_string(),
                level: 0,
            },
            DocumentPathItem {
                id: "child".to_string(),
                title: "Child".to_string(),
                level: 1,
            },
        ],
    };

    assert_eq!(response.path.len(), 2);
    assert_eq!(response.path[0].level, 0);
    assert_eq!(response.path[1].level, 1);
}

// Test VersionListResponse
#[test]
fn test_version_list_response() {
    let response = VersionListResponse {
        versions: vec![],
        total: 0,
        limit: 20,
        offset: 0,
    };

    assert!(response.versions.is_empty());
    assert_eq!(response.limit, 20);
}

// Test ExportQuery
#[test]
fn test_export_query_default() {
    let query = ExportQuery::default();
    assert_eq!(query.format, None);
}

#[test]
fn test_export_query_with_format() {
    let query = ExportQuery {
        format: Some("markdown".to_string()),
    };
    assert_eq!(query.format, Some("markdown".to_string()));
}

// Test SpaceListResponse
#[test]
fn test_space_list_response() {
    let response = SpaceListResponse {
        spaces: vec![],
        total: 0,
    };

    assert!(response.spaces.is_empty());
    assert_eq!(response.total, 0);
}

// Test MemberListResponse
#[test]
fn test_member_list_response() {
    let response = MemberListResponse {
        members: vec![],
        total: 0,
    };

    assert!(response.members.is_empty());
    assert_eq!(response.total, 0);
}

// Test ListDocumentsQuery
#[test]
fn test_list_documents_query_defaults() {
    let query = ListDocumentsQuery::default();
    assert_eq!(query.limit, None);
    assert_eq!(query.offset, None);
    assert_eq!(query.parent_id, None);
}

// Test ListVersionsQuery
#[test]
fn test_list_versions_query_defaults() {
    let query = ListVersionsQuery::default();
    assert_eq!(query.limit, None);
    assert_eq!(query.offset, None);
}

// Test RestoreVersionResponse
#[test]
fn test_restore_version_response() {
    let response = RestoreVersionResponse {
        document: DocumentResponse {
            id: "doc-123".to_string(),
            space_id: "space-456".to_string(),
            parent_id: None,
            title: "Restored".to_string(),
            icon: None,
            content: DocumentContent("content".to_string()),
            content_size: 7,
            is_archived: false,
            created_by: "user-1".to_string(),
            last_edited_by: "user-1".to_string(),
            created_at: "".to_string(),
            updated_at: "".to_string(),
        },
        message: "Restored successfully".to_string(),
        restored_from_version: 5,
    };

    assert_eq!(response.restored_from_version, 5);
    assert!(response.message.contains("successfully"));
}

// Test VersionDiffResponse
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
    assert_ne!(response.from_content, response.to_content);
}

// Test CreateVersionResponse
#[test]
fn test_create_version_response() {
    let response = CreateVersionResponse {
        id: "version-123".to_string(),
        version_number: 3,
        message: "Version created".to_string(),
        version: VersionResponse {
            id: "version-123".to_string(),
            document_id: "doc-456".to_string(),
            version_number: 3,
            title: "Title".to_string(),
            content: DocumentContent("content".to_string()),
            created_by: "user-1".to_string(),
            created_at: "".to_string(),
            change_summary: None,
        },
    };

    assert_eq!(response.version_number, 3);
    assert!(response.message.contains("created"));
}

// Test CreateDocumentResponse
#[test]
fn test_create_document_response() {
    let response = CreateDocumentResponse {
        id: "doc-123".to_string(),
        message: "Created".to_string(),
        document: DocumentResponse {
            id: "doc-123".to_string(),
            space_id: "space-456".to_string(),
            parent_id: None,
            title: "New Doc".to_string(),
            icon: None,
            content: DocumentContent("".to_string()),
            content_size: 0,
            is_archived: false,
            created_by: "user-1".to_string(),
            last_edited_by: "user-1".to_string(),
            created_at: "".to_string(),
            updated_at: "".to_string(),
        },
    };

    assert_eq!(response.id, "doc-123");
    assert!(response.message.contains("Created"));
}
