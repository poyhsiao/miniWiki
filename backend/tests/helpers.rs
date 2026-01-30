use auth_service::jwt::{JwtConfig, JwtService};
use serde_json::json;
use serde_json::Value;
use sqlx::Error as SqlxError;
use sqlx::PgPool;
use std::future::Future;
use uuid::Uuid;

const TEST_JWT_SECRET: &str = "test-secret-key-for-testing-only-do-not-use-in-production";
const TEST_PASSWORD_HASH: &str = "$2b$12$Ej0WLvZBVa6K51r5/occM.JDmozzkJr4QzzovXNjCzk8hLVjVm3Cy";

// ============================================================================
// E2E Test Response Helpers & Assertion Macros
// ============================================================================

/// Helper struct for response validation in E2E tests
pub struct ResponseValidator {
    status: reqwest::StatusCode,
    body: Value,
}

impl ResponseValidator {
    /// Create a new validator from a response
    pub async fn new(response: reqwest::Response) -> Self {
        let status = response.status();
        let body = response.json().await.unwrap_or_else(|_| json!({}));
        Self { status, body }
    }

    /// Assert the response has the expected status code
    pub fn assert_status(&self, expected: u16) -> &Self {
        let actual = self.status.as_u16();
        assert_eq!(
            actual, expected,
            "Expected status {}, but got {}. Response body: {}",
            expected, actual, self.body
        );
        self
    }

    /// Assert the response is successful (2xx)
    pub fn assert_success(&self) -> &Self {
        assert!(
            self.status.is_success(),
            "Expected success status (2xx), but got {}. Response body: {}",
            self.status(),
            self.body
        );
        self
    }

    /// Assert the response is a client error (4xx)
    pub fn assert_client_error(&self) -> &Self {
        assert!(
            self.status.is_client_error(),
            "Expected client error (4xx), but got {}. Response body: {}",
            self.status(),
            self.body
        );
        self
    }

    /// Assert the response is a server error (5xx)
    pub fn assert_server_error(&self) -> &Self {
        assert!(
            self.status.is_server_error(),
            "Expected server error (5xx), but got {}. Response body: {}",
            self.status(),
            self.body
        );
        self
    }

    /// Assert a field exists in the response body
    pub fn assert_field_exists(&self, field: &str) -> &Self {
        assert!(
            self.body.get(field).is_some(),
            "Expected field '{}' to exist in response. Body: {}",
            field,
            self.body
        );
        self
    }

    /// Assert a field does NOT exist in the response body
    pub fn assert_field_not_exists(&self, field: &str) -> &Self {
        assert!(
            self.body.get(field).is_none(),
            "Expected field '{}' to NOT exist in response. Body: {}",
            field,
            self.body
        );
        self
    }

