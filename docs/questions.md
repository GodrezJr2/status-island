# Owner decisions

## Resolved (owner approved, 2026-10-19)

- **Alert wording not covered by the spec:**
  - 50%: "{Agent} · 50% left" / "{Window} limit · resets {time}".
  - 10%: "{Agent} · 10% left" / "{Window} limit · resets {time} · pause or switch to a lighter model".
  - Limit reached without an org block: "{Agent} · limit reached" / "Resets {time}".
- **Fast-drain wording:** use the measured rate, with no new interface field: "{Agent} · draining fast" / "~{rate}%/min on {project} · ~{eta} min left at this pace".
- **App icon:** designed in Figma to match the island (black squircle, white capsule and limit ring) and exported to `app/src-tauri/icons/`.
- **Scanner `procStart` TODO:** done as the plan specifies. A Win32 check compares the process creation time with the registry's `procStart`, so a reused PID is hidden.

## Open

_None._
