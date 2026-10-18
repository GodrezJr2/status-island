//! Incremental JSONL reader. Agents append to these files while we read, so a
//! half-written last line is buffered until its newline arrives.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Upper bound per read so a huge append never stalls the snapshot loop.
const MAX_READ: u64 = 8 * 1024 * 1024;

pub struct Tail {
    path: PathBuf,
    pos: u64,
    buf: String,
}

impl Tail {
    pub fn from_start(path: &Path) -> Tail {
        Tail { path: path.to_path_buf(), pos: 0, buf: String::new() }
    }

    /// Starts `prime_bytes` before the end and returns the complete lines found there.
    pub fn from_end(path: &Path, prime_bytes: u64) -> io::Result<(Tail, Vec<serde_json::Value>)> {
        let size = std::fs::metadata(path)?.len();
        let start = size.saturating_sub(prime_bytes);
        let mut t = Tail { path: path.to_path_buf(), pos: start, buf: String::new() };
        let mut lines = t.read_new()?;
        // Starting mid-file means the first line is a fragment; parsing it may even succeed by accident.
        if start > 0 && !lines.is_empty() && !t.started_on_line_boundary(start)? {
            lines.remove(0);
        }
        Ok((t, lines))
    }

    pub fn read_new(&mut self) -> io::Result<Vec<serde_json::Value>> {
        let size = match std::fs::metadata(&self.path) {
            Ok(m) => m.len(),
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };
        if size < self.pos {
            self.pos = 0;
            self.buf.clear();
        }
        if size == self.pos {
            return Ok(Vec::new());
        }
        let len = (size - self.pos).min(MAX_READ);
        let mut f = File::open(&self.path)?;
        f.seek(SeekFrom::Start(self.pos))?;
        let mut bytes = vec![0u8; len as usize];
        f.read_exact(&mut bytes)?;
        self.pos += len;
        self.buf.push_str(&String::from_utf8_lossy(&bytes));

        let mut out = Vec::new();
        while let Some(i) = self.buf.find('\n') {
            let line: String = self.buf.drain(..=i).collect();
            if let Ok(v) = serde_json::from_str(line.trim_end()) {
                out.push(v);
            }
        }
        Ok(out)
    }

    fn started_on_line_boundary(&self, start: u64) -> io::Result<bool> {
        let mut f = File::open(&self.path)?;
        f.seek(SeekFrom::Start(start - 1))?;
        let mut b = [0u8; 1];
        f.read_exact(&mut b)?;
        Ok(b[0] == b'\n')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Unique temp folder per test (std only; tempfile pulls crates the GNU toolchain can't link).
    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("si-core-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn append(path: &std::path::Path, s: &str) {
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path).unwrap();
        f.write_all(s.as_bytes()).unwrap();
    }

    #[test]
    fn tail_buffers_partial_line() {
        let p = tmp("partial").join("t.jsonl");
        append(&p, "{\"a\":1}\n{\"b\":");
        let mut t = Tail::from_start(&p);
        let first = t.read_new().unwrap();
        assert_eq!(first, vec![serde_json::json!({"a":1})]);
        append(&p, "2}\n");
        assert_eq!(t.read_new().unwrap(), vec![serde_json::json!({"b":2})]);
    }

    #[test]
    fn tail_resets_on_truncate() {
        let p = tmp("truncate").join("t.jsonl");
        append(&p, "{\"a\":1}\n{\"a\":2}\n");
        let mut t = Tail::from_start(&p);
        assert_eq!(t.read_new().unwrap().len(), 2);
        std::fs::write(&p, "{\"c\":3}\n").unwrap();
        assert_eq!(t.read_new().unwrap(), vec![serde_json::json!({"c":3})]);
    }

    #[test]
    fn prime_reads_only_tail_bytes() {
        let p = tmp("big").join("big.jsonl");
        let mut f = std::fs::File::create(&p).unwrap();
        let mut n = 0u64;
        let mut written = 0u64;
        while written < 50_000_000 {
            let line = format!("{{\"i\":{n},\"pad\":\"{}\"}}\n", "x".repeat(200));
            f.write_all(line.as_bytes()).unwrap();
            written += line.len() as u64;
            n += 1;
        }
        drop(f);
        let (mut t, lines) = Tail::from_end(&p, 2_000_000).unwrap();
        let line_len = written / n;
        assert!((lines.len() as u64) * line_len <= 2_000_000, "read more than the primed bytes");
        assert!(!lines.is_empty());
        assert_eq!(lines.last().unwrap()["i"], serde_json::json!(n - 1));
        assert!(t.read_new().unwrap().is_empty(), "nothing new after priming");
    }
}
