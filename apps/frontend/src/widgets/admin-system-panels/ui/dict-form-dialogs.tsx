import type React from 'react';
import type { DictDataInput, DictTypeInput } from 'src/entities/system';

import MenuItem from '@mui/material/MenuItem';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { TextFieldRow, ManagementDialog } from 'src/shared/ui/admin';

import { listClassOptions } from './dict-helpers';

export function DictTypeDialog({
  open,
  editing,
  submitting,
  form,
  setForm,
  onClose,
  onSubmit,
}: DialogProps<DictTypeInput>) {
  const { t } = useTranslate('admin');
  return (
    <ManagementDialog
      open={open}
      title={editing ? t('common.edit') : t('common.create')}
      submitting={submitting}
      onClose={onClose}
      onSubmit={onSubmit}
    >
      <TextFieldRow
        label={t('fields.dictName')}
        value={form.dict_name}
        onChange={(value) => setForm((current) => ({ ...current, dict_name: value }))}
      />
      <TextFieldRow
        label={t('fields.dictType')}
        value={form.dict_type}
        onChange={(value) => setForm((current) => ({ ...current, dict_type: value }))}
      />
      <StatusField
        value={form.status}
        onChange={(status) => setForm((current) => ({ ...current, status }))}
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

export function DictDataDialog({
  open,
  editing,
  submitting,
  form,
  setForm,
  onClose,
  onSubmit,
}: DialogProps<DictDataInput>) {
  const { t } = useTranslate('admin');
  return (
    <ManagementDialog
      open={open}
      title={editing ? t('common.edit') : t('common.create')}
      submitting={submitting}
      onClose={onClose}
      onSubmit={onSubmit}
    >
      <TextFieldRow
        type="number"
        label={t('fields.dictSort')}
        value={form.dict_sort}
        onChange={(value) => setForm((current) => ({ ...current, dict_sort: Number(value) }))}
      />
      <TextFieldRow
        label={t('fields.dictLabel')}
        value={form.dict_label}
        onChange={(value) => setForm((current) => ({ ...current, dict_label: value }))}
      />
      <TextFieldRow
        label={t('fields.dictValue')}
        value={form.dict_value}
        onChange={(value) => setForm((current) => ({ ...current, dict_value: value }))}
      />
      <TextFieldRow
        label={t('fields.cssClass')}
        value={form.css_class ?? ''}
        onChange={(value) => setForm((current) => ({ ...current, css_class: value }))}
      />
      <TextFieldRow
        select
        label={t('fields.listClass')}
        value={form.list_class ?? 'default'}
        onChange={(value) => setForm((current) => ({ ...current, list_class: value }))}
      >
        {listClassOptions().map((option) => (
          <MenuItem key={option} value={option}>
            {option}
          </MenuItem>
        ))}
      </TextFieldRow>
      <TextFieldRow
        select
        label={t('fields.isDefault')}
        value={form.is_default}
        onChange={(value) => setForm((current) => ({ ...current, is_default: value }))}
      >
        <MenuItem value="Y">{t('common.yes')}</MenuItem>
        <MenuItem value="N">{t('common.no')}</MenuItem>
      </TextFieldRow>
      <StatusField
        value={form.status}
        onChange={(status) => setForm((current) => ({ ...current, status }))}
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

function StatusField({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  const { t } = useTranslate('admin');
  return (
    <TextFieldRow select label={t('common.status')} value={value} onChange={onChange}>
      <MenuItem value="0">{t('common.enabled')}</MenuItem>
      <MenuItem value="1">{t('common.disabled')}</MenuItem>
    </TextFieldRow>
  );
}

type DialogProps<T> = {
  open: boolean;
  editing: boolean;
  submitting: boolean;
  form: T;
  setForm: React.Dispatch<React.SetStateAction<T>>;
  onClose: () => void;
  onSubmit: () => void;
};
