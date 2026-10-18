use crate::engine::Signal;
use serde_json::Value;
use si_core::scan::ExternalReading;
use std::sync::mpsc::Sender;
use std::{fs::OpenOptions, io::Write};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{CloseHandle, ERROR_PIPE_CONNECTED},
        Storage::FileSystem::{ReadFile, PIPE_ACCESS_INBOUND},
        System::Pipes::{
            ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_BYTE,
            PIPE_TYPE_BYTE, PIPE_WAIT,
        },
    },
};

pub fn name() -> String {
    let user = std::env::var("USERNAME").unwrap_or_else(|_| "unknown".into());
    let safe: String = user
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    format!(r"\\.\pipe\status-island-{safe}")
}

pub fn send(kind: &str, payload: &Value) {
    let message = serde_json::json!({"kind":kind,"payload":payload});
    if let Ok(mut pipe) = OpenOptions::new().write(true).open(name()) {
        let _ = writeln!(pipe, "{message}");
    }
}

pub fn serve(tx: Sender<Signal>) {
    std::thread::spawn(move || {
        let wide: Vec<u16> = name().encode_utf16().chain(std::iter::once(0)).collect();
        loop {
            let pipe = unsafe {
                CreateNamedPipeW(
                    PCWSTR(wide.as_ptr()),
                    PIPE_ACCESS_INBOUND,
                    PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                    1,
                    0,
                    65536,
                    50,
                    None,
                )
            };
            if pipe.is_invalid() {
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
            let connected = match unsafe { ConnectNamedPipe(pipe, None) } {
                Ok(()) => true,
                Err(e) => e.code().0 as u32 == (0x8007_0000 | ERROR_PIPE_CONNECTED.0),
            };
            if connected {
                let mut pending = Vec::new();
                let mut buf = [0u8; 8192];
                loop {
                    let mut count = 0;
                    if unsafe { ReadFile(pipe, Some(&mut buf), Some(&mut count), None) }.is_err()
                        || count == 0
                    {
                        break;
                    }
                    for &byte in &buf[..count as usize] {
                        if byte == b'\n' {
                            if let Ok(v) = serde_json::from_slice::<Value>(&pending) {
                                let payload = v.get("payload").cloned();
                                let external =
                                    match (v.get("kind").and_then(Value::as_str), payload) {
                                        (Some("statusline"), Some(p)) => {
                                            Some(ExternalReading::StatusLine(p))
                                        }
                                        (Some("rate_limit_event"), Some(p)) => {
                                            Some(ExternalReading::RateLimitEvent(p))
                                        }
                                        _ => None,
                                    };
                                if let Some(event) = external {
                                    let _ = tx.send(Signal::External(event));
                                }
                            }
                            pending.clear();
                        } else if pending.len() < (1 << 20) {
                            pending.push(byte);
                        } else {
                            pending.clear();
                            break;
                        }
                    }
                }
            }
            unsafe {
                let _ = DisconnectNamedPipe(pipe);
                let _ = CloseHandle(pipe);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipe_receives_statusline() {
        let (tx, rx) = std::sync::mpsc::channel();
        serve(tx);
        for _ in 0..50 {
            send(
                "statusline",
                &serde_json::json!({"rate_limits":{"five_hour":{"used_percentage":12,"resets_at":1792329000}}}),
            );
            if let Ok(Signal::External(ExternalReading::StatusLine(v))) =
                rx.recv_timeout(std::time::Duration::from_millis(20))
            {
                assert_eq!(v["rate_limits"]["five_hour"]["used_percentage"], 12);
                return;
            }
        }
        panic!("pipe did not deliver a statusline reading");
    }
}
