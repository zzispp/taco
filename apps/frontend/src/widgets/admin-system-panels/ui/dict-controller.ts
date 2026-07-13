import type { DictData, DictType } from 'src/entities/system';
import type { useTranslate } from 'src/shared/i18n/use-locales';

import { useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';

import { systemMutations } from 'src/features/system-management';

import { useDictResources } from './dict-resources';
import { useDictToolActions } from './dict-tool-actions';
import { DEFAULT_DATA_INPUT, DEFAULT_TYPE_INPUT } from './dict-constants';
import { openDataEdit, openTypeEdit, closeDataDialog, closeTypeDialog } from './dict-helpers';

export function useDictManagementController() {
  const state = useDictState();
  const resources = useDictResources(state.selected);
  const open = useDictOpenActions({ state, activeType: resources.activeType });
  const submit = useDictSubmitActions({ state, activeType: resources.activeType, t: resources.t });
  const deletion = useDictDeleteActions({ state, t: resources.t });
  const tools = useDictToolActions({ resources, state });

  return { resources, state, actions: { ...open, ...submit, ...deletion, ...tools } };
}

export type DictManagementController = ReturnType<typeof useDictManagementController>;
export type DictManagementState = ReturnType<typeof useDictState>;

function useDictState() {
  const [selected, setSelected] = useState<DictType | null>(null);
  const [typeForm, setTypeForm] = useState(DEFAULT_TYPE_INPUT);
  const [dataForm, setDataForm] = useState(DEFAULT_DATA_INPUT);
  const [editingType, setEditingType] = useState<DictType | null>(null);
  const [editingData, setEditingData] = useState<DictData | null>(null);
  const [creatingType, setCreatingType] = useState(false);
  const [creatingData, setCreatingData] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteType, setDeleteType] = useState<DictType | null>(null);
  const [deleteData, setDeleteData] = useState<DictData | null>(null);
  const [batchDeleteTypeOpen, setBatchDeleteTypeOpen] = useState(false);
  const [batchDeleteDataOpen, setBatchDeleteDataOpen] = useState(false);
  const [selectedTypeIds, setSelectedTypeIds] = useState<string[]>([]);
  const [selectedDataIds, setSelectedDataIds] = useState<string[]>([]);

  return {
    selected,
    setSelected,
    typeForm,
    setTypeForm,
    dataForm,
    setDataForm,
    editingType,
    setEditingType,
    editingData,
    setEditingData,
    creatingType,
    setCreatingType,
    creatingData,
    setCreatingData,
    submitting,
    setSubmitting,
    deleteType,
    setDeleteType,
    deleteData,
    setDeleteData,
    batchDeleteTypeOpen,
    setBatchDeleteTypeOpen,
    batchDeleteDataOpen,
    setBatchDeleteDataOpen,
    selectedTypeIds,
    setSelectedTypeIds,
    selectedDataIds,
    setSelectedDataIds,
  };
}

type DictActionOptions = {
  state: ReturnType<typeof useDictState>;
  activeType: string;
  t: ReturnType<typeof useTranslate>['t'];
};

function useDictOpenActions({
  state,
  activeType,
}: Pick<DictActionOptions, 'state' | 'activeType'>) {
  const openCreateData = useCallback(() => {
    state.setCreatingData(true);
    state.setDataForm({ ...DEFAULT_DATA_INPUT, dict_type: activeType });
  }, [activeType, state]);
  const openType = useCallback(
    (item: DictType) => openTypeEdit(item, state.setEditingType, state.setTypeForm),
    [state]
  );
  const openData = useCallback(
    (item: DictData) => openDataEdit(item, state.setEditingData, state.setDataForm),
    [state]
  );

  return { openCreateData, openTypeEdit: openType, openDataEdit: openData };
}

function useDictSubmitActions({ state, activeType, t }: DictActionOptions) {
  const submitType = useCallback(async () => {
    state.setSubmitting(true);
    try {
      const item = state.editingType
        ? await systemMutations.updateDictType(state.editingType.dict_id, state.typeForm)
        : await systemMutations.createDictType(state.typeForm);
      state.setSelected(item);
      toast.success(t('messages.saved'));
      closeTypeDialog(state.setEditingType, state.setCreatingType, state.setTypeForm);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      state.setSubmitting(false);
    }
  }, [state, t]);
  const submitData = useSubmitDictData({ state, activeType, t });
  return { submitType, submitData };
}

function useSubmitDictData({ state, activeType, t }: DictActionOptions) {
  return useCallback(async () => {
    state.setSubmitting(true);
    try {
      const payload = { ...state.dataForm, dict_type: activeType };
      if (state.editingData)
        await systemMutations.updateDictData(state.editingData.dict_code, payload);
      else await systemMutations.createDictData(payload);
      toast.success(t('messages.saved'));
      closeDataDialog(state.setEditingData, state.setCreatingData, state.setDataForm);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      state.setSubmitting(false);
    }
  }, [activeType, state, t]);
}

function useDictDeleteActions({ state, t }: Omit<DictActionOptions, 'activeType'>) {
  const confirmDeleteType = useConfirmDeleteType(state, t);
  const confirmDeleteData = useConfirmDeleteData(state, t);
  const confirmBatchDeleteTypes = useConfirmBatchDeleteTypes(state, t);
  const confirmBatchDeleteData = useConfirmBatchDeleteData(state, t);

  return { confirmDeleteType, confirmDeleteData, confirmBatchDeleteTypes, confirmBatchDeleteData };
}

function useConfirmDeleteType(
  state: ReturnType<typeof useDictState>,
  t: ReturnType<typeof useTranslate>['t']
) {
  return useCallback(async () => {
    if (!state.deleteType) return;
    try {
      await systemMutations.deleteDictType(state.deleteType.dict_id);
      if (state.selected?.dict_id === state.deleteType.dict_id) state.setSelected(null);
      state.setSelectedTypeIds((current) =>
        current.filter((id) => id !== state.deleteType?.dict_id)
      );
      toast.success(t('messages.deleted'));
      state.setDeleteType(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [state, t]);
}

function useConfirmDeleteData(
  state: ReturnType<typeof useDictState>,
  t: ReturnType<typeof useTranslate>['t']
) {
  return useCallback(async () => {
    if (!state.deleteData) return;
    try {
      await systemMutations.deleteDictData(state.deleteData.dict_code);
      state.setSelectedDataIds((current) =>
        current.filter((id) => id !== state.deleteData?.dict_code)
      );
      toast.success(t('messages.deleted'));
      state.setDeleteData(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [state, t]);
}

function useConfirmBatchDeleteTypes(
  state: ReturnType<typeof useDictState>,
  t: ReturnType<typeof useTranslate>['t']
) {
  return useCallback(async () => {
    if (state.selectedTypeIds.length === 0) return;
    try {
      await systemMutations.deleteDictTypes(state.selectedTypeIds);
      if (state.selected && state.selectedTypeIds.includes(state.selected.dict_id))
        state.setSelected(null);
      toast.success(t('messages.deleted'));
      state.setSelectedTypeIds([]);
      state.setBatchDeleteTypeOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [state, t]);
}

function useConfirmBatchDeleteData(
  state: ReturnType<typeof useDictState>,
  t: ReturnType<typeof useTranslate>['t']
) {
  return useCallback(async () => {
    if (state.selectedDataIds.length === 0) return;
    try {
      await systemMutations.deleteDictDataBatch(state.selectedDataIds);
      toast.success(t('messages.deleted'));
      state.setSelectedDataIds([]);
      state.setBatchDeleteDataOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [state, t]);
}
