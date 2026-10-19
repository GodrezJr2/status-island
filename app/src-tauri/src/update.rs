//! Updates (spec §9 "Auto-update"): one request for the release's latest.json on GitHub.
//! The updater only installs installers signed with the project's update key, and the NSIS
//! installer runs in update mode, so the Claude connector and settings stay as they are.

use std::{sync::Mutex, time::Duration};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

const FIRST_CHECK: Duration = Duration::from_secs(60);
const EVERY: Duration = Duration::from_secs(6 * 3600);

#[derive(Default)]
pub struct UpdateState(Mutex<Option<Update>>);

/// The newer version waiting to be installed, if a check found one.
pub fn available(app: &AppHandle) -> Option<String> {
    let state = app.state::<UpdateState>();
    let pending = state.0.lock().unwrap();
    pending.as_ref().map(|u| u.version.clone())
}

/// Background check: a minute after start, then every 6 hours, while the setting is on.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(FIRST_CHECK);
        loop {
            if status_island::config::load().check_updates {
                let _ = tauri::async_runtime::block_on(check(&app));
            }
            std::thread::sleep(EVERY);
        }
    });
}

/// Asks GitHub for the latest release; returns the newer version, or None when up to date.
pub async fn check(app: &AppHandle) -> Result<Option<String>, String> {
    let update = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;
    let version = update.as_ref().map(|u| u.version.clone());
    *app.state::<UpdateState>().0.lock().unwrap() = update;
    crate::tray::show_update(app, version.as_deref());
    let _ = app.emit("update", version.clone());
    Ok(version)
}

/// Downloads, verifies the signature and runs the installer. On Windows the app exits here
/// and the installer starts it again when it finishes.
pub async fn install(app: &AppHandle) -> Result<(), String> {
    let pending = app.state::<UpdateState>().0.lock().unwrap().clone();
    let Some(update) = pending else {
        return Err("No update is waiting.".into());
    };
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())
}
