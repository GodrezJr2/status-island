# Status Island v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship Status Island v1, a Windows Dynamic-Island overlay that shows live limits, burn rate and context for Claude Code, Codex and OpenCode, with phone push, one-click setup and no admin rights.

**Architecture:**
- A Cargo workspace with a pure-Rust `si-core` crate: models, parsers, the live limit model, the estimator and alerts. It is fully unit-tested with no OS dependencies.
- A Tauri 2 app (`app/src-tauri`) owns everything that touches Windows: file watching, the named pipe, the overlay windows, Win32 signals, ntfy, and settings edits.
- The same exe also runs headless in `--wrap` mode (VS Code panel wrapper) and `--statusline` mode (Claude status line add-on).
- The front end (`app/src`) is React + Motion. It renders one island per monitor from a `Snapshot` the core pushes once a second and on every change.

**Tech Stack:** Rust stable (Tauri 2.x, `notify`, `rusqlite` with `bundled`, `serde`/`serde_json`, `windows` crate, `ureq` or `reqwest` blocking with rustls), React 18 + TypeScript + Vite, `motion` 12, `@number-flow/react`, Vitest, the Geist font (bundled).

**Spec:** `docs/superpowers/specs/2026-10-18-status-island-design.md`. Visual source of truth: https://www.figma.com/design/wSnY22zFnUATHqd2eWaUhh

## Global Constraints

**Safety rules (spec §3, all hard)**
- Never open `.credentials.json`, `*.key`, `~/.claude/ide/*.lock`, `~/.claude.json`, browser stores, `~/.codex/auth.json`, or OpenCode `account` / `control_account` / `credential` tables.
- Never make network requests with a user's account. The only outbound traffic is ntfy push and the update check.
- No proxies or relays.
- Every file the app edits is restored on uninstall or when its connector is switched off.

**Platform**
- Windows 10 and 11, x64 and ARM64, per-monitor DPI aware.
- No admin rights anywhere: NSIS `installMode: currentUser`, files only inside the user profile, per-user named pipe.
- Unsigned at launch.

**Licence and font**
- MIT licence. Do not copy GPL code (vibe-island is GPL-3.0).
- Geist is bundled (OFL). Numbers are `tabular-nums` with fixed-width slots.

**No fake numbers.** Every value is exact or labelled as an estimate with its age. Freshness: Exact ≤ 60 s → Estimated → Stale (no reading for 30 min) → Unknown.

**Opacity (owner's Figma)**
- Idle: 0.63.
- Compact/stacked: 0.95.
- Hover see-through: 0.36.
- Open dropdown: 1.0.
- Dropdown opens after 0.9 s of hover. A mouse press closes it and keeps it closed until the pointer leaves.
- The window is click-through at all times. No global keyboard or mouse hooks.

**Sizes**
- Idle 132×30.
- Compact 252×34, plus one 26 px row per extra working agent (max 3, then "+N more"), plus 6 px bottom padding. The island grows downward only.
- Dropdown 400 wide, radius `min(h/2, 26)`.
- Capsule radius = h/2.
- 6–8 px below the monitor's top edge.

**Colours (Figma tokens)**
- `island/fill` #000; `island/keyline` white 10%; `surface/card` white 6%; `surface/track` white 16%.
- `text/primary` #F5F5F7; `text/secondary` #8E8E93.
- `status/warn` #FFB020; `status/crit` #FF453A; `status/live` #30D158.
- `agent/claude` #E8835E; `agent/codex` #8AB4F8; `agent/opencode` #5EE09A.

**Alerts**
- Each alert glows in its own colour: drop shadow at 55% alpha, blur 14, no offset, height stays 34 px.
- The open dropdown casts a shadow: black 65%, blur about 8, small offset.
- Metric line detail text is 12 pt Medium. Freshness is shown on the card's location line.

**Commits:** the owner writes commit messages, and there is no AI attribution trailer. At each "Commit" step, stage the files and propose a one-line message for the owner to approve.

**Performance:** under 1% CPU at idle and about 150 MB RAM, all WebView2 processes summed.

## Review Focus

1. **VS Code `settings.json` with comments, trailing commas or odd formatting.** Editing it must change only our one key and leave every other byte intact. A corrupted user settings file is the worst possible failure. → Task 8 test `jsonc_edit_preserves_comments_and_other_keys`.
2. **A half-written last line in a JSONL file** (the agent is mid-write). The tail must buffer it and parse it on the next read, without dropping or crashing. → Task 2 test `tail_buffers_partial_line`.
3. **Huge transcripts** (100 MB+ Claude sessions). The first read primes from the last 2 MB only, never the whole file. → Task 2 test `prime_reads_only_tail_bytes`.
4. **A WSL distro that is stopped or slow.** Probing `\\wsl.localhost\...` must time out (≤ 500 ms) and never block the snapshot loop. → Task 6 test `wsl_probe_times_out`.
5. **A monitor unplugged, DPI changed, or a top taskbar.** The island must re-place itself on a visible top-centre and never end up off-screen. → Task 10 test `placement_handles_monitor_removal_and_scale`.

---

### Task 0: Toolchain without admin

**Files:**
- Create: `docs/dev-setup.md`

- [ ] **Step 1: Install Rust per user.** Run `rustup-init.exe -y --default-host x86_64-pc-windows-msvc`. Then check whether the MSVC Build Tools can install without admin.
  - If they can't, switch to `x86_64-pc-windows-gnu` (rustup's self-contained mingw linker, no admin).
  - Record which toolchain worked.
