use serde::{Deserialize, Serialize};
use si_core::alerts::{Alert, Level};
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use windows::Win32::Security::Cryptography::{BCryptGenRandom, BCRYPT_USE_SYSTEM_PREFERRED_RNG};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NtfyConfig {
    pub server: String,
    pub topic: String,
    pub token: Option<String>,
}
impl Default for NtfyConfig {
    fn default() -> Self {
        Self {
            server: "https://ntfy.sh".into(),
            topic: String::new(),
            token: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NtfyError {
    InvalidConfig,
    DailyCap,
    HttpStatus(u16),
    Transport,
    Storage,
}

#[derive(Clone, Debug)]
pub struct DailyCounter {
    path: PathBuf,
}
impl DailyCounter {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    fn reserve(&mut self) -> Result<(), NtfyError> {
        let day = utc_day();
        let current: CounterFile = fs::read(&self.path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or(CounterFile { day, count: 0 });
        let count = if current.day == day { current.count } else { 0 };
        if count >= 200 {
            return Err(NtfyError::DailyCap);
        }
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|_| NtfyError::Storage)?;
        }
        let bytes = serde_json::to_vec(&CounterFile {
            day,
            count: count + 1,
        })
        .map_err(|_| NtfyError::Storage)?;
        fs::write(&self.path, bytes).map_err(|_| NtfyError::Storage)
    }
}

#[derive(Serialize, Deserialize)]
struct CounterFile {
    day: u64,
    count: u32,
}

pub fn utc_day() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0)
}

pub fn new_topic() -> String {
    let mut random = [0u8; 24];
    let result = unsafe { BCryptGenRandom(None, &mut random, BCRYPT_USE_SYSTEM_PREFERRED_RNG) };
    assert!(result.is_ok(), "system random source unavailable");
    let alphabet = b"abcdefghijklmnopqrstuvwxyz234567";
    let mut topic = String::from("si-");
    for byte in random {
        topic.push(alphabet[(byte & 31) as usize] as char);
    }
    topic
}

pub fn send(cfg: &NtfyConfig, alert: &Alert) -> Result<(), NtfyError> {
    let local = std::env::var_os("LOCALAPPDATA").ok_or(NtfyError::Storage)?;
    let mut counter = DailyCounter::new(PathBuf::from(local).join("StatusIsland/ntfy-count.json"));
    send_with(cfg, alert, &mut counter, thread::sleep)
}

pub fn send_with(
    cfg: &NtfyConfig,
    alert: &Alert,
    counter: &mut DailyCounter,
    mut sleep: impl FnMut(Duration),
) -> Result<(), NtfyError> {
    if !(cfg.server.starts_with("https://") || cfg.server.starts_with("http://"))
        || cfg.topic.is_empty()
        || !cfg
            .topic
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(NtfyError::InvalidConfig);
    }
    let url = format!("{}/{}", cfg.server.trim_end_matches('/'), cfg.topic);
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build()
        .into();
    let tag = match alert.level {
        Level::Warn => "warning",
        Level::Crit => "rotating_light",
        Level::Info => "information_source",
    };
    let mut last_error = NtfyError::Transport;
    for attempt in 0..3 {
        counter.reserve()?;
        let mut request = agent
            .post(&url)
            .header("Title", &alert.title)
            .header("Priority", alert.priority.to_string())
            .header("Tags", tag);
        if let Some(token) = &cfg.token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }
        match request.send(alert.body.as_str()) {
            Ok(_) => return Ok(()),
            Err(ureq::Error::StatusCode(code)) => last_error = NtfyError::HttpStatus(code),
            Err(_) => last_error = NtfyError::Transport,
        }
        if attempt < 2 {
            sleep(Duration::from_secs(if attempt == 0 { 1 } else { 3 }));
        }
    }
    Err(last_error)
}
