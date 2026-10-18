# Instructions for coding agents (Codex, Claude Code, OpenCode)

Build Status Island v1 **exactly** as written. Every product decision has already been made and approved by the owner (Sheikh Mohammad Hossain, 2026-10-18). Your job is to execute the decisions, not to make new ones.

## Sources of truth (in this order)

1. `docs/superpowers/plans/2026-10-18-status-island-v1.md`: the task list. Do tasks in order, one at a time, test-first.
2. `docs/superpowers/specs/2026-10-18-status-island-design.md`: the approved design and the reasons behind it.
3. `docs/design/*.png` and `docs/design/tokens.md`: the approved visuals (exported from Figma). The UI must match them. Live file: https://www.figma.com/design/wSnY22zFnUATHqd2eWaUhh
4. `docs/dev-setup.md`: toolchain setup.

If these disagree, the earlier one wins. If something isn't covered by any of them, **stop**. Write the question to `docs/questions.md` and ask the owner. Don't guess, don't add features, and don't "improve" the design.

## Decisions already made (do not reopen)

- **Toolchain on the home PC:** MSVC (`x86_64-pc-windows-msvc`) with Visual Studio Build Tools 2022 → "Desktop development with C++". The GNU notes in `dev-setup.md` describe the work laptop only.
- **Wrapper launch (plan Task 8):** VS Code's `claudeCode.claudeProcessWrapper` setting takes an **executable path only**; it is launched as `<path> <real claude.exe> <args…>`. So the installer ships a copy of the app exe named `si-wrap.exe`. In `main()`, if the exe file name starts with `si-wrap` (case-insensitive), run wrap mode with argv[1] as the real Claude path. Otherwise check for `--wrap` / `--statusline` flags, then start the app.
- **ARM64 builds (plan Task 13):** cross-compile on the normal `windows-latest` runner: `rustup target add aarch64-pc-windows-msvc`, then `npm run tauri build -- --target aarch64-pc-windows-msvc`. Don't use ARM64 runners.
- **Crates:**
  - `notify` (file watching)
  - `rusqlite` with `bundled` (OpenCode, read-only)
  - `jsonc-parser` with the `cst` feature (settings edits)
  - `ureq` with rustls (ntfy)
  - `windows` (Win32)
  - `qrcode` (QR on the Connect screen)
  - Front end: `motion`, `@number-flow/react`, `vitest`
- **Estimator weights, alert thresholds, message copy, sizes, opacities, colours and springs** are exactly as in the plan's Global Constraints and task blocks.
- **Version:** first release `v0.1.0`, published as a GitHub Release (unsigned) from the tag workflow in Task 13.

## Safety rules (hard; from spec §3)

- Never open: `~/.claude/.credentials.json`, `*.key` files, `~/.claude/ide/*.lock`, `~/.claude.json`, browser profiles or cookies, `~/.codex/auth.json`, or the OpenCode tables `account`, `control_account` and `credential`.
- Never send requests with a user's Claude, OpenAI or other account. No proxies or relays. The only network traffic is ntfy push.
- Every settings file the app edits is backed up first and restored on uninstall or disable.
- No admin rights anywhere (per-user install). Never log ntfy topics or tokens.

## Working rules

- **Test-first:** write the test, see it fail, implement, see it pass. Run the exact verification command in each step and paste the result in your summary.
- **Real-run steps** (marked "Real-run check" in the plan) need the owner's PC and apps: Claude Code, Codex and OpenCode installed and signed in, and the ntfy app on a phone. If you can't do one, mark the step **BLOCKED: needs owner** and move on. **Never mark it passed.**
- **Commits:** the owner writes commit messages, and there is no AI attribution (no `Co-Authored-By` lines). Stage per task, then ask the owner for the message. You may use the plan's per-task messages only if the owner tells you to at the start of the session.
- **Pushing tags or releases:** only after the owner says "release".
- Keep `si-core` free of OS dependencies.
- No speculative abstractions. Smallest change that fully solves the task.
