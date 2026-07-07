import type { TranslateFn } from 'src/shared/i18n';
import type { ServerDashboard, ServerHealthStatus } from 'src/entities/system';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { Label } from 'src/shared/ui/label';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { formatBytes, formatDateTime, formatDuration, formatUnixSeconds } from './format';

const HEALTH_COLOR = { green: 'success', yellow: 'warning', red: 'error' } as const;

export function ServerInfoCard({ dashboard }: { dashboard: ServerDashboard }) {
  const { t } = useTranslate('admin');
  const { host } = dashboard;
  return (
    <Card sx={{ p: 3, height: 1 }}>
      <Typography variant="h6">{t('systemDashboard.cards.serverInfo')}</Typography>
      <Stack divider={<Divider flexItem />} spacing={1.5} sx={{ mt: 2 }}>
        <InfoLine label={t('systemDashboard.labels.hostname')} value={host.hostname ?? '-'} />
        <InfoLine label={t('systemDashboard.labels.system')} value={systemText(host)} />
        <InfoLine label={t('systemDashboard.labels.kernel')} value={host.kernel_version ?? '-'} />
        <InfoLine label={t('systemDashboard.labels.cpu')} value={host.cpu_brand ?? host.cpu_arch} />
        <InfoLine label={t('systemDashboard.labels.cpuCores')} value={cpuCoreText(host, t)} />
        <InfoLine label={t('systemDashboard.labels.memoryDisk')} value={memoryDiskText(host)} />
        <InfoLine
          label={t('systemDashboard.labels.publicIp')}
          value={host.public_ips.join(', ') || '-'}
        />
      </Stack>
    </Card>
  );
}

export function HealthCard({ dashboard }: { dashboard: ServerDashboard }) {
  const { t, currentLang } = useTranslate('admin');
  const locale = currentLang.numberFormat.code;
  return (
    <Card sx={{ p: 3, height: 1 }}>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Typography variant="h6">{t('systemDashboard.cards.healthStatus')}</Typography>
        <HealthLabel
          label={t(`systemDashboard.health.${dashboard.health.status}`)}
          status={dashboard.health.status}
        />
      </Box>
      <Stack spacing={1.5} sx={{ mt: 3 }}>
        <InfoLine
          label={t('systemDashboard.labels.uptime')}
          value={formatDuration(dashboard.host.uptime_seconds, durationLabels(t))}
        />
        <InfoLine
          label={t('systemDashboard.labels.bootTime')}
          value={formatUnixSeconds(dashboard.host.boot_time_unix_seconds, locale)}
        />
        <InfoLine
          label={t('systemDashboard.labels.sampledAt')}
          value={formatDateTime(dashboard.sampled_at, locale)}
        />
        <InfoLine label={t('systemDashboard.labels.load')} value={loadText(dashboard)} />
      </Stack>
      <Divider sx={{ my: 2 }} />
      <Typography variant="body2" color="text.secondary">
        {healthIssuesText(dashboard, t)}
      </Typography>
    </Card>
  );
}

function HealthLabel({ label, status }: { label: string; status: ServerHealthStatus }) {
  return <Label color={HEALTH_COLOR[status]}>{label}</Label>;
}

function InfoLine({ label, value }: { label: string; value: string }) {
  return (
    <Box sx={{ display: 'flex', gap: 2, justifyContent: 'space-between' }}>
      <Typography variant="body2" color="text.secondary">
        {label}
      </Typography>
      <Typography variant="body2" sx={{ textAlign: 'right' }}>
        {value}
      </Typography>
    </Box>
  );
}

function healthIssuesText(dashboard: ServerDashboard, t: TranslateFn) {
  if (!dashboard.health.issues.length) {
    return t('systemDashboard.health.allHealthy');
  }
  return dashboard.health.issues
    .map((issue) => issueText(issue, t))
    .join(t('systemDashboard.health.issueSeparator'));
}

function issueText(issue: ServerDashboard['health']['issues'][number], t: TranslateFn) {
  return t('systemDashboard.health.issue', {
    kind: t(`systemDashboard.health.kinds.${issue.kind}`),
    target: issue.target,
    usage: issue.usage_percent.toFixed(1),
  });
}

function durationLabels(t: TranslateFn) {
  return {
    day: t('systemDashboard.units.day'),
    hour: t('systemDashboard.units.hour'),
    minute: t('systemDashboard.units.minute'),
  };
}

function systemText(host: ServerDashboard['host']) {
  return [host.os_name, host.os_version].filter(Boolean).join(' ') || '-';
}

function memoryDiskText(host: ServerDashboard['host']) {
  return `${formatBytes(host.total_memory_bytes)} / ${formatBytes(host.total_disk_bytes)}`;
}

function loadText(dashboard: ServerDashboard) {
  const load = dashboard.cpu.load_average;
  return `${load.one.toFixed(2)} / ${load.five.toFixed(2)} / ${load.fifteen.toFixed(2)}`;
}

function cpuCoreText(host: ServerDashboard['host'], t: TranslateFn) {
  const physical = host.physical_core_count == null ? '-' : String(host.physical_core_count);
  return `${physical} ${t('systemDashboard.units.physical')} / ${host.logical_core_count} ${t('systemDashboard.units.logical')}`;
}
