use notify::{RecursiveMode, Watcher};
use si_core::{
    model::Snapshot,
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
            *state.0.lock().unwrap() = snapshot.clone();
            let _ = app.emit("snapshot", snapshot);
        }
    });
}
