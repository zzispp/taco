'use client';

import type { DictData, DictType } from 'src/entities/system';

import { useMemo, useState, useCallback } from 'react';

import Stack from '@mui/material/Stack';

import { toast } from 'src/shared/ui/snackbar';
import { useTable } from 'src/shared/ui/table';
import { withSelectionHead } from 'src/shared/ui/admin';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useHasPermission } from 'src/entities/session';
import { useDictData, useDictTypes } from 'src/entities/system';

import { systemMutations } from 'src/features/system-management';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

import { DictHeaderActions } from './dict-toolbar';
import { DictConfirmDialogs } from './dict-confirm-dialogs';
import { DictDataSection, DictTypeSection } from './dict-sections';
import { DictDataDialog, DictTypeDialog } from './dict-form-dialogs';
import {
  DEFAULT_DATA_INPUT,
  DEFAULT_TYPE_INPUT,
  DEFAULT_DATA_FILTERS,
  DEFAULT_TYPE_FILTERS,
} from './dict-constants';
import {
  toggle,
  dictDataHead,
  dictTypeHead,
  openDataEdit,
  openTypeEdit,
  closeDataDialog,
  closeTypeDialog,
} from './dict-helpers';

export function DictManagementPanel() {
  const { t } = useTranslate('admin');
  const typeTable = useTable({ defaultRowsPerPage: 10 });
  const dataTable = useTable({ defaultRowsPerPage: 10 });
  const [typeFilters, setTypeFilters] = useState(DEFAULT_TYPE_FILTERS);
  const [dataFilters, setDataFilters] = useState(DEFAULT_DATA_FILTERS);
  const dictTypes = useDictTypes(typeTable.page, typeTable.rowsPerPage, typeFilters);
  const [selected, setSelected] = useState<DictType | null>(null);
  const activeType = selected?.dict_type ?? dictTypes.items[0]?.dict_type ?? '';
  const dictData = useDictData(dataTable.page, dataTable.rowsPerPage, {
    ...dataFilters,
    dict_type: activeType,
  });
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
  const canAdd = useHasPermission('system:dict:add');
  const canRemove = useHasPermission('system:dict:remove');
  const canExport = useHasPermission('system:dict:export');
  const typeHead = useMemo(() => dictTypeHead(t), [t]);
  const dataHead = useMemo(() => dictDataHead(t), [t]);
  const loadingTypeHead = useMemo(
    () => (canRemove ? withSelectionHead(typeHead) : typeHead),
    [canRemove, typeHead]
  );
  const loadingDataHead = useMemo(
    () => (canRemove ? withSelectionHead(dataHead) : dataHead),
    [canRemove, dataHead]
  );

  const refreshCache = useCallback(async () => {
    try {
      await systemMutations.refreshDictCache();
      toast.success(t('messages.cacheRefreshed'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    }
  }, [t]);

  const submitType = useCallback(async () => {
    setSubmitting(true);
    try {
      const item = editingType
        ? await systemMutations.updateDictType(editingType.dict_id, typeForm)
        : await systemMutations.createDictType(typeForm);
      setSelected(item);
      toast.success(t('messages.saved'));
      closeTypeDialog(setEditingType, setCreatingType, setTypeForm);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [editingType, t, typeForm]);

  const submitData = useCallback(async () => {
    setSubmitting(true);
    try {
      const payload = { ...dataForm, dict_type: activeType };
      if (editingData) await systemMutations.updateDictData(editingData.dict_code, payload);
      else await systemMutations.createDictData(payload);
      toast.success(t('messages.saved'));
      closeDataDialog(setEditingData, setCreatingData, setDataForm);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [activeType, dataForm, editingData, t]);

  const confirmDeleteType = useCallback(async () => {
    if (!deleteType) return;
    try {
      await systemMutations.deleteDictType(deleteType.dict_id);
      if (selected?.dict_id === deleteType.dict_id) setSelected(null);
      setSelectedTypeIds((current) => current.filter((id) => id !== deleteType.dict_id));
      toast.success(t('messages.deleted'));
      setDeleteType(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteType, selected?.dict_id, t]);

  const confirmDeleteData = useCallback(async () => {
    if (!deleteData) return;
    try {
      await systemMutations.deleteDictData(deleteData.dict_code);
      setSelectedDataIds((current) => current.filter((id) => id !== deleteData.dict_code));
      toast.success(t('messages.deleted'));
      setDeleteData(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteData, t]);

  const confirmBatchDeleteTypes = useCallback(async () => {
    if (selectedTypeIds.length === 0) return;
    try {
      await systemMutations.deleteDictTypes(selectedTypeIds);
      if (selected && selectedTypeIds.includes(selected.dict_id)) setSelected(null);
      toast.success(t('messages.deleted'));
      setSelectedTypeIds([]);
      setBatchDeleteTypeOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [selected, selectedTypeIds, t]);

  const confirmBatchDeleteData = useCallback(async () => {
    if (selectedDataIds.length === 0) return;
    try {
      await systemMutations.deleteDictDataBatch(selectedDataIds);
      toast.success(t('messages.deleted'));
      setSelectedDataIds([]);
      setBatchDeleteDataOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [selectedDataIds, t]);

  const exportTypes = useCallback(async () => {
    try {
      await systemMutations.exportDictTypes(typeFilters);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.exportFailed'));
    }
  }, [t, typeFilters]);

  const exportData = useCallback(async () => {
    if (!activeType) return;
    try {
      await systemMutations.exportDictData({ ...dataFilters, dict_type: activeType });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.exportFailed'));
    }
  }, [activeType, dataFilters, t]);

  return (
    <DashboardContent>
      <AdminBreadcrumbs heading={t('pages.dictManagement')} action={headerActions()} />
      <Stack spacing={3}>
        <DictTypeSection
          table={typeTable}
          filters={typeFilters}
          resource={dictTypes}
          activeType={activeType}
          head={typeHead}
          loadingHead={loadingTypeHead}
          selectedIds={selectedTypeIds}
          canRemove={canRemove}
          onFilterChange={setTypeFilters}
          onToggleAll={(checked) =>
            setSelectedTypeIds(checked ? dictTypes.items.map((item) => item.dict_id) : [])
          }
          onToggleRow={(id) => setSelectedTypeIds(toggle(selectedTypeIds, id))}
          onSelect={setSelected}
          onEdit={(item) => openTypeEdit(item, setEditingType, setTypeForm)}
          onDelete={setDeleteType}
        />
        <DictDataSection
          table={dataTable}
          filters={dataFilters}
          resource={dictData}
          activeType={activeType}
          head={dataHead}
          loadingHead={loadingDataHead}
          selectedIds={selectedDataIds}
          canAdd={canAdd}
          canRemove={canRemove}
          canExport={canExport}
          onFilterChange={setDataFilters}
          onToggleAll={(checked) =>
            setSelectedDataIds(checked ? dictData.items.map((item) => item.dict_code) : [])
          }
          onToggleRow={(id) => setSelectedDataIds(toggle(selectedDataIds, id))}
          onEdit={(item) => openDataEdit(item, setEditingData, setDataForm)}
          onDelete={setDeleteData}
          onAdd={openCreateData}
          onBatchDelete={() => setBatchDeleteDataOpen(true)}
          onExport={exportData}
        />
      </Stack>
      {formDialogs()}
      <DictConfirmDialogs
        t={t}
        deleteType={deleteType}
        deleteData={deleteData}
        batchDeleteTypeOpen={batchDeleteTypeOpen}
        batchDeleteDataOpen={batchDeleteDataOpen}
        selectedTypeCount={selectedTypeIds.length}
        selectedDataCount={selectedDataIds.length}
        onBatchDeleteTypeClose={() => setBatchDeleteTypeOpen(false)}
        onBatchDeleteDataClose={() => setBatchDeleteDataOpen(false)}
        onDeleteTypeClose={() => setDeleteType(null)}
        onDeleteDataClose={() => setDeleteData(null)}
        onBatchDeleteTypes={confirmBatchDeleteTypes}
        onBatchDeleteData={confirmBatchDeleteData}
        onDeleteType={confirmDeleteType}
        onDeleteData={confirmDeleteData}
      />
    </DashboardContent>
  );

  function headerActions() {
    return (
      <DictHeaderActions
        t={t}
        canAdd={canAdd}
        canExport={canExport}
        canRefresh={canRemove}
        canRemove={canRemove}
        selectedCount={selectedTypeIds.length}
        onAdd={() => setCreatingType(true)}
        onExport={exportTypes}
        onRefresh={refreshCache}
        onBatchDelete={() => setBatchDeleteTypeOpen(true)}
      />
    );
  }

  function openCreateData() {
    setCreatingData(true);
    setDataForm({ ...DEFAULT_DATA_INPUT, dict_type: activeType });
  }

  function formDialogs() {
    return (
      <>
        <DictTypeDialog
          open={creatingType || !!editingType}
          form={typeForm}
          submitting={submitting}
          editing={!!editingType}
          setForm={setTypeForm}
          onClose={() => closeTypeDialog(setEditingType, setCreatingType, setTypeForm)}
          onSubmit={submitType}
        />
        <DictDataDialog
          open={creatingData || !!editingData}
          form={dataForm}
          submitting={submitting}
          editing={!!editingData}
          setForm={setDataForm}
          onClose={() => closeDataDialog(setEditingData, setCreatingData, setDataForm)}
          onSubmit={submitData}
        />
      </>
    );
  }
}