- [ ] **Step 2: Fix certificates on the company network.** npm needs `NODE_OPTIONS=--use-system-ca`. For cargo, set `CARGO_HTTP_CHECK_REVOKE=false` only if revocation checks fail, never by disabling TLS. Record what was needed.
- [ ] **Step 3: Confirm the toolchain builds.** Run `cargo new --bin hello && cargo run` in a scratch folder. Expected: `Hello, world!`. If local builds are impossible, write that down: development continues with GitHub Actions `windows-latest` builds from Task 13 onward.
- [ ] **Step 4: Write `docs/dev-setup.md`** with the exact commands that worked.

### Task 1: Repo, workspace and core models

**Files:**
- Create: `.gitignore` (ignore `spike-demo/`, `.spike-wrapper/`, `reports/`, `research_notes/`, `.agent/`, `target/`, `node_modules/`, `.claude/settings.local.json`)
- Create: `Cargo.toml` (workspace members `crates/si-core`, `app/src-tauri`)
- Create: `LICENSE` (MIT, © 2026 Sheikh Mohammad Hossain)
- Create: `crates/si-core/src/lib.rs`, `crates/si-core/src/model.rs`
- Test: `crates/si-core/src/model.rs` (`#[cfg(test)]`)

**Interfaces:**
- Produces (in `si_core::model`):
  - `enum AgentKind { Claude, Codex, OpenCode }`
  - `enum Surface { Terminal, VsCode, Unknown }`
  - `enum Activity { Working, Waiting, Idle }`
  - `enum WindowKind { FiveHour, Weekly }`
  - `enum Source { StatusLine, Wrapper, CodexRollout }`
  - `struct Reading { used_pct: f64, resets_at_ms: i64, at_ms: i64, source: Source }`
  - `enum Freshness { Exact, Estimated, Stale, Unknown }`
  - `struct WindowView { used_pct: f64, left_pct: f64, resets_at_ms: Option<i64>, freshness: Freshness, age_ms: i64, burn_pts_per_min: f64, minutes_to_empty: Option<f64>, used_elsewhere_pts: Option<f64>, estimated: bool }`
  - `struct SessionView { id: String, kind: AgentKind, model_label: String, project: String, branch: Option<String>, surface: Surface, activity: Activity, last_event_ms: i64, ctx_tokens: Option<u64>, ctx_pct: Option<u8>, tokens_per_min: u64, spend: Option<Spend> }`
  - `struct Spend { session_usd: f64, today_usd: f64, session_tokens: u64 }`
  - `struct AccountLimits { five_hour: Option<WindowView>, weekly: Option<WindowView>, overage_disabled: bool }`
  - `struct Snapshot { now_ms: i64, sessions: Vec<SessionView>, claude: AccountLimits, codex: AccountLimits, opencode_today: Option<Spend> }`
  - `fn model_label(id: &str) -> String`
  - All types `#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]`, serde `rename_all = "camelCase"`.

- [ ] **Step 1: Run `git init`**, and add `.gitignore`, `LICENSE` and the workspace `Cargo.toml`.
- [ ] **Step 2: Write the failing test** `model_label_formats_known_ids`:
  - `model_label("claude-opus-5-5") == "Opus 5.5"`
  - `model_label("claude-sonnet-5-5") == "Sonnet 5.5"`
  - `model_label("gpt-6-sol") == "GPT 6 sol"`
  - `model_label("deepseek-flash") == "DeepSeek Flash"`
  - `model_label("") == "—"`
- [ ] **Step 3: Run** `cargo test -p si-core model_label`. Expected: FAIL (not defined).
- [ ] **Step 4: Implement the types and `model_label`** in `model.rs`.
- [ ] **Step 5: Run** `cargo test -p si-core`. Expected: PASS.
- [ ] **Step 6: Commit** (stage, then propose: "Set up workspace and core models").

