import { useEffect, useLayoutEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { AnimatePresence, motion, useAnimate, useReducedMotion } from 'motion/react';
import NumberFlow from '@number-flow/react';
import { Row } from './Row';
import { Ring } from './Ring';
import { Dropdown } from './Dropdown';
import { islandSize, type IslandState } from './layout';
import { springs } from './motion';
import { alertText, levelOf, tightestLeft, type AlertKind, type Level, type LimitWindow } from './status';
import { colors, levelColor } from './tokens';
import { emptySnapshot, windowFor, type Kind, type Snapshot } from './types';

type Pointer = { inside: boolean; down: boolean };
type Alert = { agent: Kind; window: LimitWindow; kind: AlertKind; title: string; body: string; level: 'warn' | 'crit' | 'info' };

/** The limit window an alert is about, so the row shows the matching number and ring. */
function alertWindow(snapshot: Snapshot, alert: Alert) {
  const account = alert.agent === 'claude' ? snapshot.claude : alert.agent === 'codex' ? snapshot.codex : null;
  return account ? (alert.window === 'weekly' ? account.weekly : account.fiveHour) : null;
}

const enter = { opacity: 0, scale: 0.9, y: -6, filter: 'blur(5px)' };
const shown = { opacity: 1, scale: 1, y: 0, filter: 'blur(0px)' };
const exit = { opacity: 0, scale: 0.95, filter: 'blur(4px)', transition: { duration: 0.1, ease: [0.16, 1, 0.3, 1] as const } };

export function Island() {
  const [snapshot, setSnapshot] = useState<Snapshot>(emptySnapshot);
  const [pointer, setPointer] = useState<Pointer>({ inside: false, down: false });
  const [open, setOpen] = useState(false);
  const [blocked, setBlocked] = useState(false);
  const [alert, setAlert] = useState<Alert | null>(null);
  const [panelHeight, setPanelHeight] = useState(300);
  const [update, setUpdate] = useState<string | null>(null);
  const panelRef = useRef<HTMLDivElement>(null);
  const openTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const closeTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const alertTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const reduced = useReducedMotion();
  const [shellRef, animateShell] = useAnimate<HTMLDivElement>();

  useEffect(() => {
    invoke<Snapshot>('get_snapshot').then(setSnapshot).catch(() => {});
    invoke<{ available: string | null }>('update_status').then(s => setUpdate(s.available)).catch(() => {});
    const subscriptions = [
      listen<Snapshot>('snapshot', e => setSnapshot(e.payload)),
      listen<Pointer>('pointer', e => setPointer(e.payload)),
      listen<string | null>('update', e => setUpdate(e.payload)),
      listen<Alert>('alert', e => {
        setAlert(e.payload);
        if (alertTimer.current) clearTimeout(alertTimer.current);
        alertTimer.current = setTimeout(() => setAlert(null), 4800);
      }),
    ];
    return () => { subscriptions.forEach(s => s.then(unlisten => unlisten())); if (alertTimer.current) clearTimeout(alertTimer.current); };
  }, []);

  // Hover = see-through; long hover (0.9 s) = dropdown; a press closes it until the pointer leaves.
  useEffect(() => {
    if (openTimer.current) clearTimeout(openTimer.current);
    if (closeTimer.current) clearTimeout(closeTimer.current);
    if (pointer.down && pointer.inside) { setOpen(false); setBlocked(true); return; }
    if (!pointer.inside) { closeTimer.current = setTimeout(() => { setOpen(false); setBlocked(false); }, 250); return; }
    if (!blocked && !open) openTimer.current = setTimeout(() => setOpen(true), 900);
    return () => { if (openTimer.current) clearTimeout(openTimer.current); if (closeTimer.current) clearTimeout(closeTimer.current); };
  }, [pointer, blocked, open]);

  const working = useMemo(() => snapshot.sessions.filter(s => s.activity === 'working'), [snapshot.sessions]);
  const alertSession = alert ? snapshot.sessions.find(s => s.kind === alert.agent) ?? null : null;
  const showAlert = !!alert && !!alertSession && !open;
  const state: IslandState = open ? 'dropdown' : showAlert ? 'alert' : pointer.inside ? 'hover' : working.length > 1 ? 'stacked' : working.length ? 'compact' : 'idle';

  useLayoutEffect(() => {
    if (open && panelRef.current) setPanelHeight(Math.ceil(panelRef.current.scrollHeight));
  }, [open, snapshot, update]);

  const size = islandSize(state, working.length, panelHeight);
  const opacity = state === 'alert' && pointer.inside ? 0.36 : size.opacity;
  const x = (460 - size.w) / 2;
  useEffect(() => { invoke('set_island_rect', { rect: { x, y: 0, w: size.w, h: size.h } }).catch(() => {}); }, [x, size.w, size.h]);

  // The keyline stays coloured while any limit is in a warning state, not only during the alert swell.
  const worst: Level = useMemo(() => {
    const levels = working.map(s => levelOf(windowFor(snapshot, s.kind)));
    return levels.includes('crit') ? 'crit' : levels.includes('warn') ? 'warn' : 'ok';
  }, [working, snapshot]);
  // Informational alerts (50% left) don't glow; amber/red are for warning and critical only.
  const glowLevel: Level = showAlert && alert!.level !== 'info' ? (alert!.level === 'crit' ? 'crit' : 'warn') : worst;
  const glow = glowLevel === 'crit' || glowLevel === 'warn' ? levelColor[glowLevel] : null;

  const grow = state === 'idle' ? springs.shrink : open ? springs.hoverOpen : state === 'alert' ? springs.alertIn : working.length > 1 ? springs.row : springs.grow;
  const transition = reduced ? { duration: 0.15 } : { type: 'spring' as const, ...grow };
  const tight = tightestLeft(snapshot);

  // Alert swell: one soft pulse when an alert arrives (colour alone under reduced motion).
  useEffect(() => {
    if (alert && !reduced && shellRef.current) animateShell(shellRef.current, { scale: [1, 1.04, 1] }, { duration: 0.45, ease: 'easeOut' });
  }, [alert, reduced, animateShell, shellRef]);

  let content: ReactNode;
  if (open) content = <Dropdown ref={panelRef} snapshot={snapshot} update={update}/>;
  else if (showAlert) {
    const w = alertWindow(snapshot, alert!);
    content = <Row session={alertSession!} window={w} primary label={alertText(alert!.kind, w, alert!.window) || alert!.title}/>;
  }
  else if (state === 'idle') content = <div className="idle-row">
    <span className="idle-dot"/>
    {tight && <span className="idle-limit"><NumberFlow value={Math.round(tight.leftPct)} suffix="%" respectMotionPreference/><Ring left={tight.leftPct} color={colors.stale} freshness={tight.freshness} size={14}/></span>}
  </div>;
  else content = <div className="rows">
    {working.slice(0, 4).map((s, i) => <Row key={s.id} session={s} window={windowFor(snapshot, s.kind)} primary={i === 0}/>)}
    {working.length > 4 && <div className="more-row">+{working.length - 4} more</div>}
  </div>;

  return <motion.div ref={shellRef} className={`island-shell ${open ? 'is-open' : ''}`} initial={false}
    animate={{ width: size.w, height: size.h, borderRadius: size.radius, opacity, left: x }}
    transition={transition}
    style={{ boxShadow: open ? undefined : glow ? `0 0 14px ${glow}8C, inset 0 0 0 1px ${glow}99` : undefined }}>
    <AnimatePresence mode="popLayout" initial={false}>
      <motion.div key={open ? 'dropdown' : showAlert ? 'alert' : state === 'idle' ? 'idle' : 'rows'} className="content"
        initial={reduced ? { opacity: 0 } : enter} animate={shown} exit={reduced ? { opacity: 0 } : exit}
        transition={reduced ? { duration: 0.15 } : { type: 'spring', ...springs.contentEnter }}>
        {content}
      </motion.div>
    </AnimatePresence>
  </motion.div>;
}
