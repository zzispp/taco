import type { UserManagementController } from './controller';

import { UserConfirmDialogs } from './confirm-dialogs';
import { UserManagementDialogs } from './management-dialogs';

export function UserDialogSection({ resources, state, actions }: UserManagementController) {
  return (
    <>
      <UserManagementDialogs
        form={state.form}
        roles={resources.roles}
        posts={resources.posts}
        deptTree={resources.deptTree}
        editing={state.editing}
        creating={state.creating}
        submitting={state.submitting}
        roleTarget={state.roleTarget}
        assignedRoles={state.assignedRoles}
        passwordTarget={state.passwordTarget}
        newPassword={state.newPassword}
        importOpen={state.importOpen}
        importFile={state.importFile}
        updateSupport={state.updateSupport}
        setForm={state.setForm}
        onDialogClose={actions.closeDialog}
        onUserSubmit={actions.submitUser}
        onAssignedRolesChange={state.setAssignedRoles}
        onRoleClose={() => state.setRoleTarget(null)}
        onRolesSubmit={actions.submitRoles}
        onPasswordChange={state.setNewPassword}
        onPasswordClose={() => state.setPasswordTarget(null)}
        onPasswordSubmit={actions.submitPassword}
        onImportFileChange={state.setImportFile}
        onUpdateSupportChange={state.setUpdateSupport}
        onImportTemplate={actions.downloadUserImportTemplate}
        onImportClose={actions.closeImportDialog}
        onImportSubmit={actions.submitImport}
      />
      <UserConfirmDialogs
        t={resources.t}
        deleteTarget={state.deleteTarget}
        batchDeleteOpen={state.batchDeleteOpen}
        selectedCount={state.selected.length}
        onBatchDeleteClose={() => state.setBatchDeleteOpen(false)}
        onDeleteClose={() => state.setDeleteTarget(null)}
        onBatchDeleteConfirm={actions.confirmBatchDelete}
        onDeleteConfirm={actions.confirmDelete}
      />
    </>
  );
}
