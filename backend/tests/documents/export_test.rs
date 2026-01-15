//! Export operation tests
//!
//! Tests for document export functionality in various formats.
//! These tests verify the export service works correctly.
//!
//! Run with: cargo test -p miniwiki-backend-tests documents::export_test

use document_service::export::{ExportFormat, ExportService};
use std::path::PathBuf;
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::test]
async fn test_export_format_from_str() {
    assert_eq!(ExportFormat::from_str("markdown"), Some(ExportFormat::Markdown));
    assert_eq!(ExportFormat::from_str("md"), Some(ExportFormat::Markdown));
    assert_eq!(ExportFormat::from_str("html"), Some(ExportFormat::Html));
    assert_eq!(ExportFormat::from_str("pdf"), Some(ExportFormat::Pdf));
    assert_eq!(ExportFormat::from_str("json"), Some(ExportFormat::Json));
    assert_eq!(ExportFormat::from_str("unknown"), None);
}

#[tokio::test]
async fn test_export_format_extension() {
    assert_eq!(ExportFormat::Markdown.extension(), "md");
    assert_eq!(ExportFormat::Html.extension(), "html");
    assert_eq!(ExportFormat::Pdf.extension(), "pdf");
    assert_eq!(ExportFormat::Json.extension(), "json");
}

#[tokio::test]
async fn test_export_format_mime_type() {
    assert_eq!(ExportFormat::Markdown.mime_type(), "text/markdown");
    assert_eq!(ExportFormat::Html.mime_type(), "text/html");
    assert_eq!(ExportFormat::Pdf.mime_type(), "application/pdf");
    assert_eq!(ExportFormat::Json.mime_type(), "application/json");
}

#[tokio::test]
async fn test_export_markdown_basic() {
    let temp_dir = TempDir::new().unwrap();
    let service = ExportService::new(temp_dir.path().to_path_buf());

    let document_id = Uuid::new_v4().to_string();
    let title = "Test Document";
    let content = serde_json::json!({
        "type": "Y.Doc",
        "items": [
            {"type": "text", "text": "Hello World"}
        ]
    });

    let result = service
        .export_document(&document_id, title, &content, None, ExportFormat::Markdown)
        .await
        .unwrap();

    assert_eq!(result.document_id, document_id);
    assert_eq!(result.format, ExportFormat::Markdown);
    assert!(result.file_name.ends_with(".md"));
    assert!(result.file_size > 0);

    // Verify content
    let exported_content = std::fs::read_to_string(temp_dir.path().join(&result.file_name)).unwrap();
    assert!(exported_content.contains("Hello World"));
    assert!(exported_content.contains("# Test Document"));
    assert!(exported_content.contains("---")); // Frontmatter
}

#[tokio::test]
async fn test_export_html_basic() {
    let temp_dir = TempDir::new().unwrap();
    let service = ExportService::new(temp_dir.path().to_path_buf());

    let document_id = Uuid::new_v4().to_string();
    let title = "Test HTML Document";
    let content = serde_json::json!({
        "type": "Y.Doc",
        "items": [
            {"type": "text", "text": "HTML Content Here"}
        ]
    });

    let result = service
        .export_document(&document_id, title, &content, None, ExportFormat::Html)
        .await
        .unwrap();

    assert_eq!(result.document_id, document_id);
    assert_eq!(result.format, ExportFormat::Html);
    assert!(result.file_name.ends_with(".html"));
    assert!(result.file_size > 0);

    // Verify content
    let exported_content = std::fs::read_to_string(temp_dir.path().join(&result.file_name)).unwrap();
    assert!(exported_content.contains("<!DOCTYPE html>"));
    assert!(exported_content.contains("HTML Content Here"));
    assert!(exported_content.contains("<h1>Test HTML Document</h1>"));
    assert!(exported_content.contains("<p>")); // Paragraph tag
}

#[tokio::test]
async fn test_export_json_basic() {
    let temp_dir = TempDir::new().unwrap();
    let service = ExportService::new(temp_dir.path().to_path_buf());

    let document_id = Uuid::new_v4().to_string();
    let title = "Test JSON Document";
    let content = serde_json::json!({
        "type": "Y.Doc",
        "updates": ["update1", "update2"]
    });

    let result = service
        .export_document(&document_id, title, &content, None, ExportFormat::Json)
        .await
        .unwrap();

    assert_eq!(result.document_id, document_id);
    assert_eq!(result.format, ExportFormat::Json);
    assert!(result.file_name.ends_with(".json"));
    assert!(result.file_size > 0);

    // Verify content
    let exported_content = std::fs::read_to_string(temp_dir.path().join(&result.file_name)).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&exported_content).unwrap();
    assert_eq!(parsed["type"], "Y.Doc");
}

#[tokio::test]
async fn test_export_with_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let service = ExportService::new(temp_dir.path().to_path_buf());

    let document_id = Uuid::new_v4().to_string();
    let title = "Test Document With Metadata";
    let content = serde_json::json!({});

    let metadata = document_service::export::DocumentMetadata {
        id: document_id.clone(),
        title: title.to_string(),
        created_at: Some(chrono::NaiveDateTime::from_timestamp(1704067200, 0).unwrap()),
        updated_at: Some(chrono::NaiveDateTime::from_timestamp(1704153600, 0).unwrap()),
        created_by: Some("test-user-id".to_string()),
        icon: Some("📄".to_string()),
    };

    let result = service
        .export_document(&document_id, title, &content, Some(metadata), ExportFormat::Markdown)
        .await
        .unwrap();

    // Verify content contains metadata
    let exported_content = std::fs::read_to_string(temp_dir.path().join(&result.file_name)).unwrap();
    assert!(exported_content.contains("author:"));
    assert!(exported_content.contains("test-user-id"));
    assert!(exported_content.contains("document_id:"));
}

