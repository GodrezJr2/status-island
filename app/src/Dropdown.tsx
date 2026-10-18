import { AgentCard } from './AgentCard';
import type { Snapshot } from './types';

export function Dropdown({ snapshot }: { snapshot: Snapshot }) {
  return <div className="dropdown"><header><span className="brand-dot"/>Status Island <span className="dropdown-subtitle">Live activity</span></header>
    {snapshot.sessions.length ? snapshot.sessions.map(session => <AgentCard key={session.id} session={session} snapshot={snapshot}/>) : <p className="empty-copy">No active agent sessions yet.</p>}
    {snapshot.opencodeToday && <div className="card-note">OpenCode today · ${snapshot.opencodeToday.todayUsd.toFixed(2)}</div>}
  </div>;
}
