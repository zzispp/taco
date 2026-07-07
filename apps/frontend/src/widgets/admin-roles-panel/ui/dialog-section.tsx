import type { RoleManagementController } from './controller';

import { RoleManagementDialogs } from './dialogs';

type RoleDialogSectionProps = RoleManagementController;

export function RoleDialogSection({ resources, dialogs, binding, actions }: RoleDialogSectionProps) {
  const { t } = resources;

  return (
    <RoleManagementDialogs
      t={t}
      form={dialogs.form}
      creating={dialogs.creating}
      editing={dialogs.editing}
      submitting={dialogs.submitting}
      binding={{
        target: binding.target,
        type: binding.type,
        nodes: binding.nodes,
        selected: binding.selected,
        strict: binding.strict,
        dataScope: binding.dataScope,
        loading: binding.loading,
        onSelectedChange: binding.setSelected,
        onStrictChange: binding.setStrict,
        onDataScopeChange: binding.setDataScope,
        onResolvedSelectionChange: binding.setResolvedDeptBindings,
      }}
      usersTarget={dialogs.usersTarget}
      deleteTarget={dialogs.deleteTarget}
      batchDeleteOpen={dialogs.batchDeleteOpen}
      selectedCount={dialogs.selected.length}
      setForm={dialogs.setForm}
      onDialogClose={actions.closeDialog}
      onRoleSubmit={actions.submitRole}
      onBindingSubmit={actions.saveBindings}
      onBindingClose={() => binding.setTarget(null)}
      onUsersClose={() => dialogs.setUsersTarget(null)}
      onBatchDeleteClose={() => dialogs.setBatchDeleteOpen(false)}
      onBatchDeleteConfirm={actions.confirmBatchDelete}
      onDeleteClose={() => dialogs.setDeleteTarget(null)}
      onDeleteConfirm={actions.confirmDelete}
    />
  );
}
