use sqlx::PgPool;
use sqlx::Error as SqlxError;
use uuid::Uuid;
use reqwest;
use auth_service::jwt::{JwtService, JwtConfig};

const TEST_JWT_SECRET: &str = "test-secret-key-for-testing-only-do-not-use-in-production";

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

        let client = reqwest::Client::builder()
            .build()
            .expect("Failed to build reqwest client");

        let port = 8080;

        Self {
            pool,
            client,
            port,
        }
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .get(&format!("http://localhost:{}{}", self.port, path))
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .post(&format!("http://localhost:{}{}", self.port, path))
    }

    pub fn patch(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .patch(&format!("http://localhost:{}{}", self.port, path))
    }

    pub fn delete(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .delete(&format!("http://localhost:{}{}", self.port, path))
    }

    pub async fn create_test_user(&self) -> TestUser {
        let id = Uuid::new_v4();
        let email = format!("test_{}@example.com", id.to_string().replace('-', ""));
        let display_name = format!("Test User {}", id.to_string().replace('-', "").chars().take(8).collect::<String>());

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, display_name, is_active, is_email_verified, timezone, language) VALUES ($1, $2, $3, $4, true, false, 'UTC', 'en') ON CONFLICT (id) DO NOTHING"
        )
        .bind(id)
        .bind(email.clone())
        .bind("$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4aYJGYxMnC6C5.Oy")
        .bind(display_name.clone())
        .execute(&self.pool)
        .await
        .expect("Failed to create test user");

        TestUser { id, email, display_name }
    }

    pub async fn create_test_space_for_user(&self, user_id: &Uuid) -> TestSpace {
        let id = Uuid::new_v4();
        let name = format!("Test Space {}", id.to_string().replace('-', "").chars().take(8).collect::<String>());

        sqlx::query(
            "INSERT INTO spaces (id, owner_id, name, is_public) VALUES ($1, $2, $3, false) ON CONFLICT (id) DO NOTHING"
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

        TestSpace { id, owner_id: *user_id, name }
    }

    pub async fn create_test_document(&self, space_id: &Uuid, parent_id: Option<&Uuid>) -> TestDocument {
        let id = Uuid::new_v4();
        let title = format!("Test Document {}", id.to_string().replace('-', "").chars().take(8).collect::<String>());

        let content_value = serde_json::json!({
            "type": "Y.Doc",
            "update": "dGVzdCB1cGRhdGU=",
            "vector_clock": {
                "client_id": id.to_string(),
                "clock": 1
            }
        });

        let owner: (Uuid,) = sqlx::query_as(
            "SELECT owner_id FROM spaces WHERE id = $1"
        )
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

        TestDocument { id, space_id: *space_id, title }
    }

    pub async fn cleanup(&self) {
        sqlx::query("DELETE FROM document_versions WHERE document_id IN (SELECT id FROM documents WHERE title LIKE 'Test%')")
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
    let display_name = format!("Test User {}", id.to_string().replace('-', "").chars().take(8).collect::<String>());

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name, is_active, is_email_verified, timezone, language) VALUES ($1, $2, $3, $4, true, false, 'UTC', 'en') ON CONFLICT (id) DO NOTHING"
    )
    .bind(id)
    .bind(email.clone())
    .bind("$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4aYJGYxMnC6C5.Oy")
    .bind(display_name.clone())
    .execute(&app.pool)
    .await?;

    Ok(TestUser { id, email, display_name })
}

pub async fn create_test_space(app: &TestApp, owner_id: &Uuid) -> Result<TestSpace, SqlxError> {
    let id = Uuid::new_v4();
    let name = format!("Test Space {}", id.to_string().replace('-', "").chars().take(8).collect::<String>());

    sqlx::query(
        "INSERT INTO spaces (id, owner_id, name, is_public) VALUES ($1, $2, $3, false) ON CONFLICT (id) DO NOTHING"
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

    Ok(TestSpace { id, owner_id: *owner_id, name })
}

pub async fn create_test_document(app: &TestApp, space_id: &Uuid, parent_id: Option<&Uuid>, title: &str) -> Result<TestDocument, SqlxError> {
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

    let owner: (Uuid,) = sqlx::query_as(
        "SELECT owner_id FROM spaces WHERE id = $1"
    )
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

    Ok(TestDocument { id, space_id: *space_id, title: doc_title })
}

pub async fn cleanup_test_data(app: &TestApp) -> Result<(), SqlxError> {
    sqlx::query("DELETE FROM document_versions WHERE document_id IN (SELECT id FROM documents WHERE title LIKE 'Test%')")
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
