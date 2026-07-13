import type { SchedulerController } from 'src/features/scheduler-management';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { AddButton } from 'src/shared/ui/admin';
import { useTranslate } from 'src/shared/i18n/use-locales';

export function SchedulerToolbar({ controller }: { controller: SchedulerController }) {
  const { t } = useTranslate('scheduler');
  const { t: tAdmin } = useTranslate('admin');
  const { resources, state, actions, pending } = controller;
  return (
    <Stack direction="row" spacing={1} sx={{ mb: 2 }}>
      {resources.canImport && (
        <AddButton onClick={() => state.setImportOpen(true)}>{t('importJob')}</AddButton>
      )}
      {resources.canRemove && (
        <Button
          color="error"
          variant="outlined"
          disabled={state.table.selected.length === 0}
          onClick={() => state.setBatchDeleteOpen(true)}
        >
          {tAdmin('common.delete')}
        </Button>
      )}
      {resources.canExport && (
        <Button variant="outlined" loading={pending.has('export')} onClick={actions.submitExport}>
          {tAdmin('actions.export')}
        </Button>
      )}
    </Stack>
  );
}
