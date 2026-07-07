import type { DeptManagementController } from './dept-controller';

import { DeptDialog } from './dept-dialog';
import { DeptConfirmDialog } from './dept-confirm-dialog';

export function DeptDialogSection({ state, actions }: DeptManagementController) {
  return (
    <>
      <DeptDialog
        open={state.creating || !!state.editing}
        editing={!!state.editing}
        submitting={state.submitting}
        form={state.form}
        parentNodes={state.parentNodes}
        setForm={state.setForm}
        onClose={actions.closeDialog}
        onSubmit={actions.submitDept}
      />
      <DeptConfirmDialog
        target={state.deleteTarget}
        onClose={() => state.setDeleteTarget(null)}
        onConfirm={actions.confirmDelete}
      />
    </>
  );
}
