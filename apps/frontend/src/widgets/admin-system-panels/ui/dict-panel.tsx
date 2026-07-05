'use client';

import type { LabelColor } from 'src/shared/ui/label';
import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { DictData, DictType, DictDataInput, DictTypeInput } from 'src/entities/system';

import { useMemo, useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import TableRow from '@mui/material/TableRow';
import TextField from '@mui/material/TextField';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { Label } from 'src/shared/ui/label';
import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';
import { useTable, TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import {
  AddButton,
  StatusLabel,
  TextFieldRow,
  AdminBreadcrumbs,
  ManagementDialog,
  TableLoadingRows,
  withSelectionHead,
  ManagementTableHead,
} from 'src/shared/ui/admin';

import { useHasPermission } from 'src/entities/session';
import { useDictData, useDictTypes } from 'src/entities/system';

import { systemMutations } from 'src/features/system-management/api';

import { DashboardContent } from 'src/widgets/dashboard-shell';

const DEFAULT_TYPE_INPUT: DictTypeInput = { dict_name: '', dict_type: '', status: '0', remark: '' };
const DEFAULT_DATA_INPUT: DictDataInput = { dict_sort: 0, dict_label: '', dict_value: '', dict_type: '', css_class: '', list_class: 'default', is_default: 'N', status: '0', remark: '' };
const DEFAULT_TYPE_FILTERS = { dict_name: '', dict_type: '', status: '', begin_time: '', end_time: '' };
const DEFAULT_DATA_FILTERS = { dict_label: '', status: '' };

export function DictManagementPanel() {
  const { t } = useTranslate('admin');
  const typeTable = useTable({ defaultRowsPerPage: 10 });
  const dataTable = useTable({ defaultRowsPerPage: 10 });
  const [typeFilters, setTypeFilters] = useState(DEFAULT_TYPE_FILTERS);
  const [dataFilters, setDataFilters] = useState(DEFAULT_DATA_FILTERS);
  const dictTypes = useDictTypes(typeTable.page, typeTable.rowsPerPage, typeFilters);
  const [selected, setSelected] = useState<DictType | null>(null);
  const activeType = selected?.dict_type ?? dictTypes.items[0]?.dict_type ?? '';
  const dictData = useDictData(dataTable.page, dataTable.rowsPerPage, { ...dataFilters, dict_type: activeType });
  const [typeForm, setTypeForm] = useState<DictTypeInput>(DEFAULT_TYPE_INPUT);
  const [dataForm, setDataForm] = useState<DictDataInput>(DEFAULT_DATA_INPUT);
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
  const canRefresh = canRemove;
  const typeHead = useMemo(() => dictTypeHead(t), [t]);
  const dataHead = useMemo(() => dictDataHead(t), [t]);
  const loadingTypeHead = useMemo(() => canRemove ? withSelectionHead(typeHead) : typeHead, [canRemove, typeHead]);
  const loadingDataHead = useMemo(() => canRemove ? withSelectionHead(dataHead) : dataHead, [canRemove, dataHead]);

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
      const item = editingType ? await systemMutations.updateDictType(editingType.dict_id, typeForm) : await systemMutations.createDictType(typeForm);
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

  const toggleAllTypes = useCallback((checked: boolean) => {
    setSelectedTypeIds(checked ? dictTypes.items.map((item) => item.dict_id) : []);
  }, [dictTypes.items]);

  const toggleAllData = useCallback((checked: boolean) => {
    setSelectedDataIds(checked ? dictData.items.map((item) => item.dict_code) : []);
  }, [dictData.items]);

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.dictManagement')}
        action={<Stack direction="row" spacing={1}>{canRefresh && <Button variant="outlined" startIcon={<Iconify icon="solar:restart-bold" />} onClick={refreshCache}>{t('actions.refreshCache')}</Button>}{canRemove && <Button variant="outlined" color="error" disabled={selectedTypeIds.length === 0} onClick={() => setBatchDeleteTypeOpen(true)}>{t('common.delete')}</Button>}{canAdd && <AddButton onClick={() => setCreatingType(true)}>{t('actions.addDict')}</AddButton>}</Stack>}
      />
      <Stack spacing={3}>
        <Card>
          <DictTypeFilters filters={typeFilters} onChange={setTypeFilters} />
          <Scrollbar>
            <Table sx={{ minWidth: 920 }}>
              <ManagementTableHead head={typeHead} rowCount={dictTypes.items.length} numSelected={selectedTypeIds.length} onSelectAllRows={canRemove ? toggleAllTypes : undefined} />
              <TableBody>
                {dictTypes.isLoading ? <TableLoadingRows head={loadingTypeHead} rows={typeTable.rowsPerPage} /> : dictTypes.items.map((row) => <DictTypeRow key={row.dict_id} row={row} selected={row.dict_type === activeType} checked={selectedTypeIds.includes(row.dict_id)} canRemove={canRemove} onCheck={(id) => setSelectedTypeIds(toggle(selectedTypeIds, id))} onSelect={setSelected} onEdit={(item) => openTypeEdit(item, setEditingType, setTypeForm)} onDelete={setDeleteType} />)}
                <TableNoData title={t('common.noData')} notFound={!dictTypes.isLoading && dictTypes.items.length === 0} />
              </TableBody>
            </Table>
          </Scrollbar>
          <TablePaginationCustom page={typeTable.page} count={dictTypes.total} rowsPerPage={typeTable.rowsPerPage} onPageChange={typeTable.onChangePage} onRowsPerPageChange={typeTable.onChangeRowsPerPage} />
        </Card>
        <Card>
          <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ p: 2 }}>
            <Typography variant="h6">{t('fields.dictData')}：{activeType || '-'}</Typography>
            <Stack direction="row" spacing={1}>{canRemove && <Button variant="outlined" color="error" disabled={selectedDataIds.length === 0} onClick={() => setBatchDeleteDataOpen(true)}>{t('common.delete')}</Button>}
            {canAdd && <Button variant="contained" startIcon={<Iconify icon="mingcute:add-line" />} disabled={!activeType} onClick={() => { setCreatingData(true); setDataForm({ ...DEFAULT_DATA_INPUT, dict_type: activeType }); }}>{t('actions.addDictData')}</Button>}
            </Stack>
          </Stack>
          <DictDataFilters filters={dataFilters} onChange={setDataFilters} />
          <Scrollbar>
            <Table sx={{ minWidth: 1080 }}>
              <ManagementTableHead head={dataHead} rowCount={dictData.items.length} numSelected={selectedDataIds.length} onSelectAllRows={canRemove ? toggleAllData : undefined} />
              <TableBody>
                {dictData.isLoading ? <TableLoadingRows head={loadingDataHead} rows={dataTable.rowsPerPage} /> : dictData.items.map((row) => <DictDataRow key={row.dict_code} row={row} selected={selectedDataIds.includes(row.dict_code)} canRemove={canRemove} onCheck={(id) => setSelectedDataIds(toggle(selectedDataIds, id))} onEdit={(item) => openDataEdit(item, setEditingData, setDataForm)} onDelete={setDeleteData} />)}
                <TableNoData title={t('common.noData')} notFound={!dictData.isLoading && dictData.items.length === 0} />
              </TableBody>
            </Table>
          </Scrollbar>
          <TablePaginationCustom page={dataTable.page} count={dictData.total} rowsPerPage={dataTable.rowsPerPage} onPageChange={dataTable.onChangePage} onRowsPerPageChange={dataTable.onChangeRowsPerPage} />
        </Card>
      </Stack>
      <DictTypeDialog open={creatingType || !!editingType} form={typeForm} submitting={submitting} editing={!!editingType} setForm={setTypeForm} onClose={() => closeTypeDialog(setEditingType, setCreatingType, setTypeForm)} onSubmit={submitType} />
      <DictDataDialog open={creatingData || !!editingData} form={dataForm} submitting={submitting} editing={!!editingData} setForm={setDataForm} onClose={() => closeDataDialog(setEditingData, setCreatingData, setDataForm)} onSubmit={submitData} />
      <ConfirmDialog open={batchDeleteTypeOpen} onClose={() => setBatchDeleteTypeOpen(false)} title={t('common.delete')} content={t('dialogs.deleteContent', { name: String(selectedTypeIds.length) })} cancelText={t('common.cancel')} action={<Button variant="contained" color="error" onClick={confirmBatchDeleteTypes}>{t('common.delete')}</Button>} />
      <ConfirmDialog open={batchDeleteDataOpen} onClose={() => setBatchDeleteDataOpen(false)} title={t('common.delete')} content={t('dialogs.deleteContent', { name: String(selectedDataIds.length) })} cancelText={t('common.cancel')} action={<Button variant="contained" color="error" onClick={confirmBatchDeleteData}>{t('common.delete')}</Button>} />
      <ConfirmDialog open={!!deleteType} onClose={() => setDeleteType(null)} title={t('common.delete')} content={t('dialogs.deleteContent', { name: deleteType?.dict_name ?? '' })} cancelText={t('common.cancel')} action={<Button variant="contained" color="error" onClick={confirmDeleteType}>{t('common.delete')}</Button>} />
      <ConfirmDialog open={!!deleteData} onClose={() => setDeleteData(null)} title={t('common.delete')} content={t('dialogs.deleteContent', { name: deleteData?.dict_label ?? '' })} cancelText={t('common.cancel')} action={<Button variant="contained" color="error" onClick={confirmDeleteData}>{t('common.delete')}</Button>} />
    </DashboardContent>
  );
}

