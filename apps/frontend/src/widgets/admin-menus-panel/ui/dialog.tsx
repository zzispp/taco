import type { Dispatch, SetStateAction } from 'react';
import type { Menu, MenuInput } from 'src/entities/menu';

import MenuItem from '@mui/material/MenuItem';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { NAV_ICON_OPTIONS } from 'src/entities/menu';

import {
  SwitchRow,
  TextFieldRow,
  TreeSelectField,
  ManagementDialog,
} from 'src/widgets/admin-common';

import { parentMenuTree } from './helpers';

type MenuDialogProps = {
  open: boolean;
  editing: boolean;
  submitting: boolean;
  form: MenuInput;
  menus: Menu[];
  editingId?: string;
  setForm: Dispatch<SetStateAction<MenuInput>>;
  onClose: () => void;
  onSubmit: () => void;
};

export function MenuDialog(props: MenuDialogProps) {
  const { open, editing, submitting, form, menus, editingId, setForm, onClose, onSubmit } = props;
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
      <MenuParentField form={form} parentNodes={parentNodes} setForm={setForm} />
      <MenuBaseFields form={form} setForm={setForm} />
      <MenuRouteFields form={form} setForm={setForm} />
      <MenuStatusFields form={form} setForm={setForm} />
    </ManagementDialog>
  );
}

type MenuFieldGroupProps = Pick<MenuDialogProps, 'form' | 'setForm'>;
type ParentFieldProps = MenuFieldGroupProps & { parentNodes: ReturnType<typeof parentMenuTree> };

function MenuParentField({ form, parentNodes, setForm }: ParentFieldProps) {
  const { t } = useTranslate('admin');

  return (
    <TreeSelectField
      label={t('fields.parentMenu')}
      value={form.parent_id}
      nodes={parentNodes}
      onChange={(value) => setForm((current) => ({ ...current, parent_id: value }))}
    />
  );
}

function MenuBaseFields({ form, setForm }: MenuFieldGroupProps) {
  const { t } = useTranslate('admin');

  return (
    <>
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
      {form.menu_type !== 'F' && <MenuIconField form={form} setForm={setForm} />}
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
    </>
  );
}

function MenuIconField({ form, setForm }: MenuFieldGroupProps) {
  const { t } = useTranslate('admin');

  return (
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
  );
}

function MenuRouteFields({ form, setForm }: MenuFieldGroupProps) {
  const { t } = useTranslate('admin');

  return (
    <>
      {form.menu_type !== 'F' && (
        <TextFieldRow
          label={t('fields.routeName')}
          value={form.route_name}
          onChange={(value) => setForm((current) => ({ ...current, route_name: value }))}
        />
      )}
      {form.menu_type !== 'F' && <FrameField form={form} setForm={setForm} />}
      {form.menu_type !== 'F' && <PathField form={form} setForm={setForm} />}
      {form.menu_type === 'C' && <ComponentField form={form} setForm={setForm} />}
      <PermsField form={form} setForm={setForm} />
      {form.menu_type !== 'M' && <QueryField form={form} setForm={setForm} />}
      {form.menu_type === 'C' && <CacheField form={form} setForm={setForm} />}
    </>
  );
}

function FrameField({ form, setForm }: MenuFieldGroupProps) {
  const { t } = useTranslate('admin');
  return (
    <SwitchRow
      label={t('fields.isFrame')}
      checked={form.is_frame}
      onChange={(is_frame) => setForm((current) => ({ ...current, is_frame }))}
    />
  );
}

function PathField({ form, setForm }: MenuFieldGroupProps) {
  const { t } = useTranslate('admin');
  return (
    <TextFieldRow
      label={t('fields.path')}
      value={form.path}
      onChange={(value) => setForm((current) => ({ ...current, path: value }))}
    />
  );
}

function ComponentField({ form, setForm }: MenuFieldGroupProps) {
  const { t } = useTranslate('admin');
  return (
    <TextFieldRow
      label={t('fields.component')}
      value={form.component ?? ''}
      onChange={(value) => setForm((current) => ({ ...current, component: value }))}
    />
  );
}

function PermsField({ form, setForm }: MenuFieldGroupProps) {
  const { t } = useTranslate('admin');
  return (
    <TextFieldRow
      label={t('fields.perms')}
      value={form.perms ?? ''}
      onChange={(value) => setForm((current) => ({ ...current, perms: value }))}
    />
  );
}

function QueryField({ form, setForm }: MenuFieldGroupProps) {
  const { t } = useTranslate('admin');
  return (
    <TextFieldRow
      label={t('fields.query')}
      value={form.query ?? ''}
      onChange={(value) => setForm((current) => ({ ...current, query: value }))}
    />
  );
}

function CacheField({ form, setForm }: MenuFieldGroupProps) {
  const { t } = useTranslate('admin');
  return (
    <SwitchRow
      label={t('fields.isCache')}
      checked={form.is_cache}
      onChange={(is_cache) => setForm((current) => ({ ...current, is_cache }))}
    />
  );
}

function MenuStatusFields({ form, setForm }: MenuFieldGroupProps) {
  const { t } = useTranslate('admin');

  return (
    <>
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
    </>
  );
}
