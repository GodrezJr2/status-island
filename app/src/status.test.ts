import { describe, expect, it } from 'vitest';
import { agentName, alertText, levelOf, tightestLeft } from './status';
import type { Snapshot, WindowView } from './types';
import { emptySnapshot } from './types';

const win = (leftPct: number, burnPtsPerMin = 0, freshness: WindowView['freshness'] = 'exact'): WindowView => ({
  usedPct: 100 - leftPct, leftPct, resetsAtMs: Date.UTC(2026, 9, 19, 9, 40), freshness, ageMs: 4000,
  burnPtsPerMin, minutesToEmpty: null, usedElsewherePts: null, estimated: false,
});

describe('levelOf', () => {
  it('maps limit left and burn to status levels (spec §6.2)', () => {
    expect(levelOf(win(64))).toBe('ok');
    expect(levelOf(win(20))).toBe('warn');
    expect(levelOf(win(64, 2))).toBe('warn');
    expect(levelOf(win(10))).toBe('crit');
    expect(levelOf(win(0))).toBe('crit');
    expect(levelOf(win(64, 0, 'stale'))).toBe('stale');
    expect(levelOf(null)).toBe('ok');
  });
});

describe('alertText', () => {
  it('uses the approved alert wording on the island', () => {
    expect(alertText('fastDrain', win(38, 2.4))).toBe('Draining fast · 2.4%/min');
    expect(alertText('left20', win(20))).toBe('20% left');
    expect(alertText('left10', win(10))).toBe('10% left');
    expect(alertText('limitReached', win(0))).toMatch(/^Limit reached · resets \d{1,2}:\d{2} (AM|PM)$/);
  });
});

describe('tightestLeft', () => {
  it('picks the smallest exact-or-labelled limit left across agents', () => {
    const s: Snapshot = { ...emptySnapshot, claude: { fiveHour: win(64), weekly: win(52), overageDisabled: false }, codex: { fiveHour: win(85), weekly: win(98), overageDisabled: false } };
    expect(tightestLeft(s)?.leftPct).toBe(52);
    expect(tightestLeft(emptySnapshot)).toBeNull();
  });
});

describe('agentName', () => {
  it('names agents as in the design', () => {
    expect(agentName('claude')).toBe('Claude Code');
    expect(agentName('codex')).toBe('Codex');
    expect(agentName('openCode')).toBe('OpenCode');
  });
});
