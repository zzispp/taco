import type React from 'react';
import type { TranslateFn } from 'src/shared/i18n';
import type { Role, RoleInput } from 'src/entities/role';
import type { TreeSelectNode } from 'src/entities/system';

import Button from '@mui/material/Button';

import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

import { RoleDialog } from './role-dialog';
import { RoleBindingDialog } from './binding-dialog';
import { RoleUsersDialog } from './role-users-dialog';

export function RoleManagementDialogs({
  t,
  form,
  creating,
  editing,
  submitting,
  binding,
  usersTarget,
  deleteTarget,
  batchDeleteOpen,
  selectedCount,
  setForm,
  onDialogClose,
  onRoleSubmit,
  onBindingSubmit,
  onBindingClose,
  onUsersClose,
  onBatchDeleteClose,
  onBatchDeleteConfirm,
  onDeleteClose,
  onDeleteConfirm,
}: RoleManagementDialogsProps) {
  return (
    <>
      <RoleDialog
        open={creating || !!editing}
        editing={!!editing}
        submitting={submitting}
        form={form}
        setForm={setForm}
        onClose={onDialogClose}
        onSubmit={onRoleSubmit}
      />
      <RoleBindingDialog
        role={binding.target}
        type={binding.type}
        nodes={binding.nodes}
        selected={binding.selected}
        strict={binding.strict}
        dataScope={binding.dataScope}
        loading={binding.loading}
        submitting={submitting}
        onSelectedChange={binding.onSelectedChange}
        onStrictChange={binding.onStrictChange}
        onDataScopeChange={binding.onDataScopeChange}
        onResolvedSelectionChange={binding.onResolvedSelectionChange}
        onClose={onBindingClose}
        onSubmit={onBindingSubmit}
      />
      <RoleUsersDialog role={usersTarget} onClose={onUsersClose} />
      <ConfirmDialog
        open={batchDeleteOpen}
        onClose={onBatchDeleteClose}
        title={t('dialogs.deleteRole')}
        content={t('dialogs.deleteContent', { name: String(selectedCount) })}
        cancelText={t('common.cancel')}
        action={deleteAction(t, onBatchDeleteConfirm)}
      />
      <ConfirmDialog
        open={!!deleteTarget}
        onClose={onDeleteClose}
        title={t('dialogs.deleteRole')}
        content={t('dialogs.deleteContent', { name: deleteTarget?.role_name ?? '' })}
        cancelText={t('common.cancel')}
        action={deleteAction(t, onDeleteConfirm)}
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
