use crate::{
    claude,
    codex::{self, CodexEvent},
    live::LimitTracker,
    model::{AccountLimits, Activity, AgentKind, SessionView, Snapshot, Spend, Surface},
    opencode,
    tail::Tail,
};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc,
    time::Duration,
};

#[derive(Clone, Debug, Default)]
pub struct Roots {
    pub claude: Vec<PathBuf>,
    pub codex: Vec<PathBuf>,
    pub opencode_db: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub enum ExternalReading {
    StatusLine(Value),
    RateLimitEvent(Value),
}

#[derive(Default)]
struct CodexState {
    cwd: String,
    branch: Option<String>,
    model: String,
    last_event_ms: i64,
    open_task: bool,
    input: u64,
    window: u64,
    token_events: Vec<(i64, u64)>,
    last_total: u64,
}

/// Decides whether a Claude session's process is still the same live process.
/// `proc_start` is the registry's `procStart` (Windows process-creation FILETIME),
/// which guards against a reused PID. The app supplies a Win32 implementation.
pub trait ProcessProbe: Send {
    fn is_live(&self, pid: u32, proc_start: Option<&str>) -> bool;
}

/// Fallback used by `Scanner::new` (tests, non-app callers). It cannot check `procStart`.
struct TasklistProbe;

impl ProcessProbe for TasklistProbe {
    fn is_live(&self, pid: u32, _proc_start: Option<&str>) -> bool {
        pid_alive(pid)
    }
}

pub struct Scanner {
    probe: Box<dyn ProcessProbe>,
    roots: Roots,
    tails: HashMap<PathBuf, Tail>,
    codex_states: HashMap<PathBuf, CodexState>,
    claude_turns: HashMap<String, claude::ClaudeTurn>,
    claude_token_events: HashMap<String, Vec<(i64, u64)>>,
    claude_ctx_window: Option<u64>,
    claude_five: LimitTracker,
    claude_week: LimitTracker,
    codex_five: LimitTracker,
    codex_week: LimitTracker,
    claude_overage: bool,
}

pub fn default_roots(home: &Path) -> Roots {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(hidden(Command::new("wsl.exe").args(["-l", "-q"])).output());
    });
    let distros = rx
        .recv_timeout(Duration::from_millis(500))
        .ok()
        .and_then(Result::ok)
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .replace('\0', "")
                .lines()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    default_roots_with_probe(home, distros, |p| p.exists())
}

pub fn default_roots_with_probe<F>(home: &Path, distros: Vec<String>, probe: F) -> Roots
where
    F: Fn(PathBuf) -> bool + Send + Sync + 'static,
{
    let mut roots = Roots {
        claude: vec![home.join(".claude")],
        codex: vec![home.join(".codex")],
        opencode_db: vec![home.join(".local/share/opencode/opencode.db")],
    };
    let user = home
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or_default()
        .to_owned();
    let probe = std::sync::Arc::new(probe);
    for distro in distros {
        let base = PathBuf::from(format!(r"\\wsl.localhost\{}\home\{}", distro, user));
        let (tx, rx) = mpsc::channel();
        let p = std::sync::Arc::clone(&probe);
        let candidate = base.clone();
        std::thread::spawn(move || {
            let _ = tx.send(p(candidate));
        });
        if rx.recv_timeout(Duration::from_millis(500)) == Ok(true) {
            roots.claude.push(base.join(".claude"));
            roots.codex.push(base.join(".codex"));
            roots
                .opencode_db
                .push(base.join(".local/share/opencode/opencode.db"));
        }
    }
    roots
}

impl Scanner {
    pub fn new(roots: Roots) -> Self {
        Self::with_probe(roots, Box::new(TasklistProbe))
    }

    pub fn with_probe(roots: Roots, probe: Box<dyn ProcessProbe>) -> Self {
        Self {
            probe,
            roots,
            tails: HashMap::new(),
            codex_states: HashMap::new(),
            claude_turns: HashMap::new(),
            claude_token_events: HashMap::new(),
            claude_ctx_window: None,
            claude_five: LimitTracker::default(),
            claude_week: LimitTracker::default(),
            codex_five: LimitTracker::default(),
            codex_week: LimitTracker::default(),
            claude_overage: false,
        }
    }

    pub fn ingest_external(&mut self, r: ExternalReading) {
        let (five, week, overage) = match r {
            ExternalReading::StatusLine(v) => {
                let (a, b, window) = claude::parse_statusline(&v);
                self.claude_ctx_window = window.or(self.claude_ctx_window);
                (a, b, false)
            }
            ExternalReading::RateLimitEvent(v) => claude::parse_rate_limit_event(&v),
        };
        if let Some(r) = five {
            self.claude_five.ingest(r);
        }
        if let Some(r) = week {
            self.claude_week.ingest(r);
        }
        self.claude_overage |= overage;
    }

