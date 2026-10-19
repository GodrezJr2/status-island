Status Island is a Dynamic-Island-style overlay for Windows. It shows which AI coding agent is working (Claude Code, Codex, OpenCode), how much of its plan limit is left, how fast it is draining, and how full its context window is. It can send a push to your phone when a limit drains fast while you're away.

### What it promises
- **Read-only.** It reads only files the agents write themselves. It never reads your login, never sends requests with your account, and never intercepts traffic.
- **No fake numbers.** Every figure is exact, or clearly labelled as an estimate with its age.
- **No admin rights.** Per-user install, and every change it makes can be undone.

### Install
1. Download `Status.Island_0.1.0_x64-setup.exe`. On a Snapdragon or other ARM laptop, use `Status.Island_0.1.0_arm64-setup.exe`.
2. Run it. These builds aren't code-signed yet, so Windows SmartScreen may say "Windows protected your PC". Click **More info → Run anyway**. The source and the build log for this release are public on GitHub.
3. On first launch, the Connect screen opens. Switch on Claude Code for live limits, and scan the QR code with the free ntfy app if you want phone alerts.

### Undo
- Settings → Claude connector → **Turn off and restore my settings** puts back your VS Code and Claude settings exactly as they were.
- Uninstalling (Windows Settings → Apps) does the same, then removes the app's data.

If your antivirus flags the installer, please report it as a false positive to Microsoft: https://www.microsoft.com/en-us/wdsi/filesubmission

You can check an installer with [VirusTotal](https://www.virustotal.com/gui/home/upload). No VirusTotal scan result is claimed for these builds.

### Not yet verified by a human
- Install the x64 or ARM64 preview on the owner's PC without an admin prompt; confirm first launch shows Connect.
- Turn on the Claude connector and inspect the actual VS Code and Claude settings, then turn it off and confirm both files restore byte-for-byte. This automated run did not touch those settings.
- Start real Claude Code, Codex and OpenCode sessions and confirm live limits, state, and the Connect screen self-checks.
- Scan the QR code in the owner's ntfy app and send a test push. No real push was sent in this run.
- Check click-through, hover opacity, fullscreen hiding, monitor unplug/DPI changes, all island visual states, and the Figma comparison on the owner's screen.
- Measure idle CPU and summed Status Island plus WebView2 memory on the owner's PC. No representative idle performance measurement is available yet.
- Uninstall and verify both settings files restore and `%LOCALAPPDATA%\StatusIsland` is removed.

### Known preview limits
- The Claude Connect-screen self-check does not yet independently verify the VS Code wrapper's parent process or distinguish every failure reason.
- A dedicated Windows suspend notification is not wired; resume behavior needs an owner-PC check.
