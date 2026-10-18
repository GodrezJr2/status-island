# Status Island

A Dynamic-Island-style overlay for Windows that shows, live, which AI coding agent is working, how much of its plan limit is left, how fast that limit is draining, and how full its context window is. When a limit drains fast while you're away from your desk, it sends a push to your phone.

**Status:** design approved, v1 in development. Supports Claude Code, Codex and OpenCode.

![Island states](docs/design/island-states.png)

## What it promises

- **Read-only.** It reads only files the agents write themselves. It never reads your login, never sends requests with your account, and never intercepts traffic.
- **No fake numbers.** Every figure is exact, or clearly labelled as an estimate with its age.
- **No admin rights.** Per-user install, and every change it makes can be undone.

## Docs

- Design: [`docs/superpowers/specs/2026-10-18-status-island-design.md`](docs/superpowers/specs/2026-10-18-status-island-design.md)
- Build plan: [`docs/superpowers/plans/2026-10-18-status-island-v1.md`](docs/superpowers/plans/2026-10-18-status-island-v1.md)
- Visual design: [`docs/design/`](docs/design/)

MIT licensed.
