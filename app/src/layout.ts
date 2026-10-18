export type IslandState = 'idle' | 'compact' | 'stacked' | 'hover' | 'dropdown';

export function islandSize(state: IslandState, workingCount: number, dropdownHeight = 300) {
  if (state === 'idle') return { w: 132, h: 30, radius: 15, opacity: 0.63 };
  const extras = Math.max(0, Math.min(workingCount - 1, 3));
  const more = workingCount > 4 ? 20 : 0;
  const h = workingCount > 1 ? 34 + extras * 26 + more + 6 : 34;
  if (state === 'dropdown') return { w: 400, h: dropdownHeight, radius: Math.min(dropdownHeight / 2, 26), opacity: 1 };
  return { w: 252, h, radius: h / 2, opacity: state === 'hover' ? 0.36 : 0.95 };
}
