import type { Kind } from './types';
import { colors } from './tokens';

export function Glyph({ kind, size = 18 }: { kind: Kind; size?: number }) {
  const color = colors[kind];
  return <svg width={size} height={size} viewBox="0 0 24 24" fill="none" aria-hidden="true" style={{ flex: 'none' }}>
    {kind === 'claude' ? <><path d="M12 2.5v19M2.5 12h19M5.3 5.3l13.4 13.4M18.7 5.3 5.3 18.7" stroke={color} strokeWidth="2.1" strokeLinecap="round"/><circle cx="12" cy="12" r="2.2" fill={color}/></>
      : kind === 'codex' ? <><path d="M12 2.7 20.3 7.5v9L12 21.3l-8.3-4.8v-9L12 2.7Z" stroke={color} strokeWidth="2"/><path d="m8 12 2.6 2.6L16.3 9" stroke={color} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/></>
      : <><path d="M4 17.5V6.5L12 3l8 3.5v11L12 21l-8-3.5Z" stroke={color} strokeWidth="2"/><path d="m8 15 4-6 4 6" stroke={color} strokeWidth="2" strokeLinecap="round"/></>}
  </svg>;
}
