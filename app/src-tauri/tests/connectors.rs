use status_island::{connectors, jsonc_edit};
use std::{fs, path::Path};

fn fixture() -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "si-connector-test-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn jsonc_edit_preserves_comments_and_other_keys() {
    let input = "{\r\n  // keep this comment\r\n  \"editor.fontSize\": 14,\r\n  /* and this one */\r\n}\r\n";
    let value = serde_json::json!("C:\\Tools\\si-wrap.exe");
    let changed =
        jsonc_edit::set_key(input, "claudeCode.claudeProcessWrapper", Some(&value)).unwrap();
    assert!(changed.contains("// keep this comment"));
    assert!(changed.contains("/* and this one */"));
    assert!(changed.contains("\"editor.fontSize\": 14"));
    assert!(changed.contains("claudeCode.claudeProcessWrapper"));
    assert_eq!(
        jsonc_edit::set_key(&changed, "claudeCode.claudeProcessWrapper", None).unwrap(),
        input
    );
}

#[test]
fn statusline_previous_command_saved_and_restored() {
    let root = fixture();
    let vs = root.join("Code/User/settings.json");
    let claude = root.join(".claude/settings.json");
    let data = root.join("StatusIsland");
    fs::create_dir_all(vs.parent().unwrap()).unwrap();
    fs::create_dir_all(claude.parent().unwrap()).unwrap();
    fs::write(&vs, "{\"editor.fontSize\":14}\r\n").unwrap();
    let original =
        "{\r\n  \"statusLine\": {\"type\":\"command\",\"command\":\"ccstatusline\"}\r\n}\r\n";
    fs::write(&claude, original).unwrap();
    let state =
        connectors::enable_claude_at(Path::new("C:/Tools/status-island.exe"), &vs, &claude, &data)
            .unwrap();
    assert_eq!(
        fs::read_to_string(data.join("previous-statusline.txt")).unwrap(),
        "ccstatusline"
    );
    connectors::disable_claude_at(&state).unwrap();
    assert_eq!(fs::read_to_string(&claude).unwrap(), original);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn wrapper_set_and_cleared() {
    let root = fixture();
    let vs = root.join("Code/User/settings.json");
    let claude = root.join(".claude/settings.json");
    let data = root.join("StatusIsland");
    fs::create_dir_all(vs.parent().unwrap()).unwrap();
    fs::create_dir_all(claude.parent().unwrap()).unwrap();
    let original = "{\r\n // hi\r\n \"editor.fontSize\": 14,\r\n}\r\n";
    fs::write(&vs, original).unwrap();
    fs::write(&claude, "{}\n").unwrap();
    let state =
        connectors::enable_claude_at(Path::new("C:/Tools/status-island.exe"), &vs, &claude, &data)
            .unwrap();
    assert_eq!(fs::read(&state.backups[0]).unwrap(), original.as_bytes());
    let changed = fs::read_to_string(&vs).unwrap();
    assert!(changed.contains("si-wrap.exe"));
    assert!(changed.contains("editor.fontSize"));
    connectors::disable_claude_at(&state).unwrap();
    assert_eq!(fs::read_to_string(&vs).unwrap(), original);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn disable_preserves_unrelated_user_change() {
    let root = fixture();
    let vs = root.join("Code/User/settings.json");
    let claude = root.join(".claude/settings.json");
    let data = root.join("StatusIsland");
    fs::create_dir_all(vs.parent().unwrap()).unwrap();
    fs::create_dir_all(claude.parent().unwrap()).unwrap();
    fs::write(&vs, "{\"editor.fontSize\":14}\n").unwrap();
    fs::write(&claude, "{}\n").unwrap();
    let state =
        connectors::enable_claude_at(Path::new("C:/Tools/status-island.exe"), &vs, &claude, &data)
            .unwrap();
    let changed = fs::read_to_string(&vs).unwrap().replace("14", "16");
    fs::write(&vs, changed).unwrap();
    connectors::disable_claude_at(&state).unwrap();
    let after = fs::read_to_string(&vs).unwrap();
    assert!(after.contains("\"editor.fontSize\":16"));
    assert!(!after.contains("claudeCode.claudeProcessWrapper"));
    fs::remove_dir_all(root).unwrap();
}
