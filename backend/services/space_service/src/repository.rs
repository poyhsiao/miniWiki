use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Space, SpaceMembership};

pub struct SpaceRepository;

impl SpaceRepository {
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Space>, sqlx::Error> {
        sqlx::query_as!(
            Space,
            r#"
            SELECT id, owner_id, name, icon, description, is_public, created_at, updated_at
            FROM spaces
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Space>, sqlx::Error> {
        sqlx::query_as!(
            Space,
            r#"
            SELECT s.id, s.owner_id, s.name, s.icon, s.description, s.is_public, s.created_at, s.updated_at
            FROM spaces s
            LEFT JOIN space_memberships sm ON s.id = sm.space_id
            WHERE s.owner_id = $1 OR sm.user_id = $1
            GROUP BY s.id
            ORDER BY s.updated_at DESC
            "#,
            user_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        owner_id: Uuid,
        name: &str,
        icon: Option<String>,
        description: Option<String>,
        is_public: bool,
    ) -> Result<Space, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now().naive_utc();

        let space = sqlx::query_as!(
            Space,
            r#"
            INSERT INTO spaces (id, owner_id, name, icon, description, is_public, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, owner_id, name, icon, description, is_public, created_at, updated_at
            "#,
            id,
            owner_id,
            name,
            icon,
            description,
            is_public,
            now,
            now
        )
        .fetch_one(pool)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO space_memberships (id, space_id, user_id, role, joined_at, invited_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            Uuid::new_v4(),
            id,
            owner_id,
            "owner",
            now,
            owner_id
        )
        .execute(pool)
        .await?;

        Ok(space)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        name: Option<String>,
        icon: Option<String>,
        description: Option<String>,
        is_public: Option<bool>,
    ) -> Result<Space, sqlx::Error> {
        let now = chrono::Utc::now().naive_utc();
        let space = sqlx::query_as!(
            Space,
            r#"
            UPDATE spaces
            SET name = COALESCE($2, name),
                icon = COALESCE($3, icon),
                description = COALESCE($4, description),
                is_public = COALESCE($5, is_public),
                updated_at = $6
            WHERE id = $1
            RETURNING id, owner_id, name, icon, description, is_public, created_at, updated_at
            "#,
            id,
            name,
            icon,
            description,
            is_public,
            now
        )
        .fetch_one(pool)
        .await?;

        Ok(space)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM space_memberships WHERE space_id = $1", id)
            .execute(pool)
            .await?;

        sqlx::query!("DELETE FROM spaces WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn check_membership(pool: &PgPool, space_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!" FROM space_memberships
            WHERE space_id = $1 AND user_id = $2
            "#,
            space_id,
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(count > 0)
    }

    pub async fn list_members(pool: &PgPool, space_id: Uuid) -> Result<Vec<SpaceMembership>, sqlx::Error> {
        sqlx::query_as!(
            SpaceMembership,
            r#"
            SELECT id, space_id, user_id, role, joined_at, invited_by
            FROM space_memberships
            WHERE space_id = $1
            ORDER BY joined_at ASC
            "#,
            space_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn add_member(
        pool: &PgPool,
        space_id: Uuid,
        user_id: &str,
        role: &str,
        invited_by: &str,
    ) -> Result<SpaceMembership, sqlx::Error> {
        let id = Uuid::new_v4();
        let user_uuid = Uuid::parse_str(user_id).map_err(|_| sqlx::Error::Decode("Invalid user_id UUID".into()))?;
        let invited_by_uuid = Uuid::parse_str(invited_by).map_err(|_| sqlx::Error::Decode("Invalid invited_by UUID".into()))?;
        let now = chrono::Utc::now().naive_utc();

        let membership = sqlx::query_as!(
            SpaceMembership,
            r#"
            INSERT INTO space_memberships (id, space_id, user_id, role, joined_at, invited_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, space_id, user_id, role, joined_at, invited_by
            "#,
            id,
            space_id,
            user_uuid,
            role,
            now,
            invited_by_uuid
        )
        .fetch_one(pool)
        .await?;

        Ok(membership)
    }

    pub async fn get_membership(pool: &PgPool, id: Uuid) -> Result<Option<SpaceMembership>, sqlx::Error> {
        sqlx::query_as!(
            SpaceMembership,
            r#"
            SELECT id, space_id, user_id, role, joined_at, invited_by
            FROM space_memberships
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn update_member_role(
        pool: &PgPool,
        id: Uuid,
        role: &str,
    ) -> Result<SpaceMembership, sqlx::Error> {
        let _membership = sqlx::query_as!(
            SpaceMembership,
            r#"SELECT id, space_id, user_id, role, joined_at, invited_by FROM space_memberships WHERE id = $1"#,
            id
        )
        .fetch_one(pool)
        .await?;

        sqlx::query!(
            "UPDATE space_memberships SET role = $1 WHERE id = $2",
            role,
            id
        )
        .execute(pool)
        .await?;

        let updated = sqlx::query_as!(
            SpaceMembership,
            r#"SELECT id, space_id, user_id, role, joined_at, invited_by FROM space_memberships WHERE id = $1"#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }

    pub async fn remove_member(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM space_memberships WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
