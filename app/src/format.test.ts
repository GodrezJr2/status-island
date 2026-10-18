import { describe, expect, it } from 'vitest';
import { fmtReset, fmtTokens } from './format';

describe('formatting', () => {
  it('shows reset time at useful precision', () => {
    expect(fmtReset(10_620_000, 0)).toBe('resets in 2h 57m');
    expect(fmtReset(329_000_000, 0)).toBe('resets in 3d 19h');
    expect(fmtReset(-1, 0)).toBe('resetting');
    expect(fmtReset(245_000, 0)).toBe('resets in 4m 05s');
  });
  it('formats token counts', () => {
    expect(fmtTokens(29_900)).toBe('29.9k');
    expect(fmtTokens(360_000)).toBe('360k');
    expect(fmtTokens(1_200_000)).toBe('1.2M');
  });
});
