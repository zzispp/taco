import type { TranslateFn } from 'src/shared/i18n';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { Iconify } from 'src/shared/ui/iconify';
import { AddButton } from 'src/shared/ui/admin';

export function DictHeaderActions({
  t,
  canAdd,
  canExport,
  canRefresh,
  canRemove,
  selectedCount,
  onAdd,
  onExport,
  onRefresh,
  onBatchDelete,
}: {
  t: TranslateFn;
  canAdd: boolean;
  canExport: boolean;
  canRefresh: boolean;
  canRemove: boolean;
  selectedCount: number;
  onAdd: () => void;
  onExport: () => void;
  onRefresh: () => void;
  onBatchDelete: () => void;
}) {
  return (
    <Stack direction="row" spacing={1}>
      {canExport && (
        <Button
          variant="outlined"
          startIcon={<Iconify icon="solar:export-bold" />}
          onClick={onExport}
        >
          {t('actions.export')}
        </Button>
      )}
      {canRefresh && (
        <Button
          variant="outlined"
          startIcon={<Iconify icon="solar:restart-bold" />}
          onClick={onRefresh}
        >
          {t('actions.refreshCache')}
        </Button>
      )}
      {canRemove && (
        <Button
          variant="outlined"
          color="error"
          disabled={selectedCount === 0}
          onClick={onBatchDelete}
        >
          {t('common.delete')}
        </Button>
      )}
      {canAdd && <AddButton onClick={onAdd}>{t('actions.addDict')}</AddButton>}
    </Stack>
  );
}
