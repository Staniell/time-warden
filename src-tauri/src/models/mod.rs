use serde::Serialize;
use chrono::{DateTime, Utc};

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
