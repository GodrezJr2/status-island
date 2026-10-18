export type Kind = 'claude' | 'codex' | 'openCode';
export type Freshness = 'exact' | 'estimated' | 'stale' | 'unknown';
export interface WindowView { usedPct: number; leftPct: number; resetsAtMs: number | null; freshness: Freshness; ageMs: number; burnPtsPerMin: number; minutesToEmpty: number | null; usedElsewherePts: number | null; estimated: boolean }
export interface Spend { sessionUsd: number; todayUsd: number; sessionTokens: number }
export interface SessionView { id: string; kind: Kind; modelLabel: string; project: string; branch: string | null; surface: 'terminal' | 'vsCode' | 'unknown'; activity: 'working' | 'waiting' | 'idle'; lastEventMs: number; ctxTokens: number | null; ctxPct: number | null; tokensPerMin: number; spend: Spend | null }
export interface AccountLimits { fiveHour: WindowView | null; weekly: WindowView | null; overageDisabled: boolean }
export interface Snapshot { nowMs: number; sessions: SessionView[]; claude: AccountLimits; codex: AccountLimits; opencodeToday: Spend | null }
export const emptySnapshot: Snapshot = { nowMs: 0, sessions: [], claude: { fiveHour: null, weekly: null, overageDisabled: false }, codex: { fiveHour: null, weekly: null, overageDisabled: false }, opencodeToday: null };
export function windowFor(snapshot: Snapshot, kind: Kind): WindowView | null { return kind === 'claude' ? snapshot.claude.fiveHour : kind === 'codex' ? snapshot.codex.fiveHour : null; }
