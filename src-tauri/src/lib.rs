pub mod collectors;
pub mod models;
pub mod scheduler;
pub mod sessionizer;
pub mod storage;

use tauri::Manager;
use tauri_plugin_notification::NotificationExt;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use collectors::{create_collector, ForegroundCollector};
use models::Schedule;
use scheduler::SchedulerEngine;
use sessionizer::{Sessionizer, SessionizerConfig};
use storage::Database;

/// Shared application state
pub struct AppState {
    pub sessionizer: Arc<Mutex<Sessionizer>>,
    pub collector: Arc<dyn ForegroundCollector>,
    pub database: Arc<Mutex<Database>>,
    pub scheduler_engine: Arc<SchedulerEngine>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_current_app(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    let app = state.collector.get_foreground_app();
    Ok(app.map(|a| a.process_name))
}

#[tauri::command]
async fn get_idle_seconds(state: tauri::State<'_, AppState>) -> Result<u64, String> {
    Ok(state.collector.get_idle_seconds())
}

#[tauri::command]
async fn get_today_sessions(state: tauri::State<'_, AppState>) -> Result<Vec<models::Session>, String> {
    let db = state.database.lock().await;
    db.get_today_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_app_totals_today(state: tauri::State<'_, AppState>) -> Result<Vec<(String, i64)>, String> {
    use chrono::{Utc, TimeZone};
    
    let db = state.database.lock().await;
    let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
    let today_end = Utc::now().date_naive().and_hms_opt(23, 59, 59).unwrap();
    
    db.get_app_totals(
        Utc.from_utc_datetime(&today_start),
        Utc.from_utc_datetime(&today_end),
    ).map_err(|e| e.to_string())
}

// ===== Schedule CRUD Commands =====

#[tauri::command]
async fn get_all_schedules(state: tauri::State<'_, AppState>) -> Result<Vec<Schedule>, String> {
    let db = state.database.lock().await;
    db.get_all_schedules().map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_schedule(state: tauri::State<'_, AppState>, schedule: Schedule) -> Result<i64, String> {
    let db = state.database.lock().await;
    db.insert_schedule(&schedule).map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_schedule(state: tauri::State<'_, AppState>, schedule: Schedule) -> Result<(), String> {
    let db = state.database.lock().await;
    db.update_schedule(&schedule).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_schedule(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.database.lock().await;
    db.delete_schedule(id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_schedule(state: tauri::State<'_, AppState>, id: i64, enabled: bool) -> Result<(), String> {
    let db = state.database.lock().await;
    db.toggle_schedule(id, enabled).map_err(|e| e.to_string())
}

/// Start the background polling loop with scheduler integration
fn start_polling_loop(app_state: Arc<AppState>, app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                
                let app = app_state.collector.get_foreground_app();
                let idle = app_state.collector.get_idle_seconds();
                
                // Session tracking
                let mut sessionizer = app_state.sessionizer.lock().await;
                let session_completed = sessionizer.update(app.clone(), idle);
                
                if session_completed {
                    let sessions = sessionizer.take_pending_sessions();
                    let db = app_state.database.lock().await;
                    
                    for session in sessions {
                        match db.insert_session(&session) {
                            Ok(id) => {
                                println!(
                                    "[DB] Saved session {} | {} | {} | {}s",
                                    id,
                                    session.app_id,
                                    if session.is_idle { "IDLE" } else { "ACTIVE" },
                                    session.duration_seconds.unwrap_or(0)
                                );
                            }
                            Err(e) => {
                                eprintln!("[DB Error] Failed to save session: {}", e);
                            }
                        }
                    }
                }
                drop(sessionizer); // Release lock before scheduler check
                
                // Schedule compliance checking (every 5 seconds to reduce overhead)
                if idle % 5 == 0 {
                    if let Some(ref current_app) = app {
                        let db = app_state.database.lock().await;
                        if let Ok(schedules) = db.get_enabled_schedules() {
                            drop(db); // Release lock before evaluation
                            
                            for schedule in schedules {
                                let (should_notify, is_compliant) = 
                                    app_state.scheduler_engine.evaluate(&schedule, &current_app.process_name);
                                
                                // Log compliance
                                if !is_compliant {
                                    let db = app_state.database.lock().await;
                                    let _ = db.insert_compliance_log(
                                        schedule.id.unwrap_or(0),
                                        is_compliant,
                                        Some(&current_app.process_name),
                                    );
                                }
                                
                                // Send notification if needed
                                if should_notify {
                                    let _ = app_handle
                                        .notification()
                                        .builder()
                                        .title("Timewarden - Schedule Alert")
                                        .body(format!(
                                            "You're using {} during '{}'. Expected: {}",
                                            current_app.process_name,
                                            schedule.name,
                                            schedule.expected_apps.join(", ")
                                        ))
                                        .show();
                                    
                                    println!(
                                        "[Schedule] Non-compliant: {} (expected {:?})",
                                        current_app.process_name, schedule.expected_apps
                                    );
                                }
                            }
                        }
                    }
                }
                
                // Debug: Print current app every 5 seconds
                if idle % 5 == 0 {
                    if let Some(ref info) = app {
                        println!("[Tracking] {} | Idle: {}s", info.process_name, idle);
                    }
                }
            }
        });
    });
}

/// Get the database path
fn get_db_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let app_data = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    app_data.join("timewarden.db")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let db_path = get_db_path(app.handle());
            let database = Database::new(db_path).expect("Failed to initialize database");
            
            let collector = create_collector();
            let sessionizer = Arc::new(Mutex::new(Sessionizer::new(SessionizerConfig::default())));
            let database = Arc::new(Mutex::new(database));
            let scheduler_engine = Arc::new(SchedulerEngine::new());
            
            let app_state = Arc::new(AppState {
                sessionizer: sessionizer.clone(),
                collector: collector.clone(),
                database: database.clone(),
                scheduler_engine: scheduler_engine.clone(),
            });

            // Start background polling with app handle for notifications
            start_polling_loop(app_state, app.handle().clone());

            // Manage state for commands
            app.manage(AppState {
                sessionizer,
                collector,
                database,
                scheduler_engine,
            });

            // System Tray
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::TrayIconBuilder;

            let show_i = MenuItem::with_id(app, "show", "Show Timewarden", true, None::<&str>).unwrap();
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
            let menu = Menu::with_items(app, &[&show_i, &quit_i]).unwrap();

            let _tray = TrayIconBuilder::with_id("tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } => {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_current_app,
            get_idle_seconds,
            get_today_sessions,
            get_app_totals_today,
            get_all_schedules,
            create_schedule,
            update_schedule,
            delete_schedule,
            toggle_schedule
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
