//! Document Export Service
//!
//! Provides document export functionality in various formats:
//! - Markdown with frontmatter
//! - HTML with embedded styles
//! - PDF (via weasyprint - requires Python runtime)
//! - JSON (raw Yjs state)
//!
//! # Implementation Notes
//!
//! PDF export requires weasyprint which needs Python runtime.
//! For production, consider using a headless browser or wkhtmltopdf.
//!
//! Run with: cargo test -p document-service export

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Export format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Markdown,
    Html,
    Pdf,
    Json,
}

impl ExportFormat {
    /// Parse format from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Some(ExportFormat::Markdown),
            "html" | "htm" => Some(ExportFormat::Html),
            "pdf" => Some(ExportFormat::Pdf),
            "json" => Some(ExportFormat::Json),
            _ => None,
        }
    }

    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Markdown => "md",
            ExportFormat::Html => "html",
            ExportFormat::Pdf => "pdf",
            ExportFormat::Json => "json",
        }
    }

    /// Get MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ExportFormat::Markdown => "text/markdown",
            ExportFormat::Html => "text/html",
            ExportFormat::Pdf => "application/pdf",
            ExportFormat::Json => "application/json",
        }
    }
}

/// Export request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub document_id: String,
    pub format: ExportFormat,
    pub include_metadata: bool,
    pub include_versions: bool,
}

/// Export response with file path and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    pub document_id: String,
    pub format: ExportFormat,
    pub file_name: String,
    pub file_size: u64,
    pub content_type: String,
    pub exported_at: NaiveDateTime,
}

/// Export error types
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Invalid export format: {0}")]
    InvalidFormat(String),

    #[error("Export failed: {0}")]
    ExportFailed(String),

    #[error("PDF generation failed: {0}")]
    PdfGenerationFailed(String),

    #[error("Content conversion failed: {0}")]
    ConversionFailed(String),
}

/// Document metadata for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: String,
    pub title: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub icon: Option<String>,
}

/// Main export service struct
pub struct ExportService {
    /// Base output directory for exports
    output_dir: PathBuf,
    /// Weasyprint path (if available)
    weasyprint_path: Option<PathBuf>,
}

