Status Island is a Dynamic-Island-style overlay for Windows. It shows which AI coding agent is working (Claude Code, Codex, OpenCode), how much of its plan limit is left, how fast it is draining, and how full its context window is. It can send a push to your phone when a limit drains fast while you're away.

### What it promises
- **Read-only.** It reads only files the agents write themselves. It never reads your login, never sends requests with your account, and never intercepts traffic.
- **No fake numbers.** Every figure is exact, or clearly labelled as an estimate with its age.
- **No admin rights.** Per-user install, and every change it makes can be undone.

### Install
1. Download `Status Island_<version>_x64-setup.exe`. On a Snapdragon or other ARM laptop, use `…_arm64-setup.exe`.
2. Run it. These builds aren't code-signed yet, so Windows SmartScreen may say "Windows protected your PC". Click **More info → Run anyway**. The source and the build log for this release are public on GitHub.
3. On first launch, the Connect screen opens. Switch on Claude Code for live limits, and scan the QR code with the free ntfy app if you want phone alerts.

### Undo
- Settings → Claude connector → **Turn off and restore my settings** puts back your VS Code and Claude settings exactly as they were.
- Uninstalling (Windows Settings → Apps) does the same, then removes the app's data.

If your antivirus flags the installer, please report it as a false positive to Microsoft: https://www.microsoft.com/en-us/wdsi/filesubmission
