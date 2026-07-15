import type { LoginLogController } from 'src/features/audit-log-management';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

export function LoginLogToolbar({ controller }: { controller: LoginLogController }) {
  return (
    <Stack direction="row" useFlexGap flexWrap="wrap" spacing={1} sx={{ mb: 2 }}>
      <DeleteButtons controller={controller} />
      <UnlockButton controller={controller} />
      <ExportButton controller={controller} />
    </Stack>
  );
}

function DeleteButtons({ controller }: { controller: LoginLogController }) {
  const { t } = useTranslate('audit');
  const { state, resources, pending } = controller;
  if (!resources.canRemove) return null;
  return (
    <>
      <Button
        color="error"
        variant="outlined"
        disabled={!resources.selection.canDelete || pending.has('delete:batch')}
        loading={pending.has('delete:batch')}
        startIcon={<Iconify icon="solar:trash-bin-trash-bold" />}
        onClick={() => state.setBatchOpen(true)}
      >
        {t('actions.delete')}
      </Button>
      <Button
        color="error"
        variant="outlined"
        disabled={pending.has('delete:clean')}
        loading={pending.has('delete:clean')}
        startIcon={<Iconify icon="solar:eraser-bold" />}
        onClick={() => state.setCleanOpen(true)}
      >
        {t('actions.clean')}
      </Button>
    </>
  );
}

function UnlockButton({ controller }: { controller: LoginLogController }) {
  const { t } = useTranslate('audit');
  const { resources, actions, pending } = controller;
  if (!resources.canUnlock) return null;
  return (
    <Button
      color="warning"
      variant="outlined"
      disabled={!resources.selection.canUnlock || pending.has('unlock')}
      loading={pending.has('unlock')}
      startIcon={<Iconify icon="solar:lock-password-outline" />}
      onClick={actions.requestUnlock}
    >
      {t('actions.unlock')}
    </Button>
  );
}

function ExportButton({ controller }: { controller: LoginLogController }) {
  const { t } = useTranslate('audit');
  const { resources, actions, pending } = controller;
  if (!resources.canExport) return null;
  return (
    <Button
      variant="outlined"
      disabled={!resources.filtersValid || pending.has('export')}
      loading={pending.has('export')}
      startIcon={<Iconify icon="solar:export-bold" />}
      onClick={actions.submitExport}
    >
      {t('actions.export')}
    </Button>
  );
}
