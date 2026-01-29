//! Document search indexing for PostgreSQL full-text search
//!
//! This module provides a search indexing system using PostgreSQL's built-in
//! full-text search capabilities with GIN and trigram indexes.

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use tracing;
use uuid::Uuid;

/// Represents the content extracted from a document for indexing
///
/// This struct contains the essential information needed to index a document
/// for full-text search, including the document ID, title, content, and space.
#[derive(Debug, Clone)]
pub struct DocumentContent {
    /// Unique identifier of the document
    pub document_id: Uuid,
    /// Document title for search relevance
    pub title: String,
    /// Document content (can be JSON, Delta/Quill format, or plain text)
    pub content: serde_json::Value,
    /// ID of the space containing this document
    pub space_id: Uuid,
}

/// Indexer trait for document search indexing
///
/// This trait defines the interface for search indexers, allowing for
/// different implementations (PostgreSQL, Elasticsearch, etc.).
#[async_trait]
pub trait SearchIndexer {
    /// Index a single document for search
    ///
    /// # Arguments
    ///
    /// * `doc` - Reference to the document content to index
    ///
    /// # Returns
    ///
    /// `Ok(())` if indexing succeeded, `Err(sqlx::Error)` otherwise
    async fn index_document(&self, doc: &DocumentContent) -> Result<(), sqlx::Error>;

    /// Remove a document from the search index
    ///
    /// # Arguments
    ///
    /// * `document_id` - ID of the document to remove from index
    ///
    /// # Returns
    ///
    /// `Ok(())` if removal succeeded, `Err(sqlx::Error)` otherwise
    async fn remove_document(&self, document_id: &Uuid) -> Result<(), sqlx::Error>;

    /// Index multiple documents in bulk
    ///
    /// # Arguments
    ///
    /// * `docs` - Slice of document contents to index
    ///
    /// # Returns
    ///
    /// Number of successfully indexed documents
    async fn bulk_index(&self, docs: &[DocumentContent]) -> Result<usize, sqlx::Error>;

    /// Rebuild the entire search index from scratch
    ///
    /// This operation may take significant time on large datasets.
    ///
    /// # Returns
    ///
    /// Number of successfully indexed documents
    async fn rebuild_index(&self) -> Result<usize, sqlx::Error>;
}

/// PostgreSQL-based search indexer using full-text search
///
/// This indexer uses PostgreSQL's native full-text search capabilities
/// with GIN indexes for fast text search and trigram indexes for
/// fuzzy matching.
pub struct PostgresSearchIndexer {
    pool: Arc<PgPool>,
}

impl PostgresSearchIndexer {
    /// Create a new PostgreSQL search indexer
    ///
    /// # Arguments
    ///
    /// * `pool` - Arc-wrapped PostgreSQL connection pool
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Get reference to the internal connection pool
    ///
    /// This method is primarily intended for testing purposes.
    pub fn pool(&self) -> Option<&PgPool> {
        Some(&self.pool)
    }

