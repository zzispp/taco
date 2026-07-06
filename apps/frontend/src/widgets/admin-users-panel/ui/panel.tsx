'use client';

import type { RoleOption } from 'src/entities/role';
import type { IconifyName } from 'src/shared/ui/iconify';
import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { UserInput, SystemUser } from 'src/entities/user';
import type { Post, TreeSelectNode } from 'src/entities/system';

import { useMemo, useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Card from '@mui/material/Card';
import List from '@mui/material/List';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Switch from '@mui/material/Switch';
import Tooltip from '@mui/material/Tooltip';
import Collapse from '@mui/material/Collapse';
import MenuItem from '@mui/material/MenuItem';
import TableRow from '@mui/material/TableRow';
import Checkbox from '@mui/material/Checkbox';
import TextField from '@mui/material/TextField';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import DialogTitle from '@mui/material/DialogTitle';
import Autocomplete from '@mui/material/Autocomplete';
import ListItemText from '@mui/material/ListItemText';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import ListItemButton from '@mui/material/ListItemButton';
import FormControlLabel from '@mui/material/FormControlLabel';

import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';
import { useTable, TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import {
  AddButton,
  TextFieldRow,
  TreeSelectField,
  AdminBreadcrumbs,
  ManagementDialog,
  TableLoadingRows,
  withSelectionHead,
  ManagementTableHead,
} from 'src/shared/ui/admin';

import { translatedRoleName } from 'src/entities/role';
import { useHasPermission } from 'src/entities/session';
import { useUsers, useUserFormOptions } from 'src/entities/user';

import {
  createUser,
  deleteUser,
  updateUser,
  exportUsers,
  importUsers,
  deleteUsers,
  getUserRoles,
  updateUserRoles,
  updateUserStatus,
  resetUserPassword,
  downloadUserImportTemplate,
} from 'src/features/user-management/api';

import { DashboardContent } from 'src/widgets/dashboard-shell';

const DEFAULT_FORM: UserInput = {
  username: '',
  password: '',
  nick_name: '',
  dept_id: null,
  email: '',
  phonenumber: '',
  sex: '2',
  status: '0',
  remark: '',
  role_ids: [],
  post_ids: [],
};

const DEFAULT_FILTERS = {
  username: '',
  phonenumber: '',
  status: '',
  dept_id: '',
  begin_time: '',
  end_time: '',
};
const MAX_VISIBLE_SELECT_TAGS = 2;

export function UserManagementView() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10 });
  const [filters, setFilters] = useState(DEFAULT_FILTERS);
  const users = useUsers(table.page, table.rowsPerPage, filters);
  const options = useUserFormOptions();
  const roles = useMemo(() => options.data?.roles ?? [], [options.data?.roles]);
  const posts = useMemo(() => options.data?.posts ?? [], [options.data?.posts]);
  const deptTree = useMemo(() => options.data?.depts ?? [], [options.data?.depts]);
  const depts = useMemo(() => flattenDeptNames(deptTree), [deptTree]);
  const [form, setForm] = useState<UserInput>(DEFAULT_FORM);
  const [editing, setEditing] = useState<SystemUser | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<SystemUser | null>(null);
  const [batchDeleteOpen, setBatchDeleteOpen] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);
  const [passwordTarget, setPasswordTarget] = useState<SystemUser | null>(null);
  const [newPassword, setNewPassword] = useState('');
  const [roleTarget, setRoleTarget] = useState<SystemUser | null>(null);
  const [assignedRoles, setAssignedRoles] = useState<string[]>([]);
  const [importOpen, setImportOpen] = useState(false);
  const [importFile, setImportFile] = useState<File | null>(null);
  const [updateSupport, setUpdateSupport] = useState(false);
  const head = useMemo(() => userHead(t), [t]);
  const canAdd = useHasPermission('system:user:add');
  const canDelete = useHasPermission('system:user:remove');
  const canImport = useHasPermission('system:user:import');
  const canExport = useHasPermission('system:user:export');
  const loadingHead = useMemo(
    () => (canDelete ? withSelectionHead(head) : head),
    [canDelete, head]
  );
  const selectableUsers = useMemo(() => users.items.filter((user) => !user.system), [users.items]);

  const closeDialog = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_FORM);
  }, []);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_FORM, role_ids: [roles[0]?.role_id ?? ''].filter(Boolean) });
  }, [roles]);

  const openEdit = useCallback((user: SystemUser) => {
    setEditing(user);
    setForm(toInput(user));
  }, []);

  const submitUser = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editing) await updateUser(editing.user_id, form);
      else await createUser(form);
      toast.success(t('messages.saved'));
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeDialog, editing, form, t]);

  const confirmDelete = useCallback(async () => {
    if (!deleteTarget) return;
    try {
      await deleteUser(deleteTarget.user_id);
      toast.success(t('messages.deleted'));
      setDeleteTarget(null);
      setSelected((current) => current.filter((id) => id !== deleteTarget.user_id));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteTarget, t]);

  const confirmBatchDelete = useCallback(async () => {
    if (selected.length === 0) return;
    try {
      await deleteUsers(selected);
      toast.success(t('messages.deleted'));
      setSelected([]);
      setBatchDeleteOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [selected, t]);

  const toggleAll = useCallback(
    (checked: boolean) => {
      setSelected(checked ? selectableUsers.map((user) => user.user_id) : []);
    },
    [selectableUsers]
  );

  const submitPassword = useCallback(async () => {
    if (!passwordTarget) return;
    setSubmitting(true);
    try {
      await resetUserPassword(passwordTarget.user_id, newPassword);
      toast.success(t('messages.saved'));
      setPasswordTarget(null);
      setNewPassword('');
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [newPassword, passwordTarget, t]);

  const openRoles = useCallback(
    async (user: SystemUser) => {
      setRoleTarget(user);
      try {
        const payload = await getUserRoles(user.user_id);
        setAssignedRoles(payload.role_ids);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.loadBindingsFailed'));
      }
    },
    [t]
  );

  const submitRoles = useCallback(async () => {
    if (!roleTarget) return;
    setSubmitting(true);
    try {
      await updateUserRoles(roleTarget.user_id, assignedRoles);
      toast.success(t('messages.rolePermissionsUpdated'));
      setRoleTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveBindingsFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [assignedRoles, roleTarget, t]);

  const submitImport = useCallback(async () => {
    if (!importFile) return;
    setSubmitting(true);
    try {
      const result = await importUsers(importFile, updateSupport);
      toast.success(result.message || t('messages.importSuccess'));
      setImportOpen(false);
      setImportFile(null);
      setUpdateSupport(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.importFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [importFile, t, updateSupport]);

  const submitExport = useCallback(async () => {
    try {
      await exportUsers(filters);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.exportFailed'));
    }
  }, [filters, t]);

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.userManagement')}
        action={
          canDelete || canAdd || canImport || canExport ? (
            <Stack direction="row" spacing={1}>
              {canImport && (
                <Button
                  variant="outlined"
                  startIcon={<Iconify icon="solar:import-bold" />}
                  onClick={() => setImportOpen(true)}
                >
                  {t('actions.import')}
                </Button>
              )}
              {canExport && (
                <Button
                  variant="outlined"
                  startIcon={<Iconify icon="solar:export-bold" />}
                  onClick={submitExport}
                >
                  {t('actions.export')}
                </Button>
              )}
              {canDelete && (
                <Button
                  variant="outlined"
                  color="error"
                  disabled={selected.length === 0}
                  onClick={() => setBatchDeleteOpen(true)}
                >
                  {t('common.delete')}
                </Button>
              )}
              {canAdd && <AddButton onClick={openCreate}>{t('actions.addUser')}</AddButton>}
            </Stack>
          ) : null
        }
      />
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={3}>
        <Card sx={{ width: { xs: 1, md: 280 }, flexShrink: 0, alignSelf: 'flex-start' }}>
          <DeptFilterTree
            nodes={deptTree}
            selected={filters.dept_id}
            onSelect={(dept_id) => setFilters((current) => ({ ...current, dept_id }))}
          />
        </Card>
        <Card sx={{ flex: 1, minWidth: 0 }}>
          <UserFilters filters={filters} onChange={setFilters} />
          <Scrollbar>
            <Table sx={{ minWidth: 1560 }}>
              <ManagementTableHead
                head={head}
                rowCount={selectableUsers.length}
                numSelected={selected.length}
                onSelectAllRows={canDelete ? toggleAll : undefined}
              />
              <TableBody>
                {users.isLoading ? (
                  <TableLoadingRows head={loadingHead} rows={table.rowsPerPage} />
                ) : (
                  users.items.map((row) => (
                    <UserRow
                      key={row.user_id}
                      row={row}
                      selected={selected.includes(row.user_id)}
                      onToggleSelected={(id) => setSelected(toggle(selected, id))}
                      roles={roles}
                      depts={depts}
                      posts={posts}
                      onEdit={openEdit}
                      onDelete={setDeleteTarget}
                      onRoles={openRoles}
                      onResetPassword={(user) => {
                        setPasswordTarget(user);
                        setNewPassword('');
                      }}
                      onStatusChange={(status) =>
                        updateUserStatus(row.user_id, status).catch(showError(t))
                      }
                    />
                  ))
                )}
                <TableNoData
                  title={t('common.noData')}
                  notFound={!users.isLoading && users.items.length === 0}
                />
              </TableBody>
            </Table>
          </Scrollbar>
          <TablePaginationCustom
            page={table.page}
            count={users.total}
            rowsPerPage={table.rowsPerPage}
            onPageChange={table.onChangePage}
            onRowsPerPageChange={table.onChangeRowsPerPage}
          />
        </Card>
      </Stack>
      <UserDialog
        open={creating || !!editing}
        editing={!!editing}
        submitting={submitting}
        form={form}
        roles={roles}
        depts={deptTree}
        posts={posts}
        setForm={setForm}
        onClose={closeDialog}
        onSubmit={submitUser}
      />
      <RoleAssignDialog
        user={roleTarget}
        roles={roles}
        selected={assignedRoles}
        submitting={submitting}
        onSelectedChange={setAssignedRoles}
        onClose={() => setRoleTarget(null)}
        onSubmit={submitRoles}
      />
      <PasswordDialog
        user={passwordTarget}
        password={newPassword}
        submitting={submitting}
        onPasswordChange={setNewPassword}
        onClose={() => setPasswordTarget(null)}
        onSubmit={submitPassword}
      />
      <UserImportDialog
        open={importOpen}
        file={importFile}
        updateSupport={updateSupport}
        submitting={submitting}
        onFileChange={setImportFile}
        onUpdateSupportChange={setUpdateSupport}
        onTemplate={downloadUserImportTemplate}
        onClose={() => {
          setImportOpen(false);
          setImportFile(null);
          setUpdateSupport(false);
        }}
        onSubmit={submitImport}
      />
      <ConfirmDialog
        open={batchDeleteOpen}
        onClose={() => setBatchDeleteOpen(false)}
        title={t('dialogs.deleteUser')}
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
        title={t('dialogs.deleteUser')}
        content={t('dialogs.deleteContent', { name: deleteTarget?.username ?? '' })}
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

function UserFilters({
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
        label={t('common.username')}
        value={filters.username}
        onChange={(event) => write('username', event.target.value)}
      />
      <TextField
        size="small"
        label={t('fields.phone')}
        value={filters.phonenumber}
        onChange={(event) => write('phonenumber', event.target.value)}
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
      <TextField
        size="small"
        type="date"
        label={t('fields.beginTime')}
        value={filters.begin_time}
        InputLabelProps={{ shrink: true }}
        onChange={(event) => write('begin_time', event.target.value)}
      />
      <TextField
        size="small"
        type="date"
        label={t('fields.endTime')}
        value={filters.end_time}
        InputLabelProps={{ shrink: true }}
        onChange={(event) => write('end_time', event.target.value)}
      />
      <Button variant="outlined" onClick={() => onChange(DEFAULT_FILTERS)}>
        {t('common.reset')}
      </Button>
    </Stack>
  );
}

function DeptFilterTree({
  nodes,
  selected,
  onSelect,
}: {
  nodes: TreeSelectNode[];
  selected: string;
  onSelect: (id: string) => void;
}) {
  const { t } = useTranslate('admin');
  const [keyword, setKeyword] = useState('');
  const [expanded, setExpanded] = useState<string[]>([]);
  const visibleNodes = useMemo(() => filterDeptTree(nodes, keyword), [keyword, nodes]);
  const expandedIds =
    expanded.length > 0 ? expanded : flattenTree(visibleNodes).map((dept) => dept.id);
  return (
    <Box sx={{ p: 2 }}>
      <Box sx={{ typography: 'subtitle2', mb: 1 }}>{t('fields.deptTree')}</Box>
      <TextField
        fullWidth
        size="small"
        value={keyword}
        label={t('fields.deptName')}
        sx={{ mb: 1 }}
        onChange={(event) => setKeyword(event.target.value)}
      />
      <List disablePadding>
        <ListItemButton
          dense
          selected={selected === ''}
          sx={{ mb: 0.5 }}
          onClick={() => onSelect('')}
        >
          <Box sx={{ width: 34 }} />
          <ListItemText primary={t('common.all')} />
        </ListItemButton>
        {visibleNodes.map((node) => (
          <DeptFilterNode
            key={node.id}
            node={node}
            level={0}
            selected={selected}
            expanded={expandedIds}
            onToggle={(id) => setExpanded(toggle(expandedIds, id))}
            onSelect={onSelect}
          />
        ))}
      </List>
    </Box>
  );
}

function DeptFilterNode({
  node,
  level,
  selected,
  expanded,
  onToggle,
  onSelect,
}: {
  node: TreeSelectNode;
  level: number;
  selected: string;
  expanded: string[];
  onToggle: (id: string) => void;
  onSelect: (id: string) => void;
}) {
  const open = expanded.includes(node.id);
  const hasChildren = node.children.length > 0;
  return (
    <>
      <ListItemButton
        dense
        selected={selected === node.id}
        sx={{ pl: 1 + level * 2 }}
        onClick={() => onSelect(node.id)}
      >
        {hasChildren ? (
          <IconButton
            size="small"
            onClick={(event) => {
              event.stopPropagation();
              onToggle(node.id);
            }}
          >
            <Iconify icon={open ? 'eva:arrow-ios-downward-fill' : 'eva:arrow-ios-forward-fill'} />
          </IconButton>
        ) : (
          <Box sx={{ width: 34 }} />
        )}
        <ListItemText primary={node.label} />
      </ListItemButton>
      {hasChildren && (
        <Collapse in={open}>
          {node.children.map((child) => (
            <DeptFilterNode
              key={child.id}
              node={child}
              level={level + 1}
              selected={selected}
              expanded={expanded}
              onToggle={onToggle}
              onSelect={onSelect}
            />
          ))}
        </Collapse>
      )}
    </>
  );
}

function UserRow({
  row,
  selected,
  roles,
  depts,
  posts,
  onToggleSelected,
  onEdit,
  onDelete,
  onRoles,
  onResetPassword,
  onStatusChange,
}: {
  row: SystemUser;
  selected: boolean;
  roles: RoleOption[];
  depts: FlatNode[];
  posts: Post[];
  onToggleSelected: (id: string) => void;
  onEdit: (user: SystemUser) => void;
  onDelete: (user: SystemUser) => void;
  onRoles: (user: SystemUser) => void;
  onResetPassword: (user: SystemUser) => void;
  onStatusChange: (status: string) => void;
}) {
  const { t } = useTranslate('admin');
  const canEdit = useHasPermission('system:user:edit');
  const canDelete = useHasPermission('system:user:remove');
  const canReset = useHasPermission('system:user:resetPwd');
  const roleNames = displayRoles(row.role_ids, roles, t);
  const postNames = namesByIds(posts, row.post_ids, 'post_id', 'post_name');

  return (
    <TableRow hover>
      {canDelete && (
        <TableCell padding="checkbox">
          <Checkbox
            disabled={row.system}
            checked={selected}
            onChange={() => onToggleSelected(row.user_id)}
          />
        </TableCell>
      )}
      <TableCell sx={USER_CELL_SX}>{row.username}</TableCell>
      <TableCell sx={USER_CELL_SX}>{row.nick_name}</TableCell>
      <TableCell sx={USER_CELL_SX}>{nameById(depts, row.dept_id)}</TableCell>
      <TableCell sx={USER_CELL_SX}>{row.phonenumber || '-'}</TableCell>
      <TableCell sx={USER_ELLIPSIS_CELL_SX}>{row.email || '-'}</TableCell>
      <TableCell sx={USER_CELL_SX}>{sexLabel(row.sex, t)}</TableCell>
      <TableCell sx={USER_CELL_SX}>
        <Switch
          size="small"
          checked={row.status === '0'}
          disabled={row.system || !canEdit}
          onChange={(event) => onStatusChange(event.target.checked ? '0' : '1')}
        />
      </TableCell>
      <TableCell sx={USER_ELLIPSIS_CELL_SX}>{postNames}</TableCell>
      <TableCell sx={USER_ELLIPSIS_CELL_SX}>{roleNames}</TableCell>
      <TableCell sx={USER_CELL_SX}>{fAdminDateTime(row.create_time) || '-'}</TableCell>
      <TableCell align="right" sx={USER_CELL_SX}>
        <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
          <ActionIcon
            title={t('common.edit')}
            disabled={row.system || !canEdit}
            icon="solar:pen-bold"
            onClick={() => onEdit(row)}
          />
          <ActionIcon
            title={t('actions.resetPassword')}
            disabled={row.system || !canReset}
            icon="solar:restart-bold"
            onClick={() => onResetPassword(row)}
          />
          <ActionIcon
            title={t('actions.assignRoles')}
            disabled={row.system || !canEdit}
            icon="solar:user-id-bold"
            onClick={() => onRoles(row)}
          />
          <ActionIcon
            title={t('common.delete')}
            disabled={row.system || !canDelete}
            color="error"
            icon="solar:trash-bin-trash-bold"
            onClick={() => onDelete(row)}
          />
        </Box>
      </TableCell>
    </TableRow>
  );
}

function UserDialog({
  open,
  editing,
  submitting,
  form,
  roles,
  depts,
  posts,
  setForm,
  onClose,
  onSubmit,
}: {
  open: boolean;
  editing: boolean;
  submitting: boolean;
  form: UserInput;
  roles: RoleOption[];
  depts: TreeSelectNode[];
  posts: Post[];
  setForm: React.Dispatch<React.SetStateAction<UserInput>>;
  onClose: () => void;
  onSubmit: () => void;
}) {
  const { t } = useTranslate('admin');
  const roleOptions = useMemo(
    () => roles.map((role) => ({ id: role.role_id, label: translatedRoleName(role, t) })),
    [roles, t]
  );
  const postOptions = useMemo(
    () => posts.map((post) => ({ id: post.post_id, label: post.post_name })),
    [posts]
  );
  return (
    <ManagementDialog
      open={open}
      title={editing ? t('dialogs.editUser') : t('dialogs.createUser')}
      submitting={submitting}
      onClose={onClose}
      onSubmit={onSubmit}
    >
      <TextFieldRow
        required
        label={t('common.username')}
        value={form.username}
        onChange={(value) => setForm((current) => ({ ...current, username: value }))}
      />
      <TextFieldRow
        required
        label={t('fields.nickName')}
        value={form.nick_name}
        onChange={(value) => setForm((current) => ({ ...current, nick_name: value }))}
      />
      <TreeSelectField
        label={t('fields.deptName')}
        value={form.dept_id ?? ''}
        nodes={depts}
        rootValue=""
        rootLabel={t('common.none')}
        onChange={(value) => setForm((current) => ({ ...current, dept_id: value || null }))}
      />
      <TextFieldRow
        label={t('fields.phone')}
        value={form.phonenumber ?? ''}
        onChange={(value) => setForm((current) => ({ ...current, phonenumber: value }))}
      />
      <TextFieldRow
        required
        label={t('common.email')}
        value={form.email}
        onChange={(value) => setForm((current) => ({ ...current, email: value }))}
      />
      {!editing && (
        <TextFieldRow
          type="password"
          label={t('common.password')}
          helperText={t('helper.emptyPasswordUsesDefault')}
          value={form.password ?? ''}
          onChange={(value) => setForm((current) => ({ ...current, password: value }))}
        />
      )}
      <TextFieldRow
        select
        label={t('fields.sex')}
        value={form.sex}
        onChange={(value) => setForm((current) => ({ ...current, sex: value }))}
      >
        <MenuItem value="0">{t('common.male')}</MenuItem>
        <MenuItem value="1">{t('common.female')}</MenuItem>
        <MenuItem value="2">{t('common.unknown')}</MenuItem>
      </TextFieldRow>
      <TextFieldRow
        select
        label={t('common.status')}
        value={form.status}
        onChange={(value) => setForm((current) => ({ ...current, status: value }))}
      >
        <MenuItem value="0">{t('common.enabled')}</MenuItem>
        <MenuItem value="1">{t('common.disabled')}</MenuItem>
      </TextFieldRow>
      <SearchMultiSelect
        label={t('common.role')}
        values={form.role_ids}
        options={roleOptions}
        onChange={(role_ids) => setForm((current) => ({ ...current, role_ids }))}
      />
      <SearchMultiSelect
        label={t('fields.postName')}
        values={form.post_ids}
        options={postOptions}
        onChange={(post_ids) => setForm((current) => ({ ...current, post_ids }))}
      />
      <TextFieldRow
        multiline
        label={t('common.remark')}
        value={form.remark ?? ''}
        onChange={(value) => setForm((current) => ({ ...current, remark: value }))}
      />
    </ManagementDialog>
  );
}

function RoleAssignDialog({
  user,
  roles,
  selected,
  submitting,
  onSelectedChange,
  onClose,
  onSubmit,
}: {
  user: SystemUser | null;
  roles: RoleOption[];
  selected: string[];
  submitting: boolean;
  onSelectedChange: (value: string[]) => void;
  onClose: () => void;
  onSubmit: () => void;
}) {
  const { t } = useTranslate('admin');
  const roleOptions = useMemo(
    () => roles.map((role) => ({ id: role.role_id, label: translatedRoleName(role, t) })),
    [roles, t]
  );
  return (
    <ManagementDialog
      open={!!user}
      title={t('dialogs.assignRoles', { name: user?.username ?? '' })}
      submitting={submitting}
      onClose={onClose}
      onSubmit={onSubmit}
    >
      <SearchMultiSelect
        label={t('common.role')}
        values={selected}
        options={roleOptions}
        onChange={onSelectedChange}
      />
    </ManagementDialog>
  );
}

function PasswordDialog({
  user,
  password,
  submitting,
  onPasswordChange,
  onClose,
  onSubmit,
}: {
  user: SystemUser | null;
  password: string;
  submitting: boolean;
  onPasswordChange: (value: string) => void;
  onClose: () => void;
  onSubmit: () => void;
}) {
  const { t } = useTranslate('admin');
  return (
    <ManagementDialog
      open={!!user}
      title={t('dialogs.resetPassword', { name: user?.username ?? '' })}
      submitting={submitting}
      onClose={onClose}
      onSubmit={onSubmit}
    >
      <TextFieldRow
        required
        type="password"
        label={t('fields.newPassword')}
        value={password}
        onChange={onPasswordChange}
      />
    </ManagementDialog>
  );
}

function UserImportDialog({
  open,
  file,
  updateSupport,
  submitting,
  onFileChange,
  onUpdateSupportChange,
  onTemplate,
  onClose,
  onSubmit,
}: {
  open: boolean;
  file: File | null;
  updateSupport: boolean;
  submitting: boolean;
  onFileChange: (file: File | null) => void;
  onUpdateSupportChange: (value: boolean) => void;
  onTemplate: () => Promise<void>;
  onClose: () => void;
  onSubmit: () => void;
}) {
  const { t } = useTranslate('admin');
  const downloadTemplate = async () => {
    try {
      await onTemplate();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.exportFailed'));
    }
  };
  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="sm">
      <DialogTitle>{t('dialogs.importUser')}</DialogTitle>
      <DialogContent>
        <Stack spacing={2} sx={{ pt: 1 }}>
          <Button
            component="label"
            variant="outlined"
            startIcon={<Iconify icon="eva:cloud-upload-fill" />}
          >
            {file?.name ?? t('actions.selectFile')}
            <input
              hidden
              type="file"
              accept=".xlsx,.xls"
              onChange={(event) => onFileChange(event.target.files?.[0] ?? null)}
            />
          </Button>
          <FormControlLabel
            control={
              <Switch
                checked={updateSupport}
                onChange={(event) => onUpdateSupportChange(event.target.checked)}
              />
            }
            label={t('fields.updateSupport')}
          />
          <Box sx={{ typography: 'body2', color: 'text.secondary' }}>
            {t('helper.userImportTemplate')}
          </Box>
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={downloadTemplate} startIcon={<Iconify icon="solar:download-bold" />}>
          {t('actions.downloadTemplate')}
        </Button>
        <Box sx={{ flexGrow: 1 }} />
        <Button onClick={onClose}>{t('common.cancel')}</Button>
        <Button variant="contained" disabled={!file || submitting} onClick={onSubmit}>
          {t('actions.import')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function SearchMultiSelect({
  label,
  values,
  options,
  onChange,
}: {
  label: string;
  values: string[];
  options: SelectOptionItem[];
  onChange: (values: string[]) => void;
}) {
  const selectedOptions = options.filter((option) => values.includes(option.id));
  return (
    <Autocomplete
      multiple
      disableCloseOnSelect
      size="small"
      options={options}
      value={selectedOptions}
      isOptionEqualToValue={(option, value) => option.id === value.id}
      getOptionLabel={(option) => option.label}
      onChange={(_, next) => onChange(next.map((option) => option.id))}
      renderOption={(props, option, state) => (
        <OptionRow key={option.id} props={props} option={option} selected={state.selected} />
      )}
      renderValue={(selected, getItemProps) =>
        renderSelectedOptions(selected as SelectOptionItem[], getItemProps)
      }
      renderInput={(params) => <TextField {...params} label={label} />}
      slotProps={{ listbox: { sx: { maxHeight: 280 } } }}
    />
  );
}

function OptionRow({
  props,
  option,
  selected,
}: {
  props: React.HTMLAttributes<HTMLLIElement> & { key: React.Key };
  option: SelectOptionItem;
  selected: boolean;
}) {
  const { key, ...itemProps } = props;
  return (
    <li key={key} {...itemProps}>
      <Checkbox size="small" checked={selected} sx={{ mr: 1 }} />
      {option.label}
    </li>
  );
}

function renderSelectedOptions(
  selected: SelectOptionItem[],
  getItemProps: (args: { index: number }) => Record<string, unknown>
) {
  const visible = selected.slice(0, MAX_VISIBLE_SELECT_TAGS);
  const hiddenCount = selected.length - visible.length;
  return (
    <Box sx={{ display: 'flex', minWidth: 0, gap: 0.5, overflow: 'hidden' }}>
      {visible.map((option, index) => (
        <Chip
          {...getItemProps({ index })}
          key={option.id}
          size="small"
          label={option.label}
          sx={{ maxWidth: 150 }}
        />
      ))}
      {hiddenCount > 0 && <Chip size="small" label={`+${hiddenCount}`} />}
    </Box>
  );
}

function ActionIcon({
  title,
  icon,
  disabled,
  color,
  onClick,
}: {
  title: string;
  icon: IconifyName;
  disabled: boolean;
  color?: 'error';
  onClick: () => void;
}) {
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

function toInput(user: SystemUser): UserInput {
  return {
    username: user.username,
    password: '',
    nick_name: user.nick_name,
    dept_id: user.dept_id,
    email: user.email,
    phonenumber: user.phonenumber,
    sex: user.sex,
    status: user.status,
    remark: user.remark,
    role_ids: user.role_ids,
    post_ids: user.post_ids,
  };
}
const USER_HEAD_SX = { whiteSpace: 'nowrap' } as const;
const USER_CELL_SX = { whiteSpace: 'nowrap' } as const;
const USER_ELLIPSIS_CELL_SX = {
  whiteSpace: 'nowrap',
  maxWidth: 220,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
} as const;

function userHead(t: ReturnType<typeof useTranslate>['t']): TableHeadCellProps[] {
  return [
    { id: 'username', label: t('common.username'), width: 120, sx: USER_HEAD_SX },
    { id: 'nick_name', label: t('fields.nickName'), width: 120, sx: USER_HEAD_SX },
    { id: 'dept_id', label: t('fields.deptName'), width: 140, sx: USER_HEAD_SX },
    { id: 'phonenumber', label: t('fields.phone'), width: 140, sx: USER_HEAD_SX },
    { id: 'email', label: t('common.email'), width: 220, sx: USER_HEAD_SX },
    { id: 'sex', label: t('fields.sex'), width: 80, sx: USER_HEAD_SX },
    { id: 'status', label: t('common.status'), width: 90, sx: USER_HEAD_SX },
    { id: 'post_ids', label: t('fields.postName'), width: 140, sx: USER_HEAD_SX },
    { id: 'role_ids', label: t('common.role'), width: 160, sx: USER_HEAD_SX },
    { id: 'create_time', label: t('fields.createTime'), width: 190, sx: USER_HEAD_SX },
    { id: 'actions', label: t('common.actions'), align: 'right', width: 176, sx: USER_HEAD_SX },
  ];
}
function toggle(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}
function sexLabel(value: string, t: ReturnType<typeof useTranslate>['t']) {
  return value === '0'
    ? t('common.male')
    : value === '1'
      ? t('common.female')
      : t('common.unknown');
}
function displayRoles(ids: string[], roles: RoleOption[], t: ReturnType<typeof useTranslate>['t']) {
  return (
    ids
      .map((id) => roles.find((role) => role.role_id === id))
      .filter(Boolean)
      .map((role) => translatedRoleName(role as RoleOption, t))
      .join(', ') || '-'
  );
}
function namesByIds<T>(items: T[], ids: string[], idKey: keyof T, nameKey: keyof T) {
  return (
    ids.map((id) => String(items.find((item) => item[idKey] === id)?.[nameKey] ?? id)).join(', ') ||
    '-'
  );
}
function nameById(items: FlatNode[], id: string | null) {
  return id ? (items.find((item) => item.id === id)?.label ?? id) : '-';
}
function showError(t: ReturnType<typeof useTranslate>['t']) {
  return (error: unknown) =>
    toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
}
type FlatNode = { id: string; label: string };
type SelectOptionItem = { id: string; label: string };
function flattenTree(nodes: TreeSelectNode[], level = 0): FlatNode[] {
  return nodes.flatMap((node) => [
    { id: node.id, label: `${'　'.repeat(level)}${node.label}` },
    ...flattenTree(node.children, level + 1),
  ]);
}
function flattenDeptNames(nodes: TreeSelectNode[]): FlatNode[] {
  return nodes.flatMap((node) => [
    { id: node.id, label: node.label },
    ...flattenDeptNames(node.children),
  ]);
}

function filterDeptTree(nodes: TreeSelectNode[], keyword: string): TreeSelectNode[] {
  const term = keyword.trim().toLowerCase();
  if (!term) return nodes;
  return nodes.flatMap((node) => {
    const children = filterDeptTree(node.children, term);
    if (node.label.toLowerCase().includes(term) || children.length > 0)
      return [{ ...node, children }];
    return [];
  });
}
