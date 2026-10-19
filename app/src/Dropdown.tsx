import { forwardRef, useEffect, useState } from 'react';
import { AgentCard } from './AgentCard';
import type { Snapshot } from './types';

/** Long-hover dropdown (Figma "Island · dropdown"): one card per agent session, nothing else.
 *  Runs its own 1 s clock while open, so countdowns tick without the engine rescanning every second. */
export const Dropdown = forwardRef<HTMLDivElement, { snapshot: Snapshot }>(function Dropdown({ snapshot }, ref) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => { const t = setInterval(() => setNow(Date.now()), 1000); return () => clearInterval(t); }, []);
  return <div className="dropdown" ref={ref}>
    {snapshot.sessions.length
      ? snapshot.sessions.map(session => <AgentCard key={session.id} session={session} snapshot={snapshot} now={now}/>)
      : <p className="empty-copy">No agent sessions right now.</p>}
  </div>;
});
