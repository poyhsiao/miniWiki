use crate::models::{CreateDocumentRequest, CreateVersionRequest, UpdateDocumentRequest};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum DocumentValidationError {
    #[error("Title is required and must be between 1-200 characters")]
    InvalidTitle,
    #[error("Icon must be at most 50 characters")]
    InvalidIcon,
    #[error("Parent document not found in the same space")]
    InvalidParent,
    #[error("Content size exceeds maximum limit (10MB)")]
    ContentTooLarge,
    #[error("Invalid UUID format")]
    InvalidUuid,
    #[error("Version number must be positive")]
    InvalidVersionNumber,
    #[error("Change summary must be at most 500 characters")]
    InvalidChangeSummary,
}

pub fn validate_create_document(req: &CreateDocumentRequest) -> Result<(), DocumentValidationError> {
    validate_title(&req.title)?;

    if let Some(icon) = &req.icon {
        if icon.len() > 50 {
            return Err(DocumentValidationError::InvalidIcon);
        }
    }

    if let Some(content) = &req.content {
        validate_content_size(&content.to_string())?;
    }

    Ok(())
}

pub fn validate_update_document(req: &UpdateDocumentRequest) -> Result<(), DocumentValidationError> {
    if let Some(title) = &req.title {
        validate_title(title)?;
    }

    if let Some(icon) = &req.icon {
        if icon.len() > 50 {
            return Err(DocumentValidationError::InvalidIcon);
        }
    }

    if let Some(content) = &req.content {
        validate_content_size(&content.to_string())?;
    }

    Ok(())
}

pub fn validate_create_version(req: &CreateVersionRequest) -> Result<(), DocumentValidationError> {
    validate_title(&req.title)?;

    if let Some(summary) = &req.change_summary {
        if summary.len() > 500 {
            return Err(DocumentValidationError::InvalidChangeSummary);
        }
    }

    validate_content_size(&req.content.to_string())?;

    Ok(())
}

pub fn validate_title(title: &str) -> Result<(), DocumentValidationError> {
    if title.trim().is_empty() || title.len() > 200 {
        return Err(DocumentValidationError::InvalidTitle);
    }
    Ok(())
}

pub fn validate_content_size(content: &str) -> Result<(), DocumentValidationError> {
    if content.len() > 10_485_760 {
        return Err(DocumentValidationError::ContentTooLarge);
    }
    Ok(())
}

pub fn validate_uuid(uuid: &str) -> Result<(), DocumentValidationError> {
    Uuid::parse_str(uuid)
        .map(|_| ())
        .map_err(|_| DocumentValidationError::InvalidUuid)
}

pub fn validate_version_number(version: i32) -> Result<(), DocumentValidationError> {
    if version <= 0 {
        Err(DocumentValidationError::InvalidVersionNumber)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DocumentContent;
    use serde_json;

    #[test]
    fn test_validate_create_document_valid() {
        let req = CreateDocumentRequest {
            title: "Valid Title".to_string(),
            icon: Some("📝".to_string()),
            parent_id: None,
            content: Some(DocumentContent("{\"type\": \"Y.Doc\"}".to_string())),
        };
        assert!(validate_create_document(&req).is_ok());
    }

    #[test]
    fn test_validate_create_document_empty_title() {
        let req = CreateDocumentRequest {
            title: "".to_string(),
            icon: None,
            parent_id: None,
            content: None,
        };
        assert!(validate_create_document(&req).is_err());
    }

    #[test]
    fn test_validate_create_document_title_too_long() {
        let req = CreateDocumentRequest {
            title: "a".repeat(201),
            icon: None,
            parent_id: None,
            content: None,
        };
        assert!(validate_create_document(&req).is_err());
    }

    #[test]
    fn test_validate_update_document_valid() {
        let req = UpdateDocumentRequest {
            title: Some("Updated Title".to_string()),
            icon: None,
            content: None,
        };
        assert!(validate_update_document(&req).is_ok());
    }

    #[test]
    fn test_validate_update_document_partial() {
        let req = UpdateDocumentRequest {
            title: None,
            icon: Some("📄".to_string()),
            content: None,
        };
        assert!(validate_update_document(&req).is_ok());
    }

    #[test]
    fn test_validate_create_version_valid() {
        let req = CreateVersionRequest {
            content: DocumentContent("{\"text\": \"content\"}".to_string()),
            title: "Version Title".to_string(),
            change_summary: Some("Changes made".to_string()),
        };
        assert!(validate_create_version(&req).is_ok());
    }

    #[test]
    fn test_validate_uuid_valid() {
        assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
    }

    #[test]
    fn test_validate_uuid_invalid() {
        assert!(validate_uuid("invalid-uuid").is_err());
        assert!(validate_uuid("").is_err());
    }

    #[test]
    fn test_validate_version_number_valid() {
        assert!(validate_version_number(1).is_ok());
        assert!(validate_version_number(100).is_ok());
    }

    #[test]
    fn test_validate_version_number_invalid() {
        assert!(validate_version_number(0).is_err());
        assert!(validate_version_number(-1).is_err());
    }
}
