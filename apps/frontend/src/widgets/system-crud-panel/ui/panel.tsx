'use client';

import type { IconifyName } from 'src/shared/ui/iconify';
import type { TableHeadCellProps } from 'src/shared/ui/table';

import { useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Switch from '@mui/material/Switch';
import Tooltip from '@mui/material/Tooltip';
import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import TableRow from '@mui/material/TableRow';
import TextField from '@mui/material/TextField';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import FormControlLabel from '@mui/material/FormControlLabel';

import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';
import { TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import {
  AddButton,
  TextFieldRow,
  AdminBreadcrumbs,
  ManagementDialog,
  TableLoadingRows,
  withSelectionHead,
  ManagementTableHead,
} from 'src/shared/ui/admin';

import { useHasPermission } from 'src/entities/session';

import { DashboardContent } from 'src/widgets/dashboard-shell';

export type CrudField<T> = {
  key: keyof T;
  label: string;
  type?: 'text' | 'number' | 'select' | 'textarea' | 'switch' | 'boolean';
  format?: 'dateTime';
  width?: TableHeadCellProps['width'];
  options?: { value: string; label: string }[];
  disabled?: (context: {
    form: Record<string, unknown>;
    editing: Record<string, unknown> | null;
  }) => boolean;
  hiddenInTable?: boolean;
  hiddenInForm?: boolean;
};

export type CrudFilter = {
  key: string;
  label: string;
  type?: 'text' | 'select' | 'date';
  options?: { value: string; label: string }[];
};

export type CrudPanelProps<T extends Record<string, unknown>, I extends Record<string, unknown>> = {
  title: string;
  addLabel: string;
  idKey: keyof T;
  nameKey: keyof T;
  fields: CrudField<T>[];
  defaultInput: I;
  resource: { items: T[]; total: number; isLoading: boolean };
  page: number;
  rowsPerPage: number;
  filters?: CrudFilter[];
  filterValues?: Record<string, string>;
  permissionPrefix: string;
  extraActions?: (row: T) => React.ReactNode;
  toolbarAction?: React.ReactNode;
  batchDeleteItems?: (ids: string[]) => Promise<void>;
  isRowSelectable?: (row: T) => boolean;
  onFilterChange?: (filters: Record<string, string>) => void;
  onPageChange: (event: unknown, page: number) => void;
  onRowsPerPageChange: (event: React.ChangeEvent<HTMLInputElement>) => void;
  createItem: (input: I) => Promise<T>;
  updateItem: (id: string, input: I) => Promise<T>;
  deleteItem: (id: string) => Promise<void>;
  onAfterSave?: () => void;
};

export function SystemCrudPanel<
  T extends Record<string, unknown>,
  I extends Record<string, unknown>,
>({
  title,
  addLabel,
  idKey,
  nameKey,
  fields,
  defaultInput,
  resource,
  page,
  rowsPerPage,
  filters = [],
  filterValues = {},
  permissionPrefix,
  extraActions,
  toolbarAction,
  batchDeleteItems,
  isRowSelectable = () => true,
  onFilterChange,
  onPageChange,
  onRowsPerPageChange,
  createItem,
  updateItem,
  deleteItem,
  onAfterSave,
}: CrudPanelProps<T, I>) {
  const { t } = useTranslate('admin');
  const [form, setForm] = useState<I>(defaultInput);
  const [editing, setEditing] = useState<T | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<T | null>(null);
  const [batchDeleteOpen, setBatchDeleteOpen] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);
  const canAdd = useHasPermission(`${permissionPrefix}:add`);
  const canDelete = useHasPermission(`${permissionPrefix}:remove`);
  const hasBatchDelete = !!batchDeleteItems && canDelete;
  const selectableRows = resource.items.filter(isRowSelectable);
  const head = tableHead(fields, !!extraActions, hasBatchDelete, t('common.actions'));
  const bodyHead = hasBatchDelete ? withSelectionHead(head) : head;

  const closeDialog = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(defaultInput);
  }, [defaultInput]);

  const submit = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editing) await updateItem(String(editing[idKey]), form);
      else await createItem(form);
      toast.success(t('messages.saved'));
      onAfterSave?.();
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeDialog, createItem, editing, form, idKey, onAfterSave, t, updateItem]);

  const confirmDelete = useCallback(async () => {
    if (!deleteTarget) return;
    try {
      await deleteItem(String(deleteTarget[idKey]));
      toast.success(t('messages.deleted'));
      setDeleteTarget(null);
      setSelected((current) => current.filter((id) => id !== String(deleteTarget[idKey])));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteItem, deleteTarget, idKey, t]);

  const confirmBatchDelete = useCallback(async () => {
    if (!batchDeleteItems || selected.length === 0) return;
    try {
      await batchDeleteItems(selected);
      toast.success(t('messages.deleted'));
      setSelected([]);
      setBatchDeleteOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [batchDeleteItems, selected, t]);

  const toggleAll = useCallback(
    (checked: boolean) => {
      setSelected(checked ? selectableRows.map((row) => String(row[idKey])) : []);
    },
    [idKey, selectableRows]
  );

  const hasToolbarActions = Boolean(toolbarAction) || canAdd || hasBatchDelete;
  const breadcrumbAction = hasToolbarActions ? (
    <Stack direction="row" spacing={1}>
      {toolbarAction}
      {hasBatchDelete && (
        <Button
          variant="outlined"
          color="error"
          disabled={selected.length === 0}
          onClick={() => setBatchDeleteOpen(true)}
        >
          {t('common.delete')}
        </Button>
      )}
      {canAdd && <AddButton onClick={() => setCreating(true)}>{addLabel}</AddButton>}
    </Stack>
  ) : null;

  return (
    <DashboardContent>
      <AdminBreadcrumbs heading={title} action={breadcrumbAction} />
      <Card>
        <CrudFilters filters={filters} values={filterValues} onChange={onFilterChange} />
        <Scrollbar>
          <Table sx={{ minWidth: 980 }}>
            <ManagementTableHead
              head={head}
              rowCount={selectableRows.length}
              numSelected={selected.length}
              onSelectAllRows={hasBatchDelete ? toggleAll : undefined}
            />
            <TableBody>
              {resource.isLoading ? (
                <TableLoadingRows head={bodyHead} rows={rowsPerPage} />
              ) : (
                resource.items.map((row) => (
                  <TableRow key={String(row[idKey])} hover>
                    {hasBatchDelete && (
                      <TableCell padding="checkbox">
                        <Checkbox
                          disabled={!isRowSelectable(row)}
                          checked={selected.includes(String(row[idKey]))}
                          onChange={() => setSelected(toggle(selected, String(row[idKey])))}
                        />
                      </TableCell>
                    )}
                    {fields
                      .filter((field) => !field.hiddenInTable)
                      .map((field) => (
                        <TableCell key={String(field.key)} sx={fieldCellSx(field)}>
                          {displayField(row[field.key as keyof T], field)}
                        </TableCell>
                      ))}
                    <TableCell align="right">
                      <TableActions
                        permissionPrefix={permissionPrefix}
                        extra={extraActions?.(row)}
                        deleteDisabled={!isRowSelectable(row)}
                        onEdit={() => {
                          setForm(formFromRow<T, I>(row, fields));
                          setEditing(row);
                        }}
                        onDelete={() => setDeleteTarget(row)}
                      />
                    </TableCell>
                  </TableRow>
                ))
              )}
              <TableNoData
                title={t('common.noData')}
                notFound={!resource.isLoading && resource.items.length === 0}
              />
            </TableBody>
          </Table>
        </Scrollbar>
        <TablePaginationCustom
          page={page}
          count={resource.total}
          rowsPerPage={rowsPerPage}
          onPageChange={onPageChange}
          onRowsPerPageChange={onRowsPerPageChange}
        />
      </Card>
      <ManagementDialog
        open={creating || !!editing}
        title={editing ? t('common.edit') : t('common.create')}
        submitting={submitting}
        onClose={closeDialog}
        onSubmit={submit}
      >
        {fields
          .filter((field) => !field.hiddenInForm)
          .map((field) => (
            <CrudFieldControl
              key={String(field.key)}
              field={field as unknown as CrudField<I>}
              editing={editing as Record<string, unknown> | null}
              form={form}
              setForm={setForm}
            />
          ))}
      </ManagementDialog>
      <ConfirmDialog
        open={batchDeleteOpen}
        onClose={() => setBatchDeleteOpen(false)}
        title={t('common.delete')}
        content={t('dialogs.deleteContent', { name: String(selected.length) })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={confirmBatchDelete}>
            {t('common.delete')}
          </Button>
        }
      />
      <ConfirmDialog
        open={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        title={t('common.delete')}
        content={t('dialogs.deleteContent', {
          name: deleteTarget ? String(deleteTarget[nameKey]) : '',
        })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={confirmDelete}>
            {t('common.delete')}
          </Button>
        }
      />
    </DashboardContent>
  );
}

function CrudFilters({
  filters,
  values,
  onChange,
}: {
  filters: CrudFilter[];
  values: Record<string, string>;
  onChange?: (filters: Record<string, string>) => void;
}) {
  const { t } = useTranslate('admin');
  if (filters.length === 0 || !onChange) return null;
  const write = (key: string, value: string) => onChange({ ...values, [key]: value });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2 }}>
      {filters.map((filter) => (
        <TextField
          key={filter.key}
          select={filter.type === 'select'}
          type={filter.type === 'date' ? 'date' : 'text'}
          size="small"
          label={filter.label}
          value={values[filter.key] ?? ''}
          InputLabelProps={filter.type === 'date' ? { shrink: true } : undefined}
          sx={{
            minWidth: filter.type === 'select' ? 140 : filter.type === 'date' ? 170 : undefined,
          }}
          onChange={(event) => write(filter.key, event.target.value)}
        >
          {filter.options?.map((option) => (
            <MenuItem key={option.value} value={option.value}>
              {option.label}
            </MenuItem>
          ))}
        </TextField>
      ))}
      <Button
        variant="outlined"
        onClick={() => onChange(Object.fromEntries(filters.map((filter) => [filter.key, ''])))}
      >
        {t('common.reset')}
      </Button>
    </Stack>
  );
}