function DictTypeFilters({ filters, onChange }: { filters: typeof DEFAULT_TYPE_FILTERS; onChange: (filters: typeof DEFAULT_TYPE_FILTERS) => void }) {
  const { t } = useTranslate('admin');
  const write = (key: keyof typeof DEFAULT_TYPE_FILTERS, value: string) => onChange({ ...filters, [key]: value });
  return <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2 }}><TextField size="small" label={t('fields.dictName')} value={filters.dict_name} onChange={(event) => write('dict_name', event.target.value)} /><TextField size="small" label={t('fields.dictType')} value={filters.dict_type} onChange={(event) => write('dict_type', event.target.value)} /><TextField select size="small" label={t('common.status')} value={filters.status} sx={{ minWidth: 140 }} onChange={(event) => write('status', event.target.value)}><MenuItem value="">{t('common.all')}</MenuItem><MenuItem value="0">{t('common.enabled')}</MenuItem><MenuItem value="1">{t('common.disabled')}</MenuItem></TextField><TextField size="small" type="date" label={t('fields.beginTime')} value={filters.begin_time} InputLabelProps={{ shrink: true }} onChange={(event) => write('begin_time', event.target.value)} /><TextField size="small" type="date" label={t('fields.endTime')} value={filters.end_time} InputLabelProps={{ shrink: true }} onChange={(event) => write('end_time', event.target.value)} /><Button variant="outlined" onClick={() => onChange(DEFAULT_TYPE_FILTERS)}>{t('common.reset')}</Button></Stack>;
}

