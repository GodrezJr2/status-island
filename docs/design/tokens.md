# Design tokens (approved 2026-10-18)

The source is the Figma variables collection "Status Island", mode "Dark". The images in this folder are exports of the approved frames.

## Colours

| Token | Value |
|---|---|
| `island/fill` | `#000000` |
| `island/keyline` | `#FFFFFF` at 10% |
| `surface/card` | `#FFFFFF` at 6% |
| `surface/track` | `#FFFFFF` at 16% |
| `text/primary` | `#F5F5F7` |
| `text/secondary` | `#8E8E93` |
| `status/ok` | `#F5F5F7` |
| `status/warn` | `#FFB020` |
| `status/crit` | `#FF453A` |
| `status/live` | `#30D158` |
| `status/stale` | `#8E8E93` |
| `agent/claude` | `#E8835E` |
| `agent/codex` | `#8AB4F8` |
| `agent/opencode` | `#5EE09A` |

Chips (Working / Idle / Live): status colour at 16% background, with text in the status colour, 10.5 pt SemiBold, radius 8, padding 8×2.

## Type

Geist, bundled (OFL). Use tabular numbers.

| Use | Size | Weight |
|---|---|---|
| Island model name (primary row) | 13 | SemiBold |
| Island model name (extra rows) | 12 | SemiBold, 85% opacity |
| Island % left | 13 / 12 | SemiBold |
| Card title | 13 | SemiBold |
| Card location line (incl. freshness, e.g. "Status Island · VS Code · exact 4s ago") | 11 | Medium |
| Metric label | 11 | Medium |
| Metric value | 12 | SemiBold |
| Metric detail | 12 | Medium |

## Shapes and effects

| Thing | Spec |
|---|---|
| Idle | 132×30, radius 15, opacity 0.63 |
| Compact | 252×34, radius 17, opacity 0.95 |
| Stacked | 252 × (34 + 26·extra + 6), radius 22, opacity 0.95 |
| Hover see-through | opacity 0.36 |
| Dropdown | 400 wide, padding 14, gap 8, radius 26, opacity 1. Shadow: black 65%, blur ~8, offset (−3, 2), spread 4. |
| Agent card | radius 16, `surface/card`, padding 12/10/12/12 |
| Alert glow | drop shadow in the alert's status colour at 55%, blur 14, offset 0. Keyline: status colour at 60%. Height stays 34. |
| Limit ring | 18 px (primary) / 14 px (extra rows), stroke ≈ 26% of radius. Track `surface/track`; arc = fraction left, starting at 12 o'clock, clockwise. |
| Metric bar | 56×4, radius 2. Fill = fraction × 56. |
| Agent glyphs | Original marks (not company logos). SVGs: see `components.png`; the source SVGs are in the workspace repo. |
