import type { TranslateFn } from 'src/shared/i18n';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { Iconify } from 'src/shared/ui/iconify';

import { AddButton } from 'src/widgets/admin-common';

export function RoleToolbar({
  t,
  canAdd,
  canDelete,
  canExport,
  exportDisabled,
  selectedCount,
  onCreate,
  onBatchDelete,
  onExport,
}: {
  t: TranslateFn;
  canAdd: boolean;
  canDelete: boolean;
  canExport: boolean;
  exportDisabled: boolean;
  selectedCount: number;
  onCreate: () => void;
  onBatchDelete: () => void;
  onExport: () => void;
}) {
  if (!canDelete && !canAdd && !canExport) return null;
  return (
    <Stack direction="row" spacing={1}>
      {canExport && (
        <Button
          variant="outlined"
          disabled={exportDisabled}
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
      {canAdd && <AddButton onClick={onCreate}>{t('actions.addRole')}</AddButton>}
    </Stack>
  );
}