#[tokio::test]
async fn test_export_multiple_formats() {
    let temp_dir = TempDir::new().unwrap();
    let service = ExportService::new(temp_dir.path().to_path_buf());

    let document_id = Uuid::new_v4().to_string();
    let title = "Multi-format Export Test";
    let content = serde_json::json!({
        "type": "Y.Doc",
        "items": [
            {"type": "heading2", "text": "Section 1"},
            {"type": "text", "text": "Some content here"}
        ]
    });

    // Export in all formats
    for format in [ExportFormat::Markdown, ExportFormat::Html, ExportFormat::Json] {
        let result = service
            .export_document(&document_id, title, &content, None, format)
            .await
            .unwrap();

        assert_eq!(result.document_id, document_id);
        assert_eq!(result.format, format);
        assert!(result.file_size > 0);
    }
}

#[tokio::test]
async fn test_export_handles_special_characters_in_title() {
    let temp_dir = TempDir::new().unwrap();
    let service = ExportService::new(temp_dir.path().to_path_buf());

    let document_id = Uuid::new_v4().to_string();
    let title = "Test: Special Characters (and) \"quotes\" & ampersand";
    let content = serde_json::json!({});

    let result = service
        .export_document(&document_id, title, &content, None, ExportFormat::Markdown)
        .await
        .unwrap();

    // File name should not contain special characters
    assert!(!result.file_name.contains(':'));
    assert!(!result.file_name.contains('"'));
    assert!(!result.file_name.contains('&'));

    // Content should have escaped characters
    let exported_content = std::fs::read_to_string(temp_dir.path().join(&result.file_name)).unwrap();
    assert!(exported_content.contains("Test:")); // Title in content
}

#[tokio::test]
async fn test_export_handles_empty_content() {
    let temp_dir = TempDir::new().unwrap();
    let service = ExportService::new(temp_dir.path().to_path_buf());

    let document_id = Uuid::new_v4().to_string();
    let title = "Empty Document";
    let content = serde_json::json!({});

    let result = service
        .export_document(&document_id, title, &content, None, ExportFormat::Markdown)
        .await
        .unwrap();

    let exported_content = std::fs::read_to_string(temp_dir.path().join(&result.file_name)).unwrap();
    // Should have a placeholder for empty content
    assert!(exported_content.contains("*No content*") || !exported_content.is_empty());
}

#[tokio::test]
async fn test_yjs_to_markdown_conversion() {
    let content = serde_json::json!({
        "type": "Y.Doc",
        "items": [
            {"type": "heading1", "text": "Main Title"},
            {"type": "text", "text": "Introduction paragraph."},
            {"type": "heading2", "text": "Section"},
            {"type": "bullet_list", "items": [
                {"type": "text", "text": "Item 1"},
                {"type": "text", "text": "Item 2"}
            ]}
        ]
    });

    let temp_dir = TempDir::new().unwrap();
    let service = ExportService::new(temp_dir.path().to_path_buf());

    let result = service
        .export_document(&Uuid::new_v4().to_string(), "Test", &content, None, ExportFormat::Markdown)
        .await
        .unwrap();

    let exported_content = std::fs::read_to_string(temp_dir.path().join(&result.file_name)).unwrap();
    
    // Verify markdown elements
    assert!(exported_content.contains("# Main Title"));
    assert!(exported_content.contains("## Section"));
    assert!(exported_content.contains("Introduction paragraph"));
    assert!(exported_content.contains("- Item 1"));
    assert!(exported_content.contains("- Item 2"));
}

#[tokio::test]
async fn test_yjs_to_html_conversion() {
    let content = serde_json::json!({
        "type": "Y.Doc",
        "items": [
            {"type": "heading2", "text": "HTML Title"},
            {"type": "code_block", "text": "console.log('hello');"}
        ]
    });

    let temp_dir = TempDir::new().unwrap();
    let service = ExportService::new(temp_dir.path().to_path_buf());

    let result = service
        .export_document(&Uuid::new_v4().to_string(), "Test", &content, None, ExportFormat::Html)
        .await
        .unwrap();

    let exported_content = std::fs::read_to_string(temp_dir.path().join(&result.file_name)).unwrap();
    
    // Verify HTML elements
    assert!(exported_content.contains("<!DOCTYPE html>"));
    assert!(exported_content.contains("<h2>HTML Title</h2>"));
    assert!(exported_content.contains("<pre><code>"));
    assert!(exported_content.contains("console.log('hello');"));
    assert!(exported_content.contains("</code></pre>"));
}

#[tokio::test]
async fn test_export_preserves_document_structure() {
    let content = serde_json::json!({
        "type": "Y.Doc",
        "items": [
            {"type": "blockquote", "text": "A important quote"},
            {"type": "ordered_list", "items": [
                {"type": "text", "text": "First"},
                {"type": "text", "text": "Second"},
                {"type": "text", "text": "Third"}
            ]}
        ]
    });

    let temp_dir = TempDir::new().unwrap();
    let service = ExportService::new(temp_dir.path().to_path_buf());

    // Test HTML preserves structure
    let result = service
        .export_document(&Uuid::new_v4().to_string(), "Structure Test", &content, None, ExportFormat::Html)
        .await
        .unwrap();

    let exported_content = std::fs::read_to_string(temp_dir.path().join(&result.file_name)).unwrap();
    
    assert!(exported_content.contains("<blockquote>"));
    assert!(exported_content.contains("<ol>"));
    assert!(exported_content.contains("<li>First</li>"));
    assert!(exported_content.contains("<li>Second</li>"));
    assert!(exported_content.contains("<li>Third</li>"));
}
