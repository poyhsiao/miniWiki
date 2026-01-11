use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct DatabaseConnection {
    pub pool: Arc<PgPool>,
}

impl DatabaseConnection {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self {
            pool: Arc::new(pool),
        })
    }
    
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

pub async fn init_database(database_url: &str) -> Result<DatabaseConnection, sqlx::Error> {
    DatabaseConnection::new(database_url).await
}
