'use client';

import type { Menu, MenuInput } from 'src/entities/menu';
import type { IconifyName } from 'src/shared/ui/iconify';
import type { TableHeadCellProps } from 'src/shared/ui/table';

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
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';
import {
  AddButton,
  SwitchRow,
  StatusLabel,
  TextFieldRow,
  TreeSelectField,
  ManagementDialog,
  ManagementTableHead,
} from 'src/shared/ui/admin';

import { useHasPermission } from 'src/entities/session';
import { useMenus, NAV_ICON_OPTIONS, translatedMenuItem } from 'src/entities/menu';

import { createMenu, deleteMenu, updateMenu, updateMenuSorts } from 'src/features/menu-management';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

const DEFAULT_FORM: MenuInput = {
  menu_name: '',
  parent_id: '0',
  order_num: 0,
  path: '',
  component: '',
  query: '',
  route_name: '',
  is_frame: false,
  is_cache: false,
  menu_type: 'C',
  visible: '0',
  status: '0',
  perms: '',
  icon: 'icon.menu',
  remark: '',
};

const DEFAULT_FILTERS = { menu_name: '', status: '' };

export function MenuManagementView() {
  const { t } = useTranslate('admin');
  const [filters, setFilters] = useState(DEFAULT_FILTERS);
  const menus = useMenus(0, 1000, filters);
  const allMenus = useMenus(0, 1000);
  const [expanded, setExpanded] = useState<string[]>([]);
  const [form, setForm] = useState<MenuInput>(DEFAULT_FORM);
  const [editing, setEditing] = useState<Menu | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Menu | null>(null);
  const [sortEdits, setSortEdits] = useState<Record<string, number>>({});
  const head = useMemo(() => menuHead(t), [t]);
  const treeRows = useMemo(() => flattenMenuRows(menus.items, expanded), [expanded, menus.items]);
  const allIds = useMemo(() => allMenus.items.map((menu) => menu.menu_id), [allMenus.items]);
  const canAdd = useHasPermission('system:menu:add');

  const closeDialog = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_FORM);
  }, []);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm(DEFAULT_FORM);
  }, []);

  const openCreateChild = useCallback((menu: Menu) => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_FORM, parent_id: menu.menu_id });
  }, []);

  const openEdit = useCallback((menu: Menu) => {
    setEditing(menu);
    setForm(toInput(menu));
  }, []);

  const submitMenu = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editing) await updateMenu(editing.menu_id, form);
      else await createMenu(form);
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
      await updateMenuSorts(items);
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
      await deleteMenu(deleteTarget.menu_id);
      toast.success(t('messages.deleted'));
      setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteTarget, t]);

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.menuManagement')}
        action={
          canAdd ? <AddButton onClick={openCreate}>{t('actions.addMenuItem')}</AddButton> : null
        }
      />
      <Card>
        <MenuFilters filters={filters} onChange={setFilters} />
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
          <Table sx={{ minWidth: 1240 }}>
            <ManagementTableHead head={head} />
            <TableBody>
              {treeRows.map((row) => (
                <MenuRow
                  key={row.menu.menu_id}
                  row={row}
                  expanded={expanded.includes(row.menu.menu_id)}
                  onToggle={() => setExpanded(toggle(expanded, row.menu.menu_id))}
                  onEdit={openEdit}
                  onDelete={setDeleteTarget}
                  onCreateChild={openCreateChild}
                  orderValue={sortEdits[row.menu.menu_id] ?? row.menu.order_num}
                  onSort={(orderNum) =>
                    setSortEdits((current) => ({ ...current, [row.menu.menu_id]: orderNum }))
                  }
                />
              ))}
              <TableNoData
                title={t('common.noData')}
                notFound={!menus.isLoading && treeRows.length === 0}
              />
            </TableBody>
          </Table>
        </Scrollbar>
      </Card>
      <MenuDialog
        open={creating || !!editing}
        editing={!!editing}
        submitting={submitting}
        form={form}
        menus={allMenus.items}
        editingId={editing?.menu_id}
        setForm={setForm}
        onClose={closeDialog}
        onSubmit={submitMenu}
      />
      <ConfirmDialog
        open={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        title={t('dialogs.deleteMenuItem')}
        content={t('dialogs.deleteContent', { name: deleteTarget?.menu_name ?? '' })}
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

function MenuFilters({
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
        label={t('fields.menuName')}
        value={filters.menu_name}
        onChange={(event) => write('menu_name', event.target.value)}
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

function MenuRow({
  row,
  expanded,
  orderValue,
  onToggle,
  onEdit,
  onDelete,
  onCreateChild,
  onSort,
}: {
  row: MenuRowView;
  expanded: boolean;
  orderValue: number;
  onToggle: () => void;
  onEdit: (menu: Menu) => void;
  onDelete: (menu: Menu) => void;
  onCreateChild: (menu: Menu) => void;
  onSort: (orderNum: number) => void;
}) {
  const { t } = useTranslate('admin');
  const canAdd = useHasPermission('system:menu:add');
  const canEdit = useHasPermission('system:menu:edit');
  const canDelete = useHasPermission('system:menu:remove');
  const hasChildren = row.childCount > 0;
  return (
    <TableRow hover>
      <TableCell>
        <Box sx={{ display: 'flex', alignItems: 'center', pl: row.level * 2 }}>
          {hasChildren ? (
            <IconButton size="small" onClick={onToggle}>
              <Iconify
                icon={expanded ? 'eva:arrow-ios-downward-fill' : 'eva:arrow-ios-forward-fill'}
              />
            </IconButton>
          ) : (
            <Box sx={{ width: 34 }} />
          )}
          <Iconify icon={menuIcon(row.menu.icon)} width={18} sx={{ mr: 1 }} />
          {translatedMenuItem(row.menu, t)}
        </Box>
      </TableCell>
      <TableCell>{menuTypeLabel(row.menu.menu_type, t)}</TableCell>
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
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.menu.path || '-'}</TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.menu.component || '-'}</TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.menu.perms || '-'}</TableCell>
      <TableCell>{row.menu.visible === '0' ? t('common.show') : t('common.hide')}</TableCell>
      <TableCell>
        <StatusLabel status={row.menu.status} />
      </TableCell>
      <TableCell align="right">
        <Actions
          canAdd={canAdd && row.menu.menu_type !== 'F'}
          canEdit={canEdit}
          canDelete={canDelete}
          onCreateChild={() => onCreateChild(row.menu)}
          onEdit={() => onEdit(row.menu)}
          onDelete={() => onDelete(row.menu)}
        />
      </TableCell>
    </TableRow>
  );
}

