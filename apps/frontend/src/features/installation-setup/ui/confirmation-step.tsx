'use client';

import type { TranslateFn } from 'src/shared/i18n';
import type { SetupFormValues, AdvancedSetupKey } from '../model/form-values';

import { useWatch, useFormContext } from 'react-hook-form';

import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/shared/i18n';

type InstallationConfirmationProps = Readonly<{
  advancedOpen: boolean;
  advancedKeys: readonly AdvancedSetupKey[];
  error: string | null;
  submitting: boolean;
  onBack: () => void;
  onInstall: () => void;
}>;

export function InstallationConfirmation({
  advancedOpen,
  advancedKeys,
  error,
  submitting,
  onBack,
  onInstall,
}: InstallationConfirmationProps) {
  const { t } = useTranslate('setup');

  return (
    <Stack spacing={3}>
      <Stack spacing={1}>
        <Typography variant="h4">{t('steps.confirmation.title')}</Typography>
        <Typography color="text.secondary">{t('steps.confirmation.description')}</Typography>
      </Stack>
      <InstallationPreview advancedOpen={advancedOpen} advancedKeys={advancedKeys} />
      {error ? <Alert severity="error">{error}</Alert> : null}
      <Stack direction="row" justifyContent="space-between" spacing={1}>
        <Button onClick={onBack} disabled={submitting}>
          {t('actions.back')}
        </Button>
        <Button variant="contained" onClick={onInstall} loading={submitting}>
          {submitting ? t('actions.installing') : t('actions.install')}
        </Button>
      </Stack>
    </Stack>
  );
}

type InstallationPreviewProps = Pick<
  InstallationConfirmationProps,
  'advancedOpen' | 'advancedKeys'
>;

function InstallationPreview({ advancedOpen, advancedKeys }: InstallationPreviewProps) {
  const { t } = useTranslate('setup');
  const { control } = useFormContext<SetupFormValues>();
  const postgres = useWatch({ control, name: 'postgres' });
  const redis = useWatch({ control, name: 'redis' });
  const administrator = useWatch({ control, name: 'administrator' });
  const advanced = useWatch({ control, name: 'advanced' });
  const effectiveAdvancedKeys = advancedOpen ? advancedKeys : [];

  return (
    <Stack divider={<Divider flexItem />} spacing={2}>
      <SummarySection title={t('steps.confirmation.postgres')} values={postgresRows(postgres, t)} />
      <SummarySection title={t('steps.confirmation.redis')} values={redisRows(redis, t)} />
      <SummarySection
        title={t('steps.confirmation.administrator')}
        values={administratorRows(administrator, t)}
      />
      <SummarySection
        title={t('steps.confirmation.advanced')}
        values={advancedRows(advanced, effectiveAdvancedKeys, t)}
        emptyLabel={t('steps.confirmation.noAdvancedOverrides')}
      />
    </Stack>
  );
}

type SummarySectionProps = Readonly<{
  title: string;
  values: readonly SummaryValue[];
  emptyLabel?: string;
}>;

type SummaryValue = Readonly<{
  label: string;
  value: string | number;
}>;

function SummarySection({ title, values, emptyLabel }: SummarySectionProps) {
  return (
    <Stack spacing={0.5} sx={{ py: 0.5 }}>
      <Typography variant="subtitle2">{title}</Typography>
      {values.length ? (
        values.map(({ label, value }) => (
          <Typography key={`${title}-${label}`} variant="body2" color="text.secondary">
            {`${label}: ${value}`}
          </Typography>
        ))
      ) : (
        <Typography variant="body2" color="text.secondary">
          {emptyLabel}
        </Typography>
      )}
    </Stack>
  );
}

