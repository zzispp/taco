import type { TranslateFn } from 'src/shared/i18n';
import type { DashboardHistoryPoint } from './history';
import type { ServerDashboard } from 'src/entities/system';

import Card from '@mui/material/Card';
import { useTheme } from '@mui/material/styles';
import CardHeader from '@mui/material/CardHeader';

import { Chart, useChart } from 'src/shared/ui/chart';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { formatBytes, formatPercent, formatBandwidth } from './format';

export function ResourceTrendChart({ history }: { history: DashboardHistoryPoint[] }) {
  const theme = useTheme();
  const { t } = useTranslate('admin');
  const chartOptions = useChart({
    colors: [theme.palette.primary.main, theme.palette.warning.main],
    xaxis: { categories: history.map((item) => item.label) },
    yaxis: { labels: { formatter: formatPercent } },
    tooltip: { y: { formatter: formatPercent } },
  });

  return (
    <Card>
      <CardHeader
        title={t('systemDashboard.charts.resourceTrend')}
        subheader={t('systemDashboard.charts.realtimeSessionSampling')}
      />
      <Chart
        type="line"
        series={resourceSeries(history, t)}
        options={chartOptions}
        sx={{ p: 2.5, height: 360 }}
      />
    </Card>
  );
}

export function NetworkTrendChart({ history }: { history: DashboardHistoryPoint[] }) {
  const theme = useTheme();
  const { t } = useTranslate('admin');
  const chartOptions = useChart({
    colors: [theme.palette.info.main, theme.palette.success.main],
    xaxis: { categories: history.map((item) => item.label) },
    tooltip: { y: { formatter: formatBandwidth } },
  });

  return (
    <Card>
      <CardHeader
        title={t('systemDashboard.charts.networkTrend')}
        subheader={t('systemDashboard.charts.inboundOutboundBandwidth')}
      />
      <Chart
        type="area"
        series={networkSeries(history, t)}
        options={chartOptions}
        sx={{ p: 2.5, height: 360 }}
      />
    </Card>
  );
}

export function DiskUsageChart({ dashboard }: { dashboard: ServerDashboard }) {
  const theme = useTheme();
  const { t } = useTranslate('admin');
  const chartOptions = useChart({
    labels: dashboard.disks.map((disk) => disk.mount_point),
    colors: [theme.palette.primary.main, theme.palette.warning.main, theme.palette.error.main],
    tooltip: { y: { formatter: formatBytes } },
    plotOptions: { pie: { donut: { labels: donutLabels() } } },
  });

  return (
    <Card>
      <CardHeader
        title={t('systemDashboard.charts.diskUsage')}
        subheader={t('systemDashboard.charts.usedCapacity')}
      />
      <Chart
        type="donut"
        series={dashboard.disks.map((disk) => disk.used_bytes)}
        options={chartOptions}
        sx={{ p: 2.5, height: 360 }}
      />
    </Card>
  );
}

function resourceSeries(history: DashboardHistoryPoint[], t: TranslateFn) {
  return [
    { name: t('systemDashboard.charts.series.cpu'), data: history.map((item) => item.cpu) },
    { name: t('systemDashboard.charts.series.memory'), data: history.map((item) => item.memory) },
  ];
}

function networkSeries(history: DashboardHistoryPoint[], t: TranslateFn) {
  return [
    {
      name: t('systemDashboard.charts.series.inbound'),
      data: history.map((item) => item.networkIn),
    },
    {
      name: t('systemDashboard.charts.series.outbound'),
      data: history.map((item) => item.networkOut),
    },
  ];
}

function donutLabels() {
  return {
    show: true,
    value: { formatter: formatDonutBytes },
    total: { formatter: formatDonutTotal },
  };
}

function formatDonutBytes(value: string) {
  return formatBytes(Number(value));
}

function formatDonutTotal(context: { globals: { seriesTotals: number[] } }) {
  return formatBytes(context.globals.seriesTotals.reduce((sum, value) => sum + value, 0));
}
