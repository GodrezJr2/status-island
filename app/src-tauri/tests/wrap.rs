use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
    sync::Mutex,
    time::{Duration, Instant},
};

static LOCK: Mutex<()> = Mutex::new(());

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_status-island")
}
fn node() -> String {
    "node".to_owned()
}
fn script(source: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "si-wrap-test-{}-{}.cjs",
        std::process::id(),
        source.len()
    ));
    fs::write(&path, source).unwrap();
    path
}

#[test]
fn wrap_args_identical() {
    let _guard = LOCK.lock().unwrap();
    let p = script("process.stdout.write(JSON.stringify(process.argv.slice(2)))");
    let args = ["a \"quote\"", "x\\", "", "বাংলা", "{\"a\":1}"];
    let output = Command::new(bin())
        .arg("--wrap")
        .arg(node())
        .arg(&p)
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Vec<String>>(&output.stdout).unwrap(),
        args
    );
    fs::remove_file(p).unwrap();
}

#[test]
fn wrap_stdin_stdout_passthrough() {
    let _guard = LOCK.lock().unwrap();
    let p = script("process.stdin.pipe(process.stdout);process.stderr.write('err')");
    let mut child = Command::new(bin())
        .arg("--wrap")
        .arg(node())
        .arg(&p)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"\0raw\xff\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.stdout, b"\0raw\xff\n");
    assert_eq!(output.stderr, b"err");
    fs::remove_file(p).unwrap();
}

#[test]
fn wrap_exit_code_preserved() {
    let _guard = LOCK.lock().unwrap();
    let p = script("process.exit(7)");
    let output = Command::new(bin())
        .arg("--wrap")
        .arg(node())
        .arg(&p)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(7));
    fs::remove_file(p).unwrap();
}

#[test]
fn statusline_fast_without_island() {
    let _guard = LOCK.lock().unwrap();
    let empty = std::env::temp_dir().join(format!("si-empty-{}", std::process::id()));
    fs::create_dir_all(&empty).unwrap();
    let start = Instant::now();
    let output = Command::new(bin())
        .arg("--statusline")
        .env("LOCALAPPDATA", &empty)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(start.elapsed() < Duration::from_millis(150));
    fs::remove_dir_all(empty).unwrap();
}

#[test]
fn statusline_chains_previous_command() {
    let _guard = LOCK.lock().unwrap();
    let home = std::env::temp_dir().join(format!("si-statusline-{}", std::process::id()));
    let data = home.join("StatusIsland");
    fs::create_dir_all(&data).unwrap();
    let p = script("process.stdin.pipe(process.stdout)");
    fs::write(
        data.join("previous-statusline.txt"),
        format!("\"{}\" \"{}\"", node(), p.display()),
    )
    .unwrap();
    let mut child = Command::new(bin())
        .arg("--statusline")
        .env("LOCALAPPDATA", &home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"hello").unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.stdout, b"hello");
    fs::remove_file(p).unwrap();
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn wrap_kill_ends_child() {
    let _guard = LOCK.lock().unwrap();
    let pid_file = std::env::temp_dir().join(format!("si-child-pid-{}", std::process::id()));
    let p = script(
        "require('fs').writeFileSync(process.argv[2],String(process.pid));setInterval(()=>{},1000)",
    );
    let mut wrapper = Command::new(bin())
        .arg("--wrap")
        .arg(node())
        .arg(&p)
        .arg(&pid_file)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    for _ in 0..100 {
        if pid_file.exists() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(pid_file.exists());
    let pid = fs::read_to_string(&pid_file).unwrap();
    wrapper.kill().unwrap();
    wrapper.wait().unwrap();
    let mut gone = false;
    for _ in 0..100 {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
            .output()
            .unwrap();
        if !String::from_utf8_lossy(&output.stdout).lines().any(|line| {
            line.split(',')
                .nth(1)
                .is_some_and(|n| n.trim_matches('"') == pid)
        }) {
            gone = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(gone, "child process survived wrapper termination");
    fs::remove_file(p).unwrap();
    fs::remove_file(pid_file).unwrap();
}
