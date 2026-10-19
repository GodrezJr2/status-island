//! Tray menu (plan Task 12): Connect, Settings, Check for updates, Pause alerts 1 h, Quit.

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    window::Color,
    AppHandle, Emitter, Manager, Theme, WebviewUrl, WebviewWindowBuilder, Wry,
};

const CHECK_TEXT: &str = "Check for updates";

/// The update menu item, kept so its text can follow the update state.
struct UpdateItem(MenuItem<Wry>);

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let connect = MenuItem::with_id(app, "connect", "Connect", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let update = MenuItem::with_id(app, "update", CHECK_TEXT, true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pause alerts 1 h", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&connect, &settings, &update, &pause, &quit])?;
    app.manage(UpdateItem(update));
    let mut tray = TrayIconBuilder::with_id("status-island")
        .tooltip("Status Island")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "connect" => show_window(app, "connect"),
            "settings" => show_window(app, "settings"),
            "update" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if crate::update::available(&app).is_some() {
                        if crate::update::install(&app).await.is_err() {
                            set_update_text(&app, "Update failed · try again");
                        }
                    } else {
                        match crate::update::check(&app).await {
                            Ok(None) => set_update_text(
                                &app,
                                &format!("Up to date (v{})", app.package_info().version),
                            ),
                            Ok(Some(_)) => {}
                            Err(_) => set_update_text(&app, "Update check failed · try again"),
                        }
                    }
                });
            }
            "pause" => {
                let mut cfg = status_island::config::load();
                cfg.alerts_paused_until_ms = Some(now_ms() + 3_600_000);
                let _ = status_island::config::save(&cfg);
            }
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// Puts a found update on the tray menu, or returns the item to "Check for updates".
pub fn show_update(app: &AppHandle, version: Option<&str>) {
    match version {
        Some(v) => set_update_text(app, &format!("Install update {v}")),
        None => set_update_text(app, CHECK_TEXT),
    }
}

fn set_update_text(app: &AppHandle, text: &str) {
    if let Some(item) = app.try_state::<UpdateItem>() {
        let _ = item.0.set_text(text);
    }
}

/// Shows the settings window on the given view ("connect" or "settings"). The window is created
/// on demand and destroyed when closed, so no second renderer sits in memory all day.
pub fn show_window(app: &AppHandle, view: &str) {
    if let Some(window) = app.get_webview_window("connect") {
        let _ = app.emit_to("connect", "view", view);
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let view = if view == "settings" {
        "settings"
    } else {
        "connect"
    };
    let _ = WebviewWindowBuilder::new(app, "connect", WebviewUrl::App("index.html".into()))
        .initialization_script(format!("window.__SI_VIEW = '{view}';"))
        .title("Status Island")
        .inner_size(460.0, 660.0)
        .resizable(false)
        .center()
        .theme(Some(Theme::Dark))
        .background_color(Color(22, 22, 24, 255))
        .build();
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
