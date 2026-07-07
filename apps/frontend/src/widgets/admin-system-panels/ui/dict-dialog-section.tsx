import type { DictManagementController } from './dict-controller';

import { DictConfirmDialogs } from './dict-confirm-dialogs';
import { closeDataDialog, closeTypeDialog } from './dict-helpers';
import { DictDataDialog, DictTypeDialog } from './dict-form-dialogs';

export function DictDialogSection({ resources, state, actions }: DictManagementController) {
  return (
    <>
      <DictFormDialogs state={state} actions={actions} />
      <DictConfirmDialogs
        t={resources.t}
        deleteType={state.deleteType}
        deleteData={state.deleteData}
        batchDeleteTypeOpen={state.batchDeleteTypeOpen}
        batchDeleteDataOpen={state.batchDeleteDataOpen}
        selectedTypeCount={state.selectedTypeIds.length}
        selectedDataCount={state.selectedDataIds.length}
        onBatchDeleteTypeClose={() => state.setBatchDeleteTypeOpen(false)}
        onBatchDeleteDataClose={() => state.setBatchDeleteDataOpen(false)}
        onDeleteTypeClose={() => state.setDeleteType(null)}
        onDeleteDataClose={() => state.setDeleteData(null)}
        onBatchDeleteTypes={actions.confirmBatchDeleteTypes}
        onBatchDeleteData={actions.confirmBatchDeleteData}
        onDeleteType={actions.confirmDeleteType}
        onDeleteData={actions.confirmDeleteData}
      />
    </>
  );
}

type DictFormDialogsProps = Pick<DictManagementController, 'state' | 'actions'>;

function DictFormDialogs({ state, actions }: DictFormDialogsProps) {
  return (
    <>
      <DictTypeDialog
        open={state.creatingType || !!state.editingType}
        form={state.typeForm}
        submitting={state.submitting}
        editing={!!state.editingType}
        setForm={state.setTypeForm}
        onClose={() => closeTypeDialog(state.setEditingType, state.setCreatingType, state.setTypeForm)}
        onSubmit={actions.submitType}
      />
      <DictDataDialog
        open={state.creatingData || !!state.editingData}
        form={state.dataForm}
        submitting={state.submitting}
        editing={!!state.editingData}
        setForm={state.setDataForm}
        onClose={() => closeDataDialog(state.setEditingData, state.setCreatingData, state.setDataForm)}
        onSubmit={actions.submitData}
      />
    </>
  );
}
