// GUI subsystem: no console window when the island starts. The --statusline and wrap modes
// still use the stdin/stdout pipes Claude Code and VS Code hand them.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod engine;
mod fullscreen;
mod overlay;
mod pipe;
mod statusline;
mod tray;
mod update;
mod wrap;

use status_island::{config, connectors, ntfy};

fn main() {
    let exe_is_wrap = std::env::current_exe()
        .ok()
        .and_then(|p| {
            p.file_stem().map(|s| {
                s.to_string_lossy()
                    .to_ascii_lowercase()
                    .starts_with("si-wrap")
            })
        })
        .unwrap_or(false);
    let mut args = std::env::args_os().skip(1);
    let first = args.next();
    let code = if exe_is_wrap {
        wrap::run(first.into_iter().chain(args), 2)
    } else if first.as_deref() == Some(std::ffi::OsStr::new("--wrap")) {
        wrap::run(args, 3)
    } else if first.as_deref() == Some(std::ffi::OsStr::new("--statusline")) {
        statusline::run()
    } else if first.as_deref() == Some(std::ffi::OsStr::new("--uninstall-cleanup")) {
        uninstall_cleanup()
    } else {
        return run_app();
    };
    std::process::exit(code);
}

#[tauri::command]
fn get_snapshot(state: tauri::State<'_, engine::SnapshotState>) -> si_core::model::Snapshot {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
fn set_island_rect(rect: overlay::IslandRect, state: tauri::State<'_, overlay::OverlayState>) {
    *state.0.lock().unwrap() = rect;
}

#[tauri::command]
fn get_config() -> config::Config {
    config::load()
}

#[tauri::command]
fn set_config(cfg: config::Config) -> Result<(), String> {
    config::save(&cfg)
}

/// Whether the Claude connector (VS Code wrapper + status line add-on) is switched on.
#[tauri::command]
fn connector_status() -> bool {
    connector_manifest().is_some_and(|p| p.is_file())
}

#[tauri::command]
fn enable_claude() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    connectors::enable_claude(&exe).map(|_| ())
}

#[tauri::command]
fn disable_claude() -> Result<(), String> {
    connectors::disable_claude()
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Pairing {
    subscribe_url: String,
    qr_svg: String,
}

/// Creates the phone topic on first use and returns the QR the ntfy app scans. Nothing is sent anywhere.
#[tauri::command]
fn pairing() -> Result<Pairing, String> {
    let mut cfg = config::load();
    if cfg.ntfy.topic.is_empty() {
        cfg.ntfy.topic = ntfy::new_topic();
        config::save(&cfg)?;
    }
    let host = cfg
        .ntfy
        .server
        .trim_end_matches('/')
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_owned();
    let subscribe_url = format!("ntfy://{host}/{}", cfg.ntfy.topic);
    let qr_svg = qrcode::QrCode::new(subscribe_url.as_bytes())
        .map_err(|e| e.to_string())?
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(148, 148)
        .quiet_zone(true)
        .build();
    Ok(Pairing {
        subscribe_url,
        qr_svg,
    })
}

#[tauri::command]
fn send_test_push() -> Result<(), String> {
    let cfg = config::load();
    if cfg.ntfy.topic.is_empty() {
        return Err("Pair your phone first".into());
    }
    let alert = si_core::alerts::Alert {
        agent: si_core::model::AgentKind::Claude,
        window: si_core::model::WindowKind::FiveHour,
        kind: si_core::alerts::AlertKind::TaskFinished,
        title: "Status Island · test".into(),
        body: "Phone alerts are working.".into(),
        priority: 3,
        level: si_core::alerts::Level::Info,
    };
    ntfy::send(&cfg.ntfy, &alert).map_err(|e| format!("{e:?}"))
}

#[tauri::command]
fn finish_onboarding(app: tauri::AppHandle) -> Result<(), String> {
    let mut cfg = config::load();
    cfg.onboarded = true;
    config::save(&cfg)?;
    if let Some(w) = tauri::Manager::get_webview_window(&app, "connect") {
        let _ = w.close();
    }
    Ok(())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateStatus {
    current: String,
    available: Option<String>,
}

#[tauri::command]
fn update_status(app: tauri::AppHandle) -> UpdateStatus {
    UpdateStatus {
        current: app.package_info().version.to_string(),
        available: update::available(&app),
    }
}

#[tauri::command]
async fn check_update(app: tauri::AppHandle) -> Result<Option<String>, String> {
    update::check(&app).await
}

#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    update::install(&app).await
}

fn connector_manifest() -> Option<std::path::PathBuf> {
    Some(
        std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA")?)
            .join("StatusIsland/connector-state.json"),
    )
}

/// Run by the uninstaller: put back every settings file the app changed, then remove app data.
fn uninstall_cleanup() -> i32 {
    let mut code = 0;
    if connector_status() && connectors::disable_claude().is_err() {
        code = 1;
    }
    if code == 0 {
        if let Some(dir) = config::config_path().and_then(|p| p.parent().map(|d| d.to_owned())) {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
    code
}

fn run_app() {
    let initial = si_core::model::Snapshot {
        now_ms: 0,
        sessions: Vec::new(),
        claude: Default::default(),
        codex: Default::default(),
        opencode_today: None,
    };
    let state = engine::SnapshotState(std::sync::Arc::new(std::sync::Mutex::new(initial)));
    let thread_state = state.clone();
    let overlay_state = overlay::OverlayState::default();
    let overlay_thread_state = overlay_state.clone();
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(state)
        .manage(update::UpdateState::default())
        .manage(overlay_state)
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            set_island_rect,
            get_config,
            set_config,
            connector_status,
            enable_claude,
            disable_claude,
            pairing,
            send_test_push,
            finish_onboarding,
            update_status,
            check_update,
            install_update
        ])
        .setup(move |app| {
            engine::start(app.handle().clone(), thread_state.clone());
            overlay::start(app.handle().clone(), overlay_thread_state.clone());
            tray::create(app.handle())?;
            update::start(app.handle().clone());
            if !config::load().onboarded {
                tray::show_window(app.handle(), "connect");
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Tauri startup failed");
}
