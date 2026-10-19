use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::{
    Foundation::RECT,
    UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
        Shell::{
            SHQueryUserNotificationState, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
        },
        WindowsAndMessaging::{
            DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowRect, TranslateMessage,
            EVENT_SYSTEM_FOREGROUND, MSG, WINEVENT_OUTOFCONTEXT,
        },
    },
};

static FOREGROUND_CHANGED: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn on_foreground(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _window: windows::Win32::Foundation::HWND,
    _object: i32,
    _child: i32,
    _thread: u32,
    _time: u32,
) {
    FOREGROUND_CHANGED.store(true, Ordering::Relaxed);
}

pub fn start_foreground_hook() {
    std::thread::spawn(|| {
        let hook = unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                None,
                Some(on_foreground),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            )
        };
        if hook.0.is_null() {
            return;
        }
        let mut message = MSG::default();
        while unsafe { GetMessageW(&mut message, None, 0, 0) }.0 > 0 {
            unsafe {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        unsafe {
            let _ = UnhookWinEvent(hook);
        }
    });
}

pub fn foreground_changed() -> bool {
    FOREGROUND_CHANGED.swap(false, Ordering::Relaxed)
}

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
