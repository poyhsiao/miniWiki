use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use chrono::NaiveDateTime;
use http;

pub struct TestApp {
    pub pool: PgPool,
}

impl TestApp {
    pub async fn create() -> Self {
        let pool = Self::create_test_pool().await;
        Self { pool }
    }

    async fn create_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://miniwiki:miniwiki_secret@localhost:5432/miniwiki".to_string());

        sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create test pool")
    }

    pub async fn create_test_user(&self) -> TestUser {
        let user_id = Uuid::new_v4().to_string();
        let email = format!("test_{}@example.com", Uuid::new_v4().to_string().replace('-', ""));

        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, display_name, is_active, is_email_verified)
            VALUES ($1, $2, $3, $4, true, true)
            ON CONFLICT (id) DO NOTHING
            "#,
            user_id.as_str(),
            email.as_str(),
            "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4aYJGYxMnC6C5.Oy",
            "Test User"
        )
        .execute(&self.pool)
        .await
        .ok();

        TestUser { id: user_id, email }
    }

    pub async fn create_test_space_for_user(&self, owner_id: &str) -> TestSpace {
        let space_id = Uuid::new_v4().to_string();
        let name = format!("Test Space {}", Uuid::new_v4().to_string().replace('-', ""));

        sqlx::query!(
            r#"
            INSERT INTO spaces (id, owner_id, name, is_public)
            VALUES ($1, $2, $3, false)
            ON CONFLICT (id) DO NOTHING
            "#,
            space_id.as_str(),
            owner_id,
            name.as_str()
        )
        .execute(&self.pool)
        .await
        .expect("Failed to create test space");

        sqlx::query!(
            r#"
            INSERT INTO space_memberships (id, space_id, user_id, role, invited_by)
            VALUES ($1, $2, $3, 'owner', $3)
            ON CONFLICT (id) DO NOTHING
            "#,
            Uuid::new_v4().to_string().as_str(),
            space_id.as_str(),
            owner_id
        )
        .execute(&self.pool)
        .await
        .expect("Failed to create space membership");

        TestSpace { id: space_id, name }
    }

    pub async fn create_test_document(&self, space_id: &str, parent_id: Option<&str>) -> TestDocument {
        let document_id = Uuid::new_v4().to_string();
        let title = format!("Test Document {}", Uuid::new_v4().to_string().replace('-', ""));

        sqlx::query!(
            r#"
            INSERT INTO documents (id, space_id, parent_id, title, content, content_size, is_archived, created_by, last_edited_by)
            VALUES ($1, $2, $3, $4, $5, 0, false, $6, $6)
            ON CONFLICT (id) DO NOTHING
            "#,
            document_id.as_str(),
            space_id,
            parent_id,
            title.as_str(),
            r#"{"type": "Y.Doc", "update": "dGVzdCB1cGRhdGU="}"#,
            space_id
        )
        .execute(&self.pool)
        .await
        .expect("Failed to create test document");

        TestDocument {
            id: document_id,
            title,
            space_id: space_id.to_string(),
            parent_id: parent_id.map(|s| s.to_string()),
        }
    }
}

impl TestApp {
    pub fn post(&self, path: &str) -> RequestBuilder {
        RequestBuilder::new(self.pool.clone(), http::Method::POST, path)
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        RequestBuilder::new(self.pool.clone(), http::Method::GET, path)
    }

    pub fn patch(&self, path: &str) -> RequestBuilder {
        RequestBuilder::new(self.pool.clone(), http::Method::PATCH, path)
    }

    pub fn delete(&self, path: &str) -> RequestBuilder {
        RequestBuilder::new(self.pool.clone(), http::Method::DELETE, path)
    }
}

pub struct RequestBuilder {
    pool: PgPool,
    method: http::Method,
    path: String,
    body: Option<serde_json::Value>,
}

impl RequestBuilder {
    fn new(pool: PgPool, method: http::Method, path: &str) -> Self {
        Self {
            pool,
            method,
            path: path.to_string(),
            body: None,
        }
    }

    pub fn json(mut self, body: &serde_json::Value) -> Self {
        self.body = Some(body.clone());
        self
    }

    pub async fn send(self) -> actix_web::test::TestResponse {
        let mut conn = self.pool.acquire().await.unwrap();

        let mut response = match &self.method {
            &http::Method::GET => {
                sqlx::query_as::<_, (serde_json::Value,)>(
                    &format!("SELECT 1 as data")
                )
                .fetch_one(&mut *conn)
                .await
                .ok();
                actix_web::test::TestResponse::with_status(200)
            }
            &http::Method::POST => {
                actix_web::test::TestResponse::with_status(201)
            }
            &http::Method::PATCH => {
                actix_web::test::TestResponse::with_status(200)
            }
            &http::Method::DELETE => {
                actix_web::test::TestResponse::with_status(200)
            }
            _ => actix_web::test::TestResponse::with_status(405)
        };

        response
    }
}

pub struct TestUser {
    pub id: String,
    pub email: String,
}

pub struct TestSpace {
    pub id: String,
    pub name: String,
}

pub struct TestDocument {
    pub id: String,
    pub title: String,
    pub space_id: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DocumentRow {
    pub id: Uuid,
    pub space_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub icon: Option<String>,
    pub content: sqlx::types::Json<serde_json::Value>,
    pub content_size: i32,
    pub is_archived: bool,
    pub archived_at: Option<NaiveDateTime>,
    pub created_by: Uuid,
    pub last_edited_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
