import type React from 'react';
import type { RoleOption } from 'src/entities/role';
import type { UserInput, SystemUser } from 'src/entities/user';
import type { Post, TreeSelectNode } from 'src/entities/system';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Switch from '@mui/material/Switch';
import MenuItem from '@mui/material/MenuItem';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import FormControlLabel from '@mui/material/FormControlLabel';

import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { TextFieldRow, TreeSelectField, ManagementDialog } from 'src/shared/ui/admin';

import { translatedRoleName } from 'src/entities/role';

import { SearchMultiSelect } from './search-multi-select';

export function UserDialog({
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
  const roleOptions = roles.map((role) => ({
    id: role.role_id,
    label: translatedRoleName(role, t),
  }));
  const postOptions = posts.map((post) => ({ id: post.post_id, label: post.post_name }));
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

export function RoleAssignDialog({
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
  const roleOptions = roles.map((role) => ({
    id: role.role_id,
    label: translatedRoleName(role, t),
  }));
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

export function PasswordDialog({
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

export function UserImportDialog({
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