function MenuDialog({
  open,
  editing,
  submitting,
  form,
  menus,
  editingId,
  setForm,
  onClose,
  onSubmit,
}: {
  open: boolean;
  editing: boolean;
  submitting: boolean;
  form: MenuInput;
  menus: Menu[];
  editingId?: string;
  setForm: React.Dispatch<React.SetStateAction<MenuInput>>;
  onClose: () => void;
  onSubmit: () => void;
}) {
  const { t } = useTranslate('admin');
  const parentNodes = parentMenuTree(menus, editingId);
  return (
    <ManagementDialog
      open={open}
      title={editing ? t('dialogs.editMenuItem') : t('dialogs.createMenuItem')}
      submitting={submitting}
      onClose={onClose}
      onSubmit={onSubmit}
    >
      <TreeSelectField
        label={t('fields.parentMenu')}
        value={form.parent_id}
        nodes={parentNodes}
        onChange={(value) => setForm((current) => ({ ...current, parent_id: value }))}
      />
      <TextFieldRow
        select
        label={t('fields.menuType')}
        value={form.menu_type}
        onChange={(value) => setForm((current) => ({ ...current, menu_type: value }))}
      >
        <MenuItem value="M">{t('menuType.directory')}</MenuItem>
        <MenuItem value="C">{t('menuType.menu')}</MenuItem>
        <MenuItem value="F">{t('menuType.button')}</MenuItem>
      </TextFieldRow>
      {form.menu_type !== 'F' && (
        <TextFieldRow
          select
          label={t('fields.menuIcon')}
          value={form.icon}
          onChange={(value) => setForm((current) => ({ ...current, icon: value }))}
        >
          {NAV_ICON_OPTIONS.map((option) => (
            <MenuItem key={option} value={option}>
              {option}
            </MenuItem>
          ))}
        </TextFieldRow>
      )}
      <TextFieldRow
        type="number"
        label={t('fields.orderNum')}
        value={form.order_num}
        onChange={(value) => setForm((current) => ({ ...current, order_num: Number(value) }))}
      />
      <TextFieldRow
        required
        label={t('fields.menuName')}
        value={form.menu_name}
        onChange={(value) => setForm((current) => ({ ...current, menu_name: value }))}
      />
      {form.menu_type !== 'F' && (
        <TextFieldRow
          label={t('fields.routeName')}
          value={form.route_name}
          onChange={(value) => setForm((current) => ({ ...current, route_name: value }))}
        />
      )}
      {form.menu_type !== 'F' && (
        <SwitchRow
          label={t('fields.isFrame')}
          checked={form.is_frame}
          onChange={(is_frame) => setForm((current) => ({ ...current, is_frame }))}
        />
      )}
      {form.menu_type !== 'F' && (
        <TextFieldRow
          label={t('fields.path')}
          value={form.path}
          onChange={(value) => setForm((current) => ({ ...current, path: value }))}
        />
      )}
      {form.menu_type === 'C' && (
        <TextFieldRow
          label={t('fields.component')}
          value={form.component ?? ''}
          onChange={(value) => setForm((current) => ({ ...current, component: value }))}
        />
      )}
      <TextFieldRow
        label={t('fields.perms')}
        value={form.perms ?? ''}
        onChange={(value) => setForm((current) => ({ ...current, perms: value }))}
      />
      {form.menu_type !== 'M' && (
        <TextFieldRow
          label={t('fields.query')}
          value={form.query ?? ''}
          onChange={(value) => setForm((current) => ({ ...current, query: value }))}
        />
      )}
      {form.menu_type === 'C' && (
        <SwitchRow
          label={t('fields.isCache')}
          checked={form.is_cache}
          onChange={(is_cache) => setForm((current) => ({ ...current, is_cache }))}
        />
      )}
      <TextFieldRow
        select
        label={t('fields.visible')}
        value={form.visible}
        onChange={(value) => setForm((current) => ({ ...current, visible: value }))}
      >
        <MenuItem value="0">{t('common.show')}</MenuItem>
        <MenuItem value="1">{t('common.hide')}</MenuItem>
      </TextFieldRow>
      <TextFieldRow
        select
        label={t('fields.menuStatus')}
        value={form.status}
        onChange={(value) => setForm((current) => ({ ...current, status: value }))}
      >
        <MenuItem value="0">{t('common.enabled')}</MenuItem>
        <MenuItem value="1">{t('common.disabled')}</MenuItem>
      </TextFieldRow>
      <TextFieldRow
        multiline
        label={t('common.remark')}
        value={form.remark ?? ''}
        onChange={(value) => setForm((current) => ({ ...current, remark: value }))}
      />
    </ManagementDialog>
  );
}

