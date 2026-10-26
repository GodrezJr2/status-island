import { Glyph } from './Glyph';
import { MetricLine } from './MetricLine';
import { fmtAge, fmtRate, fmtReset, fmtTokens } from './format';
import { agentName, levelOf } from './status';
import { colors, levelColor } from './tokens';
import type { AccountLimits, SessionView, Snapshot, WindowView } from './types';

function freshnessText(w: WindowView, extraMs: number): string {
  const age = fmtAge(w.ageMs + extraMs);
  if (w.freshness === 'exact') return `exact ${age} ago`;
  if (w.freshness === 'stale') return `stale · ${age}`;
  if (w.freshness === 'estimated') return `est. · ${age}`;
  return `updated ${age} ago`;
}

/** Figma "Agent card": title, location (with data freshness), then 5-hour, Weekly, Context, Burn. */
export function AgentCard({ session, snapshot, now }: { session: SessionView; snapshot: Snapshot; now: number }) {
  const sinceSnapshot = Math.max(0, now - snapshot.nowMs);
  const account: AccountLimits | null = session.kind === 'claude' ? snapshot.claude : session.kind === 'codex' ? snapshot.codex : null;
  const five = account?.fiveHour ?? null;
  const week = account?.weekly ?? null;
  const surface = session.surface === 'vsCode' ? 'VS Code' : session.surface === 'terminal' ? 'Terminal' : null;
  const location = [session.project, session.branch, surface, five ? freshnessText(five, sinceSnapshot) : null].filter(Boolean).join(' · ');
  const chip = session.activity === 'working' ? 'Working' : session.activity === 'waiting' ? 'Waiting' : 'Idle';
  const fast = (five?.burnPtsPerMin ?? 0) >= 2;

  return <section className="agent-card">
    <div className="card-title"><Glyph kind={session.kind} size={16}/><strong>{session.modelLabel} · {agentName(session.kind)}</strong>
      <span className={`chip ${session.activity}`}>{session.activity === 'working' && <span className="chip-dot"/>}{chip}</span></div>
    <div className="card-location">{location}</div>
    {five && <MetricLine label="5-hour" value={`${Math.round(five.leftPct)}% left`} detail={fmtReset(five.resetsAtMs, now)} fraction={five.leftPct / 100} color={levelColor[levelOf(five)]}/>}
    {week && <MetricLine label="Weekly" value={`${Math.round(week.leftPct)}% left`} detail={fmtReset(week.resetsAtMs, now)} fraction={week.leftPct / 100} color={levelColor[levelOf(week)]}/>}
    {session.ctxTokens !== null && <MetricLine label="Context" value={session.ctxPct !== null ? `${session.ctxPct}% used` : fmtTokens(session.ctxTokens)} detail={session.ctxPct !== null ? `${fmtTokens(session.ctxTokens)} tokens` : 'tokens in context'} fraction={session.ctxPct !== null ? session.ctxPct / 100 : null} color={colors.text}/>}
    {session.spend
      ? <>
          <MetricLine label="Spent" value={`$${session.spend.sessionUsd.toFixed(2)}`} detail={`this session · $${(snapshot.opencodeToday?.todayUsd ?? session.spend.todayUsd).toFixed(2)} today`} color={colors.openCode}/>
          <MetricLine label="Tokens" value={fmtTokens(session.spend.sessionTokens)} detail="no plan limit (API key)"/>
        </>
      : <MetricLine label="Burn" value={five && five.burnPtsPerMin > 0 ? `${five.burnPtsPerMin.toFixed(1)}%/min` : 'steady'}
          detail={`${fmtRate(session.tokensPerMin)}${fast ? ' · draining fast' : ''}`} color={fast ? colors.warn : undefined}/>}
    {five?.usedElsewherePts != null && five.usedElsewherePts >= 2 && <div className="card-note">+{Math.round(five.usedElsewherePts)}% used elsewhere (web, phone or another PC)</div>}
  </section>;
}
