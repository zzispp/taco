import type { OperationLogController } from 'src/features/audit-log-management';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

export function OperationLogToolbar({ controller }: { controller: OperationLogController }) {
  const { t } = useTranslate('audit');
  const { state, resources, pending } = controller;
  return (
    <Stack direction="row" useFlexGap flexWrap="wrap" spacing={1} sx={{ mb: 2 }}>
      {resources.canRemove && (
        <Button
          color="error"
          variant="outlined"
          disabled={state.table.selected.length === 0 || pending.has('delete:batch')}
          loading={pending.has('delete:batch')}
          startIcon={<Iconify icon="solar:trash-bin-trash-bold" />}
          onClick={() => state.setBatchOpen(true)}
        >
          {t('actions.delete')}
        </Button>
      )}
      {resources.canRemove && (
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
      )}
      {resources.canExport && (
        <Button
          variant="outlined"
          disabled={!resources.filtersValid || pending.has('export')}
          loading={pending.has('export')}
          startIcon={<Iconify icon="solar:export-bold" />}
          onClick={controller.actions.submitExport}
        >
          {t('actions.export')}
        </Button>
      )}
    </Stack>
  );
}
