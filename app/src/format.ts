export function fmtReset(ms: number | null, now: number): string {
  if (ms === null) return 'reset unknown';
  const remaining = Math.max(0, ms - now);
  if (remaining === 0) return 'resetting';
  const seconds = Math.floor(remaining / 1000);
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor(seconds / 3600) % 24;
  const minutes = Math.floor(seconds / 60) % 60;
  if (days) return `resets in ${days}d ${hours}h`;
  if (seconds >= 3600) return `resets in ${Math.floor(seconds / 3600)}h ${minutes}m`;
  return `resets in ${Math.floor(seconds / 60)}m ${String(seconds % 60).padStart(2, '0')}s`;
}

export function fmtAge(ms: number): string {
  const seconds = Math.max(0, Math.floor(ms / 1000));
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
  return `${Math.floor(seconds / 3600)}h`;
}

export function fmtTokens(n: number): string {
  if (n >= 1_000_000) return `${trim(n / 1_000_000)}M`;
  if (n >= 1_000) return `${trim(n / 1_000)}k`;
  return String(n);
}

/** Tokens per minute shown as tokens per second. */
export function fmtRate(perMin: number): string {
  const perSec = perMin / 60;
  return `${perSec >= 10 ? Math.round(perSec) : perSec.toFixed(1)} tok/s`;
}

function trim(n: number): string { return n >= 100 ? Math.round(n).toString() : n.toFixed(1).replace(/\.0$/, ''); }
