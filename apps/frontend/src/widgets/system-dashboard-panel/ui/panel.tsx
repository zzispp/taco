'use client';

import type { TranslateFn } from 'src/shared/i18n';
import type { SummaryCardProps } from './summary-card';
import type { ServerDashboard } from 'src/entities/system';

import { useMemo, useState, useEffect } from 'react';

import Grid from '@mui/material/Grid';
import Alert from '@mui/material/Alert';
import Skeleton from '@mui/material/Skeleton';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { useServerDashboard } from 'src/entities/system';

import { appendHistory } from './history';
import { SummaryCard } from './summary-card';
import { HealthCard, ServerInfoCard } from './info-cards';
import { formatBytes, formatPercent, formatBandwidth } from './format';
import { CpuCoresCard, TopProcessesCard, DiskPartitionsCard } from './tables';
import { DiskUsageChart, NetworkTrendChart, ResourceTrendChart } from './charts';

const LOADING_CARD_COUNT = 8;

export function SystemDashboardPanel() {
  const dashboard = useServerDashboard();
  const dashboardData = dashboard.data;
  const [history, setHistory] = useState<ReturnType<typeof appendHistory>>([]);

  useEffect(() => {
    if (dashboardData) {
      setHistory((current) => appendHistory(current, dashboardData));
    }
  }, [dashboardData]);

  if (dashboard.error) {
    return <DashboardError message={dashboard.error.message} />;
  }

  if (!dashboardData) {
    return <DashboardLoading />;
  }

  return <DashboardLoaded dashboard={dashboardData} history={history} />;
}

function DashboardLoaded({
  dashboard,
  history,
}: {
  dashboard: ServerDashboard;
  history: ReturnType<typeof appendHistory>;
}) {
  const { t } = useTranslate('admin');
  const cards = useMemo(() => summaryCards(dashboard, history, t), [dashboard, history, t]);

  return (
    <>
      <Typography variant="h4" sx={{ mb: { xs: 3, md: 5 } }}>
        {t('systemDashboard.title')}
      </Typography>
      <Grid container spacing={3}>
        {cards.map((card) => (
          <Grid key={card.title} size={{ xs: 12, sm: 6, md: 3 }}>
            <SummaryCard {...card} />
          </Grid>
        ))}
        <DashboardGrid dashboard={dashboard} history={history} />
      </Grid>
    </>
  );
}

function DashboardGrid(props: {
  dashboard: ServerDashboard;
  history: ReturnType<typeof appendHistory>;
}) {
  const { dashboard, history } = props;
  return (
    <>
      <Grid size={{ xs: 12, md: 6 }}>
        <ServerInfoCard dashboard={dashboard} />
      </Grid>
      <Grid size={{ xs: 12, md: 6 }}>
        <HealthCard dashboard={dashboard} />
      </Grid>
      <Grid size={{ xs: 12, lg: 8 }}>
        <ResourceTrendChart history={history} />
      </Grid>
      <Grid size={{ xs: 12, lg: 4 }}>
        <DiskUsageChart dashboard={dashboard} />
      </Grid>
      <Grid size={{ xs: 12, lg: 8 }}>
        <NetworkTrendChart history={history} />
      </Grid>
      <Grid size={{ xs: 12, lg: 4 }}>
        <CpuCoresCard dashboard={dashboard} />
      </Grid>
      <Grid size={{ xs: 12, lg: 8 }}>
        <TopProcessesCard dashboard={dashboard} />
      </Grid>
      <Grid size={{ xs: 12, lg: 4 }}>
        <DiskPartitionsCard dashboard={dashboard} />
      </Grid>
    </>
  );
}

function DashboardError({ message }: { message: string }) {
  return <Alert severity="error">{message}</Alert>;
}

function DashboardLoading() {
  return (
    <>
      <Skeleton variant="text" width={240} height={56} />
      <Grid container spacing={3}>
        {Array.from({ length: LOADING_CARD_COUNT }).map((_, index) => (
          <Grid key={index} size={{ xs: 12, md: 3 }}>
            <Skeleton variant="rounded" height={220} />
          </Grid>
        ))}
      </Grid>
    </>
  );
}

function summaryCards(
  dashboard: ServerDashboard,
  history: ReturnType<typeof appendHistory>,
  t: TranslateFn
): SummaryCardProps[] {
  const categories = history.map((item) => item.label);
  const diskUsage = maxDiskUsage(dashboard);
  return [
    summary({
      title: t('systemDashboard.summary.cpuUsage'),
      value: formatPercent(dashboard.cpu.total_usage_percent),
      percent: dashboard.cpu.total_usage_percent,
      icon: 'solar:monitor-bold',
      series: history.map((item) => item.cpu),
    }),
    summary({
      title: t('systemDashboard.summary.memoryUsage'),
      value: formatBytes(dashboard.memory.used_bytes),
      percent: dashboard.memory.usage_percent,
      icon: 'solar:ssd-round-bold',
      series: history.map((item) => item.memory),
      color: 'warning',
    }),
    summary({
      title: t('systemDashboard.summary.diskUsage'),
      value: formatBytes(totalDiskUsed(dashboard)),
      percent: diskUsage,
      icon: 'solar:box-minimalistic-bold',
      series: [diskUsage],
      color: 'error',
    }),
    summary({
      title: t('systemDashboard.summary.networkInbound'),
      value: formatBandwidth(dashboard.network.received_bytes_per_second),
      percent: 0,
      icon: 'ic:baseline-wifi',
      series: history.map((item) => item.networkIn),
      color: 'info',
    }),
  ].map((card) => ({ ...card, chart: { ...card.chart, categories } }));
}

function summary(options: SummaryOptions): SummaryCardProps {
  return {
    title: options.title,
    value: options.value,
    percent: options.percent,
    icon: options.icon,
    color: options.color ?? 'primary',
    chart: { categories: [], series: options.series },
  };
}

type SummaryOptions = {
  title: string;
  value: string;
  percent: number;
  icon: SummaryCardProps['icon'];
  series: number[];
  color?: SummaryCardProps['color'];
};

function maxDiskUsage(dashboard: ServerDashboard) {
  return Math.max(0, ...dashboard.disks.map((disk) => disk.usage_percent));
}

function totalDiskUsed(dashboard: ServerDashboard) {
  return dashboard.disks.reduce((sum, disk) => sum + disk.used_bytes, 0);
}
