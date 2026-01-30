use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;
use async_trait::async_trait;
use regex::{Regex, Captures};

// Row types for search results
#[derive(sqlx::FromRow)]
pub struct SearchResultRow {
    pub document_id: Uuid,
    pub space_id: Uuid,
    pub space_name: String,
    pub title: String,
    pub content: serde_json::Value,
    pub score: f64,
}

#[async_trait]
pub trait SearchRepositoryTrait {
    async fn search(
        &self,
        user_id: &str,
        query: &str,
        space_id: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<SearchResultRow>, i64), sqlx::Error>;
}

pub struct SearchRepository {
    pool: Arc<PgPool>,
}

impl SearchRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SearchRepositoryTrait for SearchRepository {
    async fn search(
        &self,
        user_id: &str,
        query: &str,
        space_id: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<SearchResultRow>, i64), sqlx::Error> {
        let user_uuid: Uuid = user_id.parse()
            .map_err(|_| sqlx::Error::Decode("Invalid user ID format".into()))?;

        // Count total results
        let query_pattern = format!("%{}%", query);

        let total: i64 = match space_id {
            Some(sid) => {
                let count_sql = r#"
                SELECT COUNT(*) as total
                FROM documents d
                WHERE d.is_archived = false
                AND (d.title ILIKE $1 OR d.content_text ILIKE $1)
                AND d.space_id = $3
                AND EXISTS (
                    SELECT 1 FROM space_memberships sm
                    WHERE sm.space_id = d.space_id
                    AND sm.user_id = $2
                )
                "#.to_string();
                sqlx::query_as::<_, (i64,)>(&count_sql)
                    .bind(&query_pattern)
                    .bind(user_uuid)
                    .bind(sid)
                    .fetch_one(&*self.pool)
                    .await?
                    .0
            }
            None => {
                let count_sql = r#"
                SELECT COUNT(*) as total
                FROM documents d
                WHERE d.is_archived = false
                AND (d.title ILIKE $1 OR d.content_text ILIKE $1)
                AND EXISTS (
                    SELECT 1 FROM space_memberships sm
                    JOIN spaces s ON sm.space_id = s.id
                    WHERE sm.space_id = d.space_id
                    AND sm.user_id = $2
                    AND (s.is_public OR sm.user_id = $2)
                )
                "#.to_string();
                sqlx::query_as::<_, (i64,)>(&count_sql)
                    .bind(&query_pattern)
                    .bind(user_uuid)
                    .fetch_one(&*self.pool)
                    .await?
                    .0
            }
        };

        // Search with ranking
        // Using ILIKE for simple pattern matching (PostgreSQL full-text search with tsvector can be added later)
        let results: Vec<SearchResultRow> = match space_id {
            Some(sid) => {
                let search_sql = r#"
                SELECT
                    d.id as document_id,
                    d.space_id,
                    s.name as space_name,
                    d.title,
                    d.content as content,
                    (
                        CASE
                            WHEN d.title ILIKE $1 THEN 2.0
                            ELSE 1.0
                        END +
                        CASE
                            WHEN d.title ILIKE $1 || ' %' THEN 0.5
                            ELSE 0.0
                        END
                    ) as score
                FROM documents d
                JOIN spaces s ON d.space_id = s.id
                WHERE d.is_archived = false
                AND (d.title ILIKE $1 OR d.content_text ILIKE $1)
                AND d.space_id = $4
                AND EXISTS (
                    SELECT 1 FROM space_memberships sm
                    WHERE sm.space_id = d.space_id
                    AND sm.user_id = $2
                )
                ORDER BY
                    CASE WHEN d.title ILIKE $1 THEN 0 ELSE 1 END,
                    d.updated_at DESC
                LIMIT $3 OFFSET $4
                "#.to_string();
                sqlx::query_as(&search_sql)
                    .bind(&query_pattern)
                    .bind(user_uuid)
                    .bind(limit)
                    .bind(offset)
                    .bind(sid)
                    .fetch_all(&*self.pool)
                    .await?
            }
            None => {
                let search_sql = r#"
                SELECT
                    d.id as document_id,
                    d.space_id,
                    s.name as space_name,
                    d.title,
                    d.content as content,
                    (
                        CASE
                            WHEN d.title ILIKE $1 THEN 2.0
                            ELSE 1.0
                        END +
                        CASE
                            WHEN d.title ILIKE $1 || ' %' THEN 0.5
                            ELSE 0.0
                        END
                    ) as score
                FROM documents d
                JOIN spaces s ON d.space_id = s.id
                WHERE d.is_archived = false
                AND (d.title ILIKE $1 OR d.content_text ILIKE $1)
                AND EXISTS (
                    SELECT 1 FROM space_memberships sm
                    JOIN spaces s ON sm.space_id = s.id
                    WHERE sm.space_id = d.space_id
                    AND sm.user_id = $2
                    AND (s.is_public OR sm.user_id = $2)
                )
                ORDER BY
                    CASE WHEN d.title ILIKE $1 THEN 0 ELSE 1 END,
                    d.updated_at DESC
                LIMIT $3 OFFSET $4
                "#.to_string();
                sqlx::query_as(&search_sql)
                    .bind(&query_pattern)
                    .bind(user_uuid)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&*self.pool)
                    .await?
            }
        };

        // Generate snippets for each result
        let results_with_snippets: Vec<SearchResultRow> = results.into_iter()
            .map(|mut row| {
                // Extract a snippet around the match
                let snippet = generate_snippet(&row.content, query);
                row.content = serde_json::Value::String(snippet.clone());
                row
            })
            .collect();

        Ok((results_with_snippets, total))
    }
}

// Helper function to generate a search result snippet
fn generate_snippet(content: &serde_json::Value, query: &str) -> String {
    // Extract text content from JSONB
    let text = content.as_str()
        .map(|s| s.to_string())
        .or_else(|| serde_json::to_string(content).ok())
        .unwrap_or_default();

    if text.is_empty() {
        return String::new();
    }

    // Simple case-insensitive find
    let lower_text = text.to_lowercase();
    let query_lower = query.to_lowercase();

    if let Some(pos) = lower_text.find(&query_lower) {
        let start = pos.saturating_sub(50);
        let end = (pos + query.len() + 100).min(text.len());

        // Find safe UTF-8 boundaries
        let safe_start = text[..start]
            .char_indices().next_back()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);

        let safe_end = text[end..]
            .char_indices()
            .next()
            .map(|(i, _)| end + i)
            .unwrap_or_else(|| text.len());

        let mut snippet = if safe_start > 0 { "...".to_string() } else { String::new() };
        snippet.push_str(&text[safe_start..safe_end]);
        if safe_end < text.len() { snippet.push_str("..."); }

        // Highlight the match using case-insensitive regex
        let escaped_query = regex::escape(query);
        if let Ok(regex) = Regex::new(&format!("(?i){}", escaped_query)) {
            let highlighted = regex.replace_all(&snippet, |caps: &Captures| {
                format!("**{}**", &caps[0])
            });
            highlighted.into_owned()
        } else {
            snippet
        }
    } else {
        // Return first 150 chars if no match found
        let truncated = if text.len() > 150 {
            // Find safe UTF-8 boundary by iterating char indices
            let safe_boundary = text.char_indices()
                .take(150)
                .last()
                .map(|(i, _)| i)
                .unwrap_or(text.len());
            format!("{}...", &text[..safe_boundary])
        } else {
            format!("{}...", text)
        };
        truncated
    }
}
