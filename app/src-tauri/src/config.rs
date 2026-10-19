//! User settings at %LOCALAPPDATA%\StatusIsland\config.json (plan Task 12).
//! Connector state lives in its own file (connector-state.json), owned by `connectors`.

use crate::ntfy::NtfyConfig;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AlertPrefs {
    pub context_high: bool,
    pub task_finished: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    /// "active" follows the active window's monitor; otherwise a monitor id.
    pub monitor: String,
    pub offset_x: i32,
    pub ntfy: NtfyConfig,
    /// [start hour, end hour) in local time; may wrap past midnight.
    pub quiet_hours: Option<[u8; 2]>,
    pub prefs: AlertPrefs,
    pub alerts_paused_until_ms: Option<i64>,
    pub onboarded: bool,
    /// Look for a newer release on GitHub every few hours (one request, no account data).
    pub check_updates: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            monitor: "active".into(),
            offset_x: 0,
            ntfy: NtfyConfig::default(),
            quiet_hours: None,
            prefs: AlertPrefs::default(),
            alerts_paused_until_ms: None,
            onboarded: false,
            check_updates: true,
        }
    }
}

pub fn config_path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("StatusIsland/config.json"))
}

/// Missing or unreadable config means defaults; the app never fails to start over it.
pub fn load_from(path: &Path) -> Config {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

pub fn save_to(path: &Path, cfg: &Config) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(
        &tmp,
        serde_json::to_vec_pretty(cfg).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

pub fn load() -> Config {
    config_path().map(|p| load_from(&p)).unwrap_or_default()
}

pub fn save(cfg: &Config) -> Result<(), String> {
    save_to(&config_path().ok_or("local AppData unavailable")?, cfg)
}

/// Phone push is allowed when a phone is paired, alerts aren't paused, and it's outside quiet hours.
pub fn push_allowed(cfg: &Config, local_hour: u8, now_ms: i64) -> bool {
    if cfg.ntfy.topic.is_empty() {
        return false;
    }
    if cfg
        .alerts_paused_until_ms
        .is_some_and(|until| now_ms < until)
    {
        return false;
    }
    match cfg.quiet_hours {
        Some([start, end]) if start <= end => !(start..end).contains(&local_hour),
        Some([start, end]) => local_hour < start && local_hour >= end,
        None => true,
    }
}
