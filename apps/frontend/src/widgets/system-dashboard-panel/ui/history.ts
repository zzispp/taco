import type { ServerDashboard } from 'src/entities/system';

export type DashboardHistoryPoint = {
  label: string;
  cpu: number;
  memory: number;
  networkIn: number;
  networkOut: number;
};

const MAX_HISTORY_POINTS = 24;

export function appendHistory(
  current: DashboardHistoryPoint[],
  dashboard: ServerDashboard
): DashboardHistoryPoint[] {
  const next = [...current, toHistoryPoint(dashboard)];
  return next.slice(-MAX_HISTORY_POINTS);
}

function toHistoryPoint(dashboard: ServerDashboard): DashboardHistoryPoint {
  return {
    label: new Date(dashboard.sampled_at).toLocaleTimeString(),
    cpu: round(dashboard.cpu.total_usage_percent),
    memory: round(dashboard.memory.usage_percent),
    networkIn: round(dashboard.network.received_bytes_per_second),
    networkOut: round(dashboard.network.transmitted_bytes_per_second),
  };
}

function round(value: number) {
  return Number(value.toFixed(2));
}
