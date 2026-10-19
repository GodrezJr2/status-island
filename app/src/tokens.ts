// Approved Figma tokens (docs/design/tokens.md).
export const colors = {
  text: '#F5F5F7', secondary: '#8E8E93', live: '#30D158', warn: '#FFB020', crit: '#FF453A', stale: '#8E8E93',
  track: 'rgba(255,255,255,.16)', keyline: 'rgba(255,255,255,.10)',
  claude: '#E8835E', codex: '#8AB4F8', openCode: '#5EE09A',
} as const;

export const levelColor = { ok: colors.text, warn: colors.warn, crit: colors.crit, stale: colors.stale } as const;
