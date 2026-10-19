use serde_json::json;
use si_core::{claude, codex, opencode};

#[test]
fn claude_ctx_sums_three_input_fields() {
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/claude_transcript_assistant.jsonl")).unwrap();
    v["message"]["usage"] = json!({"input_tokens":2,"cache_creation_input_tokens":1494,"cache_read_input_tokens":41500,"output_tokens":300});
    assert_eq!(claude::parse_transcript_line(&v).unwrap().ctx_tokens, 42996);
}

#[test]
fn claude_statusline_accepts_epoch_and_iso() {
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/claude_statusline_stdin.jsonl")).unwrap();
    let (five, week, size) = claude::parse_statusline(&v);
    assert_eq!(five.unwrap().resets_at_ms, 1792329000000);
    assert!(week.is_some());
    assert_eq!(size, Some(1_000_000));
    v["rate_limits"]["five_hour"]["resets_at"] = json!("2026-10-18T11:50:00Z");
    assert!(claude::parse_statusline(&v).0.is_some());
}

#[test]
fn claude_event_maps_utilization_to_pct() {
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/claude_rate_limit_event.jsonl")).unwrap();
    v["rate_limit_info"]["unifiedWindows"]["five_hour"]["utilization"] = json!(0.25);
    let (five, _, disabled) = claude::parse_rate_limit_event(&v);
    assert_eq!(five.unwrap().used_pct, 25.0);
    assert!(disabled);
}

#[test]
fn codex_limits_from_token_count() {
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/codex_token_count.jsonl")).unwrap();
    v["payload"]["rate_limits"]["primary"]["used_percent"] = json!(13.0);
    assert!(codex::parse_rollout_line(&v).iter().any(|event| matches!(event, codex::CodexEvent::Limits { primary, .. } if primary.used_pct == 13.0 && primary.resets_at_ms == 1792328518000)));
}

#[test]
fn opencode_reads_only_session_v2() {
    let dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/test-opencode");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("si-oc-test-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch("CREATE TABLE session_v2(id TEXT, directory TEXT, model TEXT, cost REAL, tokens_input INTEGER, tokens_output INTEGER, tokens_reasoning INTEGER, tokens_cache_read INTEGER, tokens_cache_write INTEGER, time_updated INTEGER, time_idle INTEGER, time_archived INTEGER); CREATE TABLE credential(secret TEXT); INSERT INTO credential VALUES('sentinel'); INSERT INTO session_v2 VALUES('a','/safe','m',1.5,10,2,0,3,0,1792320000000,NULL,NULL);").unwrap();
    drop(conn);
    let (sessions, _) = opencode::read_opencode(&path).unwrap();
    assert_eq!(sessions[0].id, "a");
    std::fs::remove_file(&path).unwrap();

    // A hostile schema cannot make our SELECT read a credential table through a view.
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch("CREATE TABLE credential(id TEXT, directory TEXT, model TEXT, cost REAL, tokens_input INTEGER, tokens_output INTEGER, tokens_reasoning INTEGER, tokens_cache_read INTEGER, tokens_cache_write INTEGER, time_updated INTEGER, time_idle INTEGER, time_archived INTEGER); CREATE VIEW session_v2 AS SELECT * FROM credential;").unwrap();
    drop(conn);
    assert!(opencode::read_opencode(&path).is_err());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn parsers_ignore_unknown_shapes() {
    let v = json!({"unexpected": true});
    assert!(claude::parse_transcript_line(&v).is_none());
    assert!(claude::parse_session_file(&v).is_none());
    assert!(codex::parse_rollout_line(&v).is_empty());
}