function CrudFieldControl<I extends Record<string, unknown>>({
  field,
  editing,
  form,
  setForm,
}: {
  field: CrudField<I>;
  editing: Record<string, unknown> | null;
  form: I;
  setForm: React.Dispatch<React.SetStateAction<I>>;
}) {
  const value = form[field.key] ?? '';
  const disabled = field.disabled?.({ form, editing }) ?? false;
  const writeValue = (next: string | boolean) =>
    setForm((current) => ({ ...current, [field.key]: normalizeValue(next, field.type) }));
  if (field.type === 'switch')
    return (
      <Switch
        checked={String(value) === '0'}
        onChange={(event) => writeValue(event.target.checked ? '0' : '1')}
      />
    );
  if (field.type === 'boolean')
    return (
      <FormControlLabel
        control={
          <Switch
            checked={Boolean(value)}
            disabled={disabled}
            onChange={(event) => writeValue(event.target.checked)}
          />
        }
        label={field.label}
      />
    );
  return (
    <TextFieldRow
      disabled={disabled}
      label={field.label}
      type={field.type === 'number' ? 'number' : 'text'}
      select={field.type === 'select'}
      multiline={field.type === 'textarea'}
      value={String(value)}
      onChange={writeValue}
    >
      {field.options?.map((option) => (
        <MenuItem key={option.value} value={option.value}>
          {option.label}
        </MenuItem>
      ))}
    </TextFieldRow>
  );
}