function DictDataFilters({ filters, onChange }: { filters: typeof DEFAULT_DATA_FILTERS; onChange: (filters: typeof DEFAULT_DATA_FILTERS) => void }) {
  const { t } = useTranslate('admin');
  const write = (key: keyof typeof DEFAULT_DATA_FILTERS, value: string) => onChange({ ...filters, [key]: value });
  return <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ px: 2, pb: 2 }}><TextField size="small" label={t('fields.dictLabel')} value={filters.dict_label} onChange={(event) => write('dict_label', event.target.value)} /><TextField select size="small" label={t('common.status')} value={filters.status} sx={{ minWidth: 140 }} onChange={(event) => write('status', event.target.value)}><MenuItem value="">{t('common.all')}</MenuItem><MenuItem value="0">{t('common.enabled')}</MenuItem><MenuItem value="1">{t('common.disabled')}</MenuItem></TextField><Button variant="outlined" onClick={() => onChange(DEFAULT_DATA_FILTERS)}>{t('common.reset')}</Button></Stack>;
}

function DictTypeRow({ row, selected, checked, canRemove, onCheck, onSelect, onEdit, onDelete }: { row: DictType; selected: boolean; checked: boolean; canRemove: boolean; onCheck: (id: string) => void; onSelect: (row: DictType) => void; onEdit: (row: DictType) => void; onDelete: (row: DictType) => void }) {
  return <TableRow hover selected={selected} onClick={() => onSelect(row)}>{canRemove && <TableCell padding="checkbox"><Checkbox checked={checked} onClick={(event) => event.stopPropagation()} onChange={() => onCheck(row.dict_id)} /></TableCell>}<TableCell>{row.dict_name}</TableCell><TableCell sx={{ fontFamily: 'monospace' }}>{row.dict_type}</TableCell><TableCell><StatusLabel status={row.status} /></TableCell><TableCell>{row.remark || '-'}</TableCell><TableCell sx={DATE_TIME_CELL_SX}>{fAdminDateTime(row.create_time) || '-'}</TableCell><TableCell align="right"><RowActions onEdit={() => onEdit(row)} onDelete={() => onDelete(row)} editDisabled={!useHasPermission('system:dict:edit')} deleteDisabled={!useHasPermission('system:dict:remove')} /></TableCell></TableRow>;
}

