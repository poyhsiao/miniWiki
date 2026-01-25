use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use tracing;
use uuid::Uuid;

/// Represents the content extracted from a document for indexing
#[derive(Debug, Clone)]
pub struct DocumentContent {
    pub document_id: Uuid,
    pub title: String,
    pub content: serde_json::Value,
    pub space_id: Uuid,
}

/// Indexer trait for document search indexing
#[async_trait]
pub trait SearchIndexer {
    /// Index a document for search
    async fn index_document(&self, doc: &DocumentContent) -> Result<(), sqlx::Error>;

    /// Remove a document from the index
    async fn remove_document(&self, document_id: &Uuid) -> Result<(), sqlx::Error>;

    /// Bulk index multiple documents
    async fn bulk_index(&self, docs: &[DocumentContent]) -> Result<usize, sqlx::Error>;

    /// Rebuild the entire search index
    async fn rebuild_index(&self) -> Result<usize, sqlx::Error>;
}

/// PostgreSQL-based search indexer using full-text search
pub struct PostgresSearchIndexer {
    pool: Arc<PgPool>,
}

impl PostgresSearchIndexer {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Get reference to the pool (for testing)
    pub fn pool(&self) -> Option<&PgPool> {
        Some(&self.pool)
    }

    /// Create the full-text search index if it doesn't exist
    pub async fn create_indexes(&self) -> Result<(), sqlx::Error> {
        // Enable pg_trgm extension for trigram similarity search
        sqlx::query("CREATE EXTENSION IF NOT EXISTS pg_trgm")
            .execute(&*self.pool)
            .await?;

        // Create a GIN index on the title for fast text search
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_documents_title_search
            ON documents USING gin (title gin_trgm_ops)
            WHERE is_archived = false
            "#,
        )
        .execute(&*self.pool)
        .await?;

        // Create an index on content_text for fast text search
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_documents_content_search
            ON documents USING gin (to_tsvector('english', COALESCE(content_text, '')))
            WHERE is_archived = false
            "#,
        )
        .execute(&*self.pool)
        .await?;

        // Create updated_at index for sorting recent documents
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_documents_updated_desc
            ON documents (updated_at DESC)
            WHERE is_archived = false
            "#,
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    /// Extract plain text content from JSONB document content
    fn extract_text_content(content: &serde_json::Value) -> String {
        match content {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Object(map) => {
                // Handle Delta/Quill JSON format
                if let Some(ops) = map.get("ops") {
                    if let Some(arr) = ops.as_array() {
                        return arr
                            .iter()
                            .filter_map(|op| op.get("insert").and_then(|i| i.as_str()).map(|s| s.to_string()))
                            .collect::<Vec<String>>()
                            .join(" ")
                            .replace('\n', " ")
                            .split_whitespace()
                            .collect::<Vec<&str>>()
                            .join(" ");
                    }
                }
                serde_json::to_string(content).unwrap_or_default()
            },
            _ => serde_json::to_string(content).unwrap_or_default(),
        }
    }
}

#[async_trait]
impl SearchIndexer for PostgresSearchIndexer {
    async fn index_document(&self, doc: &DocumentContent) -> Result<(), sqlx::Error> {
        let content_text = Self::extract_text_content(&doc.content);

        sqlx::query(
            r#"
            UPDATE documents
            SET
                content_text = $1,
                updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(content_text)
        .bind(doc.document_id)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    async fn remove_document(&self, document_id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE documents
            SET content_text = NULL
            WHERE id = $1
            "#,
        )
        .bind(document_id)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    async fn bulk_index(&self, docs: &[DocumentContent]) -> Result<usize, sqlx::Error> {
        let mut indexed = 0;
        let mut failed_docs = Vec::new();

        for doc in docs {
            match self.index_document(doc).await {
                Ok(()) => indexed += 1,
                Err(e) => {
                    tracing::error!(
                        "Failed to index document: id={}, title={}, error={}",
                        doc.document_id,
                        doc.title,
                        e
                    );
                    failed_docs.push((doc.document_id, doc.title.clone(), e.to_string()));
                },
            }
        }

        if !failed_docs.is_empty() {
            tracing::warn!(
                "bulk_index: {} of {} documents failed to index",
                failed_docs.len(),
                docs.len()
            );
        }

        Ok(indexed)
    }

    async fn rebuild_index(&self) -> Result<usize, sqlx::Error> {
        // Get all non-archived documents
        let documents: Vec<(Uuid, serde_json::Value)> = sqlx::query_as(
            r#"
            SELECT id, content FROM documents WHERE is_archived = false
            "#,
        )
        .fetch_all(&*self.pool)
        .await?;

        let mut indexed = 0;

        for (id, content) in documents {
            let content_text = Self::extract_text_content(&content);

            sqlx::query(
                r#"
                UPDATE documents
                SET content_text = $1, updated_at = NOW()
                WHERE id = $2
                "#,
            )
            .bind(content_text)
            .bind(id)
            .execute(&*self.pool)
            .await?;

            indexed += 1;
        }

        Ok(indexed)
    }
}

/// Index management utilities
pub struct SearchIndexManager {
    indexer: PostgresSearchIndexer,
}

impl SearchIndexManager {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            indexer: PostgresSearchIndexer::new(pool),
        }
    }

