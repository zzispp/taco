import type { CrudPanelProps } from './types';

import { useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { withSelectionHead } from 'src/shared/ui/admin';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useHasPermission } from 'src/entities/session';

import { tableHead } from './helpers';

type CrudRecord = Record<string, unknown>;

export function useSystemCrudController<T extends CrudRecord, I extends CrudRecord>(
  props: CrudPanelProps<T, I>
) {
  const { t } = useTranslate('admin');
  const state = useCrudState<T, I>(props.defaultInput);
  const permissions = useCrudPermissions(props.permissionPrefix, props.batchDeleteItems);
  const selectableRows = props.resource.items.filter(props.isRowSelectable ?? (() => true));
  const head = tableHead({
    fields: props.fields,
    hasExtra: !!props.extraActions,
    hasSelection: permissions.hasBatchDelete,
    actionLabel: t('common.actions'),
  });
  const bodyHead = permissions.hasBatchDelete ? withSelectionHead(head) : head;
  const actions = useCrudActions({ props, state, selectableRows, t });

  return { t, state, permissions, selectableRows, head, bodyHead, actions };
}

export type SystemCrudController<
  T extends CrudRecord,
  I extends CrudRecord,
> = ReturnType<typeof useSystemCrudController<T, I>>;

function useCrudState<T extends CrudRecord, I extends CrudRecord>(defaultInput: I) {
  const [form, setForm] = useState<I>(defaultInput);
  const [editing, setEditing] = useState<T | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<T | null>(null);
  const [batchDeleteOpen, setBatchDeleteOpen] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);

  return { form, setForm, editing, setEditing, creating, setCreating, submitting, setSubmitting, deleteTarget, setDeleteTarget, batchDeleteOpen, setBatchDeleteOpen, selected, setSelected };
}

function useCrudPermissions(permissionPrefix: string, batchDeleteItems?: (ids: string[]) => Promise<void>) {
  const canAdd = useHasPermission(`${permissionPrefix}:add`);
  const canDelete = useHasPermission(`${permissionPrefix}:remove`);
  const hasBatchDelete = !!batchDeleteItems && canDelete;

  return { canAdd, canDelete, hasBatchDelete };
}

type CrudActionsOptions<T extends CrudRecord, I extends CrudRecord> = {
  props: CrudPanelProps<T, I>;
  state: ReturnType<typeof useCrudState<T, I>>;
  selectableRows: T[];
  t: ReturnType<typeof useTranslate>['t'];
};

function useCrudActions<T extends CrudRecord, I extends CrudRecord>(options: CrudActionsOptions<T, I>) {
  const { props, state, selectableRows, t } = options;
  const closeDialog = useCallback(() => {
    state.setEditing(null);
    state.setCreating(false);
    state.setForm(props.defaultInput);
  }, [props.defaultInput, state]);
  const submit = useSubmitAction({ props, state, closeDialog, t });
  const confirmDelete = useDeleteAction({ props, state, t });
  const confirmBatchDelete = useBatchDeleteAction({ props, state, t });
  const toggleAll = useCallback(
    (checked: boolean) => state.setSelected(checked ? selectableRows.map((row) => String(row[props.idKey])) : []),
    [props.idKey, selectableRows, state]
  );

  return { closeDialog, submit, confirmDelete, confirmBatchDelete, toggleAll };
}

type SubmitActionOptions<T extends CrudRecord, I extends CrudRecord> = {
  props: CrudPanelProps<T, I>;
  state: ReturnType<typeof useCrudState<T, I>>;
  closeDialog: () => void;
  t: ReturnType<typeof useTranslate>['t'];
};

function useSubmitAction<T extends CrudRecord, I extends CrudRecord>({ props, state, closeDialog, t }: SubmitActionOptions<T, I>) {
  return useCallback(async () => {
    state.setSubmitting(true);
    try {
      if (state.editing) await props.updateItem(String(state.editing[props.idKey]), state.form);
      else await props.createItem(state.form);
      toast.success(t('messages.saved'));
      props.onAfterSave?.();
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      state.setSubmitting(false);
    }
  }, [closeDialog, props, state, t]);
}

function useDeleteAction<T extends CrudRecord, I extends CrudRecord>({ props, state, t }: Omit<SubmitActionOptions<T, I>, 'closeDialog'>) {
  return useCallback(async () => {
    if (!state.deleteTarget) return;
    try {
      await props.deleteItem(String(state.deleteTarget[props.idKey]));
      toast.success(t('messages.deleted'));
      state.setDeleteTarget(null);
      state.setSelected((current) => current.filter((id) => id !== String(state.deleteTarget?.[props.idKey])));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [props, state, t]);
}

function useBatchDeleteAction<T extends CrudRecord, I extends CrudRecord>({ props, state, t }: Omit<SubmitActionOptions<T, I>, 'closeDialog'>) {
  return useCallback(async () => {
    if (!props.batchDeleteItems || state.selected.length === 0) return;
    try {
      await props.batchDeleteItems(state.selected);
      toast.success(t('messages.deleted'));
      state.setSelected([]);
      state.setBatchDeleteOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [props, state, t]);
}
