'use client';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/shared/i18n';
import { Field } from 'src/shared/ui/hook-form';

type ConnectionStepProps = Readonly<{
  onBack?: () => void;
  onNext: () => void;
  onTest: () => void;
  testing: boolean;
}>;

export function PostgresConnectionStep({ onBack, onNext, onTest, testing }: ConnectionStepProps) {
  const { t } = useTranslate('setup');

  return (
    <ConnectionStepLayout
      title={t('steps.postgres.title')}
      description={t('steps.postgres.description')}
      onBack={onBack}
      onNext={onNext}
      onTest={onTest}
      testing={testing}
    >
      <Field.Text name="postgres.host" label={t('steps.postgres.host')} />
      <Field.Text name="postgres.port" type="number" label={t('steps.postgres.port')} />
      <Field.Text name="postgres.username" label={t('steps.postgres.username')} />
      <Field.Text name="postgres.password" type="password" label={t('steps.postgres.password')} />
      <Field.Text name="postgres.database" label={t('steps.postgres.database')} />
      <Field.Switch name="postgres.use_tls" label={t('steps.postgres.useTls')} />
    </ConnectionStepLayout>
  );
}

export function RedisConnectionStep({ onBack, onNext, onTest, testing }: ConnectionStepProps) {
  const { t } = useTranslate('setup');

  return (
    <ConnectionStepLayout
      title={t('steps.redis.title')}
      description={t('steps.redis.description')}
      onBack={onBack}
      onNext={onNext}
      onTest={onTest}
      testing={testing}
    >
      <Field.Text name="redis.host" label={t('steps.redis.host')} />
      <Field.Text name="redis.port" type="number" label={t('steps.redis.port')} />
      <Field.Text name="redis.username" label={t('steps.redis.username')} />
      <Field.Text name="redis.password" type="password" label={t('steps.redis.password')} />
      <Field.Text
        name="redis.database"
        label={t('steps.redis.database')}
        slotProps={{ htmlInput: { inputMode: 'numeric', pattern: '[0-9]*' } }}
      />
      <Field.Switch name="redis.use_tls" label={t('steps.redis.useTls')} />
    </ConnectionStepLayout>
  );
}

type ConnectionStepLayoutProps = ConnectionStepProps &
  Readonly<{
    title: string;
    description: string;
    children: React.ReactNode;
  }>;

function ConnectionStepLayout({
  children,
  description,
  onBack,
  onNext,
  onTest,
  testing,
  title,
}: ConnectionStepLayoutProps) {
  const { t } = useTranslate('setup');

  return (
    <Stack spacing={3}>
      <Stack spacing={1}>
        <Typography variant="h4">{title}</Typography>
        <Typography color="text.secondary">{description}</Typography>
      </Stack>
      <Stack spacing={2}>{children}</Stack>
      <Stack direction="row" justifyContent="space-between" spacing={1}>
        {onBack ? <Button onClick={onBack}>{t('actions.back')}</Button> : <span />}
        <Stack direction="row" spacing={1}>
          <Button variant="outlined" onClick={onTest} loading={testing}>
            {testing ? t('actions.testingConnection') : t('actions.testConnection')}
          </Button>
          <Button variant="contained" onClick={onNext} disabled={testing}>
            {t('actions.next')}
          </Button>
        </Stack>
      </Stack>
    </Stack>
  );
}