### Task 2: Incremental JSONL tail

**Files:**
- Create: `crates/si-core/src/tail.rs`

**Interfaces:**
- Produces:
  - `struct Tail { path: PathBuf, pos: u64, buf: String }`
  - `Tail::from_end(path, prime_bytes: u64) -> io::Result<(Tail, Vec<serde_json::Value>)>`, which returns the parsed lines from the last `prime_bytes`, dropping the first partial line when it starts mid-file.
  - `Tail::from_start(path) -> Tail`
  - `Tail::read_new(&mut self) -> io::Result<Vec<serde_json::Value>>`, which handles truncation by resetting to 0.
- Constraint: never read more than 8 MB per call.

- [ ] **Step 1: Write the failing tests:**
  - `tail_buffers_partial_line`: write `{"a":1}\n{"b":` → `read_new` returns 1 value. Append `2}\n` → `read_new` returns `[{"b":2}]`.
  - `tail_resets_on_truncate`.
  - `prime_reads_only_tail_bytes`: on a 50 MB file of lines, `from_end(path, 2_000_000)` returns ≤ 2 MB worth of lines, and the last value equals the file's last line.
- [ ] **Step 2: Run** `cargo test -p si-core tail`. Expected: FAIL.
- [ ] **Step 3: Implement `tail.rs`.**
- [ ] **Step 4: Run** `cargo test -p si-core tail`. Expected: PASS.
- [ ] **Step 5: Commit** ("Add incremental JSONL tail").

### Task 3: Agent parsers (Claude, Codex, OpenCode)

**Files:**
- Create: `crates/si-core/src/claude.rs`, `crates/si-core/src/codex.rs`, `crates/si-core/src/opencode.rs`
- Create: `crates/si-core/tests/fixtures/*.jsonl`. Use the exact field shapes from spec §4, numbers anonymised, **no prompt text**.

**Interfaces:**
- Produces in `claude`:
  - `struct ClaudeTurn { ts_ms: i64, model: String, ctx_tokens: u64, output_tokens: u64, cwd: Option<String>, branch: Option<String> }`
  - `fn parse_transcript_line(v: &Value) -> Option<ClaudeTurn>`. Only `type == "assistant"` lines with `message.usage`. `ctx_tokens = input + cache_creation_input + cache_read_input`.
  - `struct LiveSession { pid: u32, session_id: String, cwd: String, surface: Surface, busy: bool, proc_start: Option<String> }`
  - `fn parse_session_file(v: &Value) -> Option<LiveSession>`. `entrypoint` `claude-vscode` → `VsCode`, `cli` → `Terminal`.
  - `fn parse_statusline(v: &Value) -> (Option<Reading>, Option<Reading>, Option<u64> /*ctx window*/)`. `rate_limits.five_hour/seven_day.{used_percentage, resets_at}`; `resets_at` may be epoch seconds or ISO text, so accept both.
  - `fn parse_rate_limit_event(v: &Value) -> (Option<Reading>, Option<Reading>, bool /*overage_disabled*/)`. Uses `rate_limit_info.unifiedWindows.{five_hour,seven_day}.{utilization 0–1, resetsAt s}`; `overage_disabled = overageDisabledReason == "org_level_disabled"`.
- Produces in `codex`:
  - `enum CodexEvent { Meta { cwd: String, branch: Option<String> }, Model(String), TaskStarted(i64), TaskComplete(i64), Tokens { ts_ms: i64, input: u64, window: u64, total: u64 }, Limits { ts_ms: i64, primary: Reading, secondary: Option<Reading> } }`
  - `fn parse_rollout_line(v: &Value) -> Vec<CodexEvent>`
- Produces in `opencode`:
  - `struct OcSession { id: String, directory: String, model: Option<String>, cost: f64, tokens: u64, updated_ms: i64, idle_ms: Option<i64> }`
  - `fn read_opencode(db: &Path) -> rusqlite::Result<(Vec<OcSession>, Spend /*today*/)>`
  - Open with `OpenFlags::SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX`. Query only `session_v2`, with `where time_archived is null order by time_updated desc limit 5`; today = `time_updated >= local midnight ms`.

- [ ] **Step 1: Write the failing tests:**
  - `claude_ctx_sums_three_input_fields`: input 2, cache_creation 1494, cache_read 41500 → 42996.
  - `claude_statusline_accepts_epoch_and_iso`.
  - `claude_event_maps_utilization_to_pct`: 0.25 → 25.0, resetsAt 1792329000 → 1792329000000 ms, overage_disabled true.
  - `codex_limits_from_token_count`: primary 13.0 / resets 1792328518 → Reading 13.0.
  - `opencode_reads_only_session_v2`: temp DB created in the test with `session_v2` plus a `credential` table holding a sentinel. Assert the result and that no query touched `credential`, by running with an authorizer that panics on that table.
  - `parsers_ignore_unknown_shapes`: random JSON → `None` / empty, no panic.
