export function MetricLine({ label, value, detail, left }: { label: string; value: string; detail: string; left?: number | null }) {
  return <div className="metric-line"><span className="metric-label">{label}</span><span className="metric-track">{left !== undefined && left !== null && <span style={{ width: `${Math.max(0, Math.min(100, left))}%` }}/>}</span><strong>{value}</strong><span className="metric-detail">{detail}</span></div>;
}
