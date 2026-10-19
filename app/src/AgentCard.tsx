import { Glyph } from './Glyph';
import { MetricLine } from './MetricLine';
import { fmtAge, fmtReset, fmtTokens } from './format';
import { agentName, levelOf } from './status';
import { colors, levelColor } from './tokens';
import type { AccountLimits, SessionView, Snapshot, WindowView } from './types';

function freshnessText(w: WindowView): string {
  if (w.freshness === 'exact') return `exact ${fmtAge(w.ageMs)} ago`;
  if (w.freshness === 'stale') return `stale · ${fmtAge(w.ageMs)}`;
  if (w.freshness === 'estimated') return `est. · ${fmtAge(w.ageMs)}`;
  return `updated ${fmtAge(w.ageMs)} ago`;
}

/** Figma "Agent card": title, location (with data freshness), then 5-hour, Weekly, Context, Burn. */
export function AgentCard({ session, snapshot }: { session: SessionView; snapshot: Snapshot }) {
  const account: AccountLimits | null = session.kind === 'claude' ? snapshot.claude : session.kind === 'codex' ? snapshot.codex : null;
  const five = account?.fiveHour ?? null;
  const week = account?.weekly ?? null;
  const surface = session.surface === 'vsCode' ? 'VS Code' : session.surface === 'terminal' ? 'Terminal' : null;
  const location = [session.project, session.branch, surface, five ? freshnessText(five) : null].filter(Boolean).join(' · ');
  const chip = session.activity === 'working' ? 'Working' : session.activity === 'waiting' ? 'Waiting' : 'Idle';
  const fast = (five?.burnPtsPerMin ?? 0) >= 2;

  return <section className="agent-card">
    <div className="card-title"><Glyph kind={session.kind} size={16}/><strong>{session.modelLabel} · {agentName(session.kind)}</strong>
      <span className={`chip ${session.activity}`}>{session.activity === 'working' && <span className="chip-dot"/>}{chip}</span></div>
    <div className="card-location">{location}</div>
    {five && <MetricLine label="5-hour" value={`${Math.round(five.leftPct)}% left`} detail={fmtReset(five.resetsAtMs, snapshot.nowMs)} fraction={five.leftPct / 100} color={levelColor[levelOf(five)]}/>}
    {week && <MetricLine label="Weekly" value={`${Math.round(week.leftPct)}% left`} detail={fmtReset(week.resetsAtMs, snapshot.nowMs)} fraction={week.leftPct / 100} color={levelColor[levelOf(week)]}/>}
    {session.ctxTokens !== null && <MetricLine label="Context" value={session.ctxPct !== null ? `${session.ctxPct}% used` : fmtTokens(session.ctxTokens)} detail={`${fmtTokens(session.ctxTokens)} tokens`} fraction={session.ctxPct !== null ? session.ctxPct / 100 : null} color={colors.text}/>}
    {session.spend
      ? <>
          <MetricLine label="Spent" value={`$${session.spend.sessionUsd.toFixed(2)}`} detail={`this session · $${(snapshot.opencodeToday?.todayUsd ?? session.spend.todayUsd).toFixed(2)} today`} color={colors.openCode}/>
          <MetricLine label="Tokens" value={fmtTokens(session.spend.sessionTokens)} detail="no plan limit (API key)"/>
        </>
      : <MetricLine label="Burn" value={five && five.burnPtsPerMin > 0 ? `${five.burnPtsPerMin.toFixed(1)}%/min` : 'steady'}
          detail={`${fmtTokens(session.tokensPerMin)} tokens/min${fast ? ' · draining fast' : ''}`} color={fast ? colors.warn : undefined}/>}
    {five?.usedElsewherePts != null && five.usedElsewherePts >= 2 && <div className="card-note">+{Math.round(five.usedElsewherePts)}% used elsewhere (web, phone or another PC)</div>}
  </section>;
}
