Status Island is a Dynamic-Island-style overlay for Windows. It shows which AI coding agent is working (Claude Code, Codex, OpenCode), how much of its plan limit is left, how fast it is draining, and how full its context window is. It can send a push to your phone when a limit drains fast while you're away.

### What it promises
- **Read-only.** It reads only files the agents write themselves. It never reads your login, never sends requests with your account, and never intercepts traffic.
- **No fake numbers.** Every figure is exact, or clearly labelled as an estimate with its age.
- **No admin rights.** Per-user install, and every change it makes can be undone.

### Install
1. Download `StatusIsland-x64-setup.exe`. On a Snapdragon or other ARM laptop, use `StatusIsland-arm64-setup.exe`.
2. Run it. These builds aren't code-signed yet, so Windows SmartScreen may say "Windows protected your PC". Click **More info → Run anyway**. The source and the build log for this release are public on GitHub.
3. On first launch, the Connect screen opens. Switch on Claude Code for live limits, and scan the QR code with the free ntfy app if you want phone alerts.

### Undo
- Settings → Claude connector → **Turn off and restore my settings** puts back your VS Code and Claude settings exactly as they were.
- Uninstalling (Windows Settings → Apps) does the same, then removes the app's data.

If your antivirus flags the installer, please report it as a false positive to Microsoft: https://www.microsoft.com/en-us/wdsi/filesubmission

You can check an installer with [VirusTotal](https://www.virustotal.com/gui/home/upload). No VirusTotal scan result is claimed for these builds.

### What's new in 0.1.2
- **Updates install themselves.** Status Island checks GitHub every few hours; when a new version is ready, the tray menu shows **Install update** and one click installs it, keeping your settings and the Claude connector. Only updates signed with the project's key are accepted. Turn it off in Settings.
- **On 0.1.1?** Install this version by hand once. 0.1.1's uninstaller runs during the upgrade and restores your settings, so the Connect screen opens again: switch Claude Code back on and reload VS Code. Later updates don't do this.
- OpenCode is detected on every current OpenCode version. Some versions keep sessions in a different table, and Status Island didn't find them ([#1](https://github.com/Hossain-13/status-island/issues/1)).
- Installers now have fixed names (`StatusIsland-x64-setup.exe`, `StatusIsland-arm64-setup.exe`), so download links always point at the newest release.

### 0.1.1
- No console window: the island, the VS Code wrapper and the status line add-on all run without one.
- Processes Claude starts (dev servers, builds) are no longer ended when a wrapped Claude session closes.
- The OpenCode card shows the model name properly ("DeepSeek Flash · OpenCode"), and its details aren't cut off.
- Weekly alerts say "Weekly · 50% left" and show the weekly number. Informational alerts no longer glow amber.

### Verified on the owner's PC (Windows 11, x64)
- Per-user install and upgrade with no admin prompt; the first launch opens Connect.
- Live Claude 5-hour and weekly limits through the VS Code connector. Codex and OpenCode are detected automatically.
- Uninstall removes the app and its data.
- Turning the Claude connector off restores both settings files byte-for-byte (VS Code and Claude, checked against the backups).
- Phone alerts through ntfy (QR pairing and test push), fullscreen hiding and display changes.
- Resource use while an agent was working: about 0.5% of total CPU (12 cores) and about 180 MB private memory, including WebView2.

### Still to verify
- Install on an ARM64 laptop.

### Known limits
- The Claude Connect-screen self-check does not yet independently verify the VS Code wrapper's parent process or distinguish every failure reason.
- A dedicated Windows suspend notification is not wired; resume behavior needs an owner-PC check.
