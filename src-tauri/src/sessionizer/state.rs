use chrono::{DateTime, Utc};
use crate::models::{AppInfo, Session};

/// Configuration for the sessionizer
pub struct SessionizerConfig {
    /// Idle threshold in seconds (default: 300 = 5 minutes)
    pub idle_threshold_seconds: u64,
}

impl Default for SessionizerConfig {
    fn default() -> Self {
        Self {
            idle_threshold_seconds: 300,
        }
    }
}

/// Current state of the sessionizer
#[derive(Debug, Clone)]
pub enum SessionState {
    /// No active session
    Inactive,
    /// Actively tracking an app
    Active {
        app_id: String,
        app_name: Option<String>,
        start_time: DateTime<Utc>,
    },
    /// User is idle
    Idle {
        start_time: DateTime<Utc>,
    },
}

/// The Sessionizer manages session state and handles transitions
pub struct Sessionizer {
    config: SessionizerConfig,
    state: SessionState,
    /// Completed sessions waiting to be persisted
    pending_sessions: Vec<Session>,
}

impl Sessionizer {
    pub fn new(config: SessionizerConfig) -> Self {
        Self {
            config,
            state: SessionState::Inactive,
            pending_sessions: Vec::new(),
        }
    }

    /// Process a new foreground app reading
    /// Returns true if a session was completed
    pub fn update(&mut self, app: Option<AppInfo>, idle_seconds: u64) -> bool {
        let now = Utc::now();
        let is_idle = idle_seconds >= self.config.idle_threshold_seconds;

        match (&self.state, &app, is_idle) {
            // Currently inactive, app detected, not idle -> start new session
            (SessionState::Inactive, Some(info), false) => {
                self.state = SessionState::Active {
                    app_id: info.process_name.clone(),
                    app_name: info.app_title.clone(),
                    start_time: now,
                };
                false
            }

            // Currently inactive, idle -> start idle session
            (SessionState::Inactive, _, true) => {
                self.state = SessionState::Idle { start_time: now };
                false
            }

            // Active session, same app, not idle -> continue
            (SessionState::Active { app_id, .. }, Some(info), false) if app_id == &info.process_name => {
                false
            }

            // Active session, different app or no app, not idle -> end session, start new
            (SessionState::Active { app_id, app_name, start_time }, new_app, false) => {
                // End current session
                let session = Session {
                    id: None,
                    app_id: app_id.clone(),
                    app_name: app_name.clone(),
                    start_time: *start_time,
                    end_time: Some(now),
                    duration_seconds: Some((now - *start_time).num_seconds()),
                    is_idle: false,
                };
                self.pending_sessions.push(session);

                // Start new session if app available
                if let Some(info) = new_app {
                    self.state = SessionState::Active {
                        app_id: info.process_name.clone(),
                        app_name: info.app_title.clone(),
                        start_time: now,
                    };
                } else {
                    self.state = SessionState::Inactive;
                }
                true
            }

            // Active session, now idle -> end session, start idle
            (SessionState::Active { app_id, app_name, start_time }, _, true) => {
                let session = Session {
                    id: None,
                    app_id: app_id.clone(),
                    app_name: app_name.clone(),
                    start_time: *start_time,
                    end_time: Some(now),
                    duration_seconds: Some((now - *start_time).num_seconds()),
                    is_idle: false,
                };
                self.pending_sessions.push(session);
                self.state = SessionState::Idle { start_time: now };
                true
            }

            // Idle, still idle -> continue
            (SessionState::Idle { .. }, _, true) => false,

            // Idle, no longer idle, app detected -> end idle, start new session
            (SessionState::Idle { start_time }, Some(info), false) => {
                let session = Session {
                    id: None,
                    app_id: "Idle".to_string(),
                    app_name: Some("Idle".to_string()),
                    start_time: *start_time,
                    end_time: Some(now),
                    duration_seconds: Some((now - *start_time).num_seconds()),
                    is_idle: true,
                };
                self.pending_sessions.push(session);
                self.state = SessionState::Active {
                    app_id: info.process_name.clone(),
                    app_name: info.app_title.clone(),
                    start_time: now,
                };
                true
            }

            // Idle, no longer idle, no app -> end idle, become inactive
            (SessionState::Idle { start_time }, None, false) => {
                let session = Session {
                    id: None,
                    app_id: "Idle".to_string(),
                    app_name: Some("Idle".to_string()),
                    start_time: *start_time,
                    end_time: Some(now),
                    duration_seconds: Some((now - *start_time).num_seconds()),
                    is_idle: true,
                };
                self.pending_sessions.push(session);
                self.state = SessionState::Inactive;
                true
            }

            // No app, not idle, inactive -> stay inactive
            (SessionState::Inactive, None, false) => false,
        }
    }

    /// Take and clear pending sessions
    pub fn take_pending_sessions(&mut self) -> Vec<Session> {
        std::mem::take(&mut self.pending_sessions)
    }

    /// Get current state for debugging
    pub fn current_state(&self) -> &SessionState {
        &self.state
    }
}
