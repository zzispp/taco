import type { TranslateFn } from 'src/shared/i18n';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { Iconify } from 'src/shared/ui/iconify';

import { AddButton } from 'src/widgets/admin-common';

export function DictHeaderActions(props: DictHeaderActionsProps) {
  return (
    <Stack direction="row" spacing={1}>
      <DictExportAction props={props} />
      <DictRefreshAction props={props} />
      <DictDeleteAction props={props} />
      {props.canAdd && <AddButton onClick={props.onAdd}>{props.t('actions.addDict')}</AddButton>}
    </Stack>
  );
}

function DictExportAction({ props }: { props: DictHeaderActionsProps }) {
  if (!props.canExport) return null;
  return (
    <Button
      variant="outlined"
      disabled={props.exportDisabled}
      startIcon={<Iconify icon="solar:export-bold" />}
      onClick={props.onExport}
    >
      {props.t('actions.export')}
    </Button>
  );
}

function DictRefreshAction({ props }: { props: DictHeaderActionsProps }) {
  if (!props.canRefresh) return null;
  return (
    <Button
      variant="outlined"
      startIcon={<Iconify icon="solar:restart-bold" />}
      onClick={props.onRefresh}
    >
      {props.t('actions.refreshCache')}
    </Button>
  );
}

function DictDeleteAction({ props }: { props: DictHeaderActionsProps }) {
  if (!props.canRemove) return null;
  return (
    <Button
      variant="outlined"
      color="error"
      disabled={props.selectedCount === 0}
      onClick={props.onBatchDelete}
    >
      {props.t('common.delete')}
    </Button>
  );
}

type DictHeaderActionsProps = Readonly<{
  t: TranslateFn;
  canAdd: boolean;
  canExport: boolean;
  exportDisabled: boolean;
  canRefresh: boolean;
  canRemove: boolean;
  selectedCount: number;
  onAdd: () => void;
  onExport: () => void;
  onRefresh: () => void;
  onBatchDelete: () => void;
}>;
