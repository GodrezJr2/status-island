import { Glyph } from './Glyph';
import { Ring } from './Ring';
import type { SessionView, WindowView } from './types';

export function Row({ session, window, primary = false }: { session: SessionView; window: WindowView | null; primary?: boolean }) {
  return <div className={`island-row ${primary ? 'primary' : ''}`}>
    <span className={session.activity === 'working' ? 'working-glyph' : ''}><Glyph kind={session.kind} size={primary ? 20 : 16}/></span>
    <span className="row-model">{session.modelLabel}</span>
    {window ? <><Ring left={window.leftPct} freshness={window.freshness} size={primary ? 18 : 14}/><strong className="row-percent">{Math.round(window.leftPct)}%</strong></> : session.spend ? <strong className="row-percent">${session.spend.sessionUsd.toFixed(2)}</strong> : <strong className="row-percent muted">—</strong>}
  </div>;
}
