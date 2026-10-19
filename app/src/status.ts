import type { Kind, Snapshot, WindowView } from './types';

export type Level = 'ok' | 'warn' | 'crit' | 'stale';
export type AlertKind = 'left50' | 'left20' | 'left10' | 'limitReached' | 'fastDrain' | 'contextHigh' | 'taskFinished';

/** Status level per spec §6.2: ≤10% left critical; ≤20% left or ≥2 pts/min warning. */
export function levelOf(w: WindowView | null): Level {
  if (!w) return 'ok';
  if (w.freshness === 'stale') return 'stale';
  if (w.leftPct <= 10) return 'crit';
  if (w.leftPct <= 20 || w.burnPtsPerMin >= 2) return 'warn';
  return 'ok';
}

export function clock(ms: number | null): string {
  if (ms === null) return 'unknown';
  return new Date(ms).toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
}

export type LimitWindow = 'fiveHour' | 'weekly';

/** Text shown on the island while an alert is active (spec §6.2 alert row). Weekly alerts say
 *  so, because the row's number and ring then show the weekly limit, not the 5-hour one. */
export function alertText(kind: AlertKind, w: WindowView | null, window: LimitWindow = 'fiveHour'): string {
  const weekly = window === 'weekly' ? 'Weekly · ' : '';
  switch (kind) {
    case 'fastDrain': return `${weekly}Draining fast · ${(w?.burnPtsPerMin ?? 0).toFixed(1)}%/min`;
    case 'limitReached': return `${window === 'weekly' ? 'Weekly limit' : 'Limit'} reached · resets ${clock(w?.resetsAtMs ?? null)}`;
    case 'left50': return `${weekly}50% left`;
    case 'left20': return `${weekly}20% left`;
    case 'left10': return `${weekly}10% left`;
    default: return '';
  }
}

/** The smallest limit left across agents and windows, for the idle capsule. */
export function tightestLeft(s: Snapshot): WindowView | null {
  const all = [s.claude.fiveHour, s.claude.weekly, s.codex.fiveHour, s.codex.weekly].filter((w): w is WindowView => w !== null);
  return all.reduce<WindowView | null>((min, w) => (min === null || w.leftPct < min.leftPct ? w : min), null);
}

export function agentName(kind: Kind): string {
  return kind === 'claude' ? 'Claude Code' : kind === 'codex' ? 'Codex' : 'OpenCode';
}
