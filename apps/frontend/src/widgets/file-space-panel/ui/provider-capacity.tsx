import type { FileProviderSummary } from 'src/entities/file';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Chip from '@mui/material/Chip';
import Grid from '@mui/material/Grid';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { fData } from 'src/shared/lib/format-number';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { LoadingScreen } from 'src/shared/ui/loading-screen';
import { getErrorMessage } from 'src/shared/lib/get-error-message';

import { useFileProviders, fileProviderCapacityMetrics } from 'src/entities/file';

export function ProviderCapacityPanel({ enabled }: { enabled: boolean }) {
  const { t } = useTranslate('admin');
  const resource = useFileProviders(enabled);
  if (!enabled) return null;
  if (resource.isLoading) {
    return <LoadingScreen portal={false} sx={{ minHeight: 120, mb: 3 }} />;
  }
  if (resource.error) {
    return (
      <Alert severity="error" sx={{ mb: 3 }}>
        {getErrorMessage(resource.error) || t('file.messages.providersFailed')}
      </Alert>
    );
  }
  return (
    <Box sx={{ mb: 3 }}>
      <Typography variant="h6" sx={{ mb: 1.5 }}>
        {t('file.providerCapacityTitle')}
      </Typography>
      {resource.data?.length ? (
        <Grid container spacing={2}>
          {resource.data.map((provider) => (
            <Grid key={provider.key} size={{ xs: 12, md: 6, lg: 4 }}>
              <ProviderCapacityCard provider={provider} />
            </Grid>
          ))}
        </Grid>
      ) : (
        <Alert severity="info">{t('file.noProvider')}</Alert>
      )}
    </Box>
  );
}

function ProviderCapacityCard({ provider }: { provider: FileProviderSummary }) {
  const { t } = useTranslate('admin');
  const metrics = fileProviderCapacityMetrics(provider.capacity);
  return (
    <Card variant="outlined" sx={{ p: 2.5, height: '100%' }}>
      <Stack spacing={2}>
        <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={1}>
          <Typography variant="subtitle1">{provider.key}</Typography>
          <Chip
            size="small"
            label={t(`file.capacityModes.${metrics.kind === 'bounded' ? 'bounded' : 'usageBased'}`)}
          />
        </Stack>
        <CapacityMetric label={t('file.fields.providerUsage')} value={metrics.usedBytes} />
        {metrics.kind === 'bounded' ? (
          <>
            <CapacityMetric label={t('file.fields.totalCapacity')} value={metrics.totalBytes} />
            <CapacityMetric
              label={t('file.fields.availableCapacity')}
              value={metrics.availableBytes}
            />
          </>
        ) : null}
      </Stack>
    </Card>
  );
}

function CapacityMetric({ label, value }: { label: string; value: number }) {
  return (
    <Stack direction="row" justifyContent="space-between" spacing={2}>
      <Typography variant="body2" color="text.secondary">
        {label}
      </Typography>
      <Typography variant="body2">{fData(value)}</Typography>
    </Stack>
  );
}
