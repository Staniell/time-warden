use rusqlite::{params, OptionalExtension};
use crate::models::Session;
use crate::storage::db::Database;
use chrono::{DateTime, Utc, TimeZone};

/// Session storage operations
impl Database {
    /// Insert a new session into the database
    pub fn insert_session(&self, session: &Session) -> Result<i64, rusqlite::Error> {
        let start_ts = session.start_time.timestamp();
        let end_ts = session.end_time.map(|t| t.timestamp());
        
        self.connection().execute(
            "INSERT INTO sessions (app_id, app_name, start_time, end_time, duration_seconds, is_idle, is_pending)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                session.app_id,
                session.app_name,
                start_ts,
                end_ts,
                session.duration_seconds,
                session.is_idle,
                false // Mark as not pending since it's complete
            ],
        )?;
        
        Ok(self.connection().last_insert_rowid())
    }

    /// Get sessions within a time range
    pub fn get_sessions_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Session>, rusqlite::Error> {
        let start_ts = start.timestamp();
        let end_ts = end.timestamp();
        
        let mut stmt = self.connection().prepare(
            "SELECT id, app_id, app_name, start_time, end_time, duration_seconds, is_idle
             FROM sessions
             WHERE start_time >= ?1 AND start_time <= ?2
             ORDER BY start_time ASC"
        )?;
        
        let sessions = stmt.query_map(params![start_ts, end_ts], |row| {
            let start_time: i64 = row.get(3)?;
            let end_time: Option<i64> = row.get(4)?;
            
            Ok(Session {
                id: Some(row.get(0)?),
                app_id: row.get(1)?,
                app_name: row.get(2)?,
                start_time: Utc.timestamp_opt(start_time, 0).single().unwrap_or_else(Utc::now),
                end_time: end_time.and_then(|ts| Utc.timestamp_opt(ts, 0).single()),
                duration_seconds: row.get(5)?,
                is_idle: row.get(6)?,
            })
        })?;
        
        sessions.collect()
    }

    /// Get today's sessions
    pub fn get_today_sessions(&self) -> Result<Vec<Session>, rusqlite::Error> {
        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_end = Utc::now().date_naive().and_hms_opt(23, 59, 59).unwrap();
        
        self.get_sessions_in_range(
            Utc.from_utc_datetime(&today_start),
            Utc.from_utc_datetime(&today_end),
        )
    }

    /// Get total time per app for a date range
    pub fn get_app_totals(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<(String, i64)>, rusqlite::Error> {
        let start_ts = start.timestamp();
        let end_ts = end.timestamp();
        
        let mut stmt = self.connection().prepare(
            "SELECT app_id, SUM(duration_seconds) as total
             FROM sessions
             WHERE start_time >= ?1 AND start_time <= ?2 AND is_idle = FALSE
             GROUP BY app_id
             ORDER BY total DESC"
        )?;
        
        let totals = stmt.query_map(params![start_ts, end_ts], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        
        totals.collect()
    }

    /// Get the most recent pending session (for crash recovery)
    pub fn get_pending_session(&self) -> Result<Option<Session>, rusqlite::Error> {
        let mut stmt = self.connection().prepare(
            "SELECT id, app_id, app_name, start_time, end_time, duration_seconds, is_idle
             FROM sessions
             WHERE is_pending = TRUE
             ORDER BY start_time DESC
             LIMIT 1"
        )?;
        
        stmt.query_row([], |row| {
            let start_time: i64 = row.get(3)?;
            let end_time: Option<i64> = row.get(4)?;
            
            Ok(Session {
                id: Some(row.get(0)?),
                app_id: row.get(1)?,
                app_name: row.get(2)?,
                start_time: Utc.timestamp_opt(start_time, 0).single().unwrap_or_else(Utc::now),
                end_time: end_time.and_then(|ts| Utc.timestamp_opt(ts, 0).single()),
                duration_seconds: row.get(5)?,
                is_idle: row.get(6)?,
            })
        }).optional()
    }

    /// Close a pending session (used on crash recovery)
    pub fn close_pending_sessions(&self, end_time: DateTime<Utc>) -> Result<usize, rusqlite::Error> {
        let end_ts = end_time.timestamp();
        
        self.connection().execute(
            "UPDATE sessions
             SET end_time = ?1,
                 duration_seconds = ?1 - start_time,
                 is_pending = FALSE
             WHERE is_pending = TRUE",
            params![end_ts],
        )
    }
}
