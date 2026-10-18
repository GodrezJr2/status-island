export const springs = {
  grow: { visualDuration: 0.417, bounce: 0.25 },
  shrink: { visualDuration: 0.375, bounce: 0 },
  hoverOpen: { visualDuration: 0.417, bounce: 0.20 },
  hoverClose: { visualDuration: 0.333, bounce: 0 },
  row: { visualDuration: 0.292, bounce: 0 },
  alertIn: { visualDuration: 0.417, bounce: 0.35 },
  alertOut: { visualDuration: 0.458, bounce: 0 },
  number: { visualDuration: 0.333, bounce: 0 },
  contentEnter: { visualDuration: 0.333, bounce: 0.15, delay: 0.05 },
  ring: { visualDuration: 0.5, bounce: 0 },
} as const;
