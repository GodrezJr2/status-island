import { useEffect, useMemo, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { motion, useReducedMotion } from 'motion/react';
import { Row } from './Row';
import { Dropdown } from './Dropdown';
import { islandSize, type IslandState } from './layout';
import { springs } from './motion';
import { emptySnapshot, windowFor, type Snapshot } from './types';

type Pointer = { inside: boolean; down: boolean };
type Alert = { title: string; body: string; level: 'warn' | 'crit' | 'info' };

export function Island() {
  const [snapshot, setSnapshot] = useState<Snapshot>(emptySnapshot);
  const [pointer, setPointer] = useState<Pointer>({ inside: false, down: false });
  const [open, setOpen] = useState(false);
  const [blocked, setBlocked] = useState(false);
  const [alert, setAlert] = useState<Alert | null>(null);
  const openTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const closeTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const alertTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const reduced = useReducedMotion();

  useEffect(() => {
    invoke<Snapshot>('get_snapshot').then(setSnapshot).catch(() => {});
    const subscriptions = [
      listen<Snapshot>('snapshot', event => setSnapshot(event.payload)),
      listen<Pointer>('pointer', event => setPointer(event.payload)),
      listen<Alert>('alert', event => { setAlert(event.payload); if (alertTimer.current) clearTimeout(alertTimer.current); alertTimer.current = setTimeout(() => setAlert(null), 4800); }),
    ];
    return () => { subscriptions.forEach(s => s.then(unlisten => unlisten())); if (alertTimer.current) clearTimeout(alertTimer.current); };
  }, []);

  useEffect(() => {
    if (openTimer.current) clearTimeout(openTimer.current);
    if (closeTimer.current) clearTimeout(closeTimer.current);
    if (pointer.down && pointer.inside) { setOpen(false); setBlocked(true); return; }
    if (!pointer.inside) { closeTimer.current = setTimeout(() => { setOpen(false); setBlocked(false); }, 250); return; }
    if (!blocked && !open) openTimer.current = setTimeout(() => setOpen(true), 900);
    return () => { if (openTimer.current) clearTimeout(openTimer.current); if (closeTimer.current) clearTimeout(closeTimer.current); };
  }, [pointer, blocked, open]);

  const working = useMemo(() => snapshot.sessions.filter(s => s.activity === 'working'), [snapshot.sessions]);
  const rows = working.length ? working : snapshot.sessions.slice(0, 1);
  const state: IslandState = open ? 'dropdown' : pointer.inside ? 'hover' : working.length > 1 ? 'stacked' : working.length ? 'compact' : 'idle';
  const panelHeight = 48 + Math.max(1, snapshot.sessions.length) * 125 + 14;
  const size = islandSize(state, working.length, panelHeight);
  const x = (460 - size.w) / 2;
  useEffect(() => { invoke('set_island_rect', { rect: { x, y: 0, w: size.w, h: size.h } }).catch(() => {}); }, [x, size.w, size.h]);
  const transition = reduced ? { duration: 0.1 } : { type: 'spring' as const, ...(open ? springs.hoverOpen : state === 'idle' ? springs.shrink : springs.grow) };
  const alertColor = alert?.level === 'crit' ? '#FF453A' : alert?.level === 'warn' ? '#FFB020' : undefined;

  return <motion.div className={`island-shell ${open ? 'is-open' : ''}`} initial={false}
    animate={{ width: size.w, height: size.h, borderRadius: size.radius, opacity: size.opacity, left: x }} transition={transition}
    style={{ boxShadow: alertColor ? `0 0 14px ${alertColor}88, inset 0 0 0 1px ${alertColor}99` : undefined }}>
    {open ? <Dropdown snapshot={snapshot}/> : alert ? <div className="alert-row"><span className="alert-dot" style={{ background: alertColor }}/><strong>{alert.title}</strong></div>
      : state === 'idle' ? <div className="idle-row"><span className="brand-dot"/><strong>Status Island</strong></div>
      : <div className="rows">{rows.slice(0, 4).map((session, index) => <Row key={session.id} session={session} window={windowFor(snapshot, session.kind)} primary={index === 0}/>)}{working.length > 4 && <div className="more-row">+{working.length - 4} more</div>}</div>}
  </motion.div>;
}