function TableActions({
  permissionPrefix,
  extra,
  deleteDisabled,
  onEdit,
  onDelete,
}: {
  permissionPrefix: string;
  extra?: React.ReactNode;
  deleteDisabled?: boolean;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const { t } = useTranslate('admin');
  const canEdit = useHasPermission(`${permissionPrefix}:edit`);
  const canDelete = useHasPermission(`${permissionPrefix}:remove`);
  return (
    <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
      {extra}
      <Tooltip title={t('common.edit')}>
        <span>
          <IconButton disabled={!canEdit} onClick={onEdit}>
            <Iconify icon="solar:pen-bold" />
          </IconButton>
        </span>
      </Tooltip>
      <Tooltip title={t('common.delete')}>
        <span>
          <IconButton color="error" disabled={!canDelete || deleteDisabled} onClick={onDelete}>
            <Iconify icon="solar:trash-bin-trash-bold" />
          </IconButton>
        </span>
      </Tooltip>
    </Box>
  );
}

const TABLE_HEAD_SX = { whiteSpace: 'nowrap' } as const;
const DATE_TIME_CELL_SX = { whiteSpace: 'nowrap' } as const;

function tableHead<T>(
  fields: CrudField<T>[],
  hasExtra: boolean,
  hasSelection: boolean,
  actionLabel: string
): TableHeadCellProps[] {
  return [
    ...fields
      .filter((field) => !field.hiddenInTable)
      .map((field) => ({
        id: String(field.key),
        label: field.label,
        width: field.width ?? (isDateTimeField(field) ? 190 : undefined),
        sx: TABLE_HEAD_SX,
      })),
    {
      id: 'actions',
      label: actionLabel,
      align: 'right',
      width: hasExtra || hasSelection ? 144 : 96,
      sx: TABLE_HEAD_SX,
    },
  ];
}

function toggle(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}

function formFromRow<T extends Record<string, unknown>, I extends Record<string, unknown>>(
  row: T,
  fields: CrudField<T>[]
) {
  return Object.fromEntries(
    fields.filter((field) => !field.hiddenInForm).map((field) => [field.key, row[field.key] ?? ''])
  ) as I;
}

function displayField<T>(value: unknown, field: CrudField<T>) {
  if (value === null || value === undefined || value === '') return '-';
  if (isDateTimeField(field)) return fAdminDateTime(String(value)) || '-';
  if (field.type === 'switch')
    return <Switch size="small" checked={String(value) === '0'} disabled />;
  if (typeof value === 'boolean') return value ? '是' : '否';
  return String(value);
}

function fieldCellSx<T>(field: CrudField<T>) {
  return isDateTimeField(field) ? DATE_TIME_CELL_SX : undefined;
}

function isDateTimeField<T>(field: CrudField<T>) {
  return field.format === 'dateTime' || String(field.key) === 'create_time';
}

function normalizeValue(
  value: string | boolean,
  type?: CrudField<Record<string, unknown>>['type']
) {
  if (type === 'number') return Number(value);
  if (type === 'boolean') return Boolean(value);
  if (typeof value === 'boolean') return value ? '0' : '1';
  return value;
}

export type ActionIconProps = {
  title: string;
  icon: IconifyName;
  disabled?: boolean;
  color?: 'primary' | 'error' | 'warning' | 'info' | 'success';
  onClick: () => void;
};

export function ActionIcon({ title, icon, disabled, color, onClick }: ActionIconProps) {
  return (
    <Tooltip title={title}>
      <span>
        <IconButton color={color} disabled={disabled} onClick={onClick}>
          <Iconify icon={icon} />
        </IconButton>
      </span>
    </Tooltip>
  );
}
