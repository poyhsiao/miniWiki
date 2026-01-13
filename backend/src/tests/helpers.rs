use actix_web::body::MessageBody;
use actix_web::dev::{Service, ServiceResponse};
use actix_web::web;
use actix_web::{test, App, HttpRequest, HttpResponse};
use bytes::Bytes;
use serde::Serialize;
use uuid::Uuid;

use document_service::repository::DocumentRepository;
use shared_database::connection::DatabasePool;

pub struct TestRequest {
    method: actix_web::http::Method,
    path: String,
    body: Option<actix_web::web::Bytes>,
    user_id: Option<Uuid>,
}

impl TestRequest {
    pub fn post(path: &str) -> Self {
        Self {
            method: actix_web::http::Method::POST,
            path: path.to_string(),
            body: None,
            user_id: None,
        }
    }

    pub fn get(path: &str) -> Self {
        Self {
            method: actix_web::http::Method::GET,
            path: path.to_string(),
            body: None,
            user_id: None,
        }
    }

    pub fn patch(path: &str) -> Self {
        Self {
            method: actix_web::http::Method::PATCH,
            path: path.to_string(),
            body: None,
            user_id: None,
        }
    }

    pub fn delete(path: &str) -> Self {
        Self {
            method: actix_web::http::Method::DELETE,
            path: path.to_string(),
            body: None,
            user_id: None,
        }
    }

    pub fn json<T: Serialize>(mut self, body: &T) -> Self {
        let json = serde_json::to_string(body).unwrap();
        self.body = Some(actix_web::web::Bytes::from(json));
        self
    }

    pub fn user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub async fn send(self, app: &TestApp) -> TestResponse {
        let mut test_app = test::init_service(
            App::new()
                .app_data(web::Data::new(app.repository.clone()))
                .app_data(web::Data::new(app.pool.clone()))
                .configure(document_service::config),
        )
        .await;

        let mut req = test::TestRequest::with_uri(&self.path)
            .method(self.method)
            .to_request();

        if let Some(user_id) = self.user_id {
            req.headers_mut().insert(
                actix_web::http::HeaderName::from_lowercase(b"x-user-id").unwrap(),
                user_id.to_string().parse().unwrap(),
            );
        }

        if let Some(body) = self.body {
            req.headers_mut().insert(
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::HeaderValue::from_static("application/json"),
            );
            req.set_payload(body);
        }

        let response = test_app.call(req).await.unwrap();
        TestResponse::from(response)
    }
}

pub struct TestResponse {
    response: ServiceResponse<actix_web::web::Bytes>,
}

impl TestResponse {
    pub fn status(&self) -> actix_web::http::StatusCode {
        self.response.status()
    }

    pub fn is_success(&self) -> bool {
        self.response.status().is_success()
    }

    pub async fn json<T: serde::de::DeserializeOwned>(self) -> T {
        let body = self.response.into_body();
        let bytes: Bytes = hyper::body::to_bytes(body).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }
}

impl From<ServiceResponse<actix_web::web::Bytes>> for TestResponse {
    fn from(response: ServiceResponse<actix_web::web::Bytes>) -> Self {
        Self { response }
    }
}

pub struct TestApp {
    pub pool: DatabasePool,
    pub repository: DocumentRepository,
}

impl TestApp {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://miniwiki:miniwiki_secret@localhost:5432/miniwiki".to_string());

        let pool = shared_database::connection::init_database(&database_url)
            .await
            .expect("Failed to connect to test database");

        let repository = DocumentRepository::new(pool.pool().clone());

        Self { pool, repository }
    }

    pub fn post(&self, path: &str) -> TestRequest {
        TestRequest::post(path)
    }

    pub fn get(&self, path: &str) -> TestRequest {
        TestRequest::get(path)
    }

    pub fn patch(&self, path: &str) -> TestRequest {
        TestRequest::patch(path)
    }

    pub fn delete(&self, path: &str) -> TestRequest {
        TestRequest::delete(path)
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
    .execute(&app.pool)
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

pub struct TestDocument {
    pub id: Uuid,
    pub space_id: Uuid,
    pub title: String,
}

pub async fn create_test_document(app: &TestApp, space_id: &Uuid, parent_id: Option<&Uuid>) -> sqlx::Result<TestDocument> {
    let id = Uuid::new_v4();
    let title = format!("Test Document {}", id.to_string().replace('-', "").chars().take(8).collect::<String>());

    let content = serde_json::json!({
        "type": "Y.Doc",
        "update": "dGVzdCB1cGRhdGU=",
        "vector_clock": {
            "client_id": id.to_string(),
            "clock": 1
        }
    });

    let owner = sqlx::query_as::<_, (Uuid,)>(
        "SELECT owner_id FROM spaces WHERE id = $1"
    )
    .bind(space_id)
    .fetch_optional(&app.pool)
    .await?
    .ok_or_else(|| sqlx::Error::RowNotFound)?;

    sqlx::query(
        "INSERT INTO documents (id, space_id, parent_id, title, icon, content, content_size, is_archived, created_by, last_edited_by) VALUES ($1, $2, $3, $4, $5, $6, $7, false, $8, $8) ON CONFLICT (id) DO NOTHING"
    )
    .bind(id)
    .bind(space_id)
    .bind(parent_id)
    .bind(title.clone())
    .bind(Some("📝".to_string()))
    .bind(content)
    .bind(content.to_string().len() as i32)
    .bind(owner.0)
    .execute(&app.pool)
    .await?;

    Ok(TestDocument { id, space_id: *space_id, title })
}
