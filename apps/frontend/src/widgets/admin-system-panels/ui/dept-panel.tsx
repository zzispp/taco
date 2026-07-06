'use client';

import type { Dept, TreeSelectNode } from 'src/entities/system';

import { useMemo, useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import TableBody from '@mui/material/TableBody';

import { toast } from 'src/shared/ui/snackbar';
import { TableNoData } from 'src/shared/ui/table';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { AddButton, ManagementTableHead } from 'src/shared/ui/admin';

import { useDepts } from 'src/entities/system';
import { useHasPermission } from 'src/entities/session';

import { getDeptTree, getDeptExclude, systemMutations } from 'src/features/system-management';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

import { DeptRow } from './dept-row';
import { DeptDialog } from './dept-dialog';
import { DeptFilters } from './dept-filters';
import { DeptConfirmDialog } from './dept-confirm-dialog';
import { DEFAULT_INPUT, DEFAULT_FILTERS } from './dept-constants';
import { toggle, toInput, deptHead, flattenDeptRows } from './dept-helpers';

export function DeptManagementPanel() {
  const { t } = useTranslate('admin');
  const [filters, setFilters] = useState(DEFAULT_FILTERS);
  const [expanded, setExpanded] = useState<string[]>([]);
  const [form, setForm] = useState(DEFAULT_INPUT);
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

  const loadParentNodes = useCallback(
    async (loader: () => Promise<TreeSelectNode[]>) => {
      try {
        setParentNodes(await loader());
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
      }
    },
    [t]
  );

  const closeDialog = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_INPUT);
  }, []);

  const openCreate = useCallback(async () => {
    setEditing(null);
    setCreating(true);
    setForm(DEFAULT_INPUT);
    await loadParentNodes(() => getDeptTree());
  }, [loadParentNodes]);

  const openCreateChild = useCallback(
    async (dept: Dept) => {
      setEditing(null);
      setCreating(true);
      setForm({ ...DEFAULT_INPUT, parent_id: dept.dept_id });
      await loadParentNodes(() => getDeptTree());
    },
    [loadParentNodes]
  );

  const openEdit = useCallback(
    async (dept: Dept) => {
      setEditing(dept);
      setForm(toInput(dept));
      await loadParentNodes(() => getDeptExclude(dept.dept_id));
    },
    [loadParentNodes]
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
      <DeptConfirmDialog
        target={deleteTarget}
        onClose={() => setDeleteTarget(null)}
        onConfirm={confirmDelete}
      />
    </DashboardContent>
  );
}
