# Status Island: design

- **Date:** 2026-10-18
- **Owner:** Sheikh Mohammad Hossain (GitHub `Hossain-13`)
- **Status:** draft for approval
- **Evidence:** research reports and live tests run on the owner's PC on 2026-10-18 (summarised in §12)

## 1. Intent

Status Island is a Windows overlay modelled on Apple's Dynamic Island. It sits top-centre on the screen and shows, live, which AI coding agent is working, how much of its plan limit is left, how fast that limit is draining, and how full its context window is.

**Why it exists.** The owner once watched a coding agent burn 60% of a 5-hour limit in about a minute and only noticed afterwards. Developers start agent jobs blind. The island is not a remote control. It never pauses or switches anything. It shows true numbers early enough for the developer to decide: push on, switch to a cheaper model or lower effort, or let the job finish knowingly.

**Who it's for.** The owner first. Then developers who find it through a LinkedIn launch, as a free open-source download.

## 2. Goals, non-goals, anti-goals

**Goals**
1. Live, honest usage for **Claude Code, Codex and OpenCode** (v1 scope, owner's decision).
2. Apple-grade visuals and motion: simple to look at, complete underneath.
3. Phone push when a limit drains fast or runs low while the developer is away from the desk.
4. Works on any Windows 10/11 PC: x64 and ARM64, any display scaling, multiple monitors.
5. One-click setup. Users never have to type commands, run `/usage`, or edit config files.

**Non-goals (v1)**
- Controlling agents (pause, stop, switch model).
- Other agents (Copilot, Cursor, Antigravity, JetBrains AI, Windsurf/Devin, Cline, Kiro, …) are v2+.
- macOS or Linux.
- Cost dashboards and history reports beyond the hover panel.

**Anti-goals (would count as failure even if the app works)**
- **A fake number.** Every figure is exact, or clearly labelled as an estimate with its age. The island never shows a smoothly drifting gauge it made up.
- **Any risk to the user's login or account** (see §3).
- An overlay that gets in the way: covers clicks, lags the keyboard, burns CPU when idle, or stays on screen over fullscreen video and games.
- Generic "AI-template" UI.

## 3. Safety rules (hard requirements)

The owner uses an organisation Claude account, and these rules are not negotiable. They are also a public selling point.

1. **Never read credentials.** No `.credentials.json`, OAuth tokens, `*.key` files, IDE lock files, browser cookies or keychains.
2. **Never make requests with the user's account.** No calls to Anthropic, OpenAI or claude.ai endpoints. No hidden agent sessions. Never spend the user's tokens to read their usage.
3. **No interception of network traffic** (no proxies or relays).
4. **Only official, documented mechanisms:** the Claude Code status line, the Claude VS Code extension's `claudeProcessWrapper` setting, and local files the agents write themselves.
5. **Local only.** The only outbound traffic is phone push (ntfy) and an optional update check. Push messages carry the agent name, %, project name and times. Never prompts or code.
6. **Every change can be undone, and uninstall undoes it automatically.**

## 4. Data sources (v1)

All sources below were verified on the owner's PC unless marked otherwise.

### 4.1 Claude Code

| Surface | Limit % (5h / weekly) | Context, model, project, state | How fast |
|---|---|---|---|
| Terminal (any terminal, incl. VS Code's and WSL) | **Status line JSON** `rate_limits.five_hour/seven_day.{used_percentage, resets_at}`. Org Max plan confirmed. | Status line `context_window.*`, `model.id`, `cwd` | Every reply |
| VS Code panel | **`rate_limit_event`** on the panel process's stdout, read through the wrapper (§7.2). `rate_limit_info.unifiedWindows.{five_hour,seven_day}.{utilization 0–1, resetsAt}`, plus `status`, `overageStatus`, `overageDisabledReason`. | Transcript JSONL (below) | Sent when the number changes, at most every 30 s (from Claude's code; cadence still being verified, see §11) |
| Any surface | **Not used.** `~/.claude.json` has a usage cache, but the file also holds account identity and can hold API keys, so reading it would break safety rule 1 (my call). | — | — |
| claude.ai web, desktop chat, phone | Not visible. It shows up at the next exact reading as "used elsewhere". | — | — |

- **Transcripts:** `~/.claude/projects/<slug>/<sessionId>.jsonl`, plus `subagents/`. Assistant lines carry `timestamp`, `cwd`, `gitBranch`, `message.model`, and `message.usage` (`input_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`, `output_tokens`). **Context tokens = input + cache_creation + cache_read** of the latest assistant line.
  - The context window size is not stored. Take it from the status line `context_window_size` when available, otherwise from a model table (1M for Opus 5.5 on this account).
  - Median gap between replies during work: 3.9 s.
- **Live sessions:** `~/.claude/sessions/<pid>.json` holds `pid`, `sessionId`, `cwd`, `entrypoint` (`cli` / `claude-vscode`), `status` (`busy` / `idle`), and `procStart` for detecting reused PIDs.
  - This is undocumented but proven live: `status` flips within about 4 s of each turn.
  - Never open the neighbouring `*.key` files.
- **WSL (v1, my call):** read transcripts and session files from `\\wsl.localhost\<distro>\home\<user>\.claude\`, read-only, so WSL sessions show state, context and token burn, with limits from the estimator.
  - Exact limits for WSL terminal sessions (a Linux status line build plus a file or socket bridge) are v2.
  - Codex and OpenCode files inside WSL are read the same way.
- **Plans without limits** (Claude API key, Bedrock/Vertex; Codex with an API key): no `rate_limits` arrive, so the island shows the tokens and $ view used for OpenCode. v1 supports these users; it doesn't exclude them.

### 4.2 Codex

- **Rollouts:** `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`.
  - `session_meta` → `cwd`, `git.branch`.
  - `turn_context` → `model`, `effort`.
  - `event_msg/token_count` → `info.last_token_usage`, `info.model_context_window`, and `rate_limits.{primary, secondary}.{used_percent, window_minutes, resets_at}`, `plan_type`.
- **Exact limits arrive every 2–5 s while working** (89 snapshots in 8 minutes, live test).
- **Interleaving:** several threads write snapshots, and older ones arrive out of order (13 → 12 → 13). Rule: within one `resets_at` window, usage never decreases, so show the **maximum** `used_percent` seen for the current window.
- **Active:** the newest rollout's mtime is under 30 s old, or there is a `task_started` with no matching `task_complete`.
- **Context % = `last_token_usage.input_tokens / model_context_window`.**

### 4.3 OpenCode (DeepSeek on the owner's PC)

- **`~/.local/share/opencode/opencode.db`**, opened read-only with the WAL intact.
  - `session_v2` holds `model`, `directory`, `cost`, `tokens_*`, `time_updated`, `time_idle`.
  - Some OpenCode versions use a `session` table instead, with the same columns but no `time_idle` (issue #1). The reader uses whichever exists.
  - `session_message` holds per-message tokens and cost.
  - The context limit comes from the `kv['models-dev:catalog']` model catalogue.
  - Never read the `account`, `control_account` or `credential` tables.
- **No plan limits exist for API-key providers.** The island shows **tokens and $ spent** (session and today) instead of "% left". The DeepSeek balance endpoint needs the user's API key, so it is out of scope for v1.

## 5. The live model

For each agent and each limit window, the island holds:

- **Exact reading:** value, time, source. Badge: `Exact · 4s`.
- **Estimate between readings (Claude only):** last exact value plus the tokens used since then × a learned rate (% per weighted token).
  - The rate is learned from pairs of exact readings and the local tokens used between them. Weight tokens by model price, because limits behave like cost.
  - Show it as `~31% est. · 40s`. Snap back to the exact value when a reading arrives.
- **Freshness:** **Exact** (≤ 60 s) → **Estimated** → **Stale** (no reading for 30 min, or for the 5-hour window, idle since the last reset) → **Unknown**. Each state has its own colour and wording.
- **Used elsewhere:** if an exact reading is higher than the estimate by 2 points or more, show "+N% used elsewhere (web, phone or another PC)".
- **Resets are exact.** When `resets_at` passes, the window drops to 0 (plus any new usage) on time. The reset countdown ticks every second.
- **Burn rate:** points per minute over a rolling 5-minute window, from exact readings (and the estimate for Claude). Projection: "≈ 18 min to empty at this pace", shown only when it lands before the reset.
- **Display rule:** when a new value arrives, the number rolls to it over about 0.4 s. Between updates nothing moves except clocks and countdowns.

## 6. Interface

Reference: Apple's Dynamic Island. Motion and size values come from the 2026-10-18 motion research (Apple HIG, WWDC23 "Animate with springs", converted to Motion).

### 6.1 Placement
- Top-centre of the chosen monitor, 6–8 px below the top edge, centred under the laptop bezel camera.
  - No Windows laptop has a display cutout, and Windows has no API for one, so this is a fixed position.
  - Settings offer a horizontal offset and a monitor choice: "follow active window" or a fixed monitor.
- Moves clear of a top-docked taskbar (`SHAppBarMessage`).
- Hides automatically in fullscreen video, games and presentations (`SHQueryUserNotificationState` plus a foreground-window-equals-monitor check), and comes back when they end.

### 6.2 States
| State | When | Look |
|---|---|---|
| **Idle** | No agent working | Small black capsule, about 120 × 28. A thin ring for the tightest limit at right, dimmed. |
| **Compact** | One agent working | 240 × 32 capsule. Left: agent glyph (20 px) with a soft "working" pulse. Centre: model name ("Opus 5.5"). Right: ring showing **limit left**, with the number. |
| **Stacked** | Two or more working | Same width. One extra 26 px row per agent (glyph · model · thin bar · % left). Up to 3 rows, then "+N more". The island grows **downward**, so the top row never moves (owner's decision). |
| **Hover panel** | Pointer rests 120 ms on the island; closes 250 ms after it leaves | Expands to 400 wide. One card per agent session: model, project · branch, state (Thinking / Running tool / Waiting for you / Idle), context bar, 5-hour and weekly left with reset countdown, burn-rate sparkline, freshness badge, "used elsewhere" note. |
| **Alert** | Fast drain, threshold crossed, or limit reached | The island swells briefly and takes the alert colour and icon ("Draining fast", "20% left", "Limit reached · resets 3:40 PM"), then settles back. While the condition holds, it keeps a coloured key line. |

- **Opacity per state (owner's Figma edits, 2026-10-18):**
  - idle: 63%;
  - compact/stacked: 95%;
  - hover see-through: 36%;
  - open dropdown: 100%.
- **Depth and glow (owner's edits plus the two adjustments he approved):**
  - The open dropdown casts a dark shadow (65% black, blur about 8, small offset).
  - Alerts glow softly in **their own status colour**: amber for draining fast, red for limit reached (55% alpha, blur 14, no offset). They stay at the normal 34 px height.
- **Metric lines:** detail text 12 pt Medium (owner wanted it larger; Medium keeps the values as the first thing you read). Data freshness ("exact 4s ago") sits on the card's location line, so each metric line stays on one line.
- **Fill:** `#000`, 1 px inner key line `rgba(255,255,255,0.10)`.
  - Capsules (compact, idle, rows) use `corner-shape: round` with radius = h/2.
  - The hover panel uses radius `min(h/2, 36)` with `corner-shape: squircle`, falling back to plain radius.
- **Text:** **Geist**, bundled with the app (SIL OFL licence) so it looks identical on every PC (my call; Segoe UI isn't in Figma and SF Pro can't ship on Windows). Medium weight or heavier, `tabular-nums`, fixed-width number slots so nothing reflows.
- **Visual design (v1, for owner review):** https://www.figma.com/design/wSnY22zFnUATHqd2eWaUhh. It covers tokens, components, all island states, the dropdown, phone notifications and the Connect screen.
- **Colour per state (distinct at a glance):**
  - healthy: neutral white;
  - estimated: dimmed with a dotted ring;
  - warning (≤ 20% left or fast drain): amber;
  - critical (≤ 10% or limit reached): red;
  - stale: grey with a clock icon.
- **Agent glyphs:** simple original marks in each agent's colour family, not copies of company logos.

### 6.3 Motion (Motion library springs)
| Transition | visualDuration / bounce |
|---|---|
| Idle → compact (grow) | 0.417 / 0.25 |
| Compact → idle (shrink) | 0.375 / 0 |
| Hover open / close | 0.417 / 0.20 · 0.333 / 0 |
| Row add/remove, layout move | 0.292 / 0 |
| Alert swell / settle | 0.417 / 0.35 · 0.458 / 0 |
| Number roll | 0.333 / 0 |
| Content enter | 0.333 / 0.15, 50 ms delay; from opacity 0, scale 0.9, y −6, blur 5 px |
| Content exit | 100 ms tween `[0.16,1,0.3,1]` to opacity 0, scale 0.95, blur 4 px |
| Ring change | 0.5 / 0 on stroke-dashoffset |

- **Grow with bounce, shrink without.** Width, height and radius share **one** spring per shell.
- Animate real width and height on the shell; keep content absolutely positioned. Persistent items (glyph, ring) move rather than fade.
- Interruptions carry their velocity: a quick hover in and out redirects smoothly.
- **Reduced motion** (Windows "Animation effects" off):
  - the shell resizes instantly or within 150 ms with no bounce;
  - content cross-fades with opacity only;
  - numbers change directly;
  - alerts use colour and icon instead of a swell;
  - the "working" indicator must still visibly show activity.

### 6.4 Never in the way
- **Peek-through (owner's request, 2026-10-18; the island covered a browser tab):**
  - **Hover** → the island fades to about 45% opacity, so the island and whatever is underneath (tabs, title-bar buttons) are both visible, and clicks reach what's underneath.
  - **Long hover (0.9 s)** → the dropdown panel opens.
  - A mouse press over the island closes an open panel instantly and holds it closed until the pointer leaves, so the user never clicks on something hidden.
  - (Owner tried an earlier variant on 2026-10-18 — fade to 12% while moving, open after 0.35 s still — and preferred this one.)
  - Detected by polling the cursor position and button state (`GetCursorPos`, `GetAsyncKeyState`), not by hooks.
- The window is **fully click-through at all times** (the panel is view-only). Clicks always reach the app underneath.
- No global keyboard or mouse hooks.
- **Idle budget:** under 1% CPU and about 150 MB RAM total, including WebView2 (to measure). Watch files instead of polling them. No rendering while nothing changes.

## 7. Setup and the Claude connectors

### 7.1 First launch: one "Connect" screen
The island detects installed agents and shows one switch each, with one plain line saying what it does.
- **Codex and OpenCode** need nothing; they are read automatically.
- **Claude** has one switch: "Live limits in terminal and VS Code". Turning it on applies 7.2 and 7.3.
- **Phone:** scan a QR code (§8).

### 7.2 VS Code panel wrapper (`claudeCode.claudeProcessWrapper`)
- **Shipped as a small signed Rust exe.** The extension starts it as `wrapper <real claude.exe> <args…>`.
- **Behaviour (proven by the 2026-10-18 C# test):**
  - Pass the rest of the command line through *verbatim*.
  - Pipe stdin, stdout and stderr byte for byte and keep the exit code.
  - Put the child in a Job Object with kill-on-close, so killing the wrapper never leaves Claude running.
  - Copy **only** stdout lines containing `"type":"rate_limit_event"` to the island over a local named pipe. If the island isn't running, drop them.
  - Cap line size. On any internal error, keep passing data through.
- **The setting is machine-scoped** (it can't be set per workspace). The island writes it to VS Code user `settings.json`, editing only that key, and restores it on uninstall or when the user switches it off.
- **Side effect noted in the extension's code:** with a wrapper set, the panel's starting permission mode falls back to "default" unless `claudeCode.initialPermissionMode` is set. On the owner's PC the tab still started in auto. The island checks for this and, if needed, copies the user's `permissions.defaultMode` into `claudeCode.initialPermissionMode`.
- **Self-check:** confirm the wrapper is the parent of new panel sessions (process tree) and that events arrive. Show "Claude panel · live ✓" or the reason it isn't.

### 7.3 Terminal status line add-on
- Add a `statusLine` to `~/.claude/settings.json` that runs the island's tiny status line program.
- The program forwards `rate_limits` and `context_window` to the island, then **runs the user's previous status line command** with the same input and prints its output. Existing status lines (ccstatusline etc.) keep working.
- Restored on uninstall.

## 8. Phone notifications

- **Channel:** ntfy (free, open source, Android and iOS apps, no account). Setup is a QR code holding a long random topic. Self-hosted server URL and token are optional.
  - Free ntfy.sh limit: 250 messages/day.
  - Topic names act as passwords, so messages carry no sensitive content.
- **Triggers:** each fires once per limit window and re-arms on reset.
  - 50%, 20% and 10% left.
  - Limit reached.
  - **Fast drain:** ≥ 10 points in 5 minutes, *and* the projection empties the limit before it resets.
  - Optional, off by default: context ≥ 85%, and task finished.
- **Only when away:** screen locked, no input for 2 min (`GetLastInputInfo`), or the PC suspended. At the desk, the island alerts on screen instead. Cooldown per agent and type. Optional quiet hours.
- **Wording:**
  - "**Claude Code · 20% left**", body "5-hour limit · resets 3:40 PM · review model usage"
  - "**Codex · draining fast**", body "12% used in 7 min on VWMS · ~50 min left at this pace"
  - For org accounts with overage disabled: "Limit reached · hard stop until 3:40 PM".

## 9. Platform and distribution

- **Tauri 2:** Rust core (file watching, wrapper pipe, Win32 signals, ntfy client, settings edits) and a WebView2 front end in React and Motion.
  - **Rejected:** Electron (2–3× the memory for an always-on overlay); WinUI 3 (transparency bugs, slower to iterate); WPF (weak motion); Flutter (multi-window and click-through immature).
- **Targets:** Windows 10 and 11, x64 and ARM64, per-monitor DPI aware.
- **Releases:** NSIS installers per architecture on GitHub Releases, plus winget. Per-user install, no admin. **Unsigned at launch (owner's decision, 2026-10-18).** The README explains the SmartScreen "More info → Run anyway" step and links the source. Signing (SignPath Foundation) and the Microsoft Store come later.
- **First milestone (owner's decision):** Claude Code, Codex and OpenCode together.
- **No admin rights, anywhere (owner's requirement):**
  - Installer in `currentUser` mode under `%LOCALAPPDATA%`, with no UAC prompt.
  - Every file the island edits is inside the user profile.
  - The named pipe is per user.
  - The Win32 APIs used need no elevation.
  - The WebView2 runtime installs per user if it's missing.
  - The dev toolchain must also install without admin (rustup per-user; check whether the MSVC build tools need admin, and if they do, build in CI on GitHub Actions instead).
- **Antivirus without signing (risk §11.5):**
  - The wrapper is **the island's own exe started in a `--wrap` mode**, not a second unknown binary, so antivirus sees one app.
  - Reproducible builds from GitHub Actions, with a VirusTotal link in each release.
  - Submit each release to Microsoft's free false-positive review (Defender submission portal).
  - The README shows exactly what the wrapper does and links the source.
- **Licence:** MIT (my call). Do not copy GPL code from competitors (vibe-island is GPL-3.0).
- **Auto-update** through the Tauri updater with signed manifests.

## 10. Quality bar

- Real-run checks on the owner's PC for every source. Unit tests for parsers (sample JSONL lines and DB rows), the Codex interleaving rule, the estimator, threshold re-arming and away detection.
- **Wrapper test suite:** argument fidelity (quotes, JSON, Unicode, empty arguments), stdin/stdout fidelity, exit code, orphan kill, only limit lines forwarded.
- Performance measured with all WebView2 processes summed. Check for keyboard lag. Reduced-motion pass. Mixed-DPI two-monitor pass.
- No AI attribution in commits. The owner writes commit messages.

## 11. Open questions and risks

1. **Panel event cadence: answered (§12, 15:48–15:49).** A busy wrapped session sent an event at each whole-percent step (28% → 29%). Idle tabs send nothing, because they make no requests. So the estimator is a filler between 1% steps and for usage elsewhere, not the main path.
2. The Claude Desktop "Code" tab probably writes the same `~/.claude` files. Not verified.
3. Undocumented files (`~/.claude/sessions/*.json`, Codex rollout fields) can change between versions. Parsers must fail soft, and the self-check must report it.
4. Extension updates could rename `claudeProcessWrapper` or the event format. The self-check detects it and the island falls back to the estimator.
5. Antivirus on company PCs may question an exe that launches `claude.exe`. Signing and open source reduce this risk; it is worth checking with the owner's IT.
6. VS Code Remote and WSL panel sessions run the extension on the remote side, so the wrapper must live there. That makes them v2.

## 12. Evidence log (2026-10-18, owner's PC)

| Time | Finding |
|---|---|
| 14:01 | `~/.claude.json` usage cache refreshed only by a manual `/usage` (5h 77%, weekly 43%) |
| 14:02–14:10 | Codex: 89 exact limit snapshots, 1% → 13% of the 5-hour limit in about 7 min |
| 14:20 | Terminal status line on the org Max plan: 5h 7% (after the 14:10 reset), weekly 45% |
| 14:3x | Claude VS Code extension: panel receives `rate_limit_event`. Nothing is persisted. `claudeProcessWrapper` setting exists (machine scope). |
| 14:4x | C# test wrapper: argument, stdin, stdout and exit-code fidelity pass; only the limit line logged; orphan kill pass; `claude --version` through the wrapper passes |
| 15:31 | First panel event through the wrapper: 5h 25%, weekly 47%, `overageDisabledReason: org_level_disabled`. Tab started in auto mode. |
| 15:35 | Sonnet tab, 26 replies in 1 min 40 s: one event (26%) |
| 15:45 | Terminal: 27%. The wrapped tabs were idle, so nothing was missed. |
| 15:48 | This session restarted through the wrapper to test cadence under real work |
| 15:48:15 / 15:49:54 | Busy wrapped session: events at 28% and then 29%, each at the whole-percent step, 1 min 39 s apart |

## 13. Left to the builder

- Exact React component structure, state store and IPC message shapes.
- The estimator's weighting table (model price ratios) and learning maths, within §5's rules.
- Glyph artwork, final colour values and type scale. Claude designs these in Figma, and the owner approves them before the product UI is built. That pass is a gate for the UI only; the Rust data core can be built in parallel.
- Settings screen layout (monitor, offset, notifications, quiet hours, connectors, uninstall-restore).

## 14. Calls I made (for the owner to check)

- MIT licence.
- v1 shows tokens and $ for OpenCode rather than reading the DeepSeek balance, which would need the API key.
- Fast-drain threshold: ≥ 10 points in 5 min plus projected exhaustion before reset.
- Away = locked, suspended, or 2 min without input.
- Freshness thresholds: Exact ≤ 60 s, Stale after 30 min.
- VS Code Remote/WSL panel support deferred to v2.
- Agent glyphs are original marks, not company logos.