function DictDataRow({ row, selected, canRemove, onCheck, onEdit, onDelete }: { row: DictData; selected: boolean; canRemove: boolean; onCheck: (id: string) => void; onEdit: (row: DictData) => void; onDelete: (row: DictData) => void }) {
  return <TableRow hover>{canRemove && <TableCell padding="checkbox"><Checkbox checked={selected} onChange={() => onCheck(row.dict_code)} /></TableCell>}<TableCell>{row.dict_sort}</TableCell><TableCell><DictLabel value={row.dict_label} listClass={row.list_class} /></TableCell><TableCell sx={{ fontFamily: 'monospace' }}>{row.dict_value}</TableCell><TableCell>{row.is_default}</TableCell><TableCell><StatusLabel status={row.status} /></TableCell><TableCell>{row.remark || '-'}</TableCell><TableCell sx={DATE_TIME_CELL_SX}>{fAdminDateTime(row.create_time) || '-'}</TableCell><TableCell align="right"><RowActions onEdit={() => onEdit(row)} onDelete={() => onDelete(row)} editDisabled={!useHasPermission('system:dict:edit')} deleteDisabled={!useHasPermission('system:dict:remove')} /></TableCell></TableRow>;
}


function DictLabel({ value, listClass }: { value: string; listClass: string | null }) {
  return <Label color={labelColor(listClass)} variant="soft">{value}</Label>;
}

function labelColor(value: string | null): LabelColor {
  if (value === 'primary' || value === 'success' || value === 'info' || value === 'warning') return value;
  if (value === 'danger') return 'error';
  return 'default';
}

function listClassOptions() {
  return ['default', 'primary', 'success', 'info', 'warning', 'danger'];
}

function DictTypeDialog({ open, editing, submitting, form, setForm, onClose, onSubmit }: DialogProps<DictTypeInput>) {
  const { t } = useTranslate('admin');
  return <ManagementDialog open={open} title={editing ? t('common.edit') : t('common.create')} submitting={submitting} onClose={onClose} onSubmit={onSubmit}><TextFieldRow label={t('fields.dictName')} value={form.dict_name} onChange={(value) => setForm((current) => ({ ...current, dict_name: value }))} /><TextFieldRow label={t('fields.dictType')} value={form.dict_type} onChange={(value) => setForm((current) => ({ ...current, dict_type: value }))} /><StatusField value={form.status} onChange={(status) => setForm((current) => ({ ...current, status }))} /><TextFieldRow multiline label={t('common.remark')} value={form.remark ?? ''} onChange={(value) => setForm((current) => ({ ...current, remark: value }))} /></ManagementDialog>;
}

function DictDataDialog({ open, editing, submitting, form, setForm, onClose, onSubmit }: DialogProps<DictDataInput>) {
  const { t } = useTranslate('admin');
  return <ManagementDialog open={open} title={editing ? t('common.edit') : t('common.create')} submitting={submitting} onClose={onClose} onSubmit={onSubmit}><TextFieldRow type="number" label={t('fields.dictSort')} value={form.dict_sort} onChange={(value) => setForm((current) => ({ ...current, dict_sort: Number(value) }))} /><TextFieldRow label={t('fields.dictLabel')} value={form.dict_label} onChange={(value) => setForm((current) => ({ ...current, dict_label: value }))} /><TextFieldRow label={t('fields.dictValue')} value={form.dict_value} onChange={(value) => setForm((current) => ({ ...current, dict_value: value }))} /><TextFieldRow label={t('fields.cssClass')} value={form.css_class ?? ''} onChange={(value) => setForm((current) => ({ ...current, css_class: value }))} /><TextFieldRow select label={t('fields.listClass')} value={form.list_class ?? 'default'} onChange={(value) => setForm((current) => ({ ...current, list_class: value }))}>{listClassOptions().map((option) => <MenuItem key={option} value={option}>{option}</MenuItem>)}</TextFieldRow><TextFieldRow select label={t('fields.isDefault')} value={form.is_default} onChange={(value) => setForm((current) => ({ ...current, is_default: value }))}><MenuItem value="Y">{t('common.yes')}</MenuItem><MenuItem value="N">{t('common.no')}</MenuItem></TextFieldRow><StatusField value={form.status} onChange={(status) => setForm((current) => ({ ...current, status }))} /><TextFieldRow multiline label={t('common.remark')} value={form.remark ?? ''} onChange={(value) => setForm((current) => ({ ...current, remark: value }))} /></ManagementDialog>;
}

