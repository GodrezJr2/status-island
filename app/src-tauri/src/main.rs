mod engine;
mod pipe;
mod statusline;
mod wrap;

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
    } else {
        return run_app();
    };
    std::process::exit(code);
}

#[tauri::command]
fn get_snapshot(state: tauri::State<'_, engine::SnapshotState>) -> si_core::model::Snapshot {
    state.0.lock().unwrap().clone()
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
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![get_snapshot])
        .setup(move |app| {
            engine::start(app.handle().clone(), thread_state.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Tauri startup failed");
}
