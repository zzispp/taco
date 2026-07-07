import type { CrudField, CrudPanelProps } from './types';
import type { SystemCrudController } from './controller';

import Button from '@mui/material/Button';

import { ManagementDialog } from 'src/shared/ui/admin';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

import { CrudFieldControl } from './field-control';

type CrudRecord = Record<string, unknown>;

type CrudDialogSectionProps<T extends CrudRecord, I extends CrudRecord> = {
  props: CrudPanelProps<T, I>;
  controller: SystemCrudController<T, I>;
};

export function CrudDialogSection<T extends CrudRecord, I extends CrudRecord>({
  props,
  controller,
}: CrudDialogSectionProps<T, I>) {
  return (
    <>
      <CrudEditDialog props={props} controller={controller} />
      <CrudDeleteDialogs props={props} controller={controller} />
    </>
  );
}

function CrudEditDialog<T extends CrudRecord, I extends CrudRecord>({ props, controller }: CrudDialogSectionProps<T, I>) {
  const { t, state, actions } = controller;

  return (
    <ManagementDialog
      open={state.creating || !!state.editing}
      title={state.editing ? t('common.edit') : t('common.create')}
      submitting={state.submitting}
      onClose={actions.closeDialog}
      onSubmit={actions.submit}
    >
      {props.fields.filter((field) => !field.hiddenInForm).map((field) => (
        <CrudFieldControl
          key={String(field.key)}
          field={field as unknown as CrudField<I>}
          editing={state.editing}
          form={state.form}
          setForm={state.setForm}
        />
      ))}
    </ManagementDialog>
  );
}

function CrudDeleteDialogs<T extends CrudRecord, I extends CrudRecord>({ props, controller }: CrudDialogSectionProps<T, I>) {
  const { t, state, actions } = controller;

  return (
    <>
      <ConfirmDialog
        open={state.batchDeleteOpen}
        onClose={() => state.setBatchDeleteOpen(false)}
        title={t('common.delete')}
        content={t('dialogs.deleteContent', { name: String(state.selected.length) })}
        cancelText={t('common.cancel')}
        action={<DeleteButton label={t('common.delete')} onClick={actions.confirmBatchDelete} />}
      />
      <ConfirmDialog
        open={!!state.deleteTarget}
        onClose={() => state.setDeleteTarget(null)}
        title={t('common.delete')}
        content={t('dialogs.deleteContent', { name: state.deleteTarget ? String(state.deleteTarget[props.nameKey]) : '' })}
        cancelText={t('common.cancel')}
        action={<DeleteButton label={t('common.delete')} onClick={actions.confirmDelete} />}
      />
    </>
  );
}

function DeleteButton({ label, onClick }: { label: string; onClick: () => void }) {
  return (
    <Button variant="contained" color="error" onClick={onClick}>
      {label}
    </Button>
  );
}