impl ExportService {
    /// Create a new ExportService
    pub fn new(output_dir: PathBuf) -> Self {
        let weasyprint_path = Command::new("which")
            .arg("weasyprint")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if path.is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(path))
                    }
                } else {
                    None
                }
            })
            .or_else(|| {
                let locations = [
                    PathBuf::from("/usr/local/bin/weasyprint"),
                    PathBuf::from("/usr/bin/weasyprint"),
                    PathBuf::from("/opt/homebrew/bin/weasyprint"),
                ];
                locations.iter().find(|p| p.exists()).cloned()
            });

        let _ = fs::create_dir_all(&output_dir);

        Self {
            output_dir,
            weasyprint_path,
        }
    }

    /// Get the output directory path
    pub fn output_dir(&self) -> &PathBuf {
        &self.output_dir
    }

    /// Export a document in the specified format
    pub async fn export_document(
        &self,
        document_id: &str,
        title: &str,
        content: &serde_json::Value,
        metadata: Option<DocumentMetadata>,
        format: ExportFormat,
    ) -> Result<ExportResponse, ExportError> {
        // Generate unique filename
        let file_name = format!(
            "{}_{}.{}",
            title
                .replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
                .trim_start_matches('_')
                .trim_end_matches('_'),
            document_id.split('-').next().unwrap_or(document_id),
            format.extension()
        );

        let file_path = self.output_dir.join(&file_name);

        // Generate content based on format and write to file
        match format {
            ExportFormat::Markdown => {
                let content_str = self.export_markdown(title, content, metadata.as_ref())?;
                fs::write(&file_path, content_str).map_err(|e| ExportError::ExportFailed(e.to_string()))?;
            },
            ExportFormat::Html => {
                let content_str = self.export_html(title, content, metadata.as_ref())?;
                fs::write(&file_path, content_str).map_err(|e| ExportError::ExportFailed(e.to_string()))?;
            },
            ExportFormat::Pdf => {
                let content_bytes = self.export_pdf(title, content, metadata.as_ref())?;
                fs::write(&file_path, content_bytes).map_err(|e| ExportError::ExportFailed(e.to_string()))?;
            },
            ExportFormat::Json => {
                let content_str = self.export_json(content)?;
                fs::write(&file_path, content_str).map_err(|e| ExportError::ExportFailed(e.to_string()))?;
            },
        }

        // Get file size
        let file_size = fs::metadata(&file_path)
            .map_err(|e| ExportError::ExportFailed(e.to_string()))?
            .len();

        Ok(ExportResponse {
            document_id: document_id.to_string(),
            format,
            file_name,
            file_size,
            content_type: format.mime_type().to_string(),
            exported_at: chrono::Local::now().naive_local(),
        })
    }

    /// Export as Markdown with frontmatter
    fn export_markdown(
        &self,
        title: &str,
        content: &serde_json::Value,
        metadata: Option<&DocumentMetadata>,
    ) -> Result<String, ExportError> {
        let mut output = String::new();

        // Frontmatter
        output.push_str("---\n");
        output.push_str(&format!("title: \"{}\"\n", escape_yaml(title)));

        if let Some(meta) = metadata {
            if let Some(created_at) = meta.created_at {
                output.push_str(&format!("created_at: {}\n", created_at.format("%Y-%m-%d %H:%M:%S")));
            }
            if let Some(updated_at) = meta.updated_at {
                output.push_str(&format!("updated_at: {}\n", updated_at.format("%Y-%m-%d %H:%M:%S")));
            }
            if let Some(created_by) = &meta.created_by {
                output.push_str(&format!("author: \"{}\"\n", escape_yaml(created_by)));
            }
            output.push_str(&format!("document_id: \"{}\"\n", meta.id));
        }

        output.push_str("---\n\n");

        // Title
        output.push_str(&format!("# {}\n\n", title));

        // Content - convert Yjs JSON to Markdown
        let markdown_content = Self::yjs_to_markdown(content);
        output.push_str(&markdown_content);

        Ok(output)
    }

    /// Export as HTML with embedded styles
    fn export_html(
        &self,
        title: &str,
        content: &serde_json::Value,
        metadata: Option<&DocumentMetadata>,
    ) -> Result<String, ExportError> {
        let mut output = String::new();

        // HTML header with embedded CSS
        output.push_str(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>"#,
        );
        output.push_str(&escape_html(title));
        output.push_str(r#"</title>
    <style>
        :root {
            --primary-color: #2563eb;
            --text-color: #1f2937;
            --background-color: #ffffff;
            --code-background: #f3f4f6;
            --border-color: #e5e7eb;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            line-height: 1.6;
            color: var(--text-color);
            background: var(--background-color);
            max-inline-size: 800px;
            margin-inline: auto;
            padding: 2rem;
        }
        h1 { color: var(--primary-color); border-block-end: 2px solid var(--primary-color); padding-block-end: 0.5rem; }
        h2 { margin-block-start: 1.5rem; color: #374151; }
        h3 { margin-block-start: 1.25rem; }
        code { background: var(--code-background); padding: 0.2em 0.4em; border-radius: 4px; font-family: ui-monospace, monospace; }
        pre { background: var(--code-background); padding: 1rem; border-radius: 8px; overflow-x: auto; }
        pre code { background: none; padding: 0; }
        blockquote { border-inline-start: 4px solid var(--primary-color); margin-block: 1rem; padding-inline-start: 1rem; color: #6b7280; }
        ul, ol { padding-inline-start: 1.5rem; }
        table { border-collapse: collapse; inline-size: 100%; margin-block: 1rem; }
        th, td { border: 1px solid var(--border-color); padding: 0.5rem 1rem; text-align: start; }
        th { background: var(--code-background); }
        .metadata { color: #6b7280; font-size: 0.875rem; margin-block-end: 1rem; }
        .metadata span { margin-inline-end: 1rem; }
    </style>
</head>
<body>
"#);

        // Metadata
        output.push_str("    <div class=\"metadata\">\n");
        if let Some(meta) = metadata {
            if let Some(created_at) = meta.created_at {
                write!(
                    output,
                    r#"        <span>Created: {}</span>"#,
                    created_at.format("%Y-%m-%d %H:%M")
                )
                .unwrap();
            }
            if let Some(updated_at) = meta.updated_at {
                write!(
                    output,
                    r#"        <span>Updated: {}</span>"#,
                    updated_at.format("%Y-%m-%d %H:%M")
                )
                .unwrap();
            }
        }
        output.push_str("\n    </div>\n");

        // Title
        output.push_str(&format!("    <h1>{}</h1>\n\n", escape_html(title));

        // Content - convert Yjs JSON to HTML
        let html_content = Self::yjs_to_html(content);
        output.push_str(&format!("    {}\n", html_content));

        // Footer
        output.push_str(
            r#"
</body>
</html>
"#,
        );

        Ok(output)
    }

    /// Export as PDF using weasyprint
    fn export_pdf(
        &self,
        title: &str,
        content: &serde_json::Value,
        metadata: Option<&DocumentMetadata>,
    ) -> Result<Vec<u8>, ExportError> {
        // First generate HTML
        let html = self.export_html(title, content, metadata)?;

        // Check if weasyprint is available
        let weasyprint_path = match &self.weasyprint_path {
            Some(path) => path,
            None => {
                return Err(ExportError::PdfGenerationFailed(
                    "weasyprint is not installed. Install with: pip install weasyprint".to_string(),
                ));
            },
        };

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let temp_html_path = self
            .output_dir
            .join(format!("temp_export_{}_{}.html", std::process::id(), unique));
        fs::write(&temp_html_path, &html).map_err(|e| ExportError::PdfGenerationFailed(e.to_string()))?;

        let output_path = temp_html_path.with_extension("pdf");

        let output = Command::new(&weasyprint_path)
            .arg(&temp_html_path)
            .arg(&output_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                let _ = fs::remove_file(&temp_html_path);
                ExportError::PdfGenerationFailed(e.to_string())
            })?;

        let _ = fs::remove_file(&temp_html_path);

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            let _ = fs::remove_file(&output_path);
            return Err(ExportError::PdfGenerationFailed(format!(
                "weasyprint failed: {}",
                error_msg
            )));
        }

        let pdf_bytes = fs::read(&output_path).map_err(|e| {
            let _ = fs::remove_file(&output_path);
            ExportError::PdfGenerationFailed(e.to_string())
        })?;
        let _ = fs::remove_file(&output_path);
        Ok(pdf_bytes)
    }

    /// Export as JSON (raw Yjs state)
    fn export_json(&self, content: &serde_json::Value) -> Result<String, ExportError> {
        serde_json::to_string_pretty(content).map_err(|e| ExportError::ConversionFailed(e.to_string()))
    }

    /// Convert Yjs document state to Markdown
    pub fn yjs_to_markdown(content: &serde_json::Value) -> String {
        let mut markdown = String::new();

        // Extract Yjs document info if available
        let doc_type = content.get("type").and_then(|v| v.as_str()).unwrap_or("");

        if doc_type == "Y.Doc" || doc_type == "y-doc" {
            // Extract text content from Yjs structure
            if let Some(updates) = content.get("updates").and_then(|v| v.as_array()) {
                for update in updates {
                    if let Some(text) = update.get("text").and_then(|v| v.as_str()) {
                        if !text.is_empty() {
                            markdown.push_str(text);
                            markdown.push_str("\n\n");
                        }
                    }
                }
            }

            // Check for y-text elements
            if let Some(items) = content.get("items").or(content.get("content")) {
                if let Some(arr) = items.as_array() {
                    for item in arr {
                        if let Some(text) = item.get("text").or(item.get("content")) {
                            if let Some(s) = text.as_str() {
                                if !s.is_empty() {
                                    // Determine heading level
                                    let level = item.get("level").and_then(|v| v.as_u64()).unwrap_or(0);
                                    if level > 0 {
                                        for _ in 0..level {
                                            markdown.push('#');
                                        }
                                        markdown.push(' ');
                                    }
                                    markdown.push_str(s);
                                    markdown.push_str("\n\n");
                                }
                            }
                        }
                    }
                }
            }
        } else if content.is_object() {
            // Try to extract text from various JSON structures
            extract_text_recursive(content, &mut markdown, 0);
        }

        // If no content extracted, use a placeholder
        if markdown.trim().is_empty() {
            markdown.push_str("*No content*");
        }

        markdown
    }

    /// Convert Yjs document state to HTML
    pub fn yjs_to_html(content: &serde_json::Value) -> String {
        let mut html = String::new();
        let mut in_paragraph = false;

        // Extract Yjs document info if available
        let doc_type = content.get("type").and_then(|v| v.as_str()).unwrap_or("");

        html.push_str("<div class=\"content\">\n");

        if doc_type == "Y.Doc" || doc_type == "y-doc" {
            // Process text elements
            if let Some(items) = content.get("items").or(content.get("content")) {
                if let Some(arr) = items.as_array() {
                    for item in arr {
                        process_html_item(item, &mut html, &mut in_paragraph);
                    }
                }
            }
        } else if content.is_object() {
            // Fallback: try to extract text
            extract_html_recursive(content, &mut html, &mut in_paragraph);
        }

        if in_paragraph {
            html.push_str("</p>\n");
        }

        html.push_str("</div>\n");
        html
    }
}

/// Process a Yjs item and convert to HTML
fn process_html_item(item: &serde_json::Value, html: &mut String, in_paragraph: &mut bool) {
    let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("text");

    match item_type {
        "text" | "paragraph" => {
            if let Some(text) = item.get("text").or(item.get("content")) {
                if let Some(s) = text.as_str() {
                    if !*in_paragraph {
                        html.push_str("  <p>");
                        *in_paragraph = true;
                    } else {
                        html.push_str(" ");
                    }
                    html.push_str(&escape_html(s));
                }
            }
        },
        "heading" | "heading1" => {
            if *in_paragraph {
                html.push_str("</p>\n");
                *in_paragraph = false;
            }
            let text = item.get("text").or(item.get("content")).and_then(|v| v.as_str()).unwrap_or("");
            html.push_str(&format!("  <h1>{}</h1>\n", escape_html(text)));
        },
        "heading2" => {
            if *in_paragraph {
                html.push_str("</p>\n");
                *in_paragraph = false;
            }
            let text = item.get("text").or(item.get("content")).and_then(|v| v.as_str()).unwrap_or("");
            html.push_str(&format!("  <h2>{}</h2>\n", escape_html(text)));
        },
        "heading3" => {
            if *in_paragraph {
                html.push_str("</p>\n");
                *in_paragraph = false;
            }
            let text = item.get("text").or(item.get("content")).and_then(|v| v.as_str()).unwrap_or("");
            html.push_str(&format!("  <h3>{}</h3>\n", escape_html(text)));
        },
        "bullet_list" | "list" => {
            if *in_paragraph {
                html.push_str("</p>\n");
                *in_paragraph = false;
            }
            html.push_str("  <ul>\n");
            if let Some(items) = item.get("items").and_then(|v| v.as_array()) {
                for list_item in items {
                    html.push_str("    <li>");
                    if let Some(text) = list_item.get("text").or(list_item.get("content")) {
                        if let Some(s) = text.as_str() {
                            html.push_str(&escape_html(s));
                        }
                    }
                    html.push_str("</li>\n");
                }
            }
            html.push_str("  </ul>\n");
        },
        "ordered_list" => {
            if *in_paragraph {
                html.push_str("</p>\n");
                *in_paragraph = false;
            }
            html.push_str("  <ol>\n");
            if let Some(items) = item.get("items").and_then(|v| v.as_array()) {
                for list_item in items {
                    html.push_str("    <li>");
                    if let Some(text) = list_item.get("text").or(list_item.get("content")) {
                        if let Some(s) = text.as_str() {
                            html.push_str(&escape_html(s));
                        }
                    }
                    html.push_str("</li>\n");
                }
            }
            html.push_str("  </ol>\n");
        },
        "code_block" => {
            if *in_paragraph {
                html.push_str("</p>\n");
                *in_paragraph = false;
            }
            let text = item.get("text").or(item.get("content")).and_then(|v| v.as_str()).unwrap_or("");
            html.push_str("  <pre><code>");
            html.push_str(&escape_html(text));
            html.push_str("</code></pre>\n");
        },
        "blockquote" => {
            if *in_paragraph {
                html.push_str("</p>\n");
                *in_paragraph = false;
            }
            let text = item.get("text").or(item.get("content")).and_then(|v| v.as_str()).unwrap_or("");
            html.push_str("  <blockquote>");
            html.push_str(&escape_html(text));
            html.push_str("</blockquote>\n");
        },
        "bold" | "strong" => {
            if let Some(text) = item.get("text").or(item.get("content")) {
                if let Some(s) = text.as_str() {
                    html.push_str("<strong>");
                    html.push_str(&escape_html(s));
                    html.push_str("</strong>");
                }
            }
        },
        "italic" | "em" => {
            if let Some(text) = item.get("text").or(item.get("content")) {
                if let Some(s) = text.as_str() {
                    html.push_str("<em>");
                    html.push_str(&escape_html(s));
                    html.push_str("</em>");
                }
            }
        },
        "inline_code" => {
            if let Some(text) = item.get("text").or(item.get("content")) {
                if let Some(s) = text.as_str() {
                    html.push_str("<code>");
                    html.push_str(&escape_html(s));
                    html.push_str("</code>");
                }
            }
        },
        _ => {
            // Default: try to extract text
            if let Some(text) = item.get("text").or(item.get("content")) {
                if let Some(s) = text.as_str() {
                    if !s.is_empty() {
                        if !*in_paragraph {
                            html.push_str("  <p>");
                            *in_paragraph = true;
                        }
                        html.push_str(&escape_html(s));
                    }
                }
            }
        },
    }
}

/// Extract text recursively from JSON
fn extract_text_recursive(value: &serde_json::Value, output: &mut String, _depth: usize) {
    match value {
        serde_json::Value::String(s) => {
            if !s.is_empty() {
                output.push_str(s);
                output.push_str("\n\n");
            }
        },
        serde_json::Value::Array(arr) => {
            for item in arr {
                extract_text_recursive(item, output, _depth + 1);
            }
        },
        serde_json::Value::Object(obj) => {
            // Look for common text fields
            for key in ["text", "content", "value", "body"] {
                if let Some(val) = obj.get(key) {
                    extract_text_recursive(val, output, _depth + 1);
                }
            }
        },
        _ => {},
    }
}

/// Extract HTML recursively from JSON
fn extract_html_recursive(value: &serde_json::Value, html: &mut String, in_paragraph: &mut bool) {
    match value {
        serde_json::Value::String(s) => {
            if !s.is_empty() {
                if !*in_paragraph {
                    html.push_str("  <p>");
                    *in_paragraph = true;
                }
                html.push_str(&escape_html(s));
            }
        },
        serde_json::Value::Array(arr) => {
            for item in arr {
                process_html_item(item, html, in_paragraph);
            }
        },
        serde_json::Value::Object(obj) => {
            // Look for content field
            if let Some(content) = obj.get("content").or(obj.get("text")) {
                extract_html_recursive(content, html, in_paragraph);
            }
        },
        _ => {},
    }
}

/// Escape special characters for YAML
fn escape_yaml(s: &str) -> String {
    s.replace('"', "\\\"")
}

/// Escape special characters for HTML
fn escape_html(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            c => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_from_str() {
        assert_eq!(ExportFormat::from_str("markdown"), Some(ExportFormat::Markdown));
        assert_eq!(ExportFormat::from_str("md"), Some(ExportFormat::Markdown));
        assert_eq!(ExportFormat::from_str("html"), Some(ExportFormat::Html));
        assert_eq!(ExportFormat::from_str("htm"), Some(ExportFormat::Html));
        assert_eq!(ExportFormat::from_str("pdf"), Some(ExportFormat::Pdf));
        assert_eq!(ExportFormat::from_str("json"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::from_str("unknown"), None);
    }

    #[test]
    fn test_export_format_extension() {
        assert_eq!(ExportFormat::Markdown.extension(), "md");
        assert_eq!(ExportFormat::Html.extension(), "html");
        assert_eq!(ExportFormat::Pdf.extension(), "pdf");
        assert_eq!(ExportFormat::Json.extension(), "json");
    }

    #[test]
    fn test_export_format_mime_type() {
        assert_eq!(ExportFormat::Markdown.mime_type(), "text/markdown");
        assert_eq!(ExportFormat::Html.mime_type(), "text/html");
        assert_eq!(ExportFormat::Pdf.mime_type(), "application/pdf");
        assert_eq!(ExportFormat::Json.mime_type(), "application/json");
    }

    #[test]
    fn test_yjs_to_markdown_empty() {
        let content = serde_json::json!({});
        let result = ExportService::yjs_to_markdown(&content);
        assert!(result.contains("*No content*"));
    }

    #[test]
    fn test_yjs_to_markdown_simple_text() {
        let content = serde_json::json!({
            "type": "Y.Doc",
            "items": [
                {"type": "text", "text": "Hello World"}
            ]
        });
        let result = ExportService::yjs_to_markdown(&content);
        assert!(result.contains("Hello World"));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("Hello & World <test>"), "Hello &amp; World &lt;test&gt;");
        assert_eq!(escape_html("Quote: \"test\""), "Quote: &quot;test&quot;");
    }

    #[test]
    fn test_escape_yaml() {
        assert_eq!(escape_yaml("Hello \"World\""), "Hello \\\"World\\\"");
    }
}
