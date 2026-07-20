'use client';

import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';

import { useTranslate } from 'src/shared/i18n';
import { Field } from 'src/shared/ui/hook-form';

export function AdvancedSettings() {
  const { t } = useTranslate('setup');

  return (
    <Stack spacing={2}>
      <Grid container spacing={2}>
        <NumericField
          name="advanced.http_request_timeout_ms"
          label={t('advanced.httpRequestTimeout')}
        />
        <NumericField
          name="advanced.online_session_cleanup_interval_ms"
          label={t('advanced.onlineSessionCleanupInterval')}
        />
        <NumericField
          name="advanced.online_session_cleanup_batch_size"
          label={t('advanced.onlineSessionCleanupBatchSize')}
        />
        <NumericField
          name="advanced.audit_outbox_worker_count"
          label={t('advanced.auditOutboxWorkerCount')}
        />
        <NumericField
          name="advanced.audit_outbox_claim_batch_size"
          label={t('advanced.auditOutboxClaimBatchSize')}
        />
        <NumericField
          name="advanced.audit_outbox_poll_interval_ms"
          label={t('advanced.auditOutboxPollInterval')}
        />
        <NumericField
          name="advanced.audit_outbox_lease_duration_ms"
          label={t('advanced.auditOutboxLeaseDuration')}
        />
        <NumericField
          name="advanced.audit_outbox_retry_delay_ms"
          label={t('advanced.auditOutboxRetryDelay')}
        />
        <NumericField
          name="advanced.audit_outbox_cleanup_interval_ms"
          label={t('advanced.auditOutboxCleanupInterval')}
        />
        <NumericField
          name="advanced.audit_outbox_cleanup_batch_size"
          label={t('advanced.auditOutboxCleanupBatchSize')}
        />
        <NumericField
          name="advanced.audit_outbox_processed_retention_days"
          label={t('advanced.auditOutboxProcessedRetentionDays')}
        />
        <NumericField
          name="advanced.client_ip_location_timeout_ms"
          label={t('advanced.clientIpLocationTimeout')}
        />
        <NumericField
          name="advanced.scheduler_http_timeout_ms"
          label={t('advanced.schedulerHttpTimeout')}
        />
        <NumericField
          name="advanced.scheduler_reconcile_interval_ms"
          label={t('advanced.schedulerReconcileInterval')}
        />
        <Grid size={{ xs: 12, md: 6 }}>
          <Field.Text name="advanced.redis_key_prefix" label={t('advanced.redisKeyPrefix')} />
        </Grid>
      </Grid>
      <Field.Switch name="advanced.compression_enabled" label={t('advanced.compressionEnabled')} />
      <Field.Switch name="advanced.metrics_enabled" label={t('advanced.metricsEnabled')} />
    </Stack>
  );
}

function NumericField({ name, label }: Readonly<{ name: string; label: string }>) {
  return (
    <Grid size={{ xs: 12, md: 6 }}>
      <Field.Text name={name} type="number" label={label} />
    </Grid>
  );
}