function Actions({
  canAdd,
  canEdit,
  canDelete,
  onCreateChild,
  onEdit,
  onDelete,
}: {
  canAdd: boolean;
  canEdit: boolean;
  canDelete: boolean;
  onCreateChild: () => void;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const { t } = useTranslate('admin');
  return (
    <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
      <Tooltip title={t('common.add')}>
        <span>
          <IconButton disabled={!canAdd} onClick={onCreateChild}>
            <Iconify icon="mingcute:add-line" />
          </IconButton>
        </span>
      </Tooltip>
      <Tooltip title={t('common.edit')}>
        <span>
          <IconButton disabled={!canEdit} onClick={onEdit}>
            <Iconify icon="solar:pen-bold" />
          </IconButton>
        </span>
      </Tooltip>
      <Tooltip title={t('common.delete')}>
        <span>
          <IconButton color="error" disabled={!canDelete} onClick={onDelete}>
            <Iconify icon="solar:trash-bin-trash-bold" />
          </IconButton>
        </span>
      </Tooltip>
    </Box>
  );
}

type MenuRowView = { menu: Menu; level: number; childCount: number };
function toInput(menu: Menu): MenuInput {
  return {
    menu_name: menu.menu_name,
    parent_id: menu.parent_id,
    order_num: menu.order_num,
    path: menu.path,
    component: menu.component,
    query: menu.query,
    route_name: menu.route_name,
    is_frame: menu.is_frame,
    is_cache: menu.is_cache,
    menu_type: menu.menu_type,
    visible: menu.visible,
    status: menu.status,
    perms: menu.perms,
    icon: menu.icon,
    remark: menu.remark,
  };
}
function menuTypeLabel(value: string, t: ReturnType<typeof useTranslate>['t']) {
  return value === 'M'
    ? t('menuType.directory')
    : value === 'F'
      ? t('menuType.button')
      : t('menuType.menu');
}
function menuHead(t: ReturnType<typeof useTranslate>['t']): TableHeadCellProps[] {
  return [
    { id: 'menu_name', label: t('fields.menuName') },
    { id: 'menu_type', label: t('fields.menuType') },
    { id: 'order_num', label: t('fields.orderNum') },
    { id: 'path', label: t('fields.path') },
    { id: 'component', label: t('fields.component') },
    { id: 'perms', label: t('fields.perms') },
    { id: 'visible', label: t('fields.visible') },
    { id: 'status', label: t('common.status') },
    { id: 'actions', label: t('common.actions'), align: 'right', width: 132 },
  ];
}
function toggle(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}
function menuIcon(icon: string): IconifyName {
  return (icon.startsWith('solar:') ? icon : 'solar:list-bold') as IconifyName;
}
function flattenMenuRows(menus: Menu[], expanded: string[]) {
  return walkMenus(menus, '0', 0, expanded);
}
function walkMenus(
  menus: Menu[],
  parentId: string,
  level: number,
  expanded: string[]
): MenuRowView[] {
  return menus
    .filter((menu) => menu.parent_id === parentId)
    .sort((a, b) => a.order_num - b.order_num)
    .flatMap((menu) => [
      { menu, level, childCount: menus.filter((child) => child.parent_id === menu.menu_id).length },
      ...(expanded.includes(menu.menu_id)
        ? walkMenus(menus, menu.menu_id, level + 1, expanded)
        : []),
    ]);
}
function parentMenuTree(menus: Menu[], editingId?: string) {
  return buildParentMenuNodes(
    menus.filter((menu) => menu.menu_type !== 'F' && menu.menu_id !== editingId),
    '0'
  );
}
function buildParentMenuNodes(
  menus: Menu[],
  parentId: string
): {
  id: string;
  label: string;
  disabled: boolean;
  children: ReturnType<typeof buildParentMenuNodes>;
}[] {
  return menus
    .filter((menu) => menu.parent_id === parentId)
    .sort((a, b) => a.order_num - b.order_num)
    .map((menu) => ({
      id: menu.menu_id,
      label: menu.menu_name,
      disabled: menu.status !== '0',
      children: buildParentMenuNodes(menus, menu.menu_id),
    }));
}
