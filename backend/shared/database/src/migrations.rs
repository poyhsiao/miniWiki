use sqlx::migrate::Migrator;
use sqlx::PgPool;
use std::path::Path;

/// Runs database migrations from the migrations directory.
///
/// # Arguments
///
/// * `pool` - The PostgreSQL connection pool
/// * `migrations_path` - Path to the directory containing SQL migration files
///
/// # Returns
///
/// Returns `Ok(())` if all migrations succeed, or an error if any migration fails.
pub async fn run_migrations(
    pool: &PgPool,
    migrations_path: &str,
) -> Result<(), sqlx::Error> {
    let migrator = Migrator::new(Path::new(migrations_path), pool.clone()).await?;
    migrator.run(pool).await?;
    Ok(())
}

/// Creates a new migration file with the given description.
///
/// # Arguments
///
/// * `description` - Description of the migration (snake_case recommended)
/// * `migrations_path` - Path to the migrations directory
///
/// # Returns
///
/// Returns the path to the created migration file.
pub fn create_migration_file(
    description: &str,
    migrations_path: &str,
) -> std::io::Result<std::path::PathBuf> {
    use chrono::Datelike;
    use chrono::Timelike;
    
    let now = chrono::Utc::now();
    let timestamp = format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );
    
    let safe_description = description
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
    
    let filename = format!("{}_{}.sql", timestamp, safe_description);
    let path = std::path::PathBuf::from(migrations_path).join(&filename);
    
    let content = format!(
        r#"-- Migration: {description}
-- Created: {timestamp}

-- UP migration
-- TODO: Add your migration SQL here

-- DOWN migration
-- TODO: Add rollback SQL here
"#,
        description = description,
        timestamp = timestamp
    );
    
    std::fs::write(&path, content)?;
    
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx:: PgPool;
    use sqlx::postgres::PgPoolOptions;
    
    #[tokio::test]
    async fn test_migration_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let migrations_path = temp_dir.path().to_str().unwrap();
        
        let path = create_migration_file("test_migration", migrations_path).unwrap();
        assert!(path.exists());
        
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("-- Migration: test_migration"));
        assert!(content.contains("-- UP migration"));
        assert!(content.contains("-- DOWN migration"));
    }
}
