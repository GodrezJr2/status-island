use serde_json::json;
use si_core::model::{Activity, AgentKind};
use si_core::scan::{default_roots_with_probe, ExternalReading, Roots, Scanner};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn root() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "si-scan-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

#[test]
fn snapshot_from_fixture_tree() {
    let home = root();
    let claude = home.join(".claude");
    let codex = home.join(".codex");
    write(&claude.join("sessions/live.json"), &json!({"pid":std::process::id(),"sessionId":"test","cwd":"C:/safe/project","entrypoint":"cli","status":"busy"}).to_string());
    let turn: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/claude_transcript_assistant.jsonl")).unwrap();
    write(
        &claude.join("projects/test/test.jsonl"),
        &format!("{}\n", turn),
    );
    let now = now_ms();
    let line = json!({"timestamp": chrono::Utc::now().to_rfc3339(),"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":100},"total_token_usage":{"total_tokens":200},"model_context_window":1000},"rate_limits":{"primary":{"used_percent":13.0,"resets_at":now/1000+3600}}}});
    write(
        &codex.join("sessions/2026/10/18/rollout-test.jsonl"),
        &format!("{}\n", line),
    );
    let mut scanner = Scanner::new(Roots {
        claude: vec![claude],
        codex: vec![codex],
        opencode_db: vec![],
    });
    scanner.ingest_external(ExternalReading::StatusLine(
        json!({"rate_limits":{"five_hour":{"used_percentage":20.0,"resets_at":now/1000+3600}}}),
    ));
    let snapshot = scanner.tick(now);
    assert!(snapshot
        .sessions
        .iter()
        .any(|s| s.kind == AgentKind::Claude));
    assert!(snapshot.sessions.iter().any(|s| s.kind == AgentKind::Codex));
    assert_eq!(snapshot.claude.five_hour.unwrap().used_pct, 20.0);
    assert_eq!(snapshot.codex.five_hour.unwrap().used_pct, 13.0);
    let again = scanner.tick(now + 1000);
    assert!(again
        .sessions
        .iter()
        .any(|s| s.kind == AgentKind::Claude && s.ctx_tokens.is_some()));
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn codex_activity_ignores_stale_mtime() {
    let home = root();
    let codex = home.join(".codex");
    let now = now_ms();
    let line = json!({"timestamp": chrono::Utc::now().to_rfc3339(),"type":"event_msg","payload":{"type":"task_started"}});
    let path = codex.join("sessions/2026/10/18/rollout-test.jsonl");
    write(&path, &format!("{}\n", line));
    let file = fs::File::options().write(true).open(&path).unwrap();
    file.set_times(
        fs::FileTimes::new().set_modified(std::time::SystemTime::now() - Duration::from_secs(1200)),
    )
    .unwrap();
    let mut scanner = Scanner::new(Roots {
        claude: vec![],
        codex: vec![codex],
        opencode_db: vec![],
    });
    assert_eq!(scanner.tick(now).sessions[0].activity, Activity::Working);
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn dead_pid_hidden() {
    let home = root();
    let claude = home.join(".claude");
    write(&claude.join("sessions/dead.json"), &json!({"pid":u32::MAX,"sessionId":"dead","cwd":"C:/safe","entrypoint":"cli","status":"busy"}).to_string());
    let mut scanner = Scanner::new(Roots {
        claude: vec![claude],
        codex: vec![],
        opencode_db: vec![],
    });
    assert!(scanner.tick(now_ms()).sessions.is_empty());
    fs::remove_dir_all(home).unwrap();
}

/// Records what the scanner asks, and treats only one creation time as the live process.
struct FakeProbe(std::sync::Arc<std::sync::Mutex<Vec<(u32, Option<String>)>>>);
impl si_core::scan::ProcessProbe for FakeProbe {
    fn is_live(&self, pid: u32, proc_start: Option<&str>) -> bool {
        self.0
            .lock()
            .unwrap()
            .push((pid, proc_start.map(str::to_owned)));
        proc_start == Some("134367903253677432")
    }
}

#[test]
fn reused_pid_hidden_via_proc_start() {
    let home = root();
    let claude = home.join(".claude");
    write(&claude.join("sessions/same.json"), &json!({"pid":4242,"sessionId":"same","cwd":"C:/safe/a","entrypoint":"cli","status":"busy","procStart":"134367903253677432"}).to_string());
    write(&claude.join("sessions/reused.json"), &json!({"pid":4243,"sessionId":"reused","cwd":"C:/safe/b","entrypoint":"cli","status":"busy","procStart":"999"}).to_string());
    let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut scanner = Scanner::with_probe(
        Roots {
            claude: vec![claude],
            codex: vec![],
            opencode_db: vec![],
        },
        Box::new(FakeProbe(calls.clone())),
    );
    let sessions = scanner.tick(now_ms()).sessions;
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, "same");
    let seen = calls.lock().unwrap().clone();
    assert!(seen.contains(&(4243, Some("999".to_owned()))));
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn wsl_probe_times_out() {
    let start = Instant::now();
    let roots = default_roots_with_probe(Path::new("C:/safe"), vec!["Ubuntu".to_owned()], |_| {
        std::thread::sleep(Duration::from_secs(5));
        true
    });
    assert!(start.elapsed() < Duration::from_millis(700));
    assert!(roots
        .claude
        .iter()
        .all(|p| !p.to_string_lossy().contains("wsl.localhost")));
}
