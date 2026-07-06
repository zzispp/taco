'use client';

import type { CrudField, CrudFilter, CrudPanelProps, ActionIconProps } from './types';

import { useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';

import { toast } from 'src/shared/ui/snackbar';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';
import { TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import {
  AddButton,
  ManagementDialog,
  TableLoadingRows,
  withSelectionHead,
  ManagementTableHead,
} from 'src/shared/ui/admin';

import { useHasPermission } from 'src/entities/session';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

import { CrudFilters } from './filters';
import { ActionIcon } from './action-icon';
import { TableActions } from './table-actions';
import { CrudFieldControl } from './field-control';
import { toggle, tableHead, fieldCellSx, formFromRow, displayField } from './helpers';

export type { CrudField, CrudFilter, CrudPanelProps, ActionIconProps };
export { ActionIcon };

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

  return (
    <DashboardContent>
      <AdminBreadcrumbs heading={title} action={breadcrumbAction()} />
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
                resource.items.map((row) => renderRow(row))
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
      {renderDialog()}
      {renderBatchDeleteDialog()}
      {renderDeleteDialog()}
    </DashboardContent>
  );

  function breadcrumbAction() {
    const hasToolbarActions = Boolean(toolbarAction) || canAdd || hasBatchDelete;
    if (!hasToolbarActions) return null;
    return (
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
    );
  }

  function renderRow(row: T) {
    const rowId = String(row[idKey]);
    return (
      <TableRow key={rowId} hover>
        {hasBatchDelete && (
          <TableCell padding="checkbox">
            <Checkbox
              disabled={!isRowSelectable(row)}
              checked={selected.includes(rowId)}
              onChange={() => setSelected(toggle(selected, rowId))}
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
    );
  }

  function renderDialog() {
    return (
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
    );
  }

  function renderBatchDeleteDialog() {
    return (
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
    );
  }

  function renderDeleteDialog() {
    return (
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
    );
  }
}
