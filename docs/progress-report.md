# Status Island v0.1.0 preview progress

## Environment

- Work tree: `E:\status-island`; private workspace was read only for reference.
- Rust `rustc 1.99.1`, Cargo `1.99.1`, Node `v24.16.0`, Git `2.52.0.windows.1`; GitHub CLI authenticated as `Hossain-13`.
- Local Rust used the per-repository GNU toolchain because MSVC Build Tools were unavailable here. Windows MSVC x64 and ARM64 installer builds use GitHub Actions `windows-latest`.
- No real agent, credential, settings file, or ntfy endpoint was used for testing. Parser data and connector settings tests use synthetic fixtures and temporary copies; ntfy tests use a local mock server.

## Tasks and automated checks

| Task | Status | Named tests and evidence | Commit |
| --- | --- | --- | --- |
| 3 Parsers | Done | `claude_ctx_sums_three_input_fields`, `claude_statusline_accepts_epoch_and_iso`, `claude_event_maps_utilization_to_pct`, `codex_limits_from_token_count`, `opencode_reads_only_session_v2`, `parsers_ignore_unknown_shapes` | `677d996`; SQLite read authorizer hardened in `dc13fd8` |
| 4 Live model | Done | `max_wins_within_window`, `new_window_replaces`, `reset_in_past_shows_zero`, `freshness_thresholds`, `burn_and_projection`, `estimator_learns_and_estimates`, `used_elsewhere_detected` | `79ffea7` |
| 5 Alerts | Done | `threshold_fires_once_and_rearms_on_reset`, `fast_drain_needs_projection_before_reset`, `cooldown_suppresses_repeat`, `copy_matches_spec` | `7837fc5` |
| 6 Scanner | Done | `snapshot_from_fixture_tree`, `codex_activity_ignores_stale_mtime`, `dead_pid_hidden`, `wsl_probe_times_out`; reused PID start check added | `903eca9`, `37eff1e` |
| 7 App shell | Done | `wrap_args_identical`, `wrap_stdin_stdout_passthrough`, `wrap_exit_code_preserved`, `wrap_forwards_only_limit_lines`, `wrap_kill_ends_child`, `statusline_chains_previous_command`, `statusline_fast_without_island` | `0957547` |
| 8 Connectors | Done | `jsonc_edit_preserves_comments_and_other_keys`, `statusline_previous_command_saved_and_restored`, `wrapper_set_and_cleared`; unrelated user changes also tested | `a1f95a1` |
| 9 Push and away | Done | `topic_format`, `send_builds_expected_request`, `daily_cap_enforced`, `retries_then_typed_error`; locked-session query added | `b3a63d5`, `2c1a595` |
| 10 Overlay | Done | `placement_centres_per_monitor`, `placement_handles_monitor_removal_and_scale`, `placement_respects_top_taskbar`; foreground event hook and fullscreen guard added | `d21841a`, `2c1a595` |
| 11 Island UI | Done | Front-end `fmtReset`, `fmtTokens`, `islandSize` assertions pass | `615b4dd`, `c51173f` |
| 12 Connect, settings, tray | Done | `config_roundtrip_and_defaults`, `quiet_hours_suppress_push` | `37eff1e`, `4c02b4f` |
| 13 Packaging | Automated build passed; human install pending | NSIS current-user bundle, embedded WebView2 bootstrapper, x64 and ARM64 workflow; uninstall now stops if restore fails | `036dbcc`, `04a4cb9` |

Final local commands on the updated tree: `cargo fmt --all`; `cargo test --workspace` (all suites passed); `cargo clippy --workspace --all-targets -- -D warnings` (passed); `npm test` (3 files, 8 tests passed); `npm run build` (passed). The front-end build emitted upstream `use client` bundler warnings, with exit code 0.

GitHub Actions CI run [39200791414](https://github.com/Hossain-13/status-island/actions/runs/39200791414) passed on `04a4cb9`. Manual release workflow run [39200814484](https://github.com/Hossain-13/status-island/actions/runs/39200814484) passed and uploaded both x64 and ARM64 installer artifacts on the same commit. The tag workflow [39201683669](https://github.com/Hossain-13/status-island/actions/runs/39201683669) passed all three jobs, including publication.

## BLOCKED: needs owner

These checks require the owner's actual desktop, installed agents, or phone. They were **not** marked as passed:

1. Install the preview without an admin prompt, confirm Connect appears, then turn the Claude connector on and off. Compare the real VS Code and Claude settings files byte-for-byte after restore.
2. Start real Claude Code terminal and VS Code panel sessions, Codex, and OpenCode. Confirm the live readings and self-check explanations; check the wrapper passes `claude.exe --version` through unchanged.
3. Pair the owner's phone and use **Send test**. Confirm the ntfy notification arrives. No real push was sent during this run.
4. Check browser-tab click-through, 36% hover opacity, fullscreen hide/return, a top taskbar, monitor unplug and DPI change. Compare idle, compact, stacked, dropdown, alert, and stale states with the design PNGs using a BitBlt/CAPTUREBLT capture.
5. Measure idle CPU and total Status Island plus WebView2 RAM. Target: under 1% CPU and approximately 150 MB. **Measured performance: unavailable** until this representative owner-PC run.
6. Uninstall and verify both settings files are restored and `%LOCALAPPDATA%\StatusIsland` is removed.

The Connect screen's Claude self-check currently reports live exact readings or that it is waiting for a reply; it does not independently prove the VS Code wrapper is the parent of new sessions or distinguish every stated failure reason. Verify this with the owner before a full release.

The away check covers 120-second idle time and a locked Windows session. A dedicated Windows suspend notification is not wired; the system stops normal app work while suspended, but resume behavior needs the owner-PC check before a full release.

## Decisions and release

`docs/questions.md` records the owner's resolved alert-copy and icon decisions; there are no open questions. The authorized [v0.1.0 preview](https://github.com/Hossain-13/status-island/releases/tag/v0.1.0-preview) is published as a pre-release, not a full release or latest release. The tag points at `04a4cb9`. Its assets are `Status.Island_0.1.0_x64-setup.exe` (SHA-256 `ab73ca9c9880df5aaaebecbc907fa8d4b6f9b16ed49da7d79296df88ac7e2aa6`) and `Status.Island_0.1.0_arm64-setup.exe` (SHA-256 `df4db6e80b9b14e5d53609ec1da99cfd123445d86602de8cd26cb8434bca53d8`), as reported by GitHub's release API. The winget manifests were validated locally with `winget validate packaging/winget` and have not been submitted to the community repository.

The later main-branch commits `b78c3a2`, `1f73a0c`, `9bf4ecb`, and `ea8013f` were pushed by another active run after the preview tag. Their changes are **not** in the published installers. The console-window fixes in those commits should be included in a future preview build; this tag was not moved or rewritten.

A full release remains owner-only.
