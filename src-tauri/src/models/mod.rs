use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveTime, Utc, Weekday};

#[derive(Debug, Clone, Serialize)]
pub struct AppInfo {
    pub process_name: String,
    pub app_title: Option<String>,
    pub bundle_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: Option<i64>,
    pub app_id: String,
    pub app_name: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i64>,
    pub is_idle: bool,
}

/// A schedule defines when certain apps should be used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: Option<i64>,
    pub name: String,
    pub start_time: NaiveTime,          // e.g., 09:00
    pub end_time: NaiveTime,            // e.g., 17:00
    pub days: Vec<Weekday>,             // Mon-Sun
    pub expected_apps: Vec<String>,     // List of allowed app names
    pub check_interval_secs: u32,       // Default: 300 (5 min)
    pub grace_period_secs: u32,         // Default: 60 (1 min)
    pub enabled: bool,
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            days: vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri],
            expected_apps: Vec::new(),
            check_interval_secs: 300,
            grace_period_secs: 60,
            enabled: true,
        }
    }
}

/// A log entry for compliance checks
#[derive(Debug, Clone, Serialize)]
pub struct ComplianceLog {
    pub id: Option<i64>,
    pub schedule_id: i64,
    pub timestamp: DateTime<Utc>,
    pub is_compliant: bool,
    pub current_app: Option<String>,
}