- [ ] **Step 2: Run** `cargo test -p si-core`. Expected: FAIL.
- [ ] **Step 3: Implement the three parsers.**
- [ ] **Step 4: Run** `cargo test -p si-core`. Expected: PASS.
- [ ] **Step 5: Commit** ("Add Claude, Codex and OpenCode parsers").

### Task 4: Live limit model and estimator

**Files:**
- Create: `crates/si-core/src/live.rs`, `crates/si-core/src/estimate.rs`

**Interfaces:**
- Consumes: `Reading`, `WindowView`, `Freshness` (Task 1).
- Produces:
  - `struct LimitTracker`
  - `LimitTracker::ingest(&mut self, r: Reading)`. Within one `resets_at_ms` window keep `max(used_pct)`, so stale out-of-order snapshots never lower it. A newer `resets_at_ms` starts a new window.
  - `LimitTracker::view(&self, now_ms: i64, estimate: Option<f64>) -> Option<WindowView>`
  - `struct Estimator`
  - `Estimator::learn(&mut self, prev: &Reading, next: &Reading, weighted_tokens: f64)`
  - `Estimator::estimate(&self, last: &Reading, weighted_tokens_since: f64) -> Option<f64>`
  - `fn weighted_tokens(model: &str, input: u64, cache_create: u64, cache_read: u64, output: u64) -> f64`

**Decided rules (spec §5):**
- Freshness by age of the last exact reading: ≤ 60 s Exact. Otherwise Estimated if the estimator has a rate (Claude only), else Stale once > 30 min, else Exact-with-age (an "updated Nm ago" label).
- When `resets_at_ms <= now`: `used_pct = 0`, `resets_at_ms = None`, freshness Exact.
- Burn = (latest − oldest within the last 5 min, same window) ÷ minutes (min 1).
- `minutes_to_empty = left / burn`, but only when burn > 0 and the result lands before the reset.
- `used_elsewhere_pts = exact − estimate`, shown when ≥ 2.
- Weights:
  - model multiplier: opus 5, sonnet 3, haiku 1, fable 5, unknown 3;
  - token mix: `input + 1.25·cache_create + 0.1·cache_read + 5·output`.
- `learn` keeps an exponential moving average (α = 0.3) of `(next.used − prev.used) / weighted_tokens`, ignoring pairs with Δused ≤ 0 or across windows.

- [ ] **Step 1: Write the failing tests:**
  - `max_wins_within_window`: ingest 13, 12, 13 → view 13.
  - `new_window_replaces`.
  - `reset_in_past_shows_zero`.
  - `freshness_thresholds`: ages 59 s / 61 s / 31 min.
  - `burn_and_projection`: 1 → 13 over 6 min, left 87 → burn 2.0, about 43.5 min, `None` if the reset is sooner.
  - `estimator_learns_and_estimates`.
  - `used_elsewhere_detected`: estimate 30, exact 36 → 6.
- [ ] **Step 2: Run** `cargo test -p si-core live estimate`. Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run.** Expected: PASS.
- [ ] **Step 5: Commit** ("Add live limit model and estimator").

### Task 5: Alerts and message copy

**Files:**
- Create: `crates/si-core/src/alerts.rs`

**Interfaces:**
- Produces:
  - `enum AlertKind { Left50, Left20, Left10, LimitReached, FastDrain, ContextHigh, TaskFinished }`
  - `struct Alert { agent: AgentKind, kind: AlertKind, title: String, body: String, priority: u8, level: Level }`
  - `enum Level { Warn, Crit, Info }`
  - `struct AlertEngine`
  - `AlertEngine::evaluate(&mut self, agent: AgentKind, window: WindowKind, view: &WindowView, project: &str, now_ms: i64) -> Vec<Alert>`
  - `struct Prefs { context_high: bool, task_finished: bool }` (both default false)

