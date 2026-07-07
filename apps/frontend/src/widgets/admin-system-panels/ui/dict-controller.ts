import type { DictData, DictType } from 'src/entities/system';

import { useMemo, useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTable } from 'src/shared/ui/table';
import { withSelectionHead } from 'src/shared/ui/admin';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useHasPermission } from 'src/entities/session';
import { useDictData, useDictTypes } from 'src/entities/system';

import { systemMutations } from 'src/features/system-management';

import {
  DEFAULT_DATA_INPUT,
  DEFAULT_TYPE_INPUT,
  DEFAULT_DATA_FILTERS,
  DEFAULT_TYPE_FILTERS,
} from './dict-constants';
import {
  dictDataHead,
  dictTypeHead,
  openDataEdit,
  openTypeEdit,
  closeDataDialog,
  closeTypeDialog,
} from './dict-helpers';

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

  return { selected, setSelected, typeForm, setTypeForm, dataForm, setDataForm, editingType, setEditingType, editingData, setEditingData, creatingType, setCreatingType, creatingData, setCreatingData, submitting, setSubmitting, deleteType, setDeleteType, deleteData, setDeleteData, batchDeleteTypeOpen, setBatchDeleteTypeOpen, batchDeleteDataOpen, setBatchDeleteDataOpen, selectedTypeIds, setSelectedTypeIds, selectedDataIds, setSelectedDataIds };
}

function useDictResources(selected: DictType | null) {
  const { t } = useTranslate('admin');
  const typeTable = useTable({ defaultRowsPerPage: 10 });
  const dataTable = useTable({ defaultRowsPerPage: 10 });
  const [typeFilters, setTypeFilters] = useState(DEFAULT_TYPE_FILTERS);
  const [dataFilters, setDataFilters] = useState(DEFAULT_DATA_FILTERS);
  const dictTypes = useDictTypes(typeTable.page, typeTable.rowsPerPage, typeFilters);
  const activeType = selected?.dict_type ?? dictTypes.items[0]?.dict_type ?? '';
  const dictData = useDictData(dataTable.page, dataTable.rowsPerPage, { ...dataFilters, dict_type: activeType });
  const canAdd = useHasPermission('system:dict:add');
  const canRemove = useHasPermission('system:dict:remove');
  const canExport = useHasPermission('system:dict:export');
  const typeHead = useMemo(() => dictTypeHead(t), [t]);
  const dataHead = useMemo(() => dictDataHead(t), [t]);
  const loadingTypeHead = useMemo(() => (canRemove ? withSelectionHead(typeHead) : typeHead), [canRemove, typeHead]);
  const loadingDataHead = useMemo(() => (canRemove ? withSelectionHead(dataHead) : dataHead), [canRemove, dataHead]);

  return { t, typeTable, dataTable, typeFilters, setTypeFilters, dataFilters, setDataFilters, dictTypes, activeType, dictData, canAdd, canRemove, canExport, typeHead, dataHead, loadingTypeHead, loadingDataHead };
}

type DictActionOptions = {
  state: ReturnType<typeof useDictState>;
  activeType: string;
  t: ReturnType<typeof useTranslate>['t'];
};

function useDictOpenActions({ state, activeType }: Pick<DictActionOptions, 'state' | 'activeType'>) {
  const openCreateData = useCallback(() => {
    state.setCreatingData(true);
    state.setDataForm({ ...DEFAULT_DATA_INPUT, dict_type: activeType });
  }, [activeType, state]);
  const openType = useCallback((item: DictType) => openTypeEdit(item, state.setEditingType, state.setTypeForm), [state]);
  const openData = useCallback((item: DictData) => openDataEdit(item, state.setEditingData, state.setDataForm), [state]);

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
      if (state.editingData) await systemMutations.updateDictData(state.editingData.dict_code, payload);
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

function useConfirmDeleteType(state: ReturnType<typeof useDictState>, t: ReturnType<typeof useTranslate>['t']) {
  return useCallback(async () => {
    if (!state.deleteType) return;
    try {
      await systemMutations.deleteDictType(state.deleteType.dict_id);
      if (state.selected?.dict_id === state.deleteType.dict_id) state.setSelected(null);
      state.setSelectedTypeIds((current) => current.filter((id) => id !== state.deleteType?.dict_id));
      toast.success(t('messages.deleted'));
      state.setDeleteType(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [state, t]);
}

function useConfirmDeleteData(state: ReturnType<typeof useDictState>, t: ReturnType<typeof useTranslate>['t']) {
  return useCallback(async () => {
    if (!state.deleteData) return;
    try {
      await systemMutations.deleteDictData(state.deleteData.dict_code);
      state.setSelectedDataIds((current) => current.filter((id) => id !== state.deleteData?.dict_code));
      toast.success(t('messages.deleted'));
      state.setDeleteData(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [state, t]);
}

function useConfirmBatchDeleteTypes(state: ReturnType<typeof useDictState>, t: ReturnType<typeof useTranslate>['t']) {
  return useCallback(async () => {
    if (state.selectedTypeIds.length === 0) return;
    try {
      await systemMutations.deleteDictTypes(state.selectedTypeIds);
      if (state.selected && state.selectedTypeIds.includes(state.selected.dict_id)) state.setSelected(null);
      toast.success(t('messages.deleted'));
      state.setSelectedTypeIds([]);
      state.setBatchDeleteTypeOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [state, t]);
}

function useConfirmBatchDeleteData(state: ReturnType<typeof useDictState>, t: ReturnType<typeof useTranslate>['t']) {
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

type DictToolOptions = {
  resources: ReturnType<typeof useDictResources>;
  state: ReturnType<typeof useDictState>;
};

function useDictToolActions({ resources, state }: DictToolOptions) {
  const refreshCache = useRefreshDictCache(resources.t);
  const exportTypes = useExportDictTypes(resources);
  const exportData = useExportDictData(resources);
  const toggleAllTypes = useCallback((checked: boolean) => {
    state.setSelectedTypeIds(checked ? resources.dictTypes.items.map((item) => item.dict_id) : []);
  }, [resources.dictTypes.items, state]);
  const toggleAllData = useCallback((checked: boolean) => {
    state.setSelectedDataIds(checked ? resources.dictData.items.map((item) => item.dict_code) : []);
  }, [resources.dictData.items, state]);

  return { refreshCache, exportTypes, exportData, toggleAllTypes, toggleAllData };
}

function useRefreshDictCache(t: ReturnType<typeof useTranslate>['t']) {
  return useCallback(async () => {
    try {
      await systemMutations.refreshDictCache();
      toast.success(t('messages.cacheRefreshed'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    }
  }, [t]);
}

function useExportDictTypes(resources: ReturnType<typeof useDictResources>) {
  return useCallback(async () => {
    try {
      await systemMutations.exportDictTypes(resources.typeFilters);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : resources.t('messages.exportFailed'));
    }
  }, [resources]);
}

function useExportDictData(resources: ReturnType<typeof useDictResources>) {
  return useCallback(async () => {
    if (!resources.activeType) return;
    try {
      await systemMutations.exportDictData({ ...resources.dataFilters, dict_type: resources.activeType });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : resources.t('messages.exportFailed'));
    }
  }, [resources]);
}
