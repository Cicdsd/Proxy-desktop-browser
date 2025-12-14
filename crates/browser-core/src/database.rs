use sqlx::{sqlite::SqlitePool, migrate::MigrateDatabase, Sqlite};
use anyhow::Result;
use tracing::{info, error};

/// Database connection manager using sqlx with async SQLite
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(db_path: &str) -> Result<Self> {
        info!("Initializing database at: {}", db_path);
        
        // Create database if it doesn't exist
        if !Sqlite::database_exists(db_path).await? {
            info!("Creating new database");
            Sqlite::create_database(db_path).await?;
        }
        
        // Create connection pool with optimized settings
        let pool = SqlitePool::connect(db_path).await?;
        
        // Run migrations automatically
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| {
                error!("Failed to run migrations: {}", e);
                e
            })?;
        
        info!("Database initialized successfully");
        
        Ok(Self { pool })
    }
    
    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
    
    /// Execute a raw SQL query
    pub async fn execute_query(&self, query: &str) -> Result<()> {
        sqlx::query(query)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    /// Begin a transaction
    pub async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Sqlite>> {
        Ok(self.pool.begin().await?)
    }
    
    /// Check database health
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
    
    /// Close the database connection pool
    pub async fn close(self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }
}

/// Database error type
#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(#[from] sqlx::Error),
    
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Query failed: {0}")]
    QueryFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_database_creation() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap();
        
        let db = Database::new(db_path_str).await?;
        assert!(db.health_check().await.is_ok());
        
        db.close().await?;
        Ok(())
    }
    
    #[tokio::test]
    async fn test_transaction() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_tx.db");
        let db_path_str = db_path.to_str().unwrap();
        
        let db = Database::new(db_path_str).await?;
        
        // Test transaction using begin_transaction
        let mut tx = db.begin_transaction().await?;
        sqlx::query("INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)")
            .bind("test_key")
            .bind("test_value")
            .bind(chrono::Utc::now().to_rfc3339())
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        
        // Verify the value was inserted
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM settings WHERE key = ?")
            .bind("test_key")
            .fetch_one(db.pool())
            .await?;
        assert_eq!(count, 1);
        
        db.close().await?;
        Ok(())
    }
}