    pub fn tick(&mut self, now_ms: i64) -> Snapshot {
        let mut sessions = Vec::new();
        for root in self.roots.claude.clone() {
            self.scan_claude(&root, now_ms, &mut sessions);
        }
        for root in self.roots.codex.clone() {
            self.scan_codex(&root, now_ms, &mut sessions);
        }
        let mut opencode_today = None;
        for db in &self.roots.opencode_db {
            if let Ok((found, today)) = opencode::read_opencode(db) {
                let total = opencode_today.get_or_insert(Spend {
                    session_usd: 0.0,
                    today_usd: 0.0,
                    session_tokens: 0,
                });
                total.today_usd += today.today_usd;
                total.session_tokens += today.session_tokens;
                for s in found {
                    if now_ms - s.updated_ms > 15 * 60_000 {
                        continue;
                    }
                    sessions.push(SessionView {
                        id: s.id,
                        kind: AgentKind::OpenCode,
                        model_label: s
                            .model
                            .as_deref()
                            .map(crate::model::model_label)
                            .unwrap_or_else(|| "—".into()),
                        project: project_name(Path::new(&s.directory)),
                        branch: None,
                        surface: Surface::Terminal,
                        activity: if now_ms - s.updated_ms < 30_000
                            && s.idle_ms.is_none_or(|idle| idle < s.updated_ms)
                        {
                            Activity::Working
                        } else {
                            Activity::Idle
                        },
                        last_event_ms: s.updated_ms,
                        ctx_tokens: None,
                        ctx_pct: None,
                        tokens_per_min: 0,
                        spend: Some(Spend {
                            session_usd: s.cost,
                            today_usd: today.today_usd,
                            session_tokens: s.tokens,
                        }),
                    });
                }
            }
        }
        Snapshot {
            now_ms,
            sessions,
            claude: AccountLimits {
                five_hour: self.claude_five.view(now_ms, None),
                weekly: self.claude_week.view(now_ms, None),
                overage_disabled: self.claude_overage,
            },
            codex: AccountLimits {
                five_hour: self.codex_five.view(now_ms, None),
                weekly: self.codex_week.view(now_ms, None),
                overage_disabled: false,
            },
            opencode_today,
        }
    }

