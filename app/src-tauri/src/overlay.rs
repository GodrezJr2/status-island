use serde::{Deserialize, Serialize};
use status_island::placement::{place, MonitorChoice, MonitorInfo, TaskbarInfo};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize};
use windows::Win32::{
    Foundation::POINT,
    UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON},
        Shell::{SHAppBarMessage, ABE_TOP, ABM_GETTASKBARPOS, APPBARDATA},
        WindowsAndMessaging::GetCursorPos,
    },
};

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IslandRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Default)]
pub struct OverlayState(pub Arc<Mutex<IslandRect>>);

#[derive(Clone, Copy, Serialize, PartialEq, Eq)]
struct Pointer {
    inside: bool,
    down: bool,
}

pub fn start(app: tauri::AppHandle, state: OverlayState) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let _ = window.set_ignore_cursor_events(true);
    let _ = window.set_decorations(false);
    let _ = window.set_always_on_top(true);
    let _ = window.set_skip_taskbar(true);
    let _ = window.set_focusable(false);
    std::thread::spawn(move || {
        let mut last_pointer = Pointer {
            inside: false,
            down: false,
        };
        let mut cfg = status_island::config::load();
        reposition(&window, &cfg);
        let mut last_monitor = window
            .current_monitor()
            .ok()
            .flatten()
            .map(|m| format!("{:?}", m.name()))
            .unwrap_or_default();
        let mut last_fullscreen = false;
        let mut ticks = 0u32;
        loop {
            let mut cursor = POINT::default();
            let cursor_ok = unsafe { GetCursorPos(&mut cursor) }.is_ok();
            let rect = *state.0.lock().unwrap();
            let Ok(position) = window.outer_position() else {
                std::thread::sleep(Duration::from_millis(250));
                continue;
            };
            let scale = window.scale_factor().unwrap_or(1.0);
            let left = position.x as f64 + rect.x * scale;
            let top = position.y as f64 + rect.y * scale;
            let right = left + rect.w * scale;
            let bottom = top + rect.h * scale;
            let x = cursor.x as f64;
            let y = cursor.y as f64;
            let near = cursor_ok
                && x >= left - 200.0
                && x <= right + 200.0
                && y >= top - 200.0
                && y <= bottom + 200.0;
            let pointer = Pointer {
                inside: cursor_ok && x >= left && x < right && y >= top && y < bottom,
                down: unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } < 0,
            };
            if pointer != last_pointer {
                let _ = app.emit("pointer", pointer);
                last_pointer = pointer;
            }
            ticks = ticks.wrapping_add(1);
            if ticks.is_multiple_of(20) {
                let latest = status_island::config::load();
                let settings_changed =
                    latest.monitor != cfg.monitor || latest.offset_x != cfg.offset_x;
                cfg = latest;
                if let Some(current) = window.current_monitor().ok().flatten() {
                    let monitor_id = format!("{:?}", current.name());
                    if monitor_id != last_monitor || settings_changed {
                        reposition(&window, &cfg);
                        last_monitor = monitor_id;
                    }
                    let fullscreen = crate::fullscreen::fullscreen_active(&current);
                    if fullscreen != last_fullscreen {
                        let _ = if fullscreen {
                            window.hide()
                        } else {
                            window.show()
                        };
                        last_fullscreen = fullscreen;
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(if near { 16 } else { 250 }));
        }
    });
}

fn reposition(window: &tauri::WebviewWindow, cfg: &status_island::config::Config) {
    let choice = if cfg.monitor == "active" {
        MonitorChoice::Active
    } else {
        MonitorChoice::Fixed(cfg.monitor.clone())
    };
    let Ok(monitors) = window.available_monitors() else {
        return;
    };
    let active = window.current_monitor().ok().flatten();
    let active_name = active.as_ref().and_then(|m| m.name()).cloned();
    let info: Vec<_> = monitors
        .iter()
        .enumerate()
        .map(|(index, monitor)| {
            let position = monitor.position();
            let size = monitor.size();
            let scale = monitor.scale_factor();
            let name = monitor.name().cloned().unwrap_or_else(|| index.to_string());
            MonitorInfo {
                id: name.clone(),
                x: position.x,
                y: position.y,
                w: size.width,
                h: size.height,
                work_x: position.x,
                work_y: position.y,
                work_w: size.width,
                work_h: size.height,
                scale,
                active: active_name.as_ref() == Some(&name),
            }
        })
        .collect();
    let mut bar = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        ..Default::default()
    };
    let taskbar =
        if unsafe { SHAppBarMessage(ABM_GETTASKBARPOS, &mut bar) } != 0 && bar.uEdge == ABE_TOP {
            info.iter()
                .find(|m| bar.rc.left >= m.x && bar.rc.left < m.x + m.w as i32)
                .map(|m| TaskbarInfo {
                    monitor_id: m.id.clone(),
                    top: bar.rc.top,
                    height: (bar.rc.bottom - bar.rc.top).max(0) as u32,
                })
        } else {
            None
        };
    if let Some(target) = place(&info, choice, cfg.offset_x, taskbar).first() {
        let _ = window.set_size(PhysicalSize::new(target.w, target.h));
        let _ = window.set_position(PhysicalPosition::new(target.x, target.y));
        let _ = window.show();
    }
}
