use crate::models::{ComplianceLog, Schedule};
use crate::storage::db::Database;
use chrono::{NaiveTime, Utc, Weekday};
use rusqlite::params;

impl Database {
    /// Insert a new schedule
    pub fn insert_schedule(&self, schedule: &Schedule) -> Result<i64, rusqlite::Error> {
        let days_str = schedule
            .days
            .iter()
            .map(|d| d.num_days_from_monday().to_string())
            .collect::<Vec<_>>()
            .join(",");
        let apps_str = schedule.expected_apps.join(",");

        self.connection().execute(
            r#"
            INSERT INTO schedules (name, start_time, end_time, days, expected_apps, check_interval_secs, grace_period_secs, enabled)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                schedule.name,
                schedule.start_time.format("%H:%M").to_string(),
                schedule.end_time.format("%H:%M").to_string(),
                days_str,
                apps_str,
                schedule.check_interval_secs,
                schedule.grace_period_secs,
                schedule.enabled
            ],
        )?;

        Ok(self.connection().last_insert_rowid())
    }

    /// Update an existing schedule
    pub fn update_schedule(&self, schedule: &Schedule) -> Result<(), rusqlite::Error> {
        let days_str = schedule
            .days
            .iter()
            .map(|d| d.num_days_from_monday().to_string())
            .collect::<Vec<_>>()
            .join(",");
        let apps_str = schedule.expected_apps.join(",");

        self.connection().execute(
            r#"
            UPDATE schedules 
            SET name = ?1, start_time = ?2, end_time = ?3, days = ?4, expected_apps = ?5, 
                check_interval_secs = ?6, grace_period_secs = ?7, enabled = ?8
            WHERE id = ?9
            "#,
            params![
                schedule.name,
                schedule.start_time.format("%H:%M").to_string(),
                schedule.end_time.format("%H:%M").to_string(),
                days_str,
                apps_str,
                schedule.check_interval_secs,
                schedule.grace_period_secs,
                schedule.enabled,
                schedule.id
            ],
        )?;

        Ok(())
    }

    /// Delete a schedule by ID
    pub fn delete_schedule(&self, id: i64) -> Result<(), rusqlite::Error> {
        self.connection()
            .execute("DELETE FROM schedules WHERE id = ?1", params![id])?;
        // Also delete related compliance logs
        self.connection()
            .execute("DELETE FROM compliance_logs WHERE schedule_id = ?1", params![id])?;
        Ok(())
    }

    /// Toggle schedule enabled state
    pub fn toggle_schedule(&self, id: i64, enabled: bool) -> Result<(), rusqlite::Error> {
        self.connection().execute(
            "UPDATE schedules SET enabled = ?1 WHERE id = ?2",
            params![enabled, id],
        )?;
        Ok(())
    }

    /// Get all schedules
    pub fn get_all_schedules(&self) -> Result<Vec<Schedule>, rusqlite::Error> {
        let mut stmt = self
            .connection()
            .prepare("SELECT id, name, start_time, end_time, days, expected_apps, check_interval_secs, grace_period_secs, enabled FROM schedules")?;

        let schedules = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let start_time_str: String = row.get(2)?;
                let end_time_str: String = row.get(3)?;
                let days_str: String = row.get(4)?;
                let apps_str: String = row.get(5)?;
                let check_interval_secs: u32 = row.get(6)?;
                let grace_period_secs: u32 = row.get(7)?;
                let enabled: bool = row.get(8)?;

                let start_time = NaiveTime::parse_from_str(&start_time_str, "%H:%M")
                    .unwrap_or_else(|_| NaiveTime::from_hms_opt(9, 0, 0).unwrap());
                let end_time = NaiveTime::parse_from_str(&end_time_str, "%H:%M")
                    .unwrap_or_else(|_| NaiveTime::from_hms_opt(17, 0, 0).unwrap());

                let days: Vec<Weekday> = days_str
                    .split(',')
                    .filter_map(|s| s.parse::<u32>().ok())
                    .filter_map(|n| match n {
                        0 => Some(Weekday::Mon),
                        1 => Some(Weekday::Tue),
                        2 => Some(Weekday::Wed),
                        3 => Some(Weekday::Thu),
                        4 => Some(Weekday::Fri),
                        5 => Some(Weekday::Sat),
                        6 => Some(Weekday::Sun),
                        _ => None,
                    })
                    .collect();

                let expected_apps: Vec<String> = apps_str
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();

                Ok(Schedule {
                    id: Some(id),
                    name,
                    start_time,
                    end_time,
                    days,
                    expected_apps,
                    check_interval_secs,
                    grace_period_secs,
                    enabled,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(schedules)
    }

    /// Get enabled schedules only
    pub fn get_enabled_schedules(&self) -> Result<Vec<Schedule>, rusqlite::Error> {
        let all = self.get_all_schedules()?;
        Ok(all.into_iter().filter(|s| s.enabled).collect())
    }

    /// Insert a compliance log entry
    pub fn insert_compliance_log(
        &self,
        schedule_id: i64,
        is_compliant: bool,
        current_app: Option<&str>,
    ) -> Result<i64, rusqlite::Error> {
        let timestamp = Utc::now().timestamp();

        self.connection().execute(
            r#"
            INSERT INTO compliance_logs (schedule_id, timestamp, is_compliant, current_app)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![schedule_id, timestamp, is_compliant, current_app],
        )?;

        Ok(self.connection().last_insert_rowid())
    }

    /// Get compliance logs for a schedule
    pub fn get_compliance_logs(&self, schedule_id: i64) -> Result<Vec<ComplianceLog>, rusqlite::Error> {
        let mut stmt = self.connection().prepare(
            "SELECT id, schedule_id, timestamp, is_compliant, current_app FROM compliance_logs WHERE schedule_id = ?1 ORDER BY timestamp DESC LIMIT 100",
        )?;

        let logs = stmt
            .query_map(params![schedule_id], |row| {
                let id: i64 = row.get(0)?;
                let schedule_id: i64 = row.get(1)?;
                let timestamp: i64 = row.get(2)?;
                let is_compliant: bool = row.get(3)?;
                let current_app: Option<String> = row.get(4)?;

                Ok(ComplianceLog {
                    id: Some(id),
                    schedule_id,
                    timestamp: chrono::DateTime::from_timestamp(timestamp, 0)
                        .unwrap_or_else(Utc::now),
                    is_compliant,
                    current_app,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(logs)
    }
}
