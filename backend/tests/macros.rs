//! Test helper macros for E2E tests
//!
//! This module contains utility macros for creating test data,
//! validating responses, and streamlining test code.

/// Macro for creating a quick response validator
#[macro_export]
macro_rules! validate_response {
    ($response:expr) => {
        $crate::ResponseValidator::new($response).await
    };
}

/// Macro for asserting common API response patterns
#[macro_export]
macro_rules! assert_api_success {
    ($response:expr) => {
        let validator = $crate::ResponseValidator::new($response).await;
        validator.assert_success();
    };
    ($response:expr, $status:expr) => {
        let validator = $crate::ResponseValidator::new($response).await;
        validator.assert_status($status);
    };
}

/// Macro for asserting error responses
#[macro_export]
macro_rules! assert_api_error {
    ($response:expr, $status:expr) => {
        let validator = $crate::ResponseValidator::new($response).await;
        validator.assert_status($status).assert_has_error();
    };
}

/// Macro for creating test data with custom fields
#[macro_export]
macro_rules! test_document {
    () => {
        serde_json::json!({
            "title": format!("Test Document {}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
            "content": {
                "type": "Y.Doc",
                "update": "dGVzdCB1cGRhdGU=",
                "vector_clock": {"client_id": uuid::Uuid::new_v4().to_string(), "clock": 1}
            },
            "icon": "📝"
        })
    };
    (title: $title:expr) => {
        serde_json::json!({
            "title": $title,
            "content": {
                "type": "Y.Doc",
                "update": "dGVzdCB1cGRhdGU=",
                "vector_clock": {"client_id": uuid::Uuid::new_v4().to_string(), "clock": 1}
            },
            "icon": "📝"
        })
    };
    (icon: $icon:expr) => {
        serde_json::json!({
            "title": format!("Test Document {}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
            "content": {
                "type": "Y.Doc",
                "update": "dGVzdCB1cGRhdGU=",
                "vector_clock": {"client_id": uuid::Uuid::new_v4().to_string(), "clock": 1}
            },
            "icon": $icon
        })
    };
}

/// Macro for creating test space data
#[macro_export]
macro_rules! test_space {
    () => {
        serde_json::json!({
            "name": format!("Test Space {}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
            "is_public": false
        })
    };
    (name: $name:expr) => {
        serde_json::json!({
            "name": $name,
            "is_public": false
        })
    };
}
