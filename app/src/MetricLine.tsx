/** Figma "Metric line": label · value · bar · detail. */
export function MetricLine({ label, value, detail, fraction, color }: { label: string; value: string; detail: string; fraction?: number | null; color?: string }) {
  return <div className="metric-line">
    <span className="metric-label">{label}</span>
    <strong className="metric-value" style={color ? { color } : undefined}>{value}</strong>
    {fraction !== undefined && fraction !== null
      ? <span className="metric-track"><span style={{ width: `${Math.max(0, Math.min(1, fraction)) * 100}%`, background: color }}/></span>
      : <span className="metric-track-gap"/>}
    <span className="metric-detail">{detail}</span>
  </div>;
}
