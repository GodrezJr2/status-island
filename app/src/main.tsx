import React, { useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Island } from './Island';
import { Connect } from './Connect';
import { Settings } from './Settings';
import './style.css';
import './panel.css';

function windowLabel(): string {
  try { return getCurrentWindow().label; } catch { return 'main'; }
}

/** The "connect" window hosts both the Connect and Settings views; the tray picks which one. */
function SettingsWindow() {
  const [view, setView] = useState<'connect' | 'settings'>('connect');
  useEffect(() => {
    const off = listen<string>('view', e => setView(e.payload === 'settings' ? 'settings' : 'connect'));
    return () => { off.then(f => f()); };
  }, []);
  return <div className="panel-window">
    <nav className="tabs">
      <button className={view === 'connect' ? 'active' : ''} onClick={() => setView('connect')}>Connect</button>
      <button className={view === 'settings' ? 'active' : ''} onClick={() => setView('settings')}>Settings</button>
    </nav>
    {view === 'connect' ? <Connect/> : <Settings/>}
  </div>;
}

const label = windowLabel();
if (label === 'connect') document.documentElement.classList.add('panel-root');
createRoot(document.getElementById('root')!).render(<React.StrictMode>{label === 'connect' ? <SettingsWindow/> : <Island/>}</React.StrictMode>);