    /// Initialize search indexes
    pub async fn initialize(&self) -> Result<(), sqlx::Error> {
        self.indexer.create_indexes().await?;
        Ok(())
    }

    /// Rebuild all search indexes
    pub async fn rebuild_all(&self) -> Result<usize, sqlx::Error> {
        self.indexer.rebuild_index().await
    }

    /// Index a single document
    pub async fn index(&self, doc: &DocumentContent) -> Result<(), sqlx::Error> {
        self.indexer.index_document(doc).await
    }

    /// Remove a document from index
    pub async fn remove(&self, document_id: &Uuid) -> Result<(), sqlx::Error> {
        self.indexer.remove_document(document_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_content_string() {
        let content = serde_json::json!("Hello World");
        let result = PostgresSearchIndexer::extract_text_content(&content);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_extract_text_content_quill_delta() {
        let content = serde_json::json!({
            "ops": [
                {"insert": "Hello "},
                {"insert": "World", "attributes": {"bold": true}},
                {"insert": "\n"}
            ]
        });
        let result = PostgresSearchIndexer::extract_text_content(&content);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_extract_text_content_quill_with_headers() {
        let content = serde_json::json!({
            "ops": [
                {"insert": "Title" },
                {"insert": "\n", "attributes": {"header": 1}},
                {"insert": "Body text here"}
            ]
        });
        let result = PostgresSearchIndexer::extract_text_content(&content);
        assert_eq!(result, "Title Body text here");
    }

    #[test]
    fn test_extract_text_content_empty_object() {
        let content = serde_json::json!({});
        let result = PostgresSearchIndexer::extract_text_content(&content);
        assert_eq!(result, "{}");
    }

    #[test]
    fn test_extract_text_content_array() {
        let content = serde_json::json!(["item1", "item2", "item3"]);
        let result = PostgresSearchIndexer::extract_text_content(&content);
        assert_eq!(result, r#"["item1","item2","item3"]"#);
    }

    #[test]
    fn test_extract_text_content_number() {
        let content = serde_json::json!(42);
        let result = PostgresSearchIndexer::extract_text_content(&content);
        assert_eq!(result, "42");
    }

    #[test]
    fn test_extract_text_content_null() {
        let content = serde_json::json!(null);
        let result = PostgresSearchIndexer::extract_text_content(&content);
        assert_eq!(result, "null");
    }

    #[test]
    fn test_document_content_creation() {
        let doc_id = Uuid::new_v4();
        let space_id = Uuid::new_v4();
        let content = DocumentContent {
            document_id: doc_id,
            title: "Test Document".to_string(),
            content: serde_json::json!({"text": "test content"}),
            space_id,
        };
        assert_eq!(doc_id, content.document_id);
        assert_eq!("Test Document", content.title);
        assert_eq!(space_id, content.space_id);
    }

    #[test]
    fn test_search_index_manager_creation() {
        let manager = SearchIndexManager::new(Arc::new(PgPool::connect_lazy("postgres://test").unwrap()));
        assert!(manager.indexer.pool().is_some());
    }
}
