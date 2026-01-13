use serde::Serialize;
use uuid::Uuid;

use shared_database::connection::DatabaseConnection;

pub struct TestApp {
    pub pool: DatabaseConnection,
}

impl TestApp {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://miniwiki:miniwiki_secret@localhost:5432/miniwiki".to_string());

        let pool = shared_database::connection::init_database(&database_url)
            .await
            .expect("Failed to connect to test database");

        Self { pool }
    }

    pub fn pool(&self) -> &sqlx::PgPool {
        self.pool.pool()
    }
}

pub async fn create_test_app() -> TestApp {
    TestApp::new().await
}

pub struct TestUser {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
}

pub async fn create_test_user(app: &TestApp) -> sqlx::Result<TestUser> {
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
    .execute(app.pool())
    .await?;

    Ok(TestUser { id, email, display_name })
}

pub struct TestSpace {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
}

pub async fn create_test_space(app: &TestApp, owner_id: &Uuid) -> sqlx::Result<TestSpace> {
    let id = Uuid::new_v4();
    let name = format!("Test Space {}", id.to_string().replace('-', "").chars().take(8).collect::<String>());

    sqlx::query(
        "INSERT INTO spaces (id, owner_id, name, is_public) VALUES ($1, $2, $3, false) ON CONFLICT (id) DO NOTHING"
    )
    .bind(id)
    .bind(owner_id)
    .bind(name.clone())
    .execute(app.pool())
    .await?;

    sqlx::query(
        "INSERT INTO space_memberships (id, space_id, user_id, role, invited_by) VALUES ($1, $2, $3, 'owner', $3) ON CONFLICT DO NOTHING"
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .bind(owner_id)
    .execute(app.pool())
    .await?;

    Ok(TestSpace { id, owner_id: *owner_id, name })
}

pub struct TestDocument {
    pub id: Uuid,
    pub space_id: Uuid,
    pub title: String,
}

pub async fn create_test_document(app: &TestApp, space_id: &Uuid, parent_id: Option<&Uuid>, title: &str) -> sqlx::Result<TestDocument> {
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
    .fetch_optional(app.pool())
    .await?
    .ok_or_else(|| sqlx::Error::RowNotFound)?;

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
    .execute(app.pool())
    .await?;

    Ok(TestDocument { id, space_id: *space_id, title: doc_title })
}

pub async fn cleanup_test_data(app: &TestApp) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM document_versions WHERE document_id IN (SELECT id FROM documents WHERE title LIKE 'Test%')")
        .execute(app.pool())
        .await?;
    sqlx::query("DELETE FROM documents WHERE title LIKE 'Test%'")
        .execute(app.pool())
        .await?;
    sqlx::query("DELETE FROM space_memberships WHERE space_id IN (SELECT id FROM spaces WHERE name LIKE 'Test%')")
        .execute(app.pool())
        .await?;
    sqlx::query("DELETE FROM spaces WHERE name LIKE 'Test%'")
        .execute(app.pool())
        .await?;
    sqlx::query("DELETE FROM users WHERE email LIKE 'test_%@example.com'")
        .execute(app.pool())
        .await?;
    Ok(())
}
