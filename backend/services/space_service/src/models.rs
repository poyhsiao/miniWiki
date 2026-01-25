use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_creation() {
        let ts = chrono::NaiveDateTime::from_timestamp(0, 0);
        let space = Space {
            id: Uuid::new_v4(),
            owner_id: Uuid::new_v4(),
            name: "Test Space".to_string(),
            icon: Some("🚀".to_string()),
            description: Some("A test space".to_string()),
            is_public: false,
            created_at: ts,
            updated_at: ts,
        };

        assert_eq!(space.name, "Test Space");
        assert!(!space.is_public);
        assert!(space.icon.is_some());
    }

    #[test]
    fn test_create_space_request_valid() {
        let request = CreateSpaceRequest {
            name: "My Space".to_string(),
            icon: Some("📁".to_string()),
            description: Some("Description".to_string()),
            is_public: true,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_space_request_empty_name() {
        let request = CreateSpaceRequest {
            name: "".to_string(),
            icon: None,
            description: None,
            is_public: false,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_space_request_name_too_long() {
        let request = CreateSpaceRequest {
            name: "a".repeat(201), // Max is 200
            icon: None,
            description: None,
            is_public: false,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_update_space_request_partial() {
        let request = UpdateSpaceRequest {
            name: Some("Updated Name".to_string()),
            icon: None,
            description: None,
            is_public: Some(true),
        };

        assert!(request.name.is_some());
        assert!(request.icon.is_none());
        assert!(request.is_public.is_some());
    }

    #[test]
    fn test_space_membership() {
        let joined_at = chrono::NaiveDateTime::from_timestamp(0, 0);
        let membership = SpaceMembership {
            id: Uuid::new_v4(),
            space_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            role: "editor".to_string(),
            joined_at,
            invited_by: Uuid::new_v4(),
        };

        assert_eq!(membership.role, "editor");
    }

    #[test]
    fn test_add_member_request() {
        let request = AddMemberRequest {
            user_id: Uuid::new_v4().to_string(),
            role: "viewer".to_string(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_update_member_request() {
        let request = UpdateMemberRequest {
            role: "admin".to_string(),
        };

        assert_eq!(request.role, "admin");
    }

    #[test]
    fn test_space_error_display() {
        let error = SpaceError::NotFound;
        assert_eq!(error.to_string(), "Space not found");

        let error = SpaceError::Forbidden;
        assert_eq!(error.to_string(), "Access denied");

        let error = SpaceError::Validation("Invalid name".to_string());
        assert_eq!(error.to_string(), "Validation error: Invalid name");
    }
}
