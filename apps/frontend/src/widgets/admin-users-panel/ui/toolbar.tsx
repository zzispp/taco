import type { TranslateFn } from 'src/shared/i18n';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { Iconify } from 'src/shared/ui/iconify';
import { AddButton } from 'src/shared/ui/admin';

export function UserToolbar({
  t,
  canAdd,
  canDelete,
  canImport,
  canExport,
  selectedCount,
  onCreate,
  onImport,
  onExport,
  onBatchDelete,
}: {
  t: TranslateFn;
  canAdd: boolean;
  canDelete: boolean;
  canImport: boolean;
  canExport: boolean;
  selectedCount: number;
  onCreate: () => void;
  onImport: () => void;
  onExport: () => void;
  onBatchDelete: () => void;
}) {
  if (!canDelete && !canAdd && !canImport && !canExport) return null;
  return (
    <Stack direction="row" spacing={1}>
      {canImport && (
        <Button
          variant="outlined"
          startIcon={<Iconify icon="solar:import-bold" />}
          onClick={onImport}
        >
          {t('actions.import')}
        </Button>
      )}
      {canExport && (
        <Button
          variant="outlined"
          startIcon={<Iconify icon="solar:export-bold" />}
          onClick={onExport}
        >
          {t('actions.export')}
        </Button>
      )}
      {canDelete && (
        <Button
          variant="outlined"
          color="error"
          disabled={selectedCount === 0}
          onClick={onBatchDelete}
        >
          {t('common.delete')}
        </Button>
      )}
      {canAdd && <AddButton onClick={onCreate}>{t('actions.addUser')}</AddButton>}
    </Stack>
  );
}
