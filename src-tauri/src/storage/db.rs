use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Failed to get app data directory")]
    NoAppDataDir,
    #[error("Failed to create database directory: {0}")]
    CreateDir(std::io::Error),
}

/// Database manager for Timewarden
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection
    pub fn new(db_path: PathBuf) -> Result<Self, DbError> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(DbError::CreateDir)?;
        }

        let conn = Connection::open(&db_path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            r#"
            -- Sessions table
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                app_id TEXT NOT NULL,
                app_name TEXT,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                duration_seconds INTEGER,
                is_idle BOOLEAN DEFAULT FALSE,
                is_pending BOOLEAN DEFAULT TRUE
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_time ON sessions(start_time, end_time);
            CREATE INDEX IF NOT EXISTS idx_sessions_app ON sessions(app_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_pending ON sessions(is_pending) WHERE is_pending = TRUE;

            -- Schedules table
            CREATE TABLE IF NOT EXISTS schedules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                days TEXT NOT NULL,
                expected_apps TEXT NOT NULL,
                check_interval_secs INTEGER DEFAULT 300,
                grace_period_secs INTEGER DEFAULT 60,
                enabled BOOLEAN DEFAULT TRUE
            );

            -- Compliance logs table
            CREATE TABLE IF NOT EXISTS compliance_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                schedule_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                is_compliant BOOLEAN NOT NULL,
                current_app TEXT,
                FOREIGN KEY (schedule_id) REFERENCES schedules(id)
            );
            "#,
        )?;
        Ok(())
    }

    /// Get a reference to the connection for queries
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_database_creation() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("timewarden_test.db");
        
        // Clean up if exists
        let _ = std::fs::remove_file(&db_path);
        
        let db = Database::new(db_path.clone()).expect("Failed to create database");
        
        // Verify tables exist
        let tables: Vec<String> = db
            .connection()
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"sessions".to_string()));
        assert!(tables.contains(&"schedules".to_string()));
        assert!(tables.contains(&"compliance_logs".to_string()));
        
        // Clean up
        let _ = std::fs::remove_file(&db_path);
    }
}
