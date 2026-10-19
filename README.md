# Status Island

A Dynamic-Island-style overlay for Windows that shows, live, which AI coding agent is working, how much of its plan limit is left, how fast that limit is draining, and how full its context window is. When a limit drains fast while you're away from your desk, it sends a push to your phone.

Works with **Claude Code, Codex and OpenCode** on **Windows 10 and 11** (x64 and ARM64).

![Island states](docs/design/island-states.png)

## Install

**Download the installer:**
- **Most PCs (Intel / AMD):** [StatusIsland-x64-setup.exe](https://github.com/Hossain-13/status-island/releases/latest/download/StatusIsland-x64-setup.exe)
- **ARM laptops (Snapdragon, Surface Pro X):** [StatusIsland-arm64-setup.exe](https://github.com/Hossain-13/status-island/releases/latest/download/StatusIsland-arm64-setup.exe)

All versions are on the [Releases page](https://github.com/Hossain-13/status-island/releases).

**Then:**
1. **Run the installer.** It installs just for you, so no admin rights are needed.
2. **If Windows says "Windows protected your PC"**, click **More info → Run anyway**. The app isn't code-signed yet; every installer is built in public by [this workflow](.github/workflows/release.yml) from this source.
3. **The Connect screen opens on first launch.** Codex and OpenCode are picked up automatically.
4. **For live Claude limits,** switch on **Claude Code**, then reload VS Code once (`Ctrl+Shift+P` → **Developer: Reload Window**). Claude tabs that were already open before the switch can't report until that reload.
5. **For phone alerts (optional),** install the free **ntfy** app (Android or iPhone), scan the QR code on the Connect screen, and tap **Send test**.

Status Island lives in the system tray. Right-click it for Connect, Settings, Check for updates, Pause alerts and Quit.

**Updates:** from 0.1.2 on, Status Island checks GitHub for a new version every few hours. When one is ready, the tray menu shows **Install update** and one click installs it, with no admin rights, keeping your settings and connector. It only installs updates signed with this project's key. You can turn the check off in Settings. **On 0.1.1?** Download and run the installer above once; after that, updates come to you.

If your antivirus flags the installer, please report a false positive to Microsoft: https://www.microsoft.com/en-us/wdsi/filesubmission

## Using it

- **At a glance:** the island sits top-centre and shows each working agent with how much of its limit is **left**.
- **Hover:** the island turns see-through, so you can still see and click what's underneath.
- **Keep hovering (about 1 second):** the dropdown opens with each agent's 5-hour and weekly limits, reset times, context used and burn rate.
- **Alerts:** the island glows amber when a limit drops to 20% or less or is draining fast, and red at 10% or when the limit is reached. When you're away from your desk, the same alerts go to your phone.

## What it promises

- **Read-only.** It reads only files the agents write themselves. It never reads your login, never sends requests with your account, and never intercepts traffic.
- **No fake numbers.** Every figure is exact, or clearly labelled as an estimate with its age.
- **No admin rights.** Per-user install, and every change it makes can be undone.

## Safety rules

1. **Never read credentials.** No `.credentials.json`, OAuth tokens, `*.key` files, IDE lock files, browser cookies or keychains.
2. **Never make requests with the user's account.** No calls to Anthropic, OpenAI or claude.ai endpoints. No hidden agent sessions. Never spend the user's tokens to read their usage.
3. **No interception of network traffic** (no proxies or relays).
4. **Only official, documented mechanisms:** the Claude Code status line, the Claude VS Code extension's `claudeProcessWrapper` setting, and local files the agents write themselves.
5. **Local only.** The only outbound traffic is phone push (ntfy) and an optional update check. Push messages carry the agent name, %, project name and times. Never prompts or code.
6. **Every change can be undone, and uninstall undoes it automatically.**

## How it reads your agents

| Agent | What it reads | What it changes |
|---|---|---|
| Codex | Its session files in `~/.codex/sessions` | Nothing |
| OpenCode | Its session table in its local database (read-only) | Nothing |
| Claude Code | Its session and transcript files in `~/.claude` | Only if you switch on the Claude connector (see below) |

**The Claude connector** is optional: one switch on the Connect screen. It makes Claude's limits live in two ways:

- **Terminal:** it sets Claude Code's documented `statusLine` to Status Island's small status line program. Your previous status line command is saved and still runs, so its output still shows.
- **VS Code panel:** it sets the Claude extension's documented `claudeCode.claudeProcessWrapper` to `si-wrap.exe`. VS Code then starts Claude through it. The wrapper passes everything through unchanged, and copies **only** the limit messages Claude already sends to the panel. It never stores conversation content, never touches your login, and makes no network requests.

Both settings files are backed up before the change.

## Undo

- Settings → Claude connector → **Turn off and restore my settings** puts both files back exactly as they were.
- Uninstalling (Windows Settings → Apps) does the same automatically, then removes the app's data from `%LOCALAPPDATA%\StatusIsland`.

## Docs

- Design: [`docs/superpowers/specs/2026-10-18-status-island-design.md`](docs/superpowers/specs/2026-10-18-status-island-design.md)
- Build plan: [`docs/superpowers/plans/2026-10-18-status-island-v1.md`](docs/superpowers/plans/2026-10-18-status-island-v1.md)
- Visual design: [`docs/design/`](docs/design/)

MIT licensed.
