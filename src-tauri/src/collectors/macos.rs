use crate::collectors::ForegroundCollector;
use crate::models::AppInfo;

pub struct MacOSCollector;

impl MacOSCollector {
    pub fn new() -> Self {
        Self
    }
}

impl ForegroundCollector for MacOSCollector {
    fn get_foreground_app(&self) -> Option<AppInfo> {
        None // Implementation in Phase 2
    }
    
    fn get_idle_seconds(&self) -> u64 {
        0 // Implementation in Phase 2
    }
}
