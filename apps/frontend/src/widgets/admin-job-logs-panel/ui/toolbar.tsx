import type { JobLogController } from 'src/features/scheduler-management';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

export function JobLogToolbar({ controller }: { controller: JobLogController }) {
  const { t } = useTranslate('scheduler');
  const { t: tAdmin } = useTranslate('admin');
  const { state, resources, actions, pending } = controller;
  return (
    <Stack direction="row" useFlexGap flexWrap="wrap" spacing={1} sx={{ mb: 2 }}>
      {resources.canRemove && (
        <Button
          color="error"
          variant="outlined"
          loading={pending.has('delete:batch')}
          startIcon={<Iconify icon="solar:trash-bin-trash-bold" />}
          disabled={state.table.selected.length === 0 || pending.has('delete:batch')}
          onClick={() => state.setBatchOpen(true)}
        >
          {tAdmin('common.delete')}
        </Button>
      )}
      {resources.canRemove && (
        <Button
          color="error"
          variant="outlined"
          loading={pending.has('delete:clean')}
          startIcon={<Iconify icon="solar:eraser-bold" />}
          disabled={pending.has('delete:clean')}
          onClick={() => state.setCleanOpen(true)}
        >
          {t('clearLogs')}
        </Button>
      )}
      {resources.canExport && (
        <Button
          variant="outlined"
          loading={pending.has('export')}
          startIcon={<Iconify icon="solar:export-bold" />}
          disabled={pending.has('export') || !resources.filtersValid}
          onClick={actions.submitExport}
        >
          {tAdmin('actions.export')}
        </Button>
      )}
    </Stack>
  );
}