    /// Assert a field has a specific string value
    pub fn assert_field_equals(&self, field: &str, expected: &str) -> &Self {
        let actual = self
            .body
            .get(field)
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("Field '{}' should be a string. Body: {}", field, self.body));
        assert_eq!(
            actual, expected,
            "Field '{}' should be '{}', but got '{}'",
            field, expected, actual
        );
        self
    }

    /// Assert a field has a specific numeric value
    pub fn assert_field_equals_i64(&self, field: &str, expected: i64) -> &Self {
        let actual = self
            .body
            .get(field)
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| panic!("Field '{}' should be a number. Body: {}", field, self.body));
        assert_eq!(
            actual, expected,
            "Field '{}' should be {}, but got {}",
            field, expected, actual
        );
        self
    }

    /// Assert a field is an array
    pub fn assert_field_is_array(&self, field: &str) -> &Self {
        assert!(
            self.body.get(field).map_or(false, |v| v.is_array()),
            "Field '{}' should be an array. Body: {}",
            field,
            self.body
        );
        self
    }

    /// Assert a field is an object
    pub fn assert_field_is_object(&self, field: &str) -> &Self {
        assert!(
            self.body.get(field).map_or(false, |v| v.is_object()),
            "Field '{}' should be an object. Body: {}",
            field,
            self.body
        );
        self
    }

    /// Assert an array field has a minimum length
    pub fn assert_array_min_length(&self, field: &str, min: usize) -> &Self {
        let array = self
            .body
            .get(field)
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("Field '{}' should be an array. Body: {}", field, self.body));
        assert!(
            array.len() >= min,
            "Field '{}' should have at least {} items, but has {}. Body: {}",
            field,
            min,
            array.len(),
            self.body
        );
        self
    }

    /// Assert an array field has an exact length
    pub fn assert_array_length(&self, field: &str, expected: usize) -> &Self {
        let array = self
            .body
            .get(field)
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("Field '{}' should be an array. Body: {}", field, self.body));
        assert_eq!(
            array.len(),
            expected,
            "Field '{}' should have {} items, but has {}. Body: {}",
            field,
            expected,
            array.len(),
            self.body
        );
        self
    }

    /// Assert the response contains a success flag
    pub fn assert_success_flag(&self) -> &Self {
        let success = self.body.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        assert!(success, "Expected 'success' to be true. Body: {}", self.body);
        self
    }

    /// Assert the response contains an error
    pub fn assert_has_error(&self) -> &Self {
        assert!(
            self.body.get("error").is_some() || self.body.get("message").is_some() || self.body.get("code").is_some(),
            "Expected error field in response. Body: {}",
            self.body
        );
        self
    }

    /// Get the response body for further assertions
    pub fn body(&self) -> &Value {
        &self.body
    }

    /// Get a specific field from the response body
    pub fn get_field(&self, field: &str) -> Option<&Value> {
        self.body.get(field)
    }

    /// Get the status code
    pub fn status(&self) -> reqwest::StatusCode {
        self.status
    }
}

/// Parallel test execution helper
pub async fn run_parallel_tests<T>(tests: Vec<T>) -> Vec<T::Output>
where
    T: Future,
{
    futures_util::future::join_all(tests).await
}

/// Retry helper for flaky tests
pub async fn retry_with_backoff<F, T, Fut>(max_retries: u32, initial_delay_ms: u64, mut f: F) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    if max_retries == 0 {
        return Err("max_retries must be >= 1".to_string());
    }

    let mut delay = initial_delay_ms;
    for attempt in 1..=max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(_) if attempt < max_retries => {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                delay *= 2;
                if delay > 10000 {
                    delay = 10000;
                }
            },
            Err(e) => return Err(format!("Failed after {} attempts: {}", attempt, e)),
        }
    }
    unreachable!()
}

// JWT token helper function
pub fn generate_test_jwt_token(user_id: Uuid, email: &str) -> String {
    let config = JwtConfig {
        secret: TEST_JWT_SECRET.to_string(),
        access_expiry: 3600,
        refresh_expiry: 86400,
    };
    let service = JwtService::new(config);
    service.generate_access_token(&user_id.to_string(), email, "user").unwrap()
}

pub struct TestApp {
    pub pool: PgPool,
    pub client: reqwest::Client,
    pub port: u16,
}

impl TestApp {
    pub async fn create() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://miniwiki:miniwiki_secret@localhost:5432/miniwiki".to_string());

        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let client = reqwest::Client::builder().build().expect("Failed to build reqwest client");

        let port = 8080;

