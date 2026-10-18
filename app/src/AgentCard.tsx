import { Glyph } from './Glyph';
import { MetricLine } from './MetricLine';
import { fmtAge, fmtReset, fmtTokens } from './format';
import type { AccountLimits, SessionView, Snapshot } from './types';

export function AgentCard({ session, snapshot }: { session: SessionView; snapshot: Snapshot }) {
  const account: AccountLimits | null = session.kind === 'claude' ? snapshot.claude : session.kind === 'codex' ? snapshot.codex : null;
  const freshness = account?.fiveHour?.freshness;
  const age = account?.fiveHour?.ageMs;
  return <section className="agent-card">
    <div className="card-title"><Glyph kind={session.kind}/><strong>{session.modelLabel}</strong><span className={`chip ${session.activity}`}>{session.activity === 'working' ? 'Working' : session.activity === 'waiting' ? 'Waiting' : 'Idle'}</span></div>
    <div className="card-location">{[session.project, session.branch, session.surface === 'vsCode' ? 'VS Code' : session.surface === 'terminal' ? 'Terminal' : null, freshness && age !== undefined ? `${freshness} ${fmtAge(age)} ago` : null].filter(Boolean).join(' · ')}</div>
    {session.ctxPct !== null && <MetricLine label="Context" value={`${session.ctxPct}%`} detail={`${fmtTokens(session.ctxTokens ?? 0)} tokens`} left={session.ctxPct}/>}
    {account?.fiveHour && <MetricLine label="5-hour" value={`${Math.round(account.fiveHour.leftPct)}% left`} detail={fmtReset(account.fiveHour.resetsAtMs, snapshot.nowMs)} left={account.fiveHour.leftPct}/>}
    {account?.weekly && <MetricLine label="Weekly" value={`${Math.round(account.weekly.leftPct)}% left`} detail={fmtReset(account.weekly.resetsAtMs, snapshot.nowMs)} left={account.weekly.leftPct}/>}
    {account?.fiveHour?.usedElsewherePts !== null && account?.fiveHour?.usedElsewherePts !== undefined && <div className="card-note">{Math.round(account.fiveHour.usedElsewherePts)} points used elsewhere</div>}
    {session.spend && <MetricLine label="Spend" value={`$${session.spend.sessionUsd.toFixed(2)}`} detail={`${fmtTokens(session.spend.sessionTokens)} tokens`}/>}
    {session.tokensPerMin > 0 && <div className="card-note">{fmtTokens(session.tokensPerMin)} tokens/min</div>}
  </section>;
}
