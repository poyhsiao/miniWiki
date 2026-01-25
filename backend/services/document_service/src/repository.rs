use chrono::NaiveDateTime;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

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
    // Sync-related fields
    pub version: i64,
    pub last_synced_at: Option<NaiveDateTime>,
    pub vector_clock: Option<serde_json::Value>,
    pub client_id: Option<Uuid>,
    pub sync_state: Option<String>,
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
pub struct SpaceRow {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub user_role: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct SpaceMembershipRow {
    pub id: Uuid,
    pub space_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: NaiveDateTime,
    pub invited_by: Uuid,
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

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct CommentRow {
    pub id: Uuid,
    pub document_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub author_id: Uuid,
    pub content: String,
    pub is_resolved: bool,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
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

        let document = sqlx::query_as!(DocumentRow, r#"SELECT * FROM documents WHERE id = $1"#, document_id)
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

        let documents = match parent_id {
            Some(parent_id_str) => {
                let parent_uuid =
                    Uuid::parse_str(parent_id_str).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
                sqlx::query_as!(
                    DocumentRow,
                    r#"
                    SELECT * FROM documents
                    WHERE space_id = $1 AND is_archived = false
                    AND parent_id = $2
                    ORDER BY created_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                    space_uuid,
                    parent_uuid,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?
            },
            None => {
                sqlx::query_as!(
                    DocumentRow,
                    r#"
                    SELECT * FROM documents
                    WHERE space_id = $1 AND is_archived = false
                    AND parent_id IS NULL
                    ORDER BY created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    space_uuid,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(&self.pool)
                .await?
            },
        };

        let total = match parent_id {
            Some(parent_id_str) => {
                let parent_uuid =
                    Uuid::parse_str(parent_id_str).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
                sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!" FROM documents
                    WHERE space_id = $1 AND is_archived = false
                    AND parent_id = $2
                    "#,
                    space_uuid,
                    parent_uuid
                )
                .fetch_one(&self.pool)
                .await?
                .count
            },
            None => {
                sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!" FROM documents
                    WHERE space_id = $1 AND is_archived = false
                    "#,
                    space_uuid
                )
                .fetch_one(&self.pool)
                .await?
                .count
            },
        };

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
        .filter_map(|row| Some((row.id?, row.title?, row.level?)))
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
            r#"SELECT COUNT(*) as "count!" FROM document_versions WHERE document_id = $1"#,
            doc_uuid
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        Ok((versions, total as i64))
    }

    pub async fn get_version(
        &self,
        document_id: &str,
        version_number: i32,
    ) -> Result<Option<DocumentVersionRow>, sqlx::Error> {
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

        let version_exists: Option<DocumentVersionRow> = sqlx::query_as!(
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

        sqlx::query!(
            r#"SELECT restore_document_to_version($1, $2, $3) as result"#,
            doc_uuid,
            version_number,
            restorer_uuid
        )
        .fetch_one(&self.pool)
        .await?;

        let document = sqlx::query_as!(DocumentRow, r#"SELECT * FROM documents WHERE id = $1"#, doc_uuid)
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
            doc_uuid,
            version_from
        )
        .fetch_optional(&self.pool)
        .await?;

        let to_content_row: Option<ContentRow> = sqlx::query_as!(
            ContentRow,
            r#"SELECT content FROM document_versions WHERE document_id = $1 AND version_number = $2"#,
            doc_uuid,
            version_to
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

        let result = sqlx::query_as::<_, (i32,)>(
            r#"
            SELECT 1 as found FROM spaces
            WHERE id = $1 AND owner_id = $2
            UNION
            SELECT 1 as found FROM space_memberships
            WHERE space_id = $1 AND user_id = $2
            LIMIT 1
            "#,
        )
        .bind(space_uuid)
        .bind(user_uuid)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.is_some())
    }

    pub async fn check_document_access(&self, document_id: &str, user_id: &str) -> Result<bool, sqlx::Error> {
        let doc_uuid = Uuid::parse_str(document_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let user_uuid = Uuid::parse_str(user_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let result = sqlx::query_as::<_, (i32,)>(
            r#"
            SELECT 1 as found FROM documents d
            JOIN spaces s ON d.space_id = s.id
            WHERE d.id = $1 AND (
                s.owner_id = $2
                OR s.id IN (SELECT space_id FROM space_memberships WHERE user_id = $2)
            )
            LIMIT 1
            "#,
        )
        .bind(doc_uuid)
        .bind(user_uuid)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.is_some())
    }

    // Space operations

    pub async fn list_spaces(&self, user_id: &str) -> Result<Vec<SpaceRow>, sqlx::Error> {
        let user_uuid = Uuid::parse_str(user_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let spaces = sqlx::query_as!(
            SpaceRow,
            r#"
            SELECT
                s.id, s.owner_id, s.name, s.icon, s.description,
                s.is_public, s.created_at, s.updated_at,
                sm.role as user_role
            FROM spaces s
            LEFT JOIN space_memberships sm ON s.id = sm.space_id AND sm.user_id = $1
            WHERE s.owner_id = $1 OR sm.user_id = $1 OR s.is_public = true
            ORDER BY s.updated_at DESC
            "#,
            user_uuid
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(spaces)
    }

    pub async fn create_space(
        &self,
        owner_id: &str,
        name: &str,
        icon: Option<&str>,
        description: Option<&str>,
        is_public: bool,
    ) -> Result<SpaceRow, sqlx::Error> {
        let owner_uuid = Uuid::parse_str(owner_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let space = sqlx::query_as!(
            SpaceRow,
            r#"
            INSERT INTO spaces (id, owner_id, name, icon, description, is_public)
            VALUES (gen_random_uuid(), $1, $2, $3, $4, $5)
            RETURNING id, owner_id, name, icon, description, is_public, created_at, updated_at, NULL::text as user_role
            "#,
            owner_uuid,
            name,
            icon,
            description,
            is_public
        )
        .fetch_one(&self.pool)
        .await?;

        // Add owner as member
        sqlx::query!(
            r#"
            INSERT INTO space_memberships (id, space_id, user_id, role, invited_by)
            VALUES (gen_random_uuid(), $1, $2, 'owner', $2)
            "#,
            space.id,
            owner_uuid
        )
        .execute(&self.pool)
        .await?;

        Ok(space)
    }

    pub async fn get_space(&self, space_id: &str) -> Result<Option<SpaceRow>, sqlx::Error> {
        let space_uuid = Uuid::parse_str(space_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let space = sqlx::query_as!(
            SpaceRow,
            r#"
            SELECT id, owner_id, name, icon, description, is_public, created_at, updated_at, NULL::text as user_role
            FROM spaces
            WHERE id = $1
            "#,
            space_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(space)
    }

    pub async fn update_space(
        &self,
        space_id: &str,
        name: Option<&str>,
        icon: Option<&str>,
        description: Option<&str>,
        is_public: Option<bool>,
    ) -> Result<Option<SpaceRow>, sqlx::Error> {
        let space_uuid = Uuid::parse_str(space_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let space = sqlx::query_as!(
            SpaceRow,
            r#"
            UPDATE spaces
            SET
                name = COALESCE($2, name),
                icon = COALESCE($3, icon),
                description = COALESCE($4, description),
                is_public = COALESCE($5, is_public),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, owner_id, name, icon, description, is_public, created_at, updated_at, NULL::text as user_role
            "#,
            space_uuid,
            name,
            icon,
            description,
            is_public
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(space)
    }

    pub async fn delete_space(&self, space_id: &str) -> Result<bool, sqlx::Error> {
        let space_uuid = Uuid::parse_str(space_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let result = sqlx::query!(r#"DELETE FROM spaces WHERE id = $1"#, space_uuid)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn is_space_owner(&self, space_id: &str, user_id: &str) -> Result<bool, sqlx::Error> {
        let space_uuid = Uuid::parse_str(space_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let user_uuid = Uuid::parse_str(user_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let space = sqlx::query_as!(
            SpaceRow,
            r#"
            SELECT id, owner_id, name, icon, description, is_public, created_at, updated_at, NULL::text as user_role
            FROM spaces
            WHERE id = $1 AND owner_id = $2
            "#,
            space_uuid,
            user_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(space.is_some())
    }

    pub async fn get_user_space_role(&self, space_id: &str, user_id: &str) -> Result<Option<String>, sqlx::Error> {
        let space_uuid = Uuid::parse_str(space_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let user_uuid = Uuid::parse_str(user_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let result = sqlx::query_as!(
            SpaceMembershipRow,
            r#"SELECT * FROM space_memberships WHERE space_id = $1 AND user_id = $2"#,
            space_uuid,
            user_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|r| r.role))
    }

    pub async fn list_space_members(&self, space_id: &str) -> Result<Vec<SpaceMembershipRow>, sqlx::Error> {
        let space_uuid = Uuid::parse_str(space_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let members = sqlx::query_as!(
            SpaceMembershipRow,
            r#"SELECT * FROM space_memberships WHERE space_id = $1 ORDER BY joined_at"#,
            space_uuid
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(members)
    }

    pub async fn add_space_member(
        &self,
        space_id: &str,
        user_id: &str,
        role: &str,
        invited_by: &str,
    ) -> Result<SpaceMembershipRow, sqlx::Error> {
        let space_uuid = Uuid::parse_str(space_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let user_uuid = Uuid::parse_str(user_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let inviter_uuid = Uuid::parse_str(invited_by).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let membership = sqlx::query_as!(
            SpaceMembershipRow,
            r#"
            INSERT INTO space_memberships (id, space_id, user_id, role, invited_by)
            VALUES (gen_random_uuid(), $1, $2, $3, $4)
            ON CONFLICT (space_id, user_id) DO UPDATE SET role = EXCLUDED.role
            RETURNING *
            "#,
            space_uuid,
            user_uuid,
            role,
            inviter_uuid
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(membership)
    }

    pub async fn update_space_member(
        &self,
        space_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<Option<SpaceMembershipRow>, sqlx::Error> {
        let space_uuid = Uuid::parse_str(space_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let user_uuid = Uuid::parse_str(user_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let membership = sqlx::query_as!(
            SpaceMembershipRow,
            r#"
            UPDATE space_memberships
            SET role = $3
            WHERE space_id = $1 AND user_id = $2
            RETURNING *
            "#,
            space_uuid,
            user_uuid,
            role
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(membership)
    }

    pub async fn remove_space_member(&self, space_id: &str, user_id: &str) -> Result<bool, sqlx::Error> {
        let space_uuid = Uuid::parse_str(space_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let user_uuid = Uuid::parse_str(user_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let result = sqlx::query_as!(
            (),
            r#"DELETE FROM space_memberships WHERE space_id = $1 AND user_id = $2"#,
            space_uuid,
            user_uuid
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ==================== Comment Operations ====================

    pub async fn get_comment(&self, comment_id: &str) -> Result<Option<CommentRow>, sqlx::Error> {
        let comment_uuid = Uuid::parse_str(comment_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let comment = sqlx::query_as!(CommentRow, r#"SELECT * FROM comments WHERE id = $1"#, comment_uuid)
            .fetch_optional(&self.pool)
            .await?;

        Ok(comment)
    }

    pub async fn list_comments(
        &self,
        document_id: &str,
        parent_id: Option<&str>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<(Vec<CommentRow>, i64), sqlx::Error> {
        let document_uuid = Uuid::parse_str(document_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let parent_uuid = parent_id
            .map(|s| Uuid::parse_str(s).map_err(|e| sqlx::Error::Decode(e.to_string().into())))
            .transpose()?;

        let limit_i64 = limit.unwrap_or(50) as i64;
        let offset_i64 = offset.unwrap_or(0) as i64;

        let comments = if let Some(parent) = parent_uuid {
            sqlx::query_as!(
                CommentRow,
                r#"SELECT * FROM comments WHERE document_id = $1 AND parent_id = $2 ORDER BY created_at LIMIT $3 OFFSET $4"#,
                document_uuid,
                parent,
                limit_i64,
                offset_i64
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                CommentRow,
                r#"SELECT * FROM comments WHERE document_id = $1 AND parent_id IS NULL ORDER BY created_at LIMIT $2 OFFSET $3"#,
                document_uuid,
                limit_i64,
                offset_i64
            )
            .fetch_all(&self.pool)
            .await?
        };

        let total: i64 = if let Some(parent) = parent_uuid {
            sqlx::query_scalar!(
                r#"SELECT COUNT(*) FROM comments WHERE document_id = $1 AND parent_id = $2"#,
                document_uuid,
                parent
            )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0)
        } else {
            sqlx::query_scalar!(
                r#"SELECT COUNT(*) FROM comments WHERE document_id = $1 AND parent_id IS NULL"#,
                document_uuid
            )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0)
        };

        Ok((comments, total))
    }

    pub async fn create_comment(
        &self,
        document_id: &str,
        author_id: &str,
        _author_name: &str,
        content: &str,
        parent_id: Option<&str>,
    ) -> Result<CommentRow, sqlx::Error> {
        let document_uuid = Uuid::parse_str(document_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let author_uuid = Uuid::parse_str(author_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let parent_uuid = parent_id
            .map(|s| Uuid::parse_str(s).map_err(|e| sqlx::Error::Decode(e.to_string().into())))
            .transpose()?;

        let comment = sqlx::query_as!(
            CommentRow,
            r#"
            INSERT INTO comments (id, document_id, parent_id, author_id, content)
            VALUES (gen_random_uuid(), $1, $2, $3, $4)
            RETURNING *
            "#,
            document_uuid,
            parent_uuid,
            author_uuid,
            content
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    pub async fn update_comment(&self, comment_id: &str, content: &str) -> Result<CommentRow, sqlx::Error> {
        let comment_uuid = Uuid::parse_str(comment_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let comment = sqlx::query_as!(
            CommentRow,
            r#"
            UPDATE comments
            SET content = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            comment_uuid,
            content
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    pub async fn resolve_comment(&self, comment_id: &str, resolved_by: &str) -> Result<CommentRow, sqlx::Error> {
        let comment_uuid = Uuid::parse_str(comment_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let resolver_uuid = Uuid::parse_str(resolved_by).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let comment = sqlx::query_as!(
            CommentRow,
            r#"
            UPDATE comments
            SET is_resolved = true, resolved_by = $2, resolved_at = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            comment_uuid,
            resolver_uuid
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    pub async fn unresolve_comment(&self, comment_id: &str) -> Result<CommentRow, sqlx::Error> {
        let comment_uuid = Uuid::parse_str(comment_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let comment = sqlx::query_as!(
            CommentRow,
            r#"
            UPDATE comments
            SET is_resolved = false, resolved_by = NULL, resolved_at = NULL, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            comment_uuid
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    pub async fn delete_comment(&self, comment_id: &str) -> Result<bool, sqlx::Error> {
        let comment_uuid = Uuid::parse_str(comment_id).map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

        let mut tx = self.pool.begin().await?;

        // Recursively delete all descendant comments using CTE
        sqlx::query!(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT id FROM comments WHERE parent_id = $1
                UNION ALL
                SELECT c.id FROM comments c
                INNER JOIN descendants d ON c.parent_id = d.id
            )
            DELETE FROM comments WHERE id IN (SELECT id FROM descendants)
            "#,
            comment_uuid
        )
        .execute(&mut *tx)
        .await?;

        // Then delete the comment itself
        let result = sqlx::query!(r#"DELETE FROM comments WHERE id = $1"#, comment_uuid)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(result.rows_affected() > 0)
    }
}
