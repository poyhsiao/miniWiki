//! Unit tests for shared_models entities

use chrono::Utc;
use shared_models::entities::*;
use uuid::Uuid;

// Test User
#[test]
fn test_user_default_values() {
    let user = User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        password_hash: "hash123".to_string(),
        display_name: "Test User".to_string(),
        avatar_url: None,
        timezone: "UTC".to_string(),
        language: "en".to_string(),
        is_active: true,
        is_email_verified: false,
        email_verified_at: None,
        last_login_at: None,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    assert_eq!(user.email, "test@example.com");
    assert!(user.is_active);
    assert!(!user.is_email_verified);
    assert!(user.avatar_url.is_none());
}

#[test]
fn test_user_with_avatar() {
    let user = User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        password_hash: "hash123".to_string(),
        display_name: "Test User".to_string(),
        avatar_url: Some("https://example.com/avatar.png".to_string()),
        timezone: "UTC".to_string(),
        language: "en".to_string(),
        is_active: true,
        is_email_verified: true,
        email_verified_at: Some(Utc::now().naive_utc()),
        last_login_at: None,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    assert!(user.avatar_url.is_some());
    assert!(user.email_verified_at.is_some());
}

// Test Space
#[test]
fn test_space_creation() {
    let space = Space {
        id: Uuid::new_v4(),
        owner_id: Uuid::new_v4(),
        name: "My Space".to_string(),
        icon: Some("📁".to_string()),
        description: Some("A test space".to_string()),
        is_public: false,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    assert_eq!(space.name, "My Space");
    assert!(!space.is_public);
    assert!(space.icon.is_some());
}

#[test]
fn test_space_public() {
    let space = Space {
        id: Uuid::new_v4(),
        owner_id: Uuid::new_v4(),
        name: "Public Space".to_string(),
        icon: None,
        description: None,
        is_public: true,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    assert!(space.is_public);
    assert!(space.icon.is_none());
}

// Test SpaceMembership
#[test]
fn test_space_membership() {
    let membership = SpaceMembership {
        id: Uuid::new_v4(),
        space_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        role: "editor".to_string(),
        joined_at: Utc::now().naive_utc(),
        invited_by: Uuid::new_v4(),
    };

    assert_eq!(membership.role, "editor");
}

// Test SpaceMembership roles
#[test]
fn test_space_membership_roles() {
    let roles = vec!["owner", "editor", "commenter", "viewer"];

    for role in roles {
        let membership = SpaceMembership {
            id: Uuid::new_v4(),
            space_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            role: role.to_string(),
            joined_at: Utc::now().naive_utc(),
            invited_by: Uuid::new_v4(),
        };
        assert_eq!(membership.role, role);
    }
}

// Test Document
#[test]
fn test_document_creation() {
    let doc = Document {
        id: Uuid::new_v4(),
        space_id: Uuid::new_v4(),
        parent_id: None,
        title: "Test Document".to_string(),
        icon: Some("📄".to_string()),
        content: serde_json::json!({"text": "Hello World"}),
        content_size: 11,
        is_archived: false,
        archived_at: None,
        created_by: Uuid::new_v4(),
        last_edited_by: Uuid::new_v4(),
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    assert_eq!(doc.title, "Test Document");
    assert!(doc.parent_id.is_none());
    assert!(!doc.is_archived);
}

#[test]
fn test_document_with_parent() {
    let parent_id = Uuid::new_v4();
    let doc = Document {
        id: Uuid::new_v4(),
        space_id: Uuid::new_v4(),
        parent_id: Some(parent_id),
        title: "Child Document".to_string(),
        icon: None,
        content: serde_json::json!({"text": "Child content"}),
        content_size: 12,
        is_archived: false,
        archived_at: None,
        created_by: Uuid::new_v4(),
        last_edited_by: Uuid::new_v4(),
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    assert!(doc.parent_id.is_some());
    assert_eq!(doc.parent_id.unwrap(), parent_id);
}

#[test]
fn test_document_archived() {
    let doc = Document {
        id: Uuid::new_v4(),
        space_id: Uuid::new_v4(),
        parent_id: None,
        title: "Archived Document".to_string(),
        icon: None,
        content: serde_json::json!({}),
        content_size: 0,
        is_archived: true,
        archived_at: Some(Utc::now().naive_utc()),
        created_by: Uuid::new_v4(),
        last_edited_by: Uuid::new_v4(),
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    assert!(doc.is_archived);
    assert!(doc.archived_at.is_some());
}

// Test DocumentVersion
#[test]
fn test_document_version() {
    let version = DocumentVersion {
        id: Uuid::new_v4(),
        document_id: Uuid::new_v4(),
        version_number: 1,
        content: serde_json::json!({"text": "Version 1 content"}),
        title: "Version 1".to_string(),
        created_by: Uuid::new_v4(),
        created_at: Utc::now().naive_utc(),
        change_summary: Some("Initial version".to_string()),
    };

    assert_eq!(version.version_number, 1);
    assert!(version.change_summary.is_some());
}

#[test]
fn test_document_version_no_summary() {
    let version = DocumentVersion {
        id: Uuid::new_v4(),
        document_id: Uuid::new_v4(),
        version_number: 2,
        content: serde_json::json!({"text": "Version 2"}),
        title: "Version 2".to_string(),
        created_by: Uuid::new_v4(),
        created_at: Utc::now().naive_utc(),
        change_summary: None,
    };

    assert!(version.change_summary.is_none());
}

// Test RefreshToken
#[test]
fn test_refresh_token_active() {
    let token = RefreshToken {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        token: "refresh-token-123".to_string(),
        expires_at: Utc::now().naive_utc() + chrono::Duration::days(7),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("Test Agent".to_string()),
        is_revoked: false,
        revoked_at: None,
        created_at: Utc::now().naive_utc(),
    };

    assert!(!token.is_revoked);
    assert!(token.ip_address.is_some());
    assert!(token.user_agent.is_some());
}

#[test]
fn test_refresh_token_revoked() {
    let token = RefreshToken {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        token: "refresh-token-456".to_string(),
        expires_at: Utc::now().naive_utc() + chrono::Duration::days(7),
        ip_address: None,
        user_agent: None,
        is_revoked: true,
        revoked_at: Some(Utc::now().naive_utc()),
        created_at: Utc::now().naive_utc(),
    };

    assert!(token.is_revoked);
    assert!(token.revoked_at.is_some());
}

// Test ShareLink
#[test]
fn test_share_link_active() {
    let link = ShareLink {
        id: Uuid::new_v4(),
        document_id: Uuid::new_v4(),
        created_by: Uuid::new_v4(),
        token: "share-token-123".to_string(),
        access_code: Some("code123".to_string()),
        expires_at: Some(Utc::now().naive_utc() + chrono::Duration::days(30)),
        permission: "view".to_string(),
        is_active: true,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        click_count: 0,
        max_access_count: Some(100),
    };

    assert!(link.is_active);
    assert!(link.expires_at.is_some());
    assert!(link.access_code.is_some());
}

#[test]
fn test_share_link_permissions() {
    let permissions = vec!["view", "edit", "comment"];

    for perm in permissions {
        let link = ShareLink {
            id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            created_by: Uuid::new_v4(),
            token: format!("token-{}", perm),
            access_code: None,
            expires_at: None,
            permission: perm.to_string(),
            is_active: true,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            click_count: 0,
            max_access_count: None,
        };
        assert_eq!(link.permission, perm);
    }
}

#[test]
fn test_share_link_click_counting() {
    let mut link = ShareLink {
        id: Uuid::new_v4(),
        document_id: Uuid::new_v4(),
        created_by: Uuid::new_v4(),
        token: "token-123".to_string(),
        access_code: None,
        expires_at: None,
        permission: "view".to_string(),
        is_active: true,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        click_count: 0,
        max_access_count: None,
    };

    for i in 1..=10 {
        link.click_count = i;
        assert_eq!(link.click_count, i);
    }
}

// Test serialization/deserialization
#[test]
fn test_user_serialization() {
    let user_id = Uuid::new_v4();
    let user = User {
        id: user_id,
        email: "test@example.com".to_string(),
        password_hash: "secret_hash_123".to_string(),
        display_name: "Test".to_string(),
        avatar_url: None,
        timezone: "UTC".to_string(),
        language: "en".to_string(),
        is_active: true,
        is_email_verified: false,
        email_verified_at: None,
        last_login_at: None,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    let json = serde_json::to_string(&user).unwrap();

    // Verify password_hash is NOT in the serialized JSON
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(json_value.get("password_hash").is_none(), "password_hash should not be serialized");

    // Verify other fields are correctly serialized
    assert_eq!(json_value["id"], serde_json::to_value(user_id).unwrap());
    assert_eq!(json_value["email"], "test@example.com");
    assert_eq!(json_value["display_name"], "Test");
    assert_eq!(json_value["timezone"], "UTC");
    assert_eq!(json_value["language"], "en");
    assert_eq!(json_value["is_active"], true);

    // Note: deserialization requires password_hash to be present in the JSON
    // since #[serde(skip_serializing)] only affects serialization, not deserialization
}

#[test]
fn test_document_serialization() {
    let doc = Document {
        id: Uuid::new_v4(),
        space_id: Uuid::new_v4(),
        parent_id: None,
        title: "Test".to_string(),
        icon: None,
        content: serde_json::json!({"text": "hello"}),
        content_size: 5,
        is_archived: false,
        archived_at: None,
        created_by: Uuid::new_v4(),
        last_edited_by: Uuid::new_v4(),
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    let json = serde_json::to_string(&doc).unwrap();
    let deserialized: Document = serde_json::from_str(&json).unwrap();

    assert_eq!(doc.title, deserialized.title);
    assert_eq!(doc.content_size, deserialized.content_size);
}
