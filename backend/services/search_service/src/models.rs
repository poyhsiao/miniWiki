use serde::{Deserialize, Serialize};
use validator::Validate;

// ============================================
// Request Types
// ============================================

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SearchQuery {
    #[validate(length(min = 1, max = 500))]
    pub q: String,

    pub space_id: Option<String>,

    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i32>,

    #[validate(range(min = 0))]
    pub offset: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub space_id: Option<String>,
    pub limit: i32,
    pub offset: i32,
}

// ============================================
// Response Types
// ============================================

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub document_id: String,
    pub space_id: String,
    pub space_name: String,
    pub title: String,
    pub snippet: String,
    pub score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: i64,
    pub took: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchIndexResponse {
    pub document_id: String,
    pub indexed: bool,
    pub message: String,
}

// ============================================
// API Response Wrapper
// ============================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
}

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
