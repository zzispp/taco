'use client';

import { useState } from 'react';

import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import LinearProgress from '@mui/material/LinearProgress';

import { paths } from 'src/shared/routes/paths';
import { Iconify } from 'src/shared/ui/iconify';
import { fData } from 'src/shared/lib/format-number';
import { RouterLink } from 'src/shared/routes/components';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { EmptyContent } from 'src/shared/ui/empty-content';
import { LoadingScreen } from 'src/shared/ui/loading-screen';
import { getErrorMessage } from 'src/shared/lib/get-error-message';

import { useFileOverview } from 'src/entities/file';
import { useAuthContext, usePermissionChecker } from 'src/entities/session';

import {
  fileCapabilities,
  FileSpaceSelector,
  useFileSpaceSelector,
} from 'src/features/file-management';

import { DashboardContent } from 'src/widgets/dashboard-shell';

import { UsageCard } from './usage-card';
import { RecentAssets } from './recent-assets';

export function FileStorageOverviewPanel() {
  const { t } = useTranslate('admin');
  const resources = useOverviewResources();
  if (!resources.permissions.canQuery)
    return (
      <DashboardContent>
        <Alert severity="warning">{t('file.permissionDenied')}</Alert>
      </DashboardContent>
    );
  if (resources.overview.isLoading)
    return (
      <DashboardContent>
        <LoadingScreen portal={false} sx={{ minHeight: 320 }} />
      </DashboardContent>
    );
  if (resources.overview.error) {
    return (
      <DashboardContent>
        <Alert severity="error">
          {getErrorMessage(resources.overview.error) || t('file.messages.overviewFailed')}
        </Alert>
      </DashboardContent>
    );
  }
  if (!resources.overview.data)
    return (
      <DashboardContent>
        <EmptyContent filled title={t('file.noSpace')} />
      </DashboardContent>
    );
  return <OverviewContent overview={resources.overview.data} resources={resources} />;
}

function useOverviewResources() {
  const [spaceId, setSpaceId] = useState<string | undefined>();
  const permissions = fileCapabilities(usePermissionChecker());
  const { user } = useAuthContext();
  const overview = useFileOverview(spaceId, permissions.canQuery);
  const spaceSelector = useFileSpaceSelector({
    selectedSpaceId: spaceId,
    enabled: permissions.canListSpaces,
  });
  return { spaceId, setSpaceId, permissions, user, overview, spaceSelector };
}

type Overview = NonNullable<ReturnType<typeof useFileOverview>['data']>;
type OverviewResources = ReturnType<typeof useOverviewResources>;

function OverviewContent({
  overview,
  resources,
}: {
  overview: Overview;
  resources: OverviewResources;
}) {
  return (
    <DashboardContent maxWidth="xl">
      <OverviewHeader overview={overview} resources={resources} />
      <OverviewMetrics overview={overview} />
    </DashboardContent>
  );
}

function OverviewHeader({
  overview,
  resources,
}: {
  overview: Overview;
  resources: OverviewResources;
}) {
  const { t } = useTranslate('admin');
  return (
    <Stack
      direction={{ xs: 'column', sm: 'row' }}
      alignItems={{ sm: 'center' }}
      spacing={2}
      sx={{ mb: 4 }}
    >
      <Box sx={{ flex: 1 }}>
        <Typography variant="h4">{t('file.overviewTitle')}</Typography>
        <Typography variant="body2" color="text.secondary">
          {t('file.fields.usage')}: {fData(overview.logical_asset_size)}
        </Typography>
      </Box>
      {resources.permissions.canListSpaces ? (
        <FileSpaceSelector
          selector={resources.spaceSelector}
          currentUserId={resources.user?.user_id}
          label={t('file.fields.space')}
          onChange={resources.setSpaceId}
        />
      ) : null}
      <Button
        variant="contained"
        component={RouterLink}
        href={paths.dashboard.fileManager}
        startIcon={<Iconify icon="solar:add-folder-bold" />}
      >
        {t('file.actions.open')}
      </Button>
    </Stack>
  );
}

function OverviewMetrics({ overview }: { overview: Overview }) {
  const { t } = useTranslate('admin');
  const quotaRatio =
    overview.quota_bytes > 0
      ? Math.min(100, (overview.logical_asset_size / overview.quota_bytes) * 100)
      : 0;
  return (
    <Grid container spacing={2.5}>
      <Grid size={{ xs: 12, sm: 6, md: 3 }}>
        <UsageCard label={t('file.fields.usage')} value={fData(overview.logical_asset_size)} />
      </Grid>
      <Grid size={{ xs: 12, sm: 6, md: 3 }}>
        <UsageCard label={t('file.fields.quota')} value={fData(overview.quota_bytes)} />
      </Grid>
      <Grid size={{ xs: 12, sm: 6, md: 3 }}>
        <UsageCard label={t('file.modes.trash')} value={fData(overview.recycle_bin_size)} />
      </Grid>
      <Grid size={{ xs: 12, sm: 6, md: 3 }}>
        <UsageCard label={t('file.fields.reserved')} value={fData(overview.quota_reserved_bytes)} />
      </Grid>
      <Grid size={{ xs: 12, md: 7 }}>
        <UsageBreakdown overview={overview} ratio={quotaRatio} />
      </Grid>
      <Grid size={{ xs: 12, md: 5 }}>
        <RecentAssets overview={overview} />
      </Grid>
    </Grid>
  );
}

function UsageBreakdown({ overview, ratio }: { overview: Overview; ratio: number }) {
  const { t } = useTranslate('admin');
  return (
    <Box sx={{ p: 3, border: 1, borderColor: 'divider', borderRadius: 1 }}>
      <Stack spacing={2.5}>
        <Box>
          <Typography variant="subtitle1">{t('file.fields.quota')}</Typography>
          <Typography variant="body2" color="text.secondary">
            {fData(overview.logical_asset_size)} / {fData(overview.quota_bytes)}
          </Typography>
          <LinearProgress variant="determinate" value={ratio} sx={{ mt: 1.5 }} />
        </Box>
        <Typography variant="subtitle1">{t('file.storageMetrics')}</Typography>
        <OverviewMetricRow
          label={t('file.fields.managedPhysicalUsage')}
          value={overview.managed_physical_usage}
        />
        <OverviewMetricRow
          label={t('file.fields.temporaryUploadSize')}
          value={overview.temporary_upload_size}
        />
        <OverviewMetricRow
          label={t('file.fields.deduplicationSavings')}
          value={overview.deduplication_savings}
        />
        <Typography variant="subtitle1">{t('file.fields.type')}</Typography>
        {overview.type_distribution.map((item) => (
          <Stack key={item.entry_type} direction="row" justifyContent="space-between">
            <Typography variant="body2">{t(`file.typeCategories.${item.entry_type}`)}</Typography>
            <Typography variant="body2" color="text.secondary">
              {item.count} · {fData(item.bytes)}
            </Typography>
          </Stack>
        ))}
      </Stack>
    </Box>
  );
}

function OverviewMetricRow({ label, value }: { label: string; value: number }) {
  return (
    <Stack direction="row" justifyContent="space-between" spacing={2}>
      <Typography variant="body2">{label}</Typography>
      <Typography variant="body2" color="text.secondary">
        {fData(value)}
      </Typography>
    </Stack>
  );
}
