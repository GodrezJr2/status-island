import type { Kind } from './types';
import { colors } from './tokens';

/** The approved original agent marks (Figma "Agent glyph"), not company logos. */
export function Glyph({ kind, size = 18 }: { kind: Kind; size?: number }) {
  const c = colors[kind];
  return <svg width={size} height={size} viewBox="0 0 20 20" fill="none" aria-hidden="true" style={{ flex: 'none' }}>
    {kind === 'claude'
      ? <g stroke={c} strokeWidth="2.4" strokeLinecap="round"><path d="M1.6 10H18.4"/><path d="M2.73 5.8L17.27 14.2"/><path d="M5.8 2.73L14.2 17.27"/><path d="M10 1.6V18.4"/><path d="M14.2 2.73L5.8 17.27"/><path d="M17.27 5.8L2.73 14.2"/></g>
      : kind === 'codex'
        ? <><path d="M17.27 14.2L10 18.4L2.73 14.2V5.8L10 1.6L17.27 5.8Z" stroke={c} strokeWidth="2.2" strokeLinejoin="round"/><circle cx="10" cy="10" r="2.3" fill={c}/></>
        : <path d="M7.2 2.8H2.8V17.2H7.2M12.8 2.8H17.2V17.2H12.8" stroke={c} strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round"/>}
  </svg>;
}
