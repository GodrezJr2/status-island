import NumberFlow from '@number-flow/react';
import { Glyph } from './Glyph';
import { Ring } from './Ring';
import { levelOf } from './status';
import { levelColor } from './tokens';
import type { SessionView, WindowView } from './types';

/** One agent in the compact/stacked island (Figma "Island row"): glyph · model · % left · ring. */
export function Row({ session, window, primary = false, label }: { session: SessionView; window: WindowView | null; primary?: boolean; label?: string }) {
  const color = levelColor[levelOf(window)];
  return <div className={`island-row ${primary ? 'primary' : ''}`}>
    <span className={session.activity === 'working' ? 'working-glyph' : ''}><Glyph kind={session.kind} size={primary ? 18 : 14}/></span>
    <span className="row-model" style={label ? { color } : undefined}>{label ?? session.modelLabel}</span>
    {window
      ? <><strong className="row-percent" style={{ color }}><NumberFlow value={Math.round(window.leftPct)} suffix="%" respectMotionPreference/></strong>
          <Ring left={window.leftPct} color={color} freshness={window.freshness} size={primary ? 18 : 14}/></>
      : session.spend
        ? <strong className="row-percent spend">${session.spend.sessionUsd.toFixed(2)}</strong>
        : <strong className="row-percent muted">—</strong>}
  </div>;
}
