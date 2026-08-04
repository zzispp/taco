import type React from 'react';
import type { TranslateFn } from 'src/shared/i18n';
import type { Role, RoleInput } from 'src/entities/role';
import type { TreeSelectNode } from 'src/entities/system';

import Button from '@mui/material/Button';

import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

import { RoleDialog } from './role-dialog';
import { RoleBindingDialog } from './binding-dialog';
import { RoleUsersDialog } from './role-users-dialog';

export function RoleManagementDialogs(props: RoleManagementDialogsProps) {
  return (
    <>
      <RoleDialog
        open={props.creating || !!props.editing}
        editing={Boolean(props.editing)}
        submitting={props.submitting}
        form={props.form}
        setForm={props.setForm}
        onClose={props.onDialogClose}
        onSubmit={props.onRoleSubmit}
      />
      <RoleBindingDialog
        role={props.binding.target}
        type={props.binding.type}
        nodes={props.binding.nodes}
        selected={props.binding.selected}
        strict={props.binding.strict}
        dataScope={props.binding.dataScope}
        loading={props.binding.loading}
        submitting={props.submitting}
        onSelectedChange={props.binding.onSelectedChange}
        onStrictChange={props.binding.onStrictChange}
        onDataScopeChange={props.binding.onDataScopeChange}
        onResolvedSelectionChange={props.binding.onResolvedSelectionChange}
        onClose={props.onBindingClose}
        onSubmit={props.onBindingSubmit}
      />
      <RoleUsersDialog role={props.usersTarget} onClose={props.onUsersClose} />
      <ConfirmDialog
        open={props.batchDeleteOpen}
        onClose={props.onBatchDeleteClose}
        title={props.t('dialogs.deleteRole')}
        content={props.t('dialogs.deleteContent', { name: String(props.selectedCount) })}
        cancelText={props.t('common.cancel')}
        action={deleteAction(props.t, props.onBatchDeleteConfirm)}
      />
      <ConfirmDialog
        open={Boolean(props.deleteTarget)}
        onClose={props.onDeleteClose}
        title={props.t('dialogs.deleteRole')}
        content={props.t('dialogs.deleteContent', { name: props.deleteTarget?.role_name ?? '' })}
        cancelText={props.t('common.cancel')}
        action={deleteAction(props.t, props.onDeleteConfirm)}
      />
    </>
  );
}

function deleteAction(t: TranslateFn, onClick: () => void) {
  return (
    <Button variant="contained" color="error" onClick={onClick}>
      {t('common.delete')}
    </Button>
  );
}

type RoleBindingDialogState = {
  target: Role | null;
  type: 'menus' | 'depts';
  nodes: TreeSelectNode[];
  selected: string[];
  strict: boolean;
  dataScope: string;
  loading: boolean;
  onSelectedChange: (selected: string[]) => void;
  onStrictChange: (strict: boolean) => void;
  onDataScopeChange: (dataScope: string) => void;
  onResolvedSelectionChange: (selected: string[]) => void;
};

type RoleManagementDialogsProps = {
  t: TranslateFn;
  form: RoleInput;
  creating: boolean;
  editing: Role | null;
  submitting: boolean;
  binding: RoleBindingDialogState;
  usersTarget: Role | null;
  deleteTarget: Role | null;
  batchDeleteOpen: boolean;
  selectedCount: number;
  setForm: React.Dispatch<React.SetStateAction<RoleInput>>;
  onDialogClose: () => void;
  onRoleSubmit: () => void;
  onBindingSubmit: () => void;
  onBindingClose: () => void;
  onUsersClose: () => void;
  onBatchDeleteClose: () => void;
  onBatchDeleteConfirm: () => void;
  onDeleteClose: () => void;
  onDeleteConfirm: () => void;
};
