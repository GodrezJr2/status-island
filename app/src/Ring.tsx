import { motion } from 'motion/react';
import { springs } from './motion';
import type { Freshness } from './types';
import { colors } from './tokens';

export function Ring({ left, freshness, size = 18 }: { left: number | null; freshness?: Freshness; size?: number }) {
  const radius = 7;
  const length = 2 * Math.PI * radius;
  const level = left === null ? colors.secondary : left <= 10 ? colors.crit : left <= 20 ? colors.warn : colors.text;
  return <svg width={size} height={size} viewBox="0 0 20 20" className="ring" aria-label={left === null ? 'limit unknown' : `${Math.round(left)}% left`}>
    <circle cx="10" cy="10" r={radius} stroke="rgba(255,255,255,.16)" strokeWidth="3.5" fill="none"/>
    <motion.circle cx="10" cy="10" r={radius} stroke={level} strokeWidth="3.5" fill="none" strokeLinecap="round" strokeDasharray={length} animate={{ strokeDashoffset: length * (1 - Math.max(0, Math.min(100, left ?? 0)) / 100) }} transition={{ type: 'spring', ...springs.ring }} transform="rotate(-90 10 10)" style={freshness === 'estimated' ? { strokeDasharray: '2 2' } : undefined}/>
  </svg>;
}
