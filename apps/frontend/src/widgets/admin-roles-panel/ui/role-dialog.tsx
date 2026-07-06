'use client';

import type { RoleInput } from 'src/entities/role';

import MenuItem from '@mui/material/MenuItem';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { TextFieldRow, ManagementDialog } from 'src/shared/ui/admin';

export function RoleDialog({
  open,
  editing,
  submitting,
  form,
  setForm,
  onClose,
  onSubmit,
}: {
  open: boolean;
  editing: boolean;
  submitting: boolean;
  form: RoleInput;
  setForm: React.Dispatch<React.SetStateAction<RoleInput>>;
  onClose: () => void;
  onSubmit: () => void;
}) {
  const { t } = useTranslate('admin');
  return (
    <ManagementDialog
      open={open}
      title={editing ? t('dialogs.editRole') : t('dialogs.createRole')}
      submitting={submitting}
      onClose={onClose}
      onSubmit={onSubmit}
    >
      <TextFieldRow
        required
        label={t('fields.roleName')}
        value={form.role_name}
        onChange={(value) => setForm((current) => ({ ...current, role_name: value }))}
      />
      <TextFieldRow
        required
        label={t('fields.roleKey')}
        value={form.role_key}
        onChange={(value) => setForm((current) => ({ ...current, role_key: value }))}
      />
      <TextFieldRow
        type="number"
        label={t('fields.roleSort')}
        value={form.role_sort}
        onChange={(value) => setForm((current) => ({ ...current, role_sort: Number(value) }))}
      />
      <TextFieldRow
        select
        label={t('common.status')}
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

export function dataScopeLabel(value: string, t: ReturnType<typeof useTranslate>['t']) {
  return dataScopeOptions(t).find((option) => option.value === value)?.label ?? value;
}

function dataScopeOptions(t: ReturnType<typeof useTranslate>['t']) {
  return [
    { value: '1', label: t('dataScope.all') },
    { value: '2', label: t('dataScope.custom') },
    { value: '3', label: t('dataScope.dept') },
    { value: '4', label: t('dataScope.deptAndChild') },
    { value: '5', label: t('dataScope.self') },
  ];
}
