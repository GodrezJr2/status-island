import { forwardRef } from 'react';
import { AgentCard } from './AgentCard';
import type { Snapshot } from './types';

/** Long-hover dropdown (Figma "Island · dropdown"): one card per agent session, nothing else. */
export const Dropdown = forwardRef<HTMLDivElement, { snapshot: Snapshot }>(function Dropdown({ snapshot }, ref) {
  return <div className="dropdown" ref={ref}>
    {snapshot.sessions.length
      ? snapshot.sessions.map(session => <AgentCard key={session.id} session={session} snapshot={snapshot}/>)
      : <p className="empty-copy">No agent sessions right now.</p>}
  </div>;
});
