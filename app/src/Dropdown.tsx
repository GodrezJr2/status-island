import { forwardRef, useEffect, useState } from 'react';
import { AgentCard } from './AgentCard';
import type { Snapshot } from './types';

/** Long-hover dropdown (Figma "Island · dropdown"): one card per agent session, plus a one-line
 *  notice when an update is ready. Runs its own 1 s clock while open, so countdowns tick without
 *  the engine rescanning every second. */
export const Dropdown = forwardRef<HTMLDivElement, { snapshot: Snapshot; update: string | null }>(function Dropdown({ snapshot, update }, ref) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => { const t = setInterval(() => setNow(Date.now()), 1000); return () => clearInterval(t); }, []);
  return <div className="dropdown" ref={ref}>
    {snapshot.sessions.length
      ? snapshot.sessions.map(session => <AgentCard key={session.id} session={session} snapshot={snapshot} now={now}/>)
      : <p className="empty-copy">No agent sessions right now.</p>}
    {update && <p className="update-note">Update {update} is ready · install it from the tray icon</p>}
  </div>;
});
