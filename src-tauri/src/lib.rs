pub mod collectors;
pub mod models;
pub mod scheduler;
pub mod sessionizer;
pub mod storage;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use collectors::{create_collector, ForegroundCollector};
use sessionizer::{Sessionizer, SessionizerConfig};

/// Shared application state
pub struct AppState {
    pub sessionizer: Arc<Mutex<Sessionizer>>,
    pub collector: Arc<dyn ForegroundCollector>,
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
                    for session in sessions {
                        println!(
                            "[Session] {} | {} | {}s",
                            session.app_id,
                            if session.is_idle { "IDLE" } else { "ACTIVE" },
                            session.duration_seconds.unwrap_or(0)
                        );
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let collector = create_collector();
    let sessionizer = Arc::new(Mutex::new(Sessionizer::new(SessionizerConfig::default())));
    
    let app_state = Arc::new(AppState {
        sessionizer: sessionizer.clone(),
        collector: collector.clone(),
    });

    // Start background polling
    start_polling_loop(app_state.clone());

    tauri::Builder::default()
        .manage(AppState {
            sessionizer,
            collector,
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_current_app, get_idle_seconds])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
