export type IslandState = 'idle' | 'compact' | 'stacked' | 'hover' | 'dropdown' | 'alert';

/** Sizes and opacities from the approved Figma design (docs/design/tokens.md). */
export function islandSize(state: IslandState, workingCount: number, dropdownHeight = 300) {
  if (state === 'idle') return { w: 132, h: 30, radius: 15, opacity: 0.63 };
  if (state === 'alert') return { w: 320, h: 34, radius: 17, opacity: 1 };
  if (state === 'dropdown') return { w: 400, h: dropdownHeight, radius: Math.min(dropdownHeight / 2, 26), opacity: 1 };
  const extras = Math.max(0, Math.min(workingCount - 1, 3));
  const more = workingCount > 4 ? 20 : 0;
  const h = workingCount > 1 ? 34 + extras * 26 + more + 6 : 34;
  return { w: 252, h, radius: workingCount > 1 ? 22 : 17, opacity: state === 'hover' ? 0.36 : 0.95 };
}
