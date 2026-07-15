import type React from 'react';
import type { DeptInput, TreeSelectNode } from 'src/entities/system';

import MenuItem from '@mui/material/MenuItem';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { TextFieldRow, TreeSelectField, ManagementDialog } from 'src/widgets/admin-common';

export function DeptDialog({
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
      <DeptFields form={form} parentNodes={parentNodes} setForm={setForm} />
    </ManagementDialog>
  );
}

type DeptFieldsProps = Readonly<{
  form: DeptInput;
  parentNodes: TreeSelectNode[];
  setForm: React.Dispatch<React.SetStateAction<DeptInput>>;
}>;

function DeptFields({ form, parentNodes, setForm }: DeptFieldsProps) {
  const { t } = useTranslate('admin');
  return (
    <>
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
    </>
  );
}
