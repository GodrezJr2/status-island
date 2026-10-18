use notify::{RecursiveMode, Watcher};
use si_core::{
    alerts::AlertEngine,
    model::{AgentKind, Snapshot, WindowKind},
    scan::{default_roots, ExternalReading, Scanner},
};
use std::{
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};

#[derive(Clone)]
pub struct SnapshotState(pub Arc<Mutex<Snapshot>>);

pub enum Signal {
    Wake,
    External(ExternalReading),
}

pub fn start(app: AppHandle, state: SnapshotState) {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_default();
    let roots = default_roots(&home);
    let (tx, rx) = mpsc::channel();
    let (push_tx, push_rx) =
        mpsc::channel::<(status_island::ntfy::NtfyConfig, si_core::alerts::Alert)>();
    std::thread::spawn(move || {
        for (cfg, alert) in push_rx {
            let _ = status_island::ntfy::send(&cfg, &alert);
        }
    });
    crate::pipe::serve(tx.clone());
    std::thread::spawn(move || {
        let notify_tx = tx.clone();
        let mut watcher =
            notify::recommended_watcher(move |_event: notify::Result<notify::Event>| {
                let _ = notify_tx.send(Signal::Wake);
            })
            .ok();
        if let Some(watcher) = watcher.as_mut() {
            for root in &roots.claude {
                for folder in ["sessions", "projects"] {
                    let path = root.join(folder);
                    if path.exists() {
                        let _ = watcher.watch(&path, RecursiveMode::Recursive);
                    }
                }
            }
            for root in &roots.codex {
                let path = root.join("sessions");
                if path.exists() {
                    let _ = watcher.watch(&path, RecursiveMode::Recursive);
                }
            }
            for db in &roots.opencode_db {
                if let Some(parent) = db.parent() {
                    if parent.exists() {
                        let _ = watcher.watch(parent, RecursiveMode::NonRecursive);
                    }
                }
            }
        }
        let mut scanner = Scanner::new(roots);
        let mut alerts = AlertEngine::default();
        loop {
            let signal = rx.recv_timeout(Duration::from_secs(1));
            let woke = signal.is_ok();
            if let Ok(Signal::External(reading)) = signal {
                scanner.ingest_external(reading);
            }
            while let Ok(next) = rx.try_recv() {
                if let Signal::External(reading) = next {
                    scanner.ingest_external(reading);
                }
            }
            if woke {
                std::thread::sleep(Duration::from_millis(150));
            }
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            let snapshot = scanner.tick(now_ms);
            alerts.set_overage_disabled(AgentKind::Claude, snapshot.claude.overage_disabled);
            for (agent, account) in [
                (AgentKind::Claude, &snapshot.claude),
                (AgentKind::Codex, &snapshot.codex),
            ] {
                let project = snapshot
                    .sessions
                    .iter()
                    .find(|s| s.kind == agent)
                    .map(|s| s.project.as_str())
                    .unwrap_or("");
                for (window, view) in [
                    (WindowKind::FiveHour, &account.five_hour),
                    (WindowKind::Weekly, &account.weekly),
                ] {
                    if let Some(view) = view {
                        for alert in alerts.evaluate(agent, window, view, project, now_ms) {
                            if status_island::away::is_away() {
                                if let Some(cfg) = paired_ntfy_config() {
                                    let _ = push_tx.send((cfg, alert.clone()));
                                }
                            } else {
                                let _ = app.emit("alert", alert);
                            }
                        }
                    }
                }
            }
            *state.0.lock().unwrap() = snapshot.clone();
            let _ = app.emit("snapshot", snapshot);
        }
    });
}

fn paired_ntfy_config() -> Option<status_island::ntfy::NtfyConfig> {
    let path = PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("StatusIsland/config.json");
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    let cfg: status_island::ntfy::NtfyConfig =
        serde_json::from_value(value.get("ntfy")?.clone()).ok()?;
    (!cfg.topic.is_empty()).then_some(cfg)
}
