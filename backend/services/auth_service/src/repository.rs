use sqlx::PgPool;
use uuid::Uuid;
use shared_models::entities::{User, RefreshToken};

pub struct AuthRepository {
    pool: PgPool,
}

impl AuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, display_name, avatar_url, timezone, 
             language, is_active, is_email_verified, email_verified_at, 
             last_login_at, created_at, updated_at
             FROM users WHERE email = $1 AND is_active = true"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }
    
    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, display_name, avatar_url, timezone, 
             language, is_active, is_email_verified, email_verified_at, 
             last_login_at, created_at, updated_at
             FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }
    
    pub async fn create(&self, email: &str, password_hash: &str, display_name: &str) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, password_hash, display_name) 
             VALUES ($1, $2, $3)
             RETURNING id, email, password_hash, display_name, avatar_url, timezone, 
             language, is_active, is_email_verified, email_verified_at, 
             last_login_at, created_at, updated_at"
        )
        .bind(email)
        .bind(password_hash)
        .bind(display_name)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    pub async fn update_last_login(&self, user_id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET last_login_at = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn create_refresh_token(&self, token: &RefreshToken) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO refresh_tokens (id, user_id, token, expires_at, ip_address, user_agent, is_revoked, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(&token.id)
        .bind(&token.user_id)
        .bind(&token.token)
        .bind(&token.expires_at)
        .bind(&token.ip_address)
        .bind(&token.user_agent)
        .bind(&token.is_revoked)
        .bind(&token.created_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn revoke_refresh_token(&self, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE refresh_tokens SET is_revoked = true, revoked_at = NOW() WHERE token = $1"
        )
        .bind(token)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn find_refresh_token(&self, token: &str) -> Result<Option<RefreshToken>, sqlx::Error> {
        sqlx::query_as::<_, RefreshToken>(
            "SELECT id, user_id, token, expires_at, ip_address, user_agent, 
             is_revoked, revoked_at, created_at
             FROM refresh_tokens WHERE token = $1 AND is_revoked = false 
             AND expires_at > NOW()"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
    }
}