    /// Create the full-text search indexes if they don't exist
    ///
    /// This method sets up:
    /// - `pg_trgm` extension for trigram similarity
    /// - GIN index on document titles
    /// - GIN index on document content using tsvector
    /// - B-tree index on updated_at for recent document sorting
    ///
    /// # Returns
    ///
    /// `Ok(())` if indexes were created successfully
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

/// High-level manager for search index operations
///
/// `SearchIndexManager` provides a simplified interface for managing
/// the search index, including initialization, rebuilding, and
/// individual document operations.
pub struct SearchIndexManager {
    indexer: PostgresSearchIndexer,
}

impl SearchIndexManager {
    /// Create a new search index manager
    ///
    /// # Arguments
    ///
    /// * `pool` - Arc-wrapped PostgreSQL connection pool
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            indexer: PostgresSearchIndexer::new(pool),
        }
    }

    /// Initialize search indexes in the database
    ///
    /// This method creates necessary extensions and indexes for
    /// full-text search functionality. Should be called on application startup.
    ///
    /// # Returns
    ///
    /// `Ok(())` if initialization succeeded
    pub async fn initialize(&self) -> Result<(), sqlx::Error> {
        self.indexer.create_indexes().await?;
        Ok(())
    }

    /// Rebuild all search indexes from scratch
    ///
    /// This operation re-indexes all non-archived documents in the database.
    /// May take significant time on large datasets.
    ///
    /// # Returns
    ///
    /// Number of successfully indexed documents
    pub async fn rebuild_all(&self) -> Result<usize, sqlx::Error> {
        self.indexer.rebuild_index().await
    }

    /// Index a single document
    ///
    /// # Arguments
    ///
    /// * `doc` - Reference to the document content to index
    ///
    /// # Returns
    ///
    /// `Ok(())` if indexing succeeded
    pub async fn index(&self, doc: &DocumentContent) -> Result<(), sqlx::Error> {
        self.indexer.index_document(doc).await
    }

    /// Remove a document from the search index
    ///
    /// # Arguments
    ///
    /// * `document_id` - ID of the document to remove from index
    ///
    /// # Returns
    ///
    /// `Ok(())` if removal succeeded
    pub async fn remove(&self, document_id: &Uuid) -> Result<(), sqlx::Error> {
        self.indexer.remove_document(document_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};

    // ========================================
    // Unit Tests for extract_text_content
    // ========================================

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
    fn test_extract_text_content_quill_with_newlines_normalized() {
        let content = serde_json::json!({
            "ops": [
                {"insert": "Line 1\nLine 2\nLine 3"}
            ]
        });
        let result = PostgresSearchIndexer::extract_text_content(&content);
        // Newlines should be replaced with spaces and then normalized
        assert_eq!(result, "Line 1 Line 2 Line 3");
    }

    #[test]
    fn test_extract_text_content_quill_with_multiple_spaces() {
        let content = serde_json::json!({
            "ops": [
                {"insert": "Word1  Word2   Word3"}
            ]
        });
        let result = PostgresSearchIndexer::extract_text_content(&content);
        // Multiple spaces should be normalized to single spaces
        assert_eq!(result, "Word1 Word2 Word3");
    }

    #[test]
    fn test_extract_text_content_empty_object() {
        let content = serde_json::json!({});
        let result = PostgresSearchIndexer::extract_text_content(&content);
        assert_eq!(result, "{}");
    }

    #[test]
    fn test_extract_text_content_object_without_ops() {
        let content = serde_json::json!({
            "title": "Test",
            "body": "Content"
        });
        let result = PostgresSearchIndexer::extract_text_content(&content);
        assert_eq!(result, r#"{"body":"Content","title":"Test"}"#);
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
    fn test_extract_text_content_float() {
        let content = serde_json::json!(3.14159);
        let result = PostgresSearchIndexer::extract_text_content(&content);
        assert_eq!(result, "3.14159");
    }

    #[test]
    fn test_extract_text_content_bool() {
        let content = serde_json::json!(true);
        let result = PostgresSearchIndexer::extract_text_content(&content);
        assert_eq!(result, "true");
    }

    #[test]
    fn test_extract_text_content_null() {
        let content = serde_json::json!(null);
        let result = PostgresSearchIndexer::extract_text_content(&content);
        assert_eq!(result, "null");
    }

    #[test]
    fn test_extract_text_content_nested_quill_ops() {
        let content = serde_json::json!({
            "ops": [
                {"insert": "Title", "attributes": {"bold": true, "header": 1}},
                {"insert": "\n"},
                {"insert": "Paragraph"},
                {"insert": "\n"},
                {"insert": "List item", "attributes": {"list": "bullet"}}
            ]
        });
        let result = PostgresSearchIndexer::extract_text_content(&content);
        assert_eq!(result, "Title Paragraph List item");
    }

    #[test]
    fn test_extract_text_content_empty_insert() {
        let content = serde_json::json!({
            "ops": [
                {"insert": ""},
                {"insert": "\n"},
                {"insert": "Content"}
            ]
        });
        let result = PostgresSearchIndexer::extract_text_content(&content);
        assert_eq!(result, "Content");
    }

    // ========================================
    // Mock-based Unit Tests for SearchIndexer
    // ========================================

    mock! {
        Indexer {}

        #[async_trait::async_trait]
        impl SearchIndexer for Indexer {
            async fn index_document(&self, doc: &DocumentContent) -> Result<(), sqlx::Error>;
            async fn remove_document(&self, document_id: &Uuid) -> Result<(), sqlx::Error>;
            async fn bulk_index(&self, docs: &[DocumentContent]) -> Result<usize, sqlx::Error>;
            async fn rebuild_index(&self) -> Result<usize, sqlx::Error>;
        }
    }

    #[tokio::test]
    async fn test_mock_indexer_success() {
        let mut mock_indexer = MockIndexer::new();
        let doc_id = Uuid::new_v4();
        let space_id = Uuid::new_v4();
        let doc = DocumentContent {
            document_id: doc_id,
            title: "Test Document".to_string(),
            content: serde_json::json!({"text": "test"}),
            space_id,
        };

        // Set up expectation: index_document should succeed
        mock_indexer
            .expect_index_document()
            .returning(|_| Ok(()))
            .times(1);

        let result = mock_indexer.index_document(&doc).await;
        assert!(result.is_ok(), "index_document should succeed");
    }

    #[tokio::test]
    async fn test_mock_indexer_failure() {
        let mut mock_indexer = MockIndexer::new();
        let doc_id = Uuid::new_v4();
        let space_id = Uuid::new_v4();
        let doc = DocumentContent {
            document_id: doc_id,
            title: "Test Document".to_string(),
            content: serde_json::json!({"text": "test"}),
            space_id,
        };

        // Set up expectation: index_document should fail
        mock_indexer
            .expect_index_document()
            .returning(|_| {
                Err(sqlx::Error::RowNotFound)
            })
            .times(1);

        let result = mock_indexer.index_document(&doc).await;
        assert!(result.is_err(), "index_document should fail");
    }

    #[tokio::test]
    async fn test_mock_remove_document_success() {
        let mut mock_indexer = MockIndexer::new();
        let doc_id = Uuid::new_v4();

        mock_indexer
            .expect_remove_document()
            .returning(|_| Ok(()))
            .times(1);

        let result = mock_indexer.remove_document(&doc_id).await;
        assert!(result.is_ok(), "remove_document should succeed");
    }

    #[tokio::test]
    async fn test_mock_bulk_index_partial_failure() {
        let mut mock_indexer = MockIndexer::new();
        let docs = vec![
            DocumentContent {
                document_id: Uuid::new_v4(),
                title: "Doc 1".to_string(),
                content: serde_json::json!({"text": "test1"}),
                space_id: Uuid::new_v4(),
            },
            DocumentContent {
                document_id: Uuid::new_v4(),
                title: "Doc 2".to_string(),
                content: serde_json::json!({"text": "test2"}),
                space_id: Uuid::new_v4(),
            },
        ];

        // Simulate partial failure: index 1 out of 2
        mock_indexer
            .expect_bulk_index()
            .returning(|_| Ok(1))
            .times(1);

        let result = mock_indexer.bulk_index(&docs).await;
        assert!(result.is_ok(), "bulk_index should succeed");
        assert_eq!(result.unwrap(), 1, "should report 1 indexed document");
    }

    #[tokio::test]
    async fn test_mock_bulk_index_empty_list() {
        let mut mock_indexer = MockIndexer::new();
        let docs: Vec<DocumentContent> = vec![];

        mock_indexer
            .expect_bulk_index()
            .returning(|_| Ok(0))
            .times(1);

        let result = mock_indexer.bulk_index(&docs).await;
        assert!(result.is_ok(), "bulk_index with empty list should succeed");
        assert_eq!(result.unwrap(), 0, "should report 0 indexed documents");
    }

    #[tokio::test]
    async fn test_mock_rebuild_index() {
        let mut mock_indexer = MockIndexer::new();

        mock_indexer
            .expect_rebuild_index()
            .returning(|| Ok(100))
            .times(1);

        let result = mock_indexer.rebuild_index().await;
        assert!(result.is_ok(), "rebuild_index should succeed");
        assert_eq!(result.unwrap(), 100, "should report 100 indexed documents");
    }

    // ========================================
    // Edge Case Tests
    // ========================================

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
    fn test_document_content_with_empty_title() {
        let doc_id = Uuid::new_v4();
        let space_id = Uuid::new_v4();
        let content = DocumentContent {
            document_id: doc_id,
            title: "".to_string(),
            content: serde_json::json!({}),
            space_id,
        };
        assert_eq!(content.title, "");
        assert!(content.content.is_object());
    }

    #[test]
    fn test_document_content_with_large_content() {
        let large_text = "x".repeat(10000);
        let content = DocumentContent {
            document_id: Uuid::new_v4(),
            title: "Large Document".to_string(),
            content: serde_json::json!(large_text),
            space_id: Uuid::new_v4(),
        };
        let extracted = PostgresSearchIndexer::extract_text_content(&content.content);
        assert_eq!(extracted.len(), 10000);
    }
}
