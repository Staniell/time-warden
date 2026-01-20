use crate::models::AppInfo;

pub trait ForegroundCollector: Send + Sync {
    fn get_foreground_app(&self) -> Option<AppInfo>;
    fn get_idle_seconds(&self) -> u64;
}

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

pub fn create_collector() -> std::sync::Arc<dyn ForegroundCollector> {
    #[cfg(target_os = "windows")]
    { std::sync::Arc::new(windows::WindowsCollector::new()) }
    
    #[cfg(target_os = "macos")]
    { std::sync::Arc::new(macos::MacOSCollector::new()) }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    { panic!("Unsupported platform") }
}