**Decided rules (spec §8):**
- Thresholds fire once per `(agent, window, kind, resets_at)`. They re-arm when `resets_at` changes.
- FastDrain fires when burn over 5 min ≥ 2.0 pts/min (10 points in 5 min) **and** `minutes_to_empty` is `Some`.
- Cooldown: 10 min per `(agent, kind)`.
- Priority: Left50 = 3, Left20 = 4, Left10 = 4, FastDrain = 4, LimitReached = 5.
- Copy is exact, with `{reset}` formatted as `h:mm tt` local:
  - Left20: title `"Claude Code · 20% left"`, body `"5-hour limit · resets {reset} · review model usage"`.
  - FastDrain: title `"Codex · draining fast"`, body `"{pts}% used in {min} min on {project} · ~{eta} min left at this pace"`.
  - LimitReached with overage disabled: body `"Hard stop until {reset} (org overage off)"`.

- [ ] **Step 1: Write the failing tests:**
  - `threshold_fires_once_and_rearms_on_reset`.
  - `fast_drain_needs_projection_before_reset`.
  - `cooldown_suppresses_repeat`.
  - `copy_matches_spec`: the exact strings above.
- [ ] **Step 2: Run.** Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run.** Expected: PASS.
- [ ] **Step 5: Commit** ("Add alert engine").

### Task 6: Source scanner and snapshot builder

**Files:**
- Create: `crates/si-core/src/scan.rs`

**Interfaces:**
- Consumes: Tasks 2–5.
- Produces:
  - `struct Roots { claude: Vec<PathBuf>, codex: Vec<PathBuf>, opencode_db: Vec<PathBuf> }`
  - `fn default_roots(home: &Path) -> Roots`. WSL distros come from `wsl.exe -l -q`, each probed with a 500 ms timeout on a worker thread.
  - `struct Scanner`
  - `Scanner::new(roots: Roots) -> Scanner`
  - `Scanner::ingest_external(&mut self, r: ExternalReading)`, where `enum ExternalReading { StatusLine(Value), RateLimitEvent(Value) }`
  - `Scanner::tick(&mut self, now_ms: i64) -> Snapshot`

**Decided rules:**
- **Claude:** sessions come from `sessions/*.json` (skip `*.key`) whose pid is alive and whose `procStart` matches. Each session's transcript is located once by globbing `projects/*/{sessionId}.jsonl` and primed with 2 MB. Show a session if it's busy or had activity in the last 15 min.
- **Codex:** today's and yesterday's `sessions/YYYY/MM/DD`. Activity = the maximum line timestamp, bumped to now when the file size grows. **Never use mtime**: Windows doesn't refresh it for open files. Working = activity < 20 s, or an open task with activity < 5 min.
- **OpenCode:** working = `updated < 30 s` and not idle after the update.
- `tokens_per_min` = the sum of turn tokens over 5 min ÷ 5.

- [ ] **Step 1: Write the failing tests:**
  - `snapshot_from_fixture_tree`: a temp HOME with fixture files → sessions and limits as expected.
  - `codex_activity_ignores_stale_mtime`: set mtime 20 min old while lines are 5 s old → Working.
  - `dead_pid_hidden`.
  - `wsl_probe_times_out`: inject a probe that sleeps 5 s → `default_roots` returns in < 700 ms without that distro.
- [ ] **Step 2: Run.** Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run.** Expected: PASS.
- [ ] **Step 5: Commit** ("Add scanner and snapshot builder").

### Task 7: Tauri shell, named pipe, `--wrap` and `--statusline` modes

**Files:**
- Create: `app/src-tauri/` (Tauri 2 scaffold), with `src/main.rs`, `src/pipe.rs`, `src/wrap.rs`, `src/statusline.rs`, `src/engine.rs`

**Interfaces:**
- Consumes: `Scanner`, `ExternalReading`, `AlertEngine`.
- Produces:
  - Pipe name `\\.\pipe\status-island-{USERNAME}`, carrying JSON lines `{"kind":"statusline"|"rate_limit_event","payload":{…}}`.
  - Tauri event `snapshot` with payload `Snapshot`, emitted every 1 s and on any file change (`notify` watcher, 150 ms debounce).
  - Tauri command `get_snapshot() -> Snapshot`.
- `main()` checks argv **before** Tauri initialises:
  - `--wrap <claude.exe> <args…>` → `wrap::run()`
  - `--statusline` → `statusline::run()`
  - otherwise → the app

