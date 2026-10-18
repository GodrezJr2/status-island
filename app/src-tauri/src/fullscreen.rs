use windows::Win32::{
    Foundation::RECT,
    UI::{
        Shell::{
            SHQueryUserNotificationState, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
        },
        WindowsAndMessaging::{GetForegroundWindow, GetWindowRect},
    },
};

pub fn fullscreen_active(monitor: &tauri::Monitor) -> bool {
    if let Ok(state) = unsafe { SHQueryUserNotificationState() } {
        if state == QUNS_RUNNING_D3D_FULL_SCREEN || state == QUNS_PRESENTATION_MODE {
            return true;
        }
    }
    let foreground = unsafe { GetForegroundWindow() };
    if foreground.0.is_null() {
        return false;
    }
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(foreground, &mut rect) }.is_err() {
        return false;
    }
    let position = monitor.position();
    let size = monitor.size();
    rect.left <= position.x + 2
        && rect.top <= position.y + 2
        && rect.right >= position.x + size.width as i32 - 2
        && rect.bottom >= position.y + size.height as i32 - 2
}
