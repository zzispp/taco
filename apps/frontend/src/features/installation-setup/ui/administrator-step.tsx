'use client';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Collapse from '@mui/material/Collapse';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/shared/i18n';
import { Field } from 'src/shared/ui/hook-form';

import { AdvancedSettings } from './advanced-settings';

type AdministratorStepProps = Readonly<{
  advancedOpen: boolean;
  onBack: () => void;
  onNext: () => void;
  onToggleAdvanced: () => void;
}>;

export function AdministratorStep({
  advancedOpen,
  onBack,
  onNext,
  onToggleAdvanced,
}: AdministratorStepProps) {
  const { t } = useTranslate('setup');

  return (
    <Stack spacing={3}>
      <Stack spacing={1}>
        <Typography variant="h4">{t('steps.administrator.title')}</Typography>
        <Typography color="text.secondary">{t('steps.administrator.description')}</Typography>
      </Stack>
      <AdministratorFields />
      <AdvancedSection open={advancedOpen} onToggle={onToggleAdvanced} />
      <Stack direction="row" justifyContent="space-between">
        <Button onClick={onBack}>{t('actions.back')}</Button>
        <Button variant="contained" onClick={onNext}>
          {t('actions.next')}
        </Button>
      </Stack>
    </Stack>
  );
}

function AdministratorFields() {
  const { t } = useTranslate('setup');

  return (
    <Stack spacing={2}>
      <Field.Text name="administrator.username" label={t('steps.administrator.username')} />
      <Field.Text name="administrator.email" type="email" label={t('steps.administrator.email')} />
      <Field.Text
        name="administrator.password"
        type="password"
        label={t('steps.administrator.password')}
      />
      <Field.Text
        name="administrator.password_confirmation"
        type="password"
        label={t('steps.administrator.passwordConfirmation')}
      />
    </Stack>
  );
}

type AdvancedSectionProps = Readonly<{
  open: boolean;
  onToggle: () => void;
}>;

function AdvancedSection({ open, onToggle }: AdvancedSectionProps) {
  const { t } = useTranslate('setup');

  return (
    <Box sx={{ borderTop: (theme) => `1px solid ${theme.palette.divider}`, pt: 3 }}>
      <Stack spacing={1}>
        <Typography variant="subtitle1">{t('advanced.title')}</Typography>
        <Typography variant="body2" color="text.secondary">
          {t('advanced.description')}
        </Typography>
        <Button variant="text" onClick={onToggle} sx={{ alignSelf: 'flex-start' }}>
          {open ? t('actions.closeAdvanced') : t('actions.openAdvanced')}
        </Button>
      </Stack>
      <Collapse in={open}>
        <Box sx={{ pt: 2 }}>
          <AdvancedSettings />
        </Box>
      </Collapse>
    </Box>
  );
}
