import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Glyph } from './Glyph';
import { emptySnapshot, type Snapshot } from './types';

type Pairing = { subscribeUrl: string; qrSvg: string };

function Toggle({ on, busy, onClick, label }: { on: boolean; busy: boolean; onClick: () => void; label: string }) {
  return <button className={`toggle ${on ? 'on' : ''}`} role="switch" aria-checked={on} aria-label={label} disabled={busy} onClick={onClick}><span/></button>;
}

function Chip({ live }: { live: boolean }) {
  return <span className={`chip ${live ? 'working' : 'idle'}`}>{live && <span className="chip-dot"/>}{live ? 'Live' : 'Not found'}</span>;
}

/** First-run "Connect your agents" screen (Figma "Connect screen"), with a live self-check per agent. */
export function Connect() {
  const [snapshot, setSnapshot] = useState<Snapshot>(emptySnapshot);
  const [claudeOn, setClaudeOn] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pairing, setPairing] = useState<Pairing | null>(null);
  const [test, setTest] = useState<'idle' | 'sending' | 'sent' | string>('idle');

  useEffect(() => {
    invoke<Snapshot>('get_snapshot').then(setSnapshot).catch(() => {});
    invoke<boolean>('connector_status').then(setClaudeOn).catch(() => {});
    invoke<Pairing>('pairing').then(setPairing).catch(e => setError(String(e)));
    const off = listen<Snapshot>('snapshot', e => setSnapshot(e.payload));
    return () => { off.then(f => f()); };
  }, []);

  async function toggleClaude() {
    setBusy(true); setError(null);
    try {
      await invoke(claudeOn ? 'disable_claude' : 'enable_claude');
      setClaudeOn(!claudeOn);
    } catch (e) { setError(String(e)); } finally { setBusy(false); }
  }

  async function sendTest() {
    setTest('sending');
    try { await invoke('send_test_push'); setTest('sent'); } catch (e) { setTest(String(e)); }
  }

  const claudeLive = !!snapshot.claude.fiveHour && snapshot.claude.fiveHour.freshness === 'exact';
  // Claude tabs opened before the switch was turned on can't report; that only needs fixing once.
  const claudeText = !claudeOn ? 'Live limits in terminal and VS Code'
    : claudeLive ? 'Live limits in terminal and VS Code · live ✓'
    : 'On · reload VS Code once (Ctrl+Shift+P → Reload Window) so open Claude tabs report';
  const codexLive = !!snapshot.codex.fiveHour || snapshot.sessions.some(s => s.kind === 'codex');
  const openCodeLive = !!snapshot.opencodeToday || snapshot.sessions.some(s => s.kind === 'openCode');

  return <div className="panel">
    <h1>Connect your agents</h1>
    <p className="lede">One switch each. Everything stays on this PC.</p>

    <div className="connect-row"><Glyph kind="claude" size={20}/><div className="row-text"><strong>Claude Code</strong><span>{claudeText}</span></div>
      <Toggle on={claudeOn} busy={busy} onClick={toggleClaude} label="Claude Code live limits"/></div>
    <div className="connect-row"><Glyph kind="codex" size={20}/><div className="row-text"><strong>Codex</strong><span>{codexLive ? 'Detected · read automatically' : 'Not found on this PC'}</span></div><Chip live={codexLive}/></div>
    <div className="connect-row"><Glyph kind="openCode" size={20}/><div className="row-text"><strong>OpenCode</strong><span>{openCodeLive ? 'Detected · tokens and spend' : 'Not found on this PC'}</span></div><Chip live={openCodeLive}/></div>

    <div className="connect-row phone">
      <div className="qr" aria-label="Phone pairing QR code" dangerouslySetInnerHTML={pairing ? { __html: pairing.qrSvg } : undefined}/>
      <div className="row-text"><strong>Phone alerts</strong><span className="wrap">Scan with the free ntfy app (Android / iPhone). No account needed.</span>
        <button className="link" disabled={test === 'sending' || !pairing} onClick={sendTest}>{test === 'sending' ? 'Sending…' : test === 'sent' ? 'Test sent ✓' : 'Send test'}</button>
        {test !== 'idle' && test !== 'sending' && test !== 'sent' && <span className="error">{test}</span>}
      </div>
    </div>

    {error && <p className="error">{error}</p>}
    <p className="footnote">Never reads your login · never sends requests with your account · undo anytime</p>
    <button className="primary" onClick={() => invoke('finish_onboarding')}>Done</button>
  </div>;
}
