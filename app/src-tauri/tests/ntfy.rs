use si_core::{
    alerts::{Alert, AlertKind, Level},
    model::{AgentKind, WindowKind},
};
use status_island::ntfy::{self, DailyCounter, NtfyConfig, NtfyError};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    time::Duration,
};

fn alert() -> Alert {
    Alert {
        agent: AgentKind::Claude,
        window: WindowKind::FiveHour,
        kind: AlertKind::Left20,
        title: "Claude Code · 20% left".into(),
        body: "5-hour limit · resets 3:40 PM · review model usage".into(),
        priority: 4,
        level: Level::Warn,
    }
}

fn server(
    status: u16,
    requests: usize,
) -> (String, mpsc::Receiver<String>, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::channel();
    let handle = std::thread::spawn(move || {
        for _ in 0..requests {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut bytes = Vec::new();
            let mut buffer = [0u8; 1024];
            loop {
                let n = stream.read(&mut buffer).unwrap_or(0);
                if n == 0 {
                    break;
                }
                bytes.extend_from_slice(&buffer[..n]);
                if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&bytes[..end]);
                    let len = head
                        .lines()
                        .find_map(|line| {
                            line.to_ascii_lowercase()
                                .strip_prefix("content-length: ")
                                .and_then(|n| n.trim().parse::<usize>().ok())
                        })
                        .unwrap_or(0);
                    if bytes.len() >= end + 4 + len {
                        break;
                    }
                }
            }
            tx.send(String::from_utf8_lossy(&bytes).to_string())
                .unwrap();
            write!(
                stream,
                "HTTP/1.1 {status} Test\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            )
            .unwrap();
        }
    });
    (format!("http://{addr}"), rx, handle)
}

fn counter(name: &str) -> DailyCounter {
    DailyCounter::new(
        std::env::temp_dir().join(format!("si-ntfy-{}-{name}.json", std::process::id())),
    )
}

#[test]
fn topic_format() {
    let a = ntfy::new_topic();
    let b = ntfy::new_topic();
    assert_eq!(a.len(), 27);
    assert!(a.starts_with("si-"));
    assert_ne!(a, b);
    assert!(a[3..]
        .chars()
        .all(|c| "abcdefghijklmnopqrstuvwxyz234567".contains(c)));
}

#[test]
fn send_builds_expected_request() {
    let (url, rx, handle) = server(200, 1);
    let cfg = NtfyConfig {
        server: url,
        topic: "si-testtopic".into(),
        token: None,
    };
    let mut cap = counter("request");
    ntfy::send_with(&cfg, &alert(), &mut cap, |_| {}).unwrap();
    let request = rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(request.starts_with("POST /si-testtopic HTTP/1.1"));
    assert!(request.to_ascii_lowercase().contains("title: claude code"));
    assert!(request.to_ascii_lowercase().contains("priority: 4"));
    assert!(request.to_ascii_lowercase().contains("tags: warning"));
    assert!(request.ends_with("5-hour limit · resets 3:40 PM · review model usage"));
    handle.join().unwrap();
}

#[test]
fn daily_cap_enforced() {
    let mut cap = counter("cap");
    std::fs::write(
        cap.path(),
        format!("{{\"day\":{},\"count\":200}}", ntfy::utc_day()),
    )
    .unwrap();
    let cfg = NtfyConfig {
        server: "http://127.0.0.1:1".into(),
        topic: "si-testtopic".into(),
        token: None,
    };
    assert_eq!(
        ntfy::send_with(&cfg, &alert(), &mut cap, |_| {}),
        Err(NtfyError::DailyCap)
    );
    std::fs::remove_file(cap.path()).unwrap();
}

#[test]
fn retries_then_typed_error() {
    let (url, rx, handle) = server(503, 3);
    let cfg = NtfyConfig {
        server: url,
        topic: "si-testtopic".into(),
        token: None,
    };
    let mut cap = counter("retries");
    let mut waits = Vec::new();
    assert_eq!(
        ntfy::send_with(&cfg, &alert(), &mut cap, |d| waits.push(d)),
        Err(NtfyError::HttpStatus(503))
    );
    assert_eq!(waits, vec![Duration::from_secs(1), Duration::from_secs(3)]);
    for _ in 0..3 {
        assert!(rx.recv_timeout(Duration::from_secs(2)).is_ok());
    }
    handle.join().unwrap();
}
