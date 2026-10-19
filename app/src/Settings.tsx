import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { availableMonitors } from '@tauri-apps/api/window';

export type Config = {
  monitor: string; offsetX: number; quietHours: [number, number] | null; onboarded: boolean;
  alertsPausedUntilMs: number | null; ntfy: { server: string; topic: string; token: string | null };
  prefs: { contextHigh: boolean; taskFinished: boolean }; checkUpdates: boolean;
};
type UpdateStatus = { current: string; available: string | null };

const hours = Array.from({ length: 24 }, (_, h) => h);
const hourLabel = (h: number) => new Date(2026, 0, 1, h).toLocaleTimeString('en-US', { hour: 'numeric' });

/** Settings (plan Task 12): island position, quiet hours, pause, phone server, Claude connector, updates. */
export function Settings() {
  const [cfg, setCfg] = useState<Config | null>(null);
  const [monitors, setMonitors] = useState<string[]>([]);
  const [claudeOn, setClaudeOn] = useState(false);
  const [busy, setBusy] = useState(false);
  const [note, setNote] = useState<string | null>(null);
  const [updates, setUpdates] = useState<UpdateStatus | null>(null);
  const [updateNote, setUpdateNote] = useState<string | null>(null);

  useEffect(() => {
    invoke<Config>('get_config').then(setCfg).catch(e => setNote(String(e)));
    invoke<boolean>('connector_status').then(setClaudeOn).catch(() => {});
    invoke<UpdateStatus>('update_status').then(setUpdates).catch(() => {});
    availableMonitors().then(ms => setMonitors(ms.map(m => m.name).filter((n): n is string => !!n))).catch(() => {});
  }, []);

  function update(next: Config) {
    setCfg(next);
    invoke('set_config', { cfg: next }).catch(e => setNote(String(e)));
  }

  async function turnOffClaude() {
    setBusy(true);
    try { await invoke('disable_claude'); setClaudeOn(false); setNote('Claude settings restored.'); }
    catch (e) { setNote(String(e)); } finally { setBusy(false); }
  }

  async function checkOrInstall() {
    if (!updates) return;
    setBusy(true); setUpdateNote(null);
    try {
      if (updates.available) { setUpdateNote('Installing… Status Island restarts when it finishes.'); await invoke('install_update'); }
      else {
        const available = await invoke<string | null>('check_update');
        setUpdates({ ...updates, available });
        if (!available) setUpdateNote('You have the latest version.');
      }
    } catch (e) { setUpdateNote(String(e)); } finally { setBusy(false); }
  }

  if (!cfg) return <div className="panel"><p className="lede">Loading…</p></div>;
  const paused = cfg.alertsPausedUntilMs !== null && cfg.alertsPausedUntilMs > Date.now();

  return <div className="panel settings">
    <h1>Settings</h1>

    <section><h2>Island position</h2>
      <label>Screen
        <select value={cfg.monitor} onChange={e => update({ ...cfg, monitor: e.target.value })}>
          <option value="active">Follow the island's current screen</option>
          {monitors.map(m => <option key={m} value={m}>{m}</option>)}
        </select>
      </label>
      <label>Nudge left / right <span className="value">{cfg.offsetX > 0 ? '+' : ''}{cfg.offsetX} px</span>
        <input type="range" min={-300} max={300} step={10} value={cfg.offsetX} onChange={e => update({ ...cfg, offsetX: Number(e.target.value) })}/>
      </label>
    </section>

    <section><h2>Phone alerts</h2>
      <label className="check"><input type="checkbox" checked={cfg.quietHours !== null} onChange={e => update({ ...cfg, quietHours: e.target.checked ? [22, 7] : null })}/> Quiet hours</label>
      {cfg.quietHours && <div className="inline">
        <select value={cfg.quietHours[0]} onChange={e => update({ ...cfg, quietHours: [Number(e.target.value), cfg.quietHours![1]] })}>{hours.map(h => <option key={h} value={h}>{hourLabel(h)}</option>)}</select>
        <span>to</span>
        <select value={cfg.quietHours[1]} onChange={e => update({ ...cfg, quietHours: [cfg.quietHours![0], Number(e.target.value)] })}>{hours.map(h => <option key={h} value={h}>{hourLabel(h)}</option>)}</select>
      </div>}
      <button className="secondary" onClick={() => update({ ...cfg, alertsPausedUntilMs: paused ? null : Date.now() + 3_600_000 })}>{paused ? 'Resume alerts' : 'Pause alerts for 1 hour'}</button>
      <label>ntfy server <span className="hint">(change only if you self-host)</span>
        <input type="url" value={cfg.ntfy.server} onChange={e => update({ ...cfg, ntfy: { ...cfg.ntfy, server: e.target.value } })}/>
      </label>
      <label>Access token <span className="hint">(optional, for a private server)</span>
        <input type="password" value={cfg.ntfy.token ?? ''} onChange={e => update({ ...cfg, ntfy: { ...cfg.ntfy, token: e.target.value || null } })}/>
      </label>
    </section>

    <section><h2>Claude connector</h2>
      <p className="lede">{claudeOn ? 'On. Your VS Code and Claude settings were backed up before the change.' : 'Off. Turn it on from the Connect screen.'}</p>
      {claudeOn && <button className="secondary" disabled={busy} onClick={turnOffClaude}>Turn off and restore my settings</button>}
    </section>

    <section><h2>Updates</h2>
      <p className="lede">{updates ? (updates.available ? `Version ${updates.available} is ready. You have ${updates.current}.` : `Version ${updates.current}.`) : ' '}</p>
      <label className="check"><input type="checkbox" checked={cfg.checkUpdates} onChange={e => update({ ...cfg, checkUpdates: e.target.checked })}/> Check for updates automatically</label>
      <button className="secondary" disabled={busy || !updates} onClick={checkOrInstall}>{updates?.available ? `Install update ${updates.available}` : 'Check now'}</button>
      {updateNote && <p className="footnote">{updateNote}</p>}
    </section>

    {note && <p className="footnote">{note}</p>}
  </div>;
}
