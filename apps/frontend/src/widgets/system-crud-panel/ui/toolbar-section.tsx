import type { ReactNode } from 'react';
import type { SystemCrudController } from './controller';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { AddButton } from 'src/widgets/admin-common';

type CrudRecord = Record<string, unknown>;

type CrudToolbarSectionProps<T extends CrudRecord, I extends CrudRecord> = {
  addLabel: string;
  toolbarAction?: ReactNode;
  controller: SystemCrudController<T, I>;
};

export function CrudToolbarSection<T extends CrudRecord, I extends CrudRecord>({
  addLabel,
  toolbarAction,
  controller,
}: CrudToolbarSectionProps<T, I>) {
  const { t, state, permissions } = controller;
  const hasToolbarActions =
    Boolean(toolbarAction) || permissions.canAdd || permissions.hasBatchDelete;
  if (!hasToolbarActions) return null;

  return (
    <Stack direction="row" spacing={1}>
      {toolbarAction}
      {permissions.hasBatchDelete && (
        <Button
          variant="outlined"
          color="error"
          disabled={state.selected.length === 0}
          onClick={() => state.setBatchDeleteOpen(true)}
        >
          {t('common.delete')}
        </Button>
      )}
      {permissions.canAdd && (
        <AddButton onClick={() => state.setCreating(true)}>{addLabel}</AddButton>
      )}
    </Stack>
  );
}
