import { motion } from 'motion/react';
import { springs } from './motion';
import type { Freshness } from './types';
import { colors } from './tokens';

/** Limit-left ring: arc from 12 o'clock, clockwise. Estimated readings are dotted (spec §6.2). */
export function Ring({ left, color, freshness, size = 18 }: { left: number | null; color: string; freshness?: Freshness; size?: number }) {
  const radius = 8.7; // outer edge 10, inner 7.4 = Figma ring (innerRadius 0.74)
  const length = 2 * Math.PI * radius;
  const fraction = Math.max(0, Math.min(100, left ?? 0)) / 100;
  return <svg width={size} height={size} viewBox="0 0 20 20" className="ring" aria-label={left === null ? 'limit unknown' : `${Math.round(left)}% left`} style={{ flex: 'none' }}>
    <circle cx="10" cy="10" r={radius} stroke={colors.track} strokeWidth="2.6" fill="none"/>
    <motion.circle cx="10" cy="10" r={radius} stroke={color} strokeWidth="2.6" fill="none"
      strokeDasharray={freshness === 'estimated' ? '1.6 1.6' : `${length}`}
      initial={false} animate={{ strokeDashoffset: freshness === 'estimated' ? 0 : length * (1 - fraction), opacity: freshness === 'estimated' ? 0.6 : 1 }}
      transition={{ type: 'spring', ...springs.ring }} transform="rotate(-90 10 10)"/>
  </svg>;
}
