use crate::models::Schedule;
use chrono::{Datelike, Local};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Tracks the state of each schedule for rate limiting and grace periods
#[derive(Debug, Clone)]
pub struct ScheduleState {
    pub last_check: Option<Instant>,
    pub last_notification: Option<Instant>,
    pub grace_started: Option<Instant>,
    pub consecutive_non_compliant: u32,
}

impl Default for ScheduleState {
    fn default() -> Self {
        Self {
            last_check: None,
            last_notification: None,
            grace_started: None,
            consecutive_non_compliant: 0,
        }
    }
}

/// Scheduler engine for evaluating compliance
pub struct SchedulerEngine {
    /// State for each schedule (keyed by schedule ID)
    states: Arc<Mutex<HashMap<i64, ScheduleState>>>,
}

impl SchedulerEngine {
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if the current time falls within the schedule's time window
    pub fn is_within_schedule(&self, schedule: &Schedule) -> bool {
        let now = Local::now();
        let current_time = now.time();
        let current_day = now.weekday();

        // Check if today is in the schedule's days
        if !schedule.days.contains(&current_day) {
            return false;
        }

        // Check if current time is within the time window
        if schedule.start_time <= schedule.end_time {
            // Normal case: e.g., 09:00 - 17:00
            current_time >= schedule.start_time && current_time <= schedule.end_time
        } else {
            // Overnight case: e.g., 22:00 - 06:00
            current_time >= schedule.start_time || current_time <= schedule.end_time
        }
    }

    /// Check if the current app is compliant with the schedule
    pub fn is_compliant(&self, schedule: &Schedule, current_app: &str) -> bool {
        // If no expected apps are specified, any app is compliant
        if schedule.expected_apps.is_empty() {
            return true;
        }

        // Check if current app matches any expected app (case-insensitive)
        let current_lower = current_app.to_lowercase();
        schedule
            .expected_apps
            .iter()
            .any(|app| current_lower.contains(&app.to_lowercase()))
    }

    /// Determine if enough time has passed since the last check
    pub fn should_check(&self, schedule_id: i64, check_interval_secs: u32) -> bool {
        let states = self.states.lock().unwrap();
        if let Some(state) = states.get(&schedule_id) {
            if let Some(last_check) = state.last_check {
                return last_check.elapsed().as_secs() >= check_interval_secs as u64;
            }
        }
        true // No previous check, should check
    }

    /// Update the last check time for a schedule
    pub fn mark_checked(&self, schedule_id: i64) {
        let mut states = self.states.lock().unwrap();
        let state = states.entry(schedule_id).or_default();
        state.last_check = Some(Instant::now());
    }

    /// Check if we should send a notification (respecting grace period and rate limiting)
    pub fn should_notify(&self, schedule_id: i64, grace_period_secs: u32) -> bool {
        let mut states = self.states.lock().unwrap();
        let state = states.entry(schedule_id).or_default();

        // Check grace period
        if let Some(grace_started) = state.grace_started {
            if grace_started.elapsed().as_secs() < grace_period_secs as u64 {
                return false; // Still in grace period
            }
        }

        // Check rate limiting (don't notify more than once per check interval)
        if let Some(last_notification) = state.last_notification {
            if last_notification.elapsed().as_secs() < 300 {
                // 5 minute rate limit
                return false;
            }
        }

        true
    }

    /// Start grace period for a schedule
    pub fn start_grace(&self, schedule_id: i64) {
        let mut states = self.states.lock().unwrap();
        let state = states.entry(schedule_id).or_default();
        if state.grace_started.is_none() {
            state.grace_started = Some(Instant::now());
        }
    }

    /// Reset grace period (user became compliant)
    pub fn reset_grace(&self, schedule_id: i64) {
        let mut states = self.states.lock().unwrap();
        if let Some(state) = states.get_mut(&schedule_id) {
            state.grace_started = None;
            state.consecutive_non_compliant = 0;
        }
    }

    /// Mark that a notification was sent
    pub fn mark_notified(&self, schedule_id: i64) {
        let mut states = self.states.lock().unwrap();
        let state = states.entry(schedule_id).or_default();
        state.last_notification = Some(Instant::now());
        state.consecutive_non_compliant += 1;
    }

    /// Evaluate a schedule and return if notification should be triggered
    /// Returns: (should_notify, is_compliant)
    pub fn evaluate(
        &self,
        schedule: &Schedule,
        current_app: &str,
    ) -> (bool, bool) {
        let schedule_id = schedule.id.unwrap_or(0);

        // Check if we should even evaluate this schedule now
        if !schedule.enabled {
            return (false, true);
        }

        if !self.is_within_schedule(schedule) {
            return (false, true);
        }

        if !self.should_check(schedule_id, schedule.check_interval_secs) {
            return (false, true); // Not time to check yet
        }

        self.mark_checked(schedule_id);

        let is_compliant = self.is_compliant(schedule, current_app);

        if is_compliant {
            self.reset_grace(schedule_id);
            return (false, true);
        }

        // Non-compliant: start/continue grace period
        self.start_grace(schedule_id);

        let should_notify = self.should_notify(schedule_id, schedule.grace_period_secs);
        if should_notify {
            self.mark_notified(schedule_id);
        }

        (should_notify, false)
    }
}

impl Default for SchedulerEngine {
    fn default() -> Self {
        Self::new()
    }
}