function StatusField({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  const { t } = useTranslate('admin');
  return <TextFieldRow select label={t('common.status')} value={value} onChange={onChange}><MenuItem value="0">{t('common.enabled')}</MenuItem><MenuItem value="1">{t('common.disabled')}</MenuItem></TextFieldRow>;
}

function RowActions({ onEdit, onDelete, editDisabled, deleteDisabled }: { onEdit: () => void; onDelete: () => void; editDisabled: boolean; deleteDisabled: boolean }) {
  const { t } = useTranslate('admin');
  return <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}><Tooltip title={t('common.edit')}><span><IconButton disabled={editDisabled} onClick={(event) => { event.stopPropagation(); onEdit(); }}><Iconify icon="solar:pen-bold" /></IconButton></span></Tooltip><Tooltip title={t('common.delete')}><span><IconButton color="error" disabled={deleteDisabled} onClick={(event) => { event.stopPropagation(); onDelete(); }}><Iconify icon="solar:trash-bin-trash-bold" /></IconButton></span></Tooltip></Box>;
}

function openTypeEdit(item: DictType, setEditing: (item: DictType) => void, setForm: (item: DictTypeInput) => void) { setEditing(item); setForm({ dict_name: item.dict_name, dict_type: item.dict_type, status: item.status, remark: item.remark }); }
function openDataEdit(item: DictData, setEditing: (item: DictData) => void, setForm: (item: DictDataInput) => void) { setEditing(item); setForm({ dict_sort: item.dict_sort, dict_label: item.dict_label, dict_value: item.dict_value, dict_type: item.dict_type, css_class: item.css_class, list_class: item.list_class, is_default: item.is_default, status: item.status, remark: item.remark }); }
function closeTypeDialog(setEditing: (item: DictType | null) => void, setCreating: (value: boolean) => void, setForm: (item: DictTypeInput) => void) { setEditing(null); setCreating(false); setForm(DEFAULT_TYPE_INPUT); }
function closeDataDialog(setEditing: (item: DictData | null) => void, setCreating: (value: boolean) => void, setForm: (item: DictDataInput) => void) { setEditing(null); setCreating(false); setForm(DEFAULT_DATA_INPUT); }
const TABLE_HEAD_SX = { whiteSpace: 'nowrap' } as const;
const DATE_TIME_CELL_SX = { whiteSpace: 'nowrap' } as const;
function dictTypeHead(t: ReturnType<typeof useTranslate>['t']): TableHeadCellProps[] { return [{ id: 'dict_name', label: t('fields.dictName') }, { id: 'dict_type', label: t('fields.dictType') }, { id: 'status', label: t('common.status') }, { id: 'remark', label: t('common.remark') }, { id: 'create_time', label: t('fields.createTime'), width: 190, sx: TABLE_HEAD_SX }, { id: 'actions', label: t('common.actions'), align: 'right', width: 96, sx: TABLE_HEAD_SX }]; }
function dictDataHead(t: ReturnType<typeof useTranslate>['t']): TableHeadCellProps[] { return [{ id: 'dict_sort', label: t('fields.dictSort') }, { id: 'dict_label', label: t('fields.dictLabel') }, { id: 'dict_value', label: t('fields.dictValue') }, { id: 'is_default', label: t('fields.isDefault') }, { id: 'status', label: t('common.status') }, { id: 'remark', label: t('common.remark') }, { id: 'create_time', label: t('fields.createTime'), width: 190, sx: TABLE_HEAD_SX }, { id: 'actions', label: t('common.actions'), align: 'right', width: 96, sx: TABLE_HEAD_SX }]; }
function toggle(values: string[], value: string) { return values.includes(value) ? values.filter((item) => item !== value) : [...values, value]; }
type DialogProps<T> = { open: boolean; editing: boolean; submitting: boolean; form: T; setForm: React.Dispatch<React.SetStateAction<T>>; onClose: () => void; onSubmit: () => void };