        Self { pool, client, port }
    }

    /// Get auth data - returns (token, user_id). Creates user if not provided.
    pub async fn get_auth_data(&self, user_id: Option<Uuid>, email: Option<String>) -> (String, String) {
        let user = if let Some(id) = user_id {
            // Use existing user
            let email = email.unwrap_or_else(|| format!("test_{}@example.com", id.to_string().replace('-', "")));
            TestUser {
                id,
                email,
                display_name: format!("Test User {}", id.to_string().chars().take(8).collect::<String>()),
            }
        } else {
            self.create_test_user().await
        };
        let token = generate_test_jwt_token(user.id, &user.email);
        (token, user.id.to_string())
    }

    /// Create a request builder with Authorization header and X-User-Id header
    pub async fn auth_get(&self, path: &str, user_id: Option<Uuid>, email: Option<String>) -> reqwest::RequestBuilder {
        let (token, user_id_str) = self.get_auth_data(user_id, email).await;
        self.client
            .get(format!("http://localhost:{}{}", self.port, path))
            .header("Authorization", format!("Bearer {}", token))
            .header("X-User-Id", user_id_str)
    }

    /// Create a POST request with Authorization header and X-User-Id header
    pub async fn auth_post(&self, path: &str, user_id: Option<Uuid>, email: Option<String>) -> reqwest::RequestBuilder {
        let (token, user_id_str) = self.get_auth_data(user_id, email).await;
        self.client
            .post(format!("http://localhost:{}{}", self.port, path))
            .header("Authorization", format!("Bearer {}", token))
            .header("X-User-Id", user_id_str)
    }

    /// Create a PATCH request with Authorization header and X-User-Id header
    pub async fn auth_patch(
        &self,
        path: &str,
        user_id: Option<Uuid>,
        email: Option<String>,
    ) -> reqwest::RequestBuilder {
        let (token, user_id_str) = self.get_auth_data(user_id, email).await;
        self.client
            .patch(format!("http://localhost:{}{}", self.port, path))
            .header("Authorization", format!("Bearer {}", token))
            .header("X-User-Id", user_id_str)
    }

    /// Create a DELETE request with Authorization header and X-User-Id header
    pub async fn auth_delete(
        &self,
        path: &str,
        user_id: Option<Uuid>,
        email: Option<String>,
    ) -> reqwest::RequestBuilder {
        let (token, user_id_str) = self.get_auth_data(user_id, email).await;
        self.client
            .delete(format!("http://localhost:{}{}", self.port, path))
            .header("Authorization", format!("Bearer {}", token))
            .header("X-User-Id", user_id_str)
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.get(format!("http://localhost:{}{}", self.port, path))
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.post(format!("http://localhost:{}{}", self.port, path))
    }

    /// Perform login via POST /api/v1/auth/login and return server-issued token
    pub async fn login_user(&self, email: &str, password: &str) -> Result<String, String> {
        let response = self
            .client
            .post(format!("http://localhost:{}/api/v1/auth/login", self.port))
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .send()
            .await
            .map_err(|e| format!("Login request failed: {}", e))?;

        if response.status() != 200 {
            return Err(format!("Login failed with status: {}", response.status()));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse login response: {}", e))?;

        body.get("token")
            .or_else(|| body.get("access_token"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "Token not found in response".to_string())
    }

    pub fn patch(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.patch(format!("http://localhost:{}{}", self.port, path))
    }

    pub fn delete(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.delete(format!("http://localhost:{}{}", self.port, path))
    }

    pub async fn create_test_user(&self) -> TestUser {
        let id = Uuid::new_v4();
        let email = format!("test_{}@example.com", id.to_string().replace('-', ""));
        let display_name = format!(
            "Test User {}",
            id.to_string().replace('-', "").chars().take(8).collect::<String>()
        );

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, display_name, is_active, is_email_verified, timezone, language) VALUES ($1, $2, $3, $4, true, false, 'UTC', 'en') ON CONFLICT (id) DO NOTHING"
        )
        .bind(id)
        .bind(email.clone())
        .bind(TEST_PASSWORD_HASH)
        .bind(display_name.clone())
        .execute(&self.pool)
        .await
        .expect("Failed to create test user");

        TestUser {
            id,
            email,
            display_name,
        }
    }

    pub async fn create_test_space_for_user(&self, user_id: &Uuid) -> TestSpace {
        let id = Uuid::new_v4();
        let name = format!(
            "Test Space {}",
            id.to_string().replace('-', "").chars().take(8).collect::<String>()
        );

        sqlx::query(
            "INSERT INTO spaces (id, owner_id, name, is_public) VALUES ($1, $2, $3, false) ON CONFLICT (id) DO NOTHING",
        )
        .bind(id)
        .bind(user_id)
        .bind(name.clone())
        .execute(&self.pool)
        .await
        .expect("Failed to create test space");

        sqlx::query(
            "INSERT INTO space_memberships (id, space_id, user_id, role, invited_by) VALUES ($1, $2, $3, 'owner', $3) ON CONFLICT DO NOTHING"
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .expect("Failed to create space membership");

        TestSpace {
            id,
            owner_id: *user_id,
            name,
        }
    }

    pub async fn create_test_document(&self, space_id: &Uuid, parent_id: Option<&Uuid>) -> TestDocument {
        let id = Uuid::new_v4();
        let title = format!(
            "Test Document {}",
            id.to_string().replace('-', "").chars().take(8).collect::<String>()
        );

        let content_value = serde_json::json!({
            "type": "Y.Doc",
            "update": "dGVzdCB1cGRhdGU=",
            "vector_clock": {
                "client_id": id.to_string(),
                "clock": 1
            }
        });

        let owner: (Uuid,) = sqlx::query_as("SELECT owner_id FROM spaces WHERE id = $1")
            .bind(space_id)
            .fetch_one(&self.pool)
            .await
            .expect("Failed to get space owner");

        let content_size = content_value.to_string().len() as i32;

        sqlx::query(
            "INSERT INTO documents (id, space_id, parent_id, title, icon, content, content_size, is_archived, created_by, last_edited_by) VALUES ($1, $2, $3, $4, $5, $6, $7, false, $8, $8) ON CONFLICT (id) DO NOTHING"
        )
        .bind(id)
        .bind(space_id)
        .bind(parent_id)
        .bind(title.clone())
        .bind(Some("📝".to_string()))
        .bind(content_value)
        .bind(content_size)
        .bind(owner.0)
        .execute(&self.pool)
        .await
        .expect("Failed to create test document");

        TestDocument {
            id,
            space_id: *space_id,
            title,
        }
    }

    /// Create a test document with custom title
    pub async fn create_test_document_with_title(
        &self,
        space_id: &Uuid,
        parent_id: Option<&Uuid>,
        title: &str,
    ) -> TestDocument {
        let id = Uuid::new_v4();

        let content_value = serde_json::json!({
            "type": "Y.Doc",
            "update": "dGVzdCB1cGRhdGU=",
            "vector_clock": {
                "client_id": id.to_string(),
                "clock": 1
            }
        });

        let owner: (Uuid,) = sqlx::query_as("SELECT owner_id FROM spaces WHERE id = $1")
            .bind(space_id)
            .fetch_one(&self.pool)
            .await
            .expect("Failed to get space owner");

        let content_size = content_value.to_string().len() as i32;

        sqlx::query(
            "INSERT INTO documents (id, space_id, parent_id, title, icon, content, content_size, is_archived, created_by, last_edited_by) VALUES ($1, $2, $3, $4, $5, $6, $7, false, $8, $8) ON CONFLICT (id) DO NOTHING"
        )
        .bind(id)
        .bind(space_id)
        .bind(parent_id)
        .bind(title.to_string())
        .bind(Some("📝".to_string()))
        .bind(content_value)
        .bind(content_size)
        .bind(owner.0)
        .execute(&self.pool)
        .await
        .expect("Failed to create test document");

        TestDocument {
            id,
            space_id: *space_id,
            title: title.to_string(),
        }
    }

    /// Create multiple test documents for pagination/listing tests
    pub async fn create_test_documents(&self, space_id: &Uuid, count: u32) -> Vec<TestDocument> {
        let mut documents = Vec::new();
        for i in 0..count {
            let doc = self
                .create_test_document_with_title(space_id, None, &format!("Test Document {}", i))
                .await;
            documents.push(doc);
        }
        documents
    }

    /// Create a test space member
    pub async fn add_space_member(&self, space_id: &Uuid, user_id: &Uuid, role: &str) {
        sqlx::query(
            "INSERT INTO space_memberships (id, space_id, user_id, role, invited_by) VALUES ($1, $2, $3, $4, $3) ON CONFLICT DO NOTHING"
        )
        .bind(Uuid::new_v4())
        .bind(space_id)
        .bind(user_id)
        .bind(role)
        .execute(&self.pool)
        .await
        .expect("Failed to add space member");
    }

    /// Create a test document version
    pub async fn create_test_document_version(&self, document_id: &Uuid, created_by: &Uuid) -> TestVersion {
        let id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO document_versions (id, document_id, name, description, content_snapshot, created_by) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT DO NOTHING"
        )
        .bind(id)
        .bind(document_id)
        .bind(format!("Version {}", id.to_string().chars().take(8).collect::<String>()))
        .bind("Test version description")
        .bind(serde_json::json!({"type": "Y.Doc", "update": "dGVzdCB2ZXJzaW9u"}))
        .bind(created_by)
        .execute(&self.pool)
        .await
        .expect("Failed to create document version");

        TestVersion {
            id,
            document_id: *document_id,
        }
    }

    pub async fn cleanup(&self) {
        sqlx::query(
            "DELETE FROM document_versions WHERE document_id IN (SELECT id FROM documents WHERE title LIKE 'Test%')",
        )
        .execute(&self.pool)
        .await
        .expect("Cleanup failed");
        sqlx::query("DELETE FROM documents WHERE title LIKE 'Test%'")
            .execute(&self.pool)
            .await
            .expect("Cleanup failed");
        sqlx::query("DELETE FROM space_memberships WHERE space_id IN (SELECT id FROM spaces WHERE name LIKE 'Test%')")
            .execute(&self.pool)
            .await
            .expect("Cleanup failed");
        sqlx::query("DELETE FROM spaces WHERE name LIKE 'Test%'")
            .execute(&self.pool)
            .await
            .expect("Cleanup failed");
        sqlx::query("DELETE FROM users WHERE email LIKE 'test_%@example.com'")
            .execute(&self.pool)
            .await
            .expect("Cleanup failed");
    }

    pub async fn cleanup_test_user(&self, user_id: &Uuid) {
        sqlx::query(
            "DELETE FROM document_versions WHERE document_id IN (SELECT id FROM documents WHERE created_by = $1)",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .expect("Cleanup failed");
        sqlx::query("DELETE FROM documents WHERE created_by = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("Cleanup failed");
        // Delete memberships for spaces owned by this user (to avoid FK violations)
        sqlx::query("DELETE FROM space_memberships WHERE space_id IN (SELECT id FROM spaces WHERE owner_id = $1)")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("Cleanup failed");
        // Delete memberships where this user is a member
        sqlx::query("DELETE FROM space_memberships WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("Cleanup failed");
        sqlx::query("DELETE FROM spaces WHERE owner_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("Cleanup failed");
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("Cleanup failed");
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        // Don't close pool in test as it may be shared
    }
}

#[derive(Debug)]
pub struct TestUser {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
}

#[derive(Debug)]
pub struct TestSpace {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
}

#[derive(Debug)]
pub struct TestDocument {
    pub id: Uuid,
    pub space_id: Uuid,
    pub title: String,
}

/// Test document version for version-related tests
#[derive(Debug)]
pub struct TestVersion {
    pub id: Uuid,
    pub document_id: Uuid,
}

// Legacy functions for backward compatibility with spaces tests
pub async fn create_test_app() -> TestApp {
    TestApp::create().await
}

pub async fn test_app() -> TestApp {
    create_test_app().await
}

pub async fn create_test_user(app: &TestApp) -> Result<TestUser, SqlxError> {
    let id = Uuid::new_v4();
    let email = format!("test_{}@example.com", id.to_string().replace('-', ""));
    let display_name = format!(
        "Test User {}",
        id.to_string().replace('-', "").chars().take(8).collect::<String>()
    );

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_active, is_email_verified, timezone, language) VALUES ($1, $2, $3, $4, true, false, 'UTC', 'en') ON CONFLICT (id) DO NOTHING"
    )
    .bind(id)
    .bind(email.clone())
    .bind(TEST_PASSWORD_HASH)
    .bind(display_name.clone())
    .execute(&app.pool)
    .await?;

    Ok(TestUser {
        id,
        email,
        display_name,
    })
}

pub async fn create_test_space(app: &TestApp, owner_id: &Uuid) -> Result<TestSpace, SqlxError> {
    let id = Uuid::new_v4();
    let name = format!(
        "Test Space {}",
        id.to_string().replace('-', "").chars().take(8).collect::<String>()
    );

    sqlx::query(
        "INSERT INTO spaces (id, owner_id, name, is_public) VALUES ($1, $2, $3, false) ON CONFLICT (id) DO NOTHING",
    )
    .bind(id)
    .bind(owner_id)
    .bind(name.clone())
    .execute(&app.pool)
    .await?;

    sqlx::query(
        "INSERT INTO space_memberships (id, space_id, user_id, role, invited_by) VALUES ($1, $2, $3, 'owner', $3) ON CONFLICT DO NOTHING"
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .bind(owner_id)
    .execute(&app.pool)
    .await?;

    Ok(TestSpace {
        id,
        owner_id: *owner_id,
        name,
    })
}

pub async fn create_test_document(
    app: &TestApp,
    space_id: &Uuid,
    parent_id: Option<&Uuid>,
    title: &str,
) -> Result<TestDocument, SqlxError> {
    let id = Uuid::new_v4();
    let doc_title = title.to_string();

    let content_value = serde_json::json!({
        "type": "Y.Doc",
        "update": "dGVzdCB1cGRhdGU=",
        "vector_clock": {
            "client_id": id.to_string(),
            "clock": 1
        }
    });

    let owner: (Uuid,) = sqlx::query_as("SELECT owner_id FROM spaces WHERE id = $1")
        .bind(space_id)
        .fetch_optional(&app.pool)
        .await?
        .ok_or_else(|| SqlxError::RowNotFound)?;

    let content_size = content_value.to_string().len() as i32;

    sqlx::query(
        "INSERT INTO documents (id, space_id, parent_id, title, icon, content, content_size, is_archived, created_by, last_edited_by) VALUES ($1, $2, $3, $4, $5, $6, $7, false, $8, $8) ON CONFLICT (id) DO NOTHING"
    )
    .bind(id)
    .bind(space_id)
    .bind(parent_id)
    .bind(doc_title.clone())
    .bind(Some("📝".to_string()))
    .bind(content_value)
    .bind(content_size)
    .bind(owner.0)
    .execute(&app.pool)
    .await?;

    Ok(TestDocument {
        id,
        space_id: *space_id,
        title: doc_title,
    })
}

pub async fn cleanup_test_data(app: &TestApp) -> Result<(), SqlxError> {
    sqlx::query(
        "DELETE FROM document_versions WHERE document_id IN (SELECT id FROM documents WHERE title LIKE 'Test%')",
    )
    .execute(&app.pool)
    .await?;
    sqlx::query("DELETE FROM documents WHERE title LIKE 'Test%'")
        .execute(&app.pool)
        .await?;
    sqlx::query("DELETE FROM space_memberships WHERE space_id IN (SELECT id FROM spaces WHERE name LIKE 'Test%')")
        .execute(&app.pool)
        .await?;
    sqlx::query("DELETE FROM spaces WHERE name LIKE 'Test%'")
        .execute(&app.pool)
        .await?;
    sqlx::query("DELETE FROM users WHERE email LIKE 'test_%@example.com'")
        .execute(&app.pool)
        .await?;
    Ok(())
}