**wrap::run (port of the proven C# test, spec §7.2):**
- Pass the raw command line after the first two tokens through verbatim.
- Pipe stdin, stdout and stderr byte for byte, and return the child's exit code.
- Use a Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.
- Forward only stdout lines containing `"type":"rate_limit_event"` to the pipe, without blocking. If the pipe is missing, drop the line.
- 1 MB line cap. Any internal error falls back to pass-through.

**statusline::run:**
- Read all of stdin.
- Send `{"kind":"statusline"}` to the pipe with a 50 ms connect timeout.
- If `%LOCALAPPDATA%\StatusIsland\previous-statusline.txt` holds a command, run it with the same stdin and print its stdout. Otherwise print `""`.
- Must finish in < 150 ms when the island isn't running.

- [ ] **Step 1: Write the failing tests** (`app/src-tauri/tests/wrap.rs`, port of today's harness, using a fake child):
  - `wrap_args_identical`: JSON with quotes, spaces, trailing backslash, an empty argument, Unicode.
  - `wrap_stdin_stdout_passthrough`.
  - `wrap_exit_code_preserved` (7).
  - `wrap_forwards_only_limit_lines`: content lines never reach the pipe.
  - `wrap_kill_ends_child`.
  - `statusline_chains_previous_command`.
  - `statusline_fast_without_island` (< 150 ms).
- [ ] **Step 2: Run** `cargo test -p status-island`. Expected: FAIL.
- [ ] **Step 3: Implement** `pipe.rs`, `wrap.rs`, `statusline.rs`, `engine.rs` (scanner thread + watcher + pipe server → `snapshot` events).
- [ ] **Step 4: Run.** Expected: PASS.
- [ ] **Step 5: Real-run check.** With the island running, start a terminal `claude` session that uses the add-on → a `statusline` reading arrives. Then try `--wrap` with `claude.exe --version` → `2.1.x (Claude Code)`, exit 0.
- [ ] **Step 6: Commit** ("Add app shell, pipe, wrap and statusline modes").

### Task 8: Connectors (settings edits with restore)

**Files:**
- Create: `app/src-tauri/src/connectors.rs`, `app/src-tauri/src/jsonc_edit.rs`

**Interfaces:**
- Produces:
  - `fn set_key(text: &str, key: &str, value: Option<&serde_json::Value>) -> Result<String>`. A minimal JSONC text edit: replace or insert or remove one top-level key, preserving comments, ordering and formatting. Use the `jsonc-parser` crate's CST or an equivalent.
  - `fn enable_claude(app_exe: &Path) -> Result<ConnectorState>`, `fn disable_claude() -> Result<()>`. This edits:
    - VS Code user `settings.json`: `claudeCode.claudeProcessWrapper` = `"<app_exe>" --wrap` form. Check the extension accepts arguments; if it doesn't, ship a `si-wrap.exe` copy of the app exe that implies `--wrap`.
    - `~/.claude/settings.json`: `statusLine` → `{"type":"command","command":"\"<app_exe>\" --statusline"}`, saving the old `statusLine.command` to `previous-statusline.txt`.
    - `claudeCode.initialPermissionMode` = the user's `permissions.defaultMode`, only if that is set and the initial mode isn't.
  - `struct ConnectorState { wrapper_set: bool, statusline_set: bool, backups: Vec<PathBuf> }`
- Before every edit, write a timestamped backup to `%LOCALAPPDATA%\StatusIsland\backups\`.
- `disable_claude` restores exactly the previous values.

- [ ] **Step 1: Write the failing tests:**
  - `jsonc_edit_preserves_comments_and_other_keys`: an input with `//` and `/* */` comments, trailing commas and CRLF. After set and then remove, the output is byte-identical to the input.
  - `statusline_previous_command_saved_and_restored`.
  - `wrapper_set_and_cleared`.
- [ ] **Step 2: Run.** Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run.** Expected: PASS.
- [ ] **Step 5: Commit** ("Add Claude connectors with backup and restore").

### Task 9: Push notifications and away detection

**Files:**
- Create: `app/src-tauri/src/ntfy.rs`, `app/src-tauri/src/away.rs`

**Interfaces:**
- Produces:
  - `struct NtfyConfig { server: String /*default https://ntfy.sh*/, topic: String, token: Option<String> }`
  - `fn new_topic() -> String`: `"si-"` + 24 random lowercase base32 characters.
  - `fn send(cfg: &NtfyConfig, alert: &Alert) -> Result<(), NtfyError>`: POST `{server}/{topic}` with headers `Title`, `Priority`, `Tags` (`warning` / `rotating_light` / `information_source`), body = `alert.body`. Timeout 5 s, 3 retries with backoff of 1 s, then 3 s. Daily cap of 200 sends, under ntfy's free limit of 250.
  - `fn is_away() -> bool`: `GetLastInputInfo` idle ≥ 120 s, **or** the session is locked (`WTSRegisterSessionNotification`), **or** the PC is suspending.
- The token comes from the app config only, never logged. The topic acts as a password, so it is never logged either.

- [ ] **Step 1: Write the failing tests:**
  - `topic_format`.
  - `send_builds_expected_request`: against a local mock HTTP server.
  - `daily_cap_enforced`.
  - `retries_then_typed_error`.
- [ ] **Step 2: Run.** Expected: FAIL.
- [ ] **Step 3: Implement.** In `engine.rs`, alerts call `send` only when `is_away()`; at the desk they only set the on-screen alert state.
- [ ] **Step 4: Run.** Expected: PASS. Real run: send one test push to the owner's phone from the Connect screen's "Send test" button.
- [ ] **Step 5: Commit** ("Add ntfy push and away detection").

### Task 10: Overlay windows, placement and hover

**Files:**
- Create: `app/src-tauri/src/overlay.rs`, `app/src-tauri/src/placement.rs`, `app/src-tauri/src/fullscreen.rs`

**Interfaces:**
- Produces:
  - `fn place(monitors: &[MonitorInfo], choice: MonitorChoice, offset_x: i32, taskbar: Option<TaskbarInfo>) -> Vec<Placement>`, where `struct Placement { monitor_id: String, x: i32, y: i32, w: u32, h: u32, scale: f64 }`.
  - The window is a fixed 460×520 logical size, centred, `y = top + 6` (below a top-docked taskbar if present).
  - Tauri event `pointer` with payload `{ inside: bool, down: bool }`, polled every 16 ms with `GetCursorPos` and `GetAsyncKeyState`, only while the cursor is within 200 px of the island. Otherwise poll every 250 ms.
  - Tauri command `set_island_rect(rect)`: the front end reports the current shell bounds for hit-testing.
  - `fn fullscreen_active(monitor) -> bool`: `SHQueryUserNotificationState` ∈ {`QUNS_RUNNING_D3D_FULL_SCREEN`, `QUNS_PRESENTATION_MODE`}, or the foreground window rect equals the monitor rect. Re-check on `EVENT_SYSTEM_FOREGROUND` (`SetWinEventHook`, out-of-context).
- Window flags: transparent, undecorated, always on top, skip taskbar, not focusable, `set_ignore_cursor_events(true)` permanently.

- [ ] **Step 1: Write the failing tests:**
  - `placement_centres_per_monitor`.
  - `placement_handles_monitor_removal_and_scale`: monitor list 2 → 1, scale 1.25 → 1.5; every placement lies inside a visible monitor's work area.
  - `placement_respects_top_taskbar`.
- [ ] **Step 2: Run.** Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run.** Expected: PASS.
- [ ] **Step 5: Real-run check on the owner's PC:**
  - Clicks pass through to browser tabs.
  - Hover turns the island see-through.
  - A 0.9 s hover opens the dropdown, and a click closes it.
  - A fullscreen YouTube video hides the island.
- [ ] **Step 6: Commit** ("Add overlay windows, placement and hover").

### Task 11: Island front end (states, motion, dropdown)

**Files:**
- Create: `app/src/main.tsx`, `app/src/Island.tsx`, `app/src/Row.tsx`, `app/src/Dropdown.tsx`, `app/src/AgentCard.tsx`, `app/src/MetricLine.tsx`, `app/src/Ring.tsx`, `app/src/Glyph.tsx`, `app/src/tokens.ts`, `app/src/motion.ts`, `app/src/format.ts`, `app/src/layout.ts`, `app/src/fonts/Geist*.woff2`
- Test: `app/src/format.test.ts`, `app/src/layout.test.ts`

**Interfaces:**
- Consumes: the `snapshot` and `pointer` events and `set_island_rect` (Task 10), and the `Snapshot` JSON (Task 1).
- Produces:
  - `format.ts`: `fmtReset(ms: number | null, now: number): string` gives `"resets in 2h 57m"`, `"resets in 3d 19h"`, `"resets in 4m 05s"`. `fmtAge(ms: number): string` gives `"4s"`, `"5m"`, `"2h"`. `fmtTokens(n: number): string` gives `"29.9k"`, `"360k"`, `"1.2M"`.
  - `layout.ts`: `islandSize(state: IslandState, workingCount: number, dropdownHeight?: number): { w: number; h: number; radius: number; opacity: number }`, with the sizes and opacities from Global Constraints.
  - `motion.ts` springs, as `{visualDuration, bounce}`:
    - grow 0.417/0.25; shrink 0.375/0;
    - hoverOpen 0.417/0.20; hoverClose 0.333/0;
    - row 0.292/0;
    - alertIn 0.417/0.35; alertOut 0.458/0;
    - number 0.333/0;
    - contentEnter 0.333/0.15 with a 50 ms delay;
    - ring 0.5/0 (ring change).
  - Content enter: from `{opacity: 0, scale: 0.9, y: -6, filter: "blur(5px)"}`. Exit: 100 ms `[0.16, 1, 0.3, 1]` to `{opacity: 0, scale: 0.95, filter: "blur(4px)"}`.
- Rules:
  - Width, height and radius share one spring.
  - Animate only `transform`, `opacity`, `width` and `height` on the shell. No `backdrop-filter`.
  - Under `prefers-reduced-motion`: instant or ≤ 150 ms resize, opacity-only fades, direct number swaps, colour instead of swell, and the working pulse stays visible.
  - Visuals match the Figma frames (Island states, Dropdown). Freshness goes on the card's location line. Metric detail text is 12 pt Medium.

- [ ] **Step 1: Write the failing tests:**
  - `fmtReset`: cases 10 620 000 ms → `"resets in 2h 57m"`, 329 000 000 → `"resets in 3d 19h"`, past → `"resetting"`.
  - `fmtTokens`.
  - `islandSize`: idle → 132×30/0.63; 1 working → 252×34/0.95; 2 → 252×66; 5 → 252×(34+3·26+20+6); hover → opacity 0.36; dropdown → w 400, opacity 1.
- [ ] **Step 2: Run** `npm test`. Expected: FAIL.
- [ ] **Step 3: Implement** `format.ts`, `layout.ts`, `motion.ts`, `tokens.ts`, then the components.
- [ ] **Step 4: Run** `npm test`. Expected: PASS.
- [ ] **Step 5: Real-run visual check against Figma** (screenshot with the BitBlt/CAPTUREBLT capture). Check every state: idle, compact, stacked, hover, dropdown, amber alert, red alert, stale. No clipped text, and digits don't move as numbers change.
- [ ] **Step 6: Commit** ("Add island front end").

### Task 12: Connect screen, settings and tray

**Files:**
- Create: `app/src/Connect.tsx`, `app/src/Settings.tsx`, `app/src-tauri/src/config.rs`, `app/src-tauri/src/tray.rs`

**Interfaces:**
- Produces:
  - Config file at `%LOCALAPPDATA%\StatusIsland\config.json` with `{ monitor: "active"|id, offsetX, ntfy: NtfyConfig, quietHours?: [start, end], prefs: Prefs, connectors: ConnectorState }`.
  - Tauri commands: `enable_claude`, `disable_claude`, `send_test_push`, `get_config`, `set_config`.
  - A self-check row per agent: "Claude panel · live ✓", or the reason it isn't (wrapper not parent of new sessions, no events in 10 min while busy, file format changed).
  - Tray menu: Show Connect, Settings, Pause alerts 1 h, Quit.
- The QR code encodes `ntfy://{host}/{topic}`, generated locally (the `qrcode` crate). Nothing is sent anywhere.
- Copy follows the Figma "Connect screen" frame exactly.

- [ ] **Step 1: Write the failing tests:**
  - `config_roundtrip_and_defaults`.
  - `quiet_hours_suppress_push`.
- [ ] **Step 2: Run.** Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run.** Expected: PASS. Real run: the first launch shows Connect; enabling Claude edits both settings files with backups; disabling restores them byte-for-byte.
- [ ] **Step 5: Commit** ("Add Connect screen, settings and tray").

### Task 13: Packaging, CI and release

**Files:**
- Create: `.github/workflows/release.yml`, `app/src-tauri/tauri.conf.json` (bundle section), `packaging/winget/StatusIsland.yaml`, `README.md`

**Decided:**
- **CI:** matrix `x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc` on `windows-latest`. Steps: `cargo test`, `npm test`, `tauri build`.
- **Bundle:** NSIS with `installMode: "currentUser"`, plus the WebView2 `embedBootstrapper` per-user.
- **Release assets** go on GitHub Releases, with a VirusTotal link in the notes.
- **winget** manifest `Hossain-13.StatusIsland`.
- **README** sections:
  - What it shows.
  - The safety promise (spec §3, verbatim bullets).
  - Exactly what the wrapper and status line add-on do.
  - SmartScreen "More info → Run anyway".
  - How to undo.
  - Microsoft Defender false-positive submission link.
- The Tauri updater signs updates with its own free key (separate from code signing). It shipped in 0.1.2 (owner-approved 2026-10-19).

- [ ] **Step 1: Write `release.yml`**, push a tag to a test branch, and confirm both installers build. Expected: two `.exe` assets.
- [ ] **Step 2: Install on the owner's PC with no admin prompt.** Measure idle CPU and RAM with all WebView2 processes summed (target < 1% and ~150 MB) and record the figures in the README.
- [ ] **Step 3: Uninstall.** Confirm both settings files are restored and `%LOCALAPPDATA%\StatusIsland` is removed.
- [ ] **Step 4: Commit** ("Add packaging and release pipeline").
