'use client';

import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { Dept, DeptInput, TreeSelectNode } from 'src/entities/system';

import { useMemo, useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import MenuItem from '@mui/material/MenuItem';
import TableRow from '@mui/material/TableRow';
import TextField from '@mui/material/TextField';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';

import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { TableNoData } from 'src/shared/ui/table';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';
import {
  AddButton,
  StatusLabel,
  TextFieldRow,
  TreeSelectField,
  AdminBreadcrumbs,
  ManagementDialog,
  ManagementTableHead,
} from 'src/shared/ui/admin';

import { useDepts } from 'src/entities/system';
import { useHasPermission } from 'src/entities/session';

import { getDeptTree, getDeptExclude, systemMutations } from 'src/features/system-management/api';

import { DashboardContent } from 'src/widgets/dashboard-shell';

const DEFAULT_INPUT: DeptInput = {
  parent_id: '0',
  dept_name: '',
  order_num: 0,
  leader: '',
  phone: '',
  email: '',
  status: '0',
};
const DEFAULT_FILTERS = { dept_name: '', status: '' };

export function DeptManagementPanel() {
  const { t } = useTranslate('admin');
  const [filters, setFilters] = useState(DEFAULT_FILTERS);
  const [expanded, setExpanded] = useState<string[]>([]);
  const [form, setForm] = useState<DeptInput>(DEFAULT_INPUT);
  const [editing, setEditing] = useState<Dept | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Dept | null>(null);
  const [sortEdits, setSortEdits] = useState<Record<string, number>>({});
  const [parentNodes, setParentNodes] = useState<TreeSelectNode[]>([]);
  const resource = useDepts(0, 1000, filters);
  const head = useMemo(() => deptHead(t), [t]);
  const rows = useMemo(() => flattenDeptRows(resource.items, expanded), [expanded, resource.items]);
  const allIds = useMemo(() => resource.items.map((dept) => dept.dept_id), [resource.items]);
  const canAdd = useHasPermission('system:dept:add');

  const closeDialog = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_INPUT);
  }, []);
  const openCreate = useCallback(async () => {
    setEditing(null);
    setCreating(true);
    setForm(DEFAULT_INPUT);
    try {
      setParentNodes(await getDeptTree());
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    }
  }, [t]);

  const openCreateChild = useCallback(
    async (dept: Dept) => {
      setEditing(null);
      setCreating(true);
      setForm({ ...DEFAULT_INPUT, parent_id: dept.dept_id });
      try {
        setParentNodes(await getDeptTree());
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
      }
    },
    [t]
  );

  const openEdit = useCallback(
    async (dept: Dept) => {
      setEditing(dept);
      setForm(toInput(dept));
      try {
        setParentNodes(await getDeptExclude(dept.dept_id));
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
      }
    },
    [t]
  );

  const submitDept = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editing) await systemMutations.updateDept(editing.dept_id, form);
      else await systemMutations.createDept(form);
      toast.success(t('messages.saved'));
      setSortEdits({});
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeDialog, editing, form, t]);

  const saveSorts = useCallback(async () => {
    const items = Object.entries(sortEdits).map(([id, order_num]) => ({ id, order_num }));
    if (items.length === 0) return;
    setSubmitting(true);
    try {
      await systemMutations.updateDeptSorts(items);
      toast.success(t('messages.sortSaved'));
      setSortEdits({});
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [sortEdits, t]);

  const confirmDelete = useCallback(async () => {
    if (!deleteTarget) return;
    try {
      await systemMutations.deleteDept(deleteTarget.dept_id);
      toast.success(t('messages.deleted'));
      setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteTarget, t]);

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.deptManagement')}
        action={canAdd ? <AddButton onClick={openCreate}>{t('actions.addDept')}</AddButton> : null}
      />
      <Card>
        <DeptFilters filters={filters} onChange={setFilters} />
        <Stack direction="row" spacing={1} sx={{ px: 2, pb: 2 }}>
          <Button size="small" onClick={() => setExpanded(allIds)}>
            {t('actions.expandAll')}
          </Button>
          <Button size="small" onClick={() => setExpanded([])}>
            {t('actions.collapseAll')}
          </Button>
          <Button
            size="small"
            variant="contained"
            disabled={Object.keys(sortEdits).length === 0 || submitting}
            onClick={saveSorts}
          >
            {t('actions.saveSort')}
          </Button>
        </Stack>
        <Scrollbar>
          <Table sx={{ minWidth: 1080 }}>
            <ManagementTableHead head={head} />
            <TableBody>
              {rows.map((row) => (
                <DeptRow
                  key={row.dept.dept_id}
                  row={row}
                  expanded={expanded.includes(row.dept.dept_id)}
                  orderValue={sortEdits[row.dept.dept_id] ?? row.dept.order_num}
                  onToggle={() => setExpanded(toggle(expanded, row.dept.dept_id))}
                  onSort={(orderNum) =>
                    setSortEdits((current) => ({ ...current, [row.dept.dept_id]: orderNum }))
                  }
                  onCreateChild={openCreateChild}
                  onEdit={openEdit}
                  onDelete={setDeleteTarget}
                />
              ))}
              <TableNoData
                title={t('common.noData')}
                notFound={!resource.isLoading && rows.length === 0}
              />
            </TableBody>
          </Table>
        </Scrollbar>
      </Card>
      <DeptDialog
        open={creating || !!editing}
        editing={!!editing}
        submitting={submitting}
        form={form}
        parentNodes={parentNodes}
        setForm={setForm}
        onClose={closeDialog}
        onSubmit={submitDept}
      />
      <ConfirmDialog
        open={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        title={t('common.delete')}
        content={t('dialogs.deleteContent', { name: deleteTarget?.dept_name ?? '' })}
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

function DeptFilters({
  filters,
  onChange,
}: {
  filters: typeof DEFAULT_FILTERS;
  onChange: (filters: typeof DEFAULT_FILTERS) => void;
}) {
  const { t } = useTranslate('admin');
  const write = (key: keyof typeof DEFAULT_FILTERS, value: string) =>
    onChange({ ...filters, [key]: value });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2 }}>
      <TextField
        size="small"
        label={t('fields.deptName')}
        value={filters.dept_name}
        onChange={(event) => write('dept_name', event.target.value)}
      />
      <TextField
        select
        size="small"
        label={t('common.status')}
        value={filters.status}
        sx={{ minWidth: 140 }}
        onChange={(event) => write('status', event.target.value)}
      >
        <MenuItem value="">{t('common.all')}</MenuItem>
        <MenuItem value="0">{t('common.enabled')}</MenuItem>
        <MenuItem value="1">{t('common.disabled')}</MenuItem>
      </TextField>
      <Button variant="outlined" onClick={() => onChange(DEFAULT_FILTERS)}>
        {t('common.reset')}
      </Button>
    </Stack>
  );
}

function DeptRow({
  row,
  expanded,
  orderValue,
  onToggle,
  onSort,
  onCreateChild,
  onEdit,
  onDelete,
}: {
  row: DeptRowView;
  expanded: boolean;
  orderValue: number;
  onToggle: () => void;
  onSort: (value: number) => void;
  onCreateChild: (dept: Dept) => void;
  onEdit: (dept: Dept) => void;
  onDelete: (dept: Dept) => void;
}) {
  const { t } = useTranslate('admin');
  const canAdd = useHasPermission('system:dept:add');
  const canEdit = useHasPermission('system:dept:edit');
  const canDelete = useHasPermission('system:dept:remove');
  return (
    <TableRow hover>
      <TableCell>
        <Box sx={{ display: 'flex', alignItems: 'center', pl: row.level * 2 }}>
          {row.childCount > 0 ? (
            <IconButton size="small" onClick={onToggle}>
              <Iconify
                icon={expanded ? 'eva:arrow-ios-downward-fill' : 'eva:arrow-ios-forward-fill'}
              />
            </IconButton>
          ) : (
            <Box sx={{ width: 34 }} />
          )}
          {row.dept.dept_name}
        </Box>
      </TableCell>
      <TableCell>
        <TextField
          size="small"
          type="number"
          value={orderValue}
          disabled={!canEdit}
          sx={{ width: 88 }}
          onChange={(event) => onSort(Number(event.target.value))}
        />
      </TableCell>
      <TableCell>{row.dept.leader || '-'}</TableCell>
      <TableCell>{row.dept.phone || '-'}</TableCell>
      <TableCell>{row.dept.email || '-'}</TableCell>
      <TableCell>
        <StatusLabel status={row.dept.status} />
      </TableCell>
      <TableCell sx={DATE_TIME_CELL_SX}>{fAdminDateTime(row.dept.create_time) || '-'}</TableCell>
      <TableCell align="right">
        <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
          <Tooltip title={t('common.add')}>
            <span>
              <IconButton disabled={!canAdd} onClick={() => onCreateChild(row.dept)}>
                <Iconify icon="mingcute:add-line" />
              </IconButton>
            </span>
          </Tooltip>
          <Tooltip title={t('common.edit')}>
            <span>
              <IconButton disabled={!canEdit} onClick={() => onEdit(row.dept)}>
                <Iconify icon="solar:pen-bold" />
              </IconButton>
            </span>
          </Tooltip>
          <Tooltip title={t('common.delete')}>
            <span>
              <IconButton color="error" disabled={!canDelete} onClick={() => onDelete(row.dept)}>
                <Iconify icon="solar:trash-bin-trash-bold" />
              </IconButton>
            </span>
          </Tooltip>
        </Box>
      </TableCell>
    </TableRow>
  );
}

function DeptDialog({
  open,
  editing,
  submitting,
  form,
  parentNodes,
  setForm,
  onClose,
  onSubmit,
}: {
  open: boolean;
  editing: boolean;
  submitting: boolean;
  form: DeptInput;
  parentNodes: TreeSelectNode[];
  setForm: React.Dispatch<React.SetStateAction<DeptInput>>;
  onClose: () => void;
  onSubmit: () => void;
}) {
  const { t } = useTranslate('admin');
  return (
    <ManagementDialog
      open={open}
      title={editing ? t('common.edit') : t('common.create')}
      submitting={submitting}
      onClose={onClose}
      onSubmit={onSubmit}
    >
      <TreeSelectField
        label={t('fields.parentDept')}
        value={form.parent_id}
        nodes={parentNodes}
        onChange={(value) => setForm((current) => ({ ...current, parent_id: value }))}
      />
      <TextFieldRow
        required
        label={t('fields.deptName')}
        value={form.dept_name}
        onChange={(value) => setForm((current) => ({ ...current, dept_name: value }))}
      />
      <TextFieldRow
        type="number"
        label={t('common.sort')}
        value={form.order_num}
        onChange={(value) => setForm((current) => ({ ...current, order_num: Number(value) }))}
      />
      <TextFieldRow
        label={t('fields.leader')}
        value={form.leader ?? ''}
        onChange={(value) => setForm((current) => ({ ...current, leader: value }))}
      />
      <TextFieldRow
        label={t('fields.phone')}
        value={form.phone ?? ''}
        onChange={(value) => setForm((current) => ({ ...current, phone: value }))}
      />
      <TextFieldRow
        label={t('common.email')}
        value={form.email ?? ''}
        onChange={(value) => setForm((current) => ({ ...current, email: value }))}
      />
      <TextFieldRow
        select
        label={t('common.status')}
        value={form.status}
        onChange={(status) => setForm((current) => ({ ...current, status }))}
      >
        <MenuItem value="0">{t('common.enabled')}</MenuItem>
        <MenuItem value="1">{t('common.disabled')}</MenuItem>
      </TextFieldRow>
    </ManagementDialog>
  );
}

type DeptRowView = { dept: Dept; level: number; childCount: number };
function toInput(dept: Dept): DeptInput {
  return {
    parent_id: dept.parent_id,
    dept_name: dept.dept_name,
    order_num: dept.order_num,
    leader: dept.leader,
    phone: dept.phone,
    email: dept.email,
    status: dept.status,
  };
}
const TABLE_HEAD_SX = { whiteSpace: 'nowrap' } as const;
const DATE_TIME_CELL_SX = { whiteSpace: 'nowrap' } as const;
function deptHead(t: ReturnType<typeof useTranslate>['t']): TableHeadCellProps[] {
  return [
    { id: 'dept_name', label: t('fields.deptName') },
    { id: 'order_num', label: t('common.sort') },
    { id: 'leader', label: t('fields.leader') },
    { id: 'phone', label: t('fields.phone') },
    { id: 'email', label: t('common.email') },
    { id: 'status', label: t('common.status') },
    { id: 'create_time', label: t('fields.createTime'), width: 190, sx: TABLE_HEAD_SX },
    { id: 'actions', label: t('common.actions'), align: 'right', width: 132, sx: TABLE_HEAD_SX },
  ];
}
function toggle(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}
function flattenDeptRows(depts: Dept[], expanded: string[]) {
  const ids = new Set(depts.map((dept) => dept.dept_id));
  const roots = sortDepts(
    depts.filter((dept) => dept.parent_id === '0' || !ids.has(dept.parent_id))
  );
  return roots.flatMap((dept) => flattenDeptBranch(depts, dept, 0, expanded));
}

function flattenDeptBranch(
  depts: Dept[],
  dept: Dept,
  level: number,
  expanded: string[]
): DeptRowView[] {
  const children = sortDepts(depts.filter((child) => child.parent_id === dept.dept_id));
  const row = { dept, level, childCount: children.length };
  if (!expanded.includes(dept.dept_id)) return [row];
  return [
    row,
    ...children.flatMap((child) => flattenDeptBranch(depts, child, level + 1, expanded)),
  ];
}

function sortDepts(depts: Dept[]) {
  return [...depts].sort((a, b) => a.order_num - b.order_num);
}
