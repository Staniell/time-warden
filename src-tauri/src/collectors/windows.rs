use crate::collectors::ForegroundCollector;
use crate::models::AppInfo;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::HWND,
    Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId},
    Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    Win32::System::ProcessStatus::GetModuleBaseNameW,
    Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
};

pub struct WindowsCollector;

impl WindowsCollector {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "windows")]
impl ForegroundCollector for WindowsCollector {
    fn get_foreground_app(&self) -> Option<AppInfo> {
        unsafe {
            let hwnd: HWND = GetForegroundWindow();
            if hwnd.0.is_null() {
                return None;
            }

            // Get window title
            let mut title_buf = [0u16; 512];
            let title_len = GetWindowTextW(hwnd, &mut title_buf);
            let app_title = if title_len > 0 {
                Some(String::from_utf16_lossy(&title_buf[..title_len as usize]))
            } else {
                None
            };

            // Get process ID
            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id));

            if process_id == 0 {
                return None;
            }

            // Get process name
            let process_name = get_process_name(process_id).unwrap_or_else(|| "Unknown".to_string());

            Some(AppInfo {
                process_name,
                app_title,
                bundle_id: None,
            })
        }
    }

    fn get_idle_seconds(&self) -> u64 {
        unsafe {
            let mut last_input = LASTINPUTINFO {
                cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
                dwTime: 0,
            };

            if GetLastInputInfo(&mut last_input).as_bool() {
                let tick_count = windows::Win32::System::SystemInformation::GetTickCount();
                let idle_ms = tick_count.saturating_sub(last_input.dwTime);
                (idle_ms / 1000) as u64
            } else {
                0
            }
        }
    }
}

#[cfg(target_os = "windows")]
unsafe fn get_process_name(process_id: u32) -> Option<String> {
    let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, process_id).ok()?;
    
    let mut name_buf = [0u16; 260];
    let len = GetModuleBaseNameW(handle, None, &mut name_buf);
    
    // Close the handle
    let _ = windows::Win32::Foundation::CloseHandle(handle);

    if len > 0 {
        Some(String::from_utf16_lossy(&name_buf[..len as usize]))
    } else {
        None
    }
}

#[cfg(not(target_os = "windows"))]
impl ForegroundCollector for WindowsCollector {
    fn get_foreground_app(&self) -> Option<AppInfo> {
        None
    }

    fn get_idle_seconds(&self) -> u64 {
        0
    }
}
