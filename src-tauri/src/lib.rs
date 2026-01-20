pub mod collectors;
pub mod models;
pub mod scheduler;
pub mod sessionizer;
pub mod storage;

use tauri::Manager;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use collectors::{create_collector, ForegroundCollector};
use sessionizer::{Sessionizer, SessionizerConfig};
use storage::Database;

/// Shared application state
pub struct AppState {
    pub sessionizer: Arc<Mutex<Sessionizer>>,
    pub collector: Arc<dyn ForegroundCollector>,
    pub database: Arc<Mutex<Database>>,
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

/// Start the background polling loop
fn start_polling_loop(app_state: Arc<AppState>) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                
                let app = app_state.collector.get_foreground_app();
                let idle = app_state.collector.get_idle_seconds();
                
                let mut sessionizer = app_state.sessionizer.lock().await;
                let session_completed = sessionizer.update(app.clone(), idle);
                
                if session_completed {
                    let sessions = sessionizer.take_pending_sessions();
                    let db = app_state.database.lock().await;
                    
                    for session in sessions {
                        // Persist to database
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
        .setup(|app| {
            let db_path = get_db_path(app.handle());
            let database = Database::new(db_path).expect("Failed to initialize database");
            
            let collector = create_collector();
            let sessionizer = Arc::new(Mutex::new(Sessionizer::new(SessionizerConfig::default())));
            let database = Arc::new(Mutex::new(database));
            
            let app_state = Arc::new(AppState {
                sessionizer: sessionizer.clone(),
                collector: collector.clone(),
                database: database.clone(),
            });

            // Start background polling
            start_polling_loop(app_state);

            // Manage state for commands
            app.manage(AppState {
                sessionizer,
                collector,
                database,
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_current_app,
            get_idle_seconds,
            get_today_sessions,
            get_app_totals_today
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
