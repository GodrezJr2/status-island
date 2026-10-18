use std::{
    ffi::OsString,
    io::{Read, Write},
    os::windows::{io::AsRawHandle, process::CommandExt},
    process::{Command, Stdio},
    thread,
};
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::{
        Environment::GetCommandLineW,
        JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
            SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        },
    },
};

pub fn run(args: impl Iterator<Item = OsString>, skip_raw_tokens: usize) -> i32 {
    let mut args = args;
    let Some(target) = args.next() else {
        return 2;
    };
    let mut command = Command::new(target);
    let raw = unsafe { GetCommandLineW().to_string().ok() }
        .and_then(|s| raw_after_tokens(&s, skip_raw_tokens));
    if let Some(raw) = raw {
        if !raw.is_empty() {
            command.raw_arg(raw);
        }
    } else {
        command.args(args);
    }
    let mut child = match command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return 1,
    };
    let job = Job::attach(&child);
    let Some(mut child_stdin) = child.stdin.take() else {
        return 1;
    };
    let Some(mut child_stdout) = child.stdout.take() else {
        return 1;
    };
    let Some(mut child_stderr) = child.stderr.take() else {
        return 1;
    };
    let _input = thread::spawn(move || {
        let _ = std::io::copy(&mut std::io::stdin(), &mut child_stdin);
    });
    let output = thread::spawn(move || {
        let mut stdout = std::io::stdout().lock();
        let mut buffer = [0u8; 65536];
        let mut tap = LineTap::default();
        loop {
            match child_stdout.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if stdout.write_all(&buffer[..n]).is_err() {
                        break;
                    }
                    let _ = stdout.flush();
                    tap.feed(&buffer[..n], &mut |v| {
                        crate::pipe::send("rate_limit_event", v)
                    });
                }
            }
        }
    });
    let errors = thread::spawn(move || {
        let _ = std::io::copy(&mut child_stderr, &mut std::io::stderr());
    });
    let status = child.wait();
    let _ = output.join();
    let _ = errors.join();
    drop(job);
    status.ok().and_then(|s| s.code()).unwrap_or(1)
}

fn raw_after_tokens(command: &str, count: usize) -> Option<String> {
    let bytes = command.as_bytes();
    let mut i = 0;
    for _ in 0..count {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i == bytes.len() {
            return None;
        }
        let mut quoted = false;
        while i < bytes.len() {
            if bytes[i] == b'\\' {
                let start = i;
                while i < bytes.len() && bytes[i] == b'\\' {
                    i += 1;
                }
                if i < bytes.len() && bytes[i] == b'"' {
                    if (i - start) % 2 == 0 {
                        quoted = !quoted;
                    }
                    i += 1;
                }
                continue;
            }
            if bytes[i] == b'"' {
                quoted = !quoted;
                i += 1;
                continue;
            }
            if !quoted && bytes[i].is_ascii_whitespace() {
                break;
            }
            i += 1;
        }
    }
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    Some(command[i..].to_owned())
}

#[derive(Default)]
struct LineTap {
    line: Vec<u8>,
    overflow: bool,
}
impl LineTap {
    fn feed(&mut self, data: &[u8], callback: &mut impl FnMut(&serde_json::Value)) {
        for &byte in data {
            if byte == b'\n' {
                if !self.overflow {
                    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&self.line) {
                        if v.get("type").and_then(serde_json::Value::as_str)
                            == Some("rate_limit_event")
                        {
                            callback(&v);
                        }
                    }
                }
                self.line.clear();
                self.overflow = false;
            } else if !self.overflow {
                if self.line.len() >= 1 << 20 {
                    self.line.clear();
                    self.overflow = true;
                } else {
                    self.line.push(byte);
                }
            }
        }
    }
}

struct Job(HANDLE);
impl Job {
    fn attach(child: &std::process::Child) -> Option<Self> {
        unsafe {
            let handle = CreateJobObjectW(None, None).ok()?;
            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let set = SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const std::ffi::c_void,
                std::mem::size_of_val(&info) as u32,
            );
            let assigned =
                set.and_then(|_| AssignProcessToJobObject(handle, HANDLE(child.as_raw_handle())));
            if assigned.is_err() {
                let _ = CloseHandle(handle);
                None
            } else {
                Some(Self(handle))
            }
        }
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_forwards_only_limit_lines() {
        let mut tap = LineTap::default();
        let mut seen = Vec::new();
        tap.feed(b"{\"type\":\"assistant\",\"content\":\"private\"}\n{\"type\":\"rate_limit_event\",\"rate_limit_info\":{}}\n", &mut |v| seen.push(v.clone()));
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0]["type"], "rate_limit_event");
        tap.feed(&vec![b'x'; (1 << 20) + 1], &mut |v| seen.push(v.clone()));
        tap.feed(b"\n", &mut |v| seen.push(v.clone()));
        assert_eq!(seen.len(), 1);
    }

    #[test]
    fn raw_tail_keeps_quotes_and_unicode() {
        assert_eq!(
            raw_after_tokens(r#""C:\Program Files\si-wrap.exe" "C:\Program Files\claude.exe" "a \"quote\"" "" বাংলা"#, 2).as_deref(),
            Some(r#""a \"quote\"" "" বাংলা"#)
        );
    }
}
