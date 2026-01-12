use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use chrono::NaiveDateTime;

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

#[derive(Debug, Clone, FromRow)]
pub struct DocumentVersionRow {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version_number: i32,
    pub content: sqlx::types::Json<serde_json::Value>,
    pub title: String,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub change_summary: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
struct ContentRow {
    content: serde_json::Value,
}

#[derive(Debug, Clone, FromRow)]
struct DocumentPathRow {
    id: Option<Uuid>,
    title: Option<String>,
    level: Option<i32>,
}

pub struct DocumentRepository {
    pool: PgPool,
}

impl DocumentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        space_id: &str,
        parent_id: Option<&str>,
        title: &str,
        icon: Option<&str>,
        content: Option<serde_json::Value>,
        created_by: &str,
    ) -> Result<DocumentRow, sqlx::Error> {
        let space_uuid = Uuid::parse_str(space_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let parent_uuid = match parent_id {
            Some(id) => Some(Uuid::parse_str(id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?),
            None => None,
        };
        let created_by_uuid = Uuid::parse_str(created_by).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let content_value = content.unwrap_or_else(|| serde_json::json!({}));
        let content_size = content_value.to_string().len() as i32;

        let document = sqlx::query_as!(
            DocumentRow,
            r#"
            INSERT INTO documents (
                id, space_id, parent_id, title, icon, content,
                content_size, is_archived, created_by, last_edited_by
            ) VALUES (
                gen_random_uuid(), $1, $2, $3, $4, $5,
                $6, false, $7, $7
            )
            RETURNING *
            "#,
            space_uuid,
            parent_uuid,
            title,
            icon,
            content_value,
            content_size,
            created_by_uuid
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(document)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<DocumentRow>, sqlx::Error> {
        let document_id = Uuid::parse_str(id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let document = sqlx::query_as!(
            DocumentRow,
            r#"SELECT * FROM documents WHERE id = $1"#,
            document_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(document)
    }

    pub async fn update(
        &self,
        id: &str,
        title: Option<&str>,
        icon: Option<&str>,
        content: Option<serde_json::Value>,
        last_edited_by: &str,
    ) -> Result<Option<DocumentRow>, sqlx::Error> {
        let document_id = Uuid::parse_str(id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let editor_uuid = Uuid::parse_str(last_edited_by).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let document = sqlx::query_as!(
            DocumentRow,
            r#"
            UPDATE documents
            SET
                title = COALESCE($2, title),
                icon = COALESCE($3, icon),
                content = COALESCE($4, content),
                content_size = COALESCE(length($4::text), content_size),
                last_edited_by = $5,
                updated_at = NOW()
            WHERE id = $1 AND is_archived = false
            RETURNING *
            "#,
            document_id,
            title,
            icon,
            content,
            editor_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(document)
    }

    pub async fn delete(&self, id: &str) -> Result<bool, sqlx::Error> {
        let document_id = Uuid::parse_str(id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let result = sqlx::query!(
            r#"
            UPDATE documents
            SET is_archived = true, archived_at = NOW()
            WHERE id = $1 AND is_archived = false
            "#,
            document_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list_in_space(
        &self,
        space_id: &str,
        parent_id: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<DocumentRow>, i64), sqlx::Error> {
        let space_uuid = Uuid::parse_str(space_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let parent_uuid = match parent_id {
            Some(id) => Some(Uuid::parse_str(id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?),
            None => None,
        };

        let documents = sqlx::query_as!(
            DocumentRow,
            r#"
            SELECT * FROM documents
            WHERE space_id = $1 AND is_archived = false
            AND parent_id IS NOT DISTINCT FROM $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            space_uuid,
            parent_uuid,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM documents
            WHERE space_id = $1 AND is_archived = false
            AND parent_id IS NOT DISTINCT FROM $2
            "#,
            space_uuid,
            parent_uuid
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);

        Ok((documents, total as i64))
    }

    pub async fn get_children(&self, parent_id: &str) -> Result<(Vec<DocumentRow>, i64), sqlx::Error> {
        let parent_uuid = Uuid::parse_str(parent_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let documents = sqlx::query_as!(
            DocumentRow,
            r#"
            SELECT * FROM documents
            WHERE parent_id = $1 AND is_archived = false
            ORDER BY created_at DESC
            "#,
            parent_uuid
        )
        .fetch_all(&self.pool)
        .await?;

        let total = documents.len() as i64;

        Ok((documents, total))
    }

    pub async fn get_document_path(&self, document_id: &str) -> Result<Vec<(Uuid, String, i32)>, sqlx::Error> {
        let doc_uuid = Uuid::parse_str(document_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        // Use the SQL function we created
        let path = sqlx::query_as!(
            DocumentPathRow,
            r#"
            SELECT id, title, level FROM get_document_path($1)
            "#,
            doc_uuid
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .filter_map(|row| {
            Some((
                row.id?,
                row.title?,
                row.level?,
            ))
        })
        .collect();

        Ok(path)
    }

    // Version operations

    pub async fn create_version(
        &self,
        document_id: &str,
        content: serde_json::Value,
        title: &str,
        created_by: &str,
        change_summary: Option<&str>,
    ) -> Result<DocumentVersionRow, sqlx::Error> {
        let doc_uuid = Uuid::parse_str(document_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let creator_uuid = Uuid::parse_str(created_by).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        // Convert change_summary to String for sqlx compatibility
        let summary_str = change_summary.map(|s| s.to_string()).unwrap_or_default();

        // Call the SQL function for version creation
        let version_id = sqlx::query_scalar!(
            r#"SELECT create_document_version($1, $2, $3, $4, $5) as id"#,
            doc_uuid,
            content,
            title,
            creator_uuid,
            summary_str
        )
        .fetch_one(&self.pool)
        .await?;

        // Fetch the created version
        let version = sqlx::query_as!(
            DocumentVersionRow,
            r#"SELECT * FROM document_versions WHERE id = $1"#,
            version_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(version)
    }

    pub async fn list_versions(
        &self,
        document_id: &str,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<DocumentVersionRow>, i64), sqlx::Error> {
        let doc_uuid = Uuid::parse_str(document_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let versions = sqlx::query_as!(
            DocumentVersionRow,
            r#"
            SELECT * FROM document_versions
            WHERE document_id = $1
            ORDER BY version_number DESC
            LIMIT $2 OFFSET $3
            "#,
            doc_uuid,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM document_versions WHERE document_id = $1"#,
            doc_uuid
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);

        Ok((versions, total as i64))
    }

    pub async fn get_version(&self, document_id: &str, version_number: i32) -> Result<Option<DocumentVersionRow>, sqlx::Error> {
        let doc_uuid = Uuid::parse_str(document_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let version = sqlx::query_as!(
            DocumentVersionRow,
            r#"
            SELECT * FROM document_versions
            WHERE document_id = $1 AND version_number = $2
            "#,
            doc_uuid,
            version_number
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(version)
    }

    pub async fn restore_version(
        &self,
        document_id: &str,
        version_number: i32,
        restored_by: &str,
    ) -> Result<Option<DocumentRow>, sqlx::Error> {
        let doc_uuid = Uuid::parse_str(document_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let restorer_uuid = Uuid::parse_str(restored_by).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        // First, check if the version exists
        let version_exists = sqlx::query_as!(
            DocumentVersionRow,
            r#"SELECT * FROM document_versions WHERE document_id = $1 AND version_number = $2 LIMIT 1"#,
            doc_uuid,
            version_number
        )
        .fetch_optional(&self.pool)
        .await?;

        if version_exists.is_none() {
            return Ok(None);
        }

        // Call the SQL function for version restore
        sqlx::query!(
            r#"SELECT restore_document_to_version($1, $2, $3) as result"#,
            doc_uuid,
            version_number,
            restorer_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        // Fetch the restored document
        let document = sqlx::query_as!(
            DocumentRow,
            r#"SELECT * FROM documents WHERE id = $1"#,
            doc_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(document)
    }

    pub async fn get_version_diff(
        &self,
        document_id: &str,
        version_from: i32,
        version_to: i32,
    ) -> Result<Option<(serde_json::Value, serde_json::Value)>, sqlx::Error> {
        let doc_uuid = Uuid::parse_str(document_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let from_content_row: Option<ContentRow> = sqlx::query_as!(
            ContentRow,
            r#"SELECT content FROM document_versions WHERE document_id = $1 AND version_number = $2"#,
            doc_uuid, version_from
        )
        .fetch_optional(&self.pool)
        .await?;

        let to_content_row: Option<ContentRow> = sqlx::query_as!(
            ContentRow,
            r#"SELECT content FROM document_versions WHERE document_id = $1 AND version_number = $2"#,
            doc_uuid, version_to
        )
        .fetch_optional(&self.pool)
        .await?;

        match (from_content_row, to_content_row) {
            (Some(from), Some(to)) => Ok(Some((from.content, to.content))),
            _ => Ok(None),
        }
    }

    pub async fn check_space_access(&self, space_id: &str, user_id: &str) -> Result<bool, sqlx::Error> {
        let space_uuid = Uuid::parse_str(space_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let user_uuid = Uuid::parse_str(user_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let result = sqlx::query!(
            r#"
            SELECT 1 as found FROM spaces
            WHERE id = $1 AND owner_id = $2
            UNION
            SELECT 1 as found FROM space_memberships
            WHERE space_id = $1 AND user_id = $2
            LIMIT 1
            "#,
            space_uuid,
            user_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.is_some())
    }

    pub async fn check_document_access(&self, document_id: &str, user_id: &str) -> Result<bool, sqlx::Error> {
        let doc_uuid = Uuid::parse_str(document_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let user_uuid = Uuid::parse_str(user_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let result = sqlx::query!(
            r#"
            SELECT 1 as found FROM documents d
            JOIN spaces s ON d.space_id = s.id
            WHERE d.id = $1 AND (
                s.owner_id = $2
                OR s.id IN (SELECT space_id FROM space_memberships WHERE user_id = $2)
            )
            LIMIT 1
            "#,
            doc_uuid,
            user_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.is_some())
    }
}