const ADVANCED_LABEL_KEYS = {
  http_request_timeout_ms: 'advanced.httpRequestTimeout',
  compression_enabled: 'advanced.compressionEnabled',
  metrics_enabled: 'advanced.metricsEnabled',
  online_session_cleanup_interval_ms: 'advanced.onlineSessionCleanupInterval',
  online_session_cleanup_batch_size: 'advanced.onlineSessionCleanupBatchSize',
  audit_outbox_worker_count: 'advanced.auditOutboxWorkerCount',
  audit_outbox_claim_batch_size: 'advanced.auditOutboxClaimBatchSize',
  audit_outbox_poll_interval_ms: 'advanced.auditOutboxPollInterval',
  audit_outbox_lease_duration_ms: 'advanced.auditOutboxLeaseDuration',
  audit_outbox_retry_delay_ms: 'advanced.auditOutboxRetryDelay',
  audit_outbox_cleanup_interval_ms: 'advanced.auditOutboxCleanupInterval',
  audit_outbox_cleanup_batch_size: 'advanced.auditOutboxCleanupBatchSize',
  audit_outbox_processed_retention_days: 'advanced.auditOutboxProcessedRetentionDays',
  client_ip_location_timeout_ms: 'advanced.clientIpLocationTimeout',
  scheduler_http_timeout_ms: 'advanced.schedulerHttpTimeout',
  scheduler_reconcile_interval_ms: 'advanced.schedulerReconcileInterval',
  redis_key_prefix: 'advanced.redisKeyPrefix',
} as const satisfies Record<AdvancedSetupKey, string>;

function advancedRows(
  advanced: SetupFormValues['advanced'] | undefined,
  keys: readonly AdvancedSetupKey[],
  t: TranslateFn
): readonly SummaryValue[] {
  if (!advanced) return [];

  return keys.map((key) => ({
    label: t(ADVANCED_LABEL_KEYS[key]),
    value: advancedValue(advanced[key], t),
  }));
}

function postgresRows(
  postgres: SetupFormValues['postgres'] | undefined,
  t: TranslateFn
): readonly SummaryValue[] {
  return [
    { label: t('steps.postgres.host'), value: configuredText(postgres?.host, t) },
    { label: t('steps.postgres.port'), value: configuredValue(postgres?.port, t) },
    { label: t('steps.postgres.database'), value: configuredText(postgres?.database, t) },
    { label: t('steps.postgres.username'), value: configuredText(postgres?.username, t) },
    { label: t('steps.postgres.password'), value: passwordStatus(postgres?.password, t) },
    { label: t('steps.postgres.useTls'), value: booleanStatus(postgres?.use_tls, t) },
  ];
}

function redisRows(
  redis: SetupFormValues['redis'] | undefined,
  t: TranslateFn
): readonly SummaryValue[] {
  return [
    { label: t('steps.redis.host'), value: configuredText(redis?.host, t) },
    { label: t('steps.redis.port'), value: configuredValue(redis?.port, t) },
    { label: t('steps.redis.database'), value: configuredText(redis?.database, t) },
    { label: t('steps.redis.username'), value: configuredText(redis?.username, t) },
    { label: t('steps.redis.password'), value: passwordStatus(redis?.password, t) },
    { label: t('steps.redis.useTls'), value: booleanStatus(redis?.use_tls, t) },
  ];
}

function administratorRows(
  administrator: SetupFormValues['administrator'] | undefined,
  t: TranslateFn
): readonly SummaryValue[] {
  return [
    { label: t('steps.administrator.username'), value: configuredText(administrator?.username, t) },
    { label: t('steps.administrator.email'), value: configuredText(administrator?.email, t) },
    { label: t('steps.administrator.password'), value: passwordStatus(administrator?.password, t) },
  ];
}

function configuredText(value: string | undefined, t: TranslateFn): string {
  const trimmed = value?.trim();
  return trimmed || t('steps.confirmation.notConfigured');
}

function configuredValue(value: number | undefined, t: TranslateFn): string | number {
  return value ?? t('steps.confirmation.notConfigured');
}

function passwordStatus(value: string | undefined, t: TranslateFn): string {
  return value?.trim()
    ? t('steps.confirmation.passwordConfigured')
    : t('steps.confirmation.passwordNotConfigured');
}

function booleanStatus(value: boolean | undefined, t: TranslateFn): string {
  return value ? t('steps.confirmation.enabled') : t('steps.confirmation.disabled');
}

function advancedValue(value: string | number | boolean, t: TranslateFn): string | number {
  return typeof value === 'boolean' ? booleanStatus(value, t) : value;
}