    fn scan_claude(&mut self, root: &Path, now_ms: i64, out: &mut Vec<SessionView>) {
        let Ok(files) = fs::read_dir(root.join("sessions")) else {
            return;
        };
        for entry in files.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let Ok(bytes) = fs::read(&path) else {
                continue;
            };
            let Ok(v) = serde_json::from_slice::<Value>(&bytes) else {
                continue;
            };
            let Some(s) = claude::parse_session_file(&v) else {
                continue;
            };
            if !self.probe.is_live(s.pid, s.proc_start.as_deref()) {
                continue;
            }
            let mut latest = self.claude_turns.get(&s.session_id).cloned();
            if let Ok(projects) = fs::read_dir(root.join("projects")) {
                for project in projects.flatten() {
                    let transcript = project.path().join(format!("{}.jsonl", s.session_id));
                    if !transcript.is_file() {
                        continue;
                    }
                    for v in self.read_tail(&transcript) {
                        if let Some(turn) = claude::parse_transcript_line(&v) {
                            self.claude_token_events
                                .entry(s.session_id.clone())
                                .or_default()
                                .push((turn.ts_ms, turn.output_tokens));
                            if latest
                                .as_ref()
                                .is_none_or(|old: &claude::ClaudeTurn| turn.ts_ms >= old.ts_ms)
                            {
                                latest = Some(turn);
                            }
                        }
                    }
                    break;
                }
            }
            let last_ms = v["updatedAt"]
                .as_i64()
                .unwrap_or_else(|| latest.as_ref().map(|x| x.ts_ms).unwrap_or(0));
            if !s.busy && now_ms - last_ms > 15 * 60_000 {
                continue;
            }
            let ctx = latest.as_ref().map(|x| x.ctx_tokens);
            if let Some(turn) = latest.clone() {
                self.claude_turns.insert(s.session_id.clone(), turn);
            }
            let model = latest
                .as_ref()
                .map(|x| crate::model::model_label(&x.model))
                .unwrap_or_else(|| "—".into());
            let token_events = self
                .claude_token_events
                .entry(s.session_id.clone())
                .or_default();
            token_events.retain(|(ts, _)| now_ms - *ts <= 5 * 60_000);
            let tokens_per_min = token_events.iter().map(|(_, n)| *n).sum::<u64>() / 5;
            let ctx_pct = ctx.zip(self.claude_ctx_window).and_then(|(used, total)| {
                (total > 0).then(|| ((used as f64 / total as f64 * 100.0).round() as u8).min(100))
            });
            out.push(SessionView {
                id: s.session_id,
                kind: AgentKind::Claude,
                model_label: model,
                project: project_name(Path::new(&s.cwd)),
                branch: latest.and_then(|x| x.branch),
                surface: s.surface,
                activity: if s.busy {
                    Activity::Working
                } else {
                    Activity::Idle
                },
                last_event_ms: last_ms,
                ctx_tokens: ctx,
                ctx_pct,
                tokens_per_min,
                spend: None,
            });
        }
    }

    fn scan_codex(&mut self, root: &Path, now_ms: i64, out: &mut Vec<SessionView>) {
        let mut files = Vec::new();
        collect_jsonl(&root.join("sessions"), 4, &mut files);
        for path in files {
            let had_tail = self.tails.contains_key(&path);
            let values = self.read_tail(&path);
            let grew = had_tail && !values.is_empty();
            let state = self.codex_states.entry(path.clone()).or_default();
            for v in values {
                for event in codex::parse_rollout_line(&v) {
                    match event {
                        CodexEvent::Meta { cwd, branch } => {
                            state.cwd = cwd;
                            state.branch = branch;
                        }
                        CodexEvent::Model(model) => state.model = model,
                        CodexEvent::TaskStarted(ts) => {
                            state.open_task = true;
                            state.last_event_ms = state.last_event_ms.max(ts);
                        }
                        CodexEvent::TaskComplete(ts) => {
                            state.open_task = false;
                            state.last_event_ms = state.last_event_ms.max(ts);
                        }
                        CodexEvent::Tokens {
                            ts_ms,
                            input,
                            window,
                            total,
                        } => {
                            state.input = input;
                            state.window = window;
                            state
                                .token_events
                                .push((ts_ms, total.saturating_sub(state.last_total)));
                            state.last_total = total;
                            state.last_event_ms = state.last_event_ms.max(ts_ms);
                        }
                        CodexEvent::Limits {
                            ts_ms,
                            primary,
                            secondary,
                        } => {
                            self.codex_five.ingest(primary);
                            if let Some(r) = secondary {
                                self.codex_week.ingest(r);
                            }
                            state.last_event_ms = state.last_event_ms.max(ts_ms);
                        }
                    }
                }
            }
            if grew {
                state.last_event_ms = now_ms;
            }
            if now_ms - state.last_event_ms > 15 * 60_000 {
                continue;
            }
            let activity = if now_ms - state.last_event_ms < 20_000
                || (state.open_task && now_ms - state.last_event_ms < 5 * 60_000)
            {
                Activity::Working
            } else {
                Activity::Idle
            };
            let ctx_pct = (state.window > 0).then(|| {
                ((state.input as f64 / state.window as f64 * 100.0).round() as u8).min(100)
            });
            state
                .token_events
                .retain(|(ts, _)| now_ms - *ts <= 5 * 60_000);
            let tokens_per_min = state.token_events.iter().map(|(_, n)| *n).sum::<u64>() / 5;
            out.push(SessionView {
                id: path
                    .file_stem()
                    .and_then(|x| x.to_str())
                    .unwrap_or_default()
                    .to_owned(),
                kind: AgentKind::Codex,
                model_label: crate::model::model_label(&state.model),
                project: project_name(Path::new(&state.cwd)),
                branch: state.branch.clone(),
                surface: Surface::Unknown,
                activity,
                last_event_ms: state.last_event_ms,
                ctx_tokens: Some(state.input),
                ctx_pct,
                tokens_per_min,
                spend: None,
            });
        }
    }

    fn read_tail(&mut self, path: &Path) -> Vec<Value> {
        if let Some(t) = self.tails.get_mut(path) {
            return t.read_new().unwrap_or_default();
        }
        match Tail::from_end(path, 2_000_000) {
            Ok((tail, values)) => {
                self.tails.insert(path.to_owned(), tail);
                values
            }
            Err(_) => Vec::new(),
        }
    }
}

fn collect_jsonl(dir: &Path, depth: u8, out: &mut Vec<PathBuf>) {
    if depth == 0 {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_jsonl(&p, depth - 1, out);
            } else if p.extension().and_then(|x| x.to_str()) == Some("jsonl") {
                out.push(p);
            }
        }
    }
}

fn project_name(path: &Path) -> String {
    path.file_name()
        .and_then(|x| x.to_str())
        .unwrap_or_default()
        .to_owned()
}

/// Child processes from a GUI app would otherwise flash a console window.
fn hidden(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd
}

fn pid_alive(pid: u32) -> bool {
    if pid == std::process::id() {
        return true;
    }
    if pid == 0 || pid == u32::MAX {
        return false;
    }
    hidden(Command::new("tasklist").args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"]))
        .output()
        .ok()
        .filter(|o| o.status.success())
        .is_some_and(|o| {
            String::from_utf8_lossy(&o.stdout).lines().any(|line| {
                line.split(',')
                    .nth(1)
                    .is_some_and(|n| n.trim_matches('"') == pid.to_string())
            })
        })
}
