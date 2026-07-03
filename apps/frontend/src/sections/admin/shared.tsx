'use client';

import type { Role, ApiPermission } from 'src/types/rbac';
import type { TableHeadCellProps } from 'src/components/table';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import MenuItem from '@mui/material/MenuItem';
import TableCell from '@mui/material/TableCell';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import FormControlLabel from '@mui/material/FormControlLabel';

import { paths } from 'src/routes/paths';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { TableHeadCustom } from 'src/components/table';
import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

// ----------------------------------------------------------------------

export { NAV_ICONS, NAV_ICON_OPTIONS, translatedMenuItem, translatedMenuSection } from './nav-metadata';

export const METHOD_OPTIONS = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE'];

export type AdminT = ReturnType<typeof useTranslate>['t'];

export function AdminBreadcrumbs({
  heading,
  action,
}: {
  heading: string;
  action?: React.ReactNode;
}) {
  const { t } = useTranslate('admin');

  return (
    <CustomBreadcrumbs
      heading={heading}
      links={[
        { name: t('nav.dashboard'), href: paths.dashboard.root },
        { name: t('nav.systemManagement') },
        { name: heading },
      ]}
      action={action}
      sx={{ mb: { xs: 3, md: 5 } }}
    />
  );
}

export function AddButton({ onClick, children }: { onClick: () => void; children: React.ReactNode }) {
  return (
    <Button variant="contained" startIcon={<Iconify icon="mingcute:add-line" />} onClick={onClick}>
      {children}
    </Button>
  );
}

export function EnabledLabel({ enabled }: { enabled: boolean }) {
  const { t } = useTranslate('admin');

  return (
    <Label color={enabled ? 'success' : 'default'} variant="soft">
      {enabled ? t('common.enabled') : t('common.disabled')}
    </Label>
  );
}

export function BooleanLabel({ enabled, trueText, falseText }: { enabled: boolean; trueText: string; falseText: string }) {
  return (
    <Label color={enabled ? 'info' : 'default'} variant="soft">
      {enabled ? trueText : falseText}
    </Label>
  );
}

export function MethodLabel({ method }: { method: string }) {
  const color =
    (method === 'GET' && 'success') ||
    (method === 'POST' && 'info') ||
    (method === 'PUT' && 'warning') ||
    (method === 'PATCH' && 'warning') ||
    (method === 'DELETE' && 'error') ||
    'default';

  return (
    <Label color={color} variant="soft">
      {method}
    </Label>
  );
}

export function TableLoadingRows({
  head,
  rows = 5,
}: {
  head: TableHeadCellProps[];
  rows?: number;
}) {
  const { t } = useTranslate('admin');

  return (
    <>
      {Array.from({ length: rows }).map((_, rowIndex) => (
        <TableRow key={rowIndex}>
          {head.map((cell) => (
            <TableCell key={cell.id || cell.label?.toString() || 'action'} sx={{ color: 'text.disabled' }}>
              {t('common.loading')}
            </TableCell>
          ))}
        </TableRow>
      ))}
    </>
  );
}

export function ManagementTableHead({ head }: { head: TableHeadCellProps[] }) {
  return <TableHeadCustom headCells={head} />;
}

export function TextFieldRow({
  label,
  value,
  onChange,
  required,
  type,
  select,
  children,
  helperText,
  disabled,
}: {
  label: string;
  value: string | number;
  onChange: (value: string) => void;
  required?: boolean;
  type?: React.InputHTMLAttributes<unknown>['type'];
  select?: boolean;
  children?: React.ReactNode;
  helperText?: React.ReactNode;
  disabled?: boolean;
}) {
  return (
    <TextField
      fullWidth
      select={select}
      required={required}
      type={type}
      label={label}
      value={value}
      disabled={disabled}
      helperText={helperText}
      onChange={(event) => onChange(event.target.value)}
    >
      {children}
    </TextField>
  );
}

export function SwitchRow({
  checked,
  label,
  onChange,
}: {
  checked: boolean;
  label: string;
  onChange: (checked: boolean) => void;
}) {
  return (
    <FormControlLabel
      control={<Checkbox checked={checked} onChange={(event) => onChange(event.target.checked)} />}
      label={label}
    />
  );
}

export function ManagementDialog({
  open,
  title,
  children,
  submitting,
  onClose,
  onSubmit,
}: {
  open: boolean;
  title: string;
  children: React.ReactNode;
  submitting: boolean;
  onClose: () => void;
  onSubmit: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Dialog fullWidth maxWidth="md" open={open} onClose={onClose}>
      <DialogTitle>{title}</DialogTitle>
      <DialogContent>
        <Stack sx={{ pt: 1, gap: 2.5 }}>{children}</Stack>
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button variant="contained" loading={submitting} onClick={onSubmit}>
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

export function SelectOption({ value, label }: { value: string; label: string }) {
  return <MenuItem value={value}>{label}</MenuItem>;
}

export function translatedRoleName(role: Role, t: AdminT) {
  const keyByCode: Record<string, string> = {
    admin: 'roles.admin.name',
    user: 'roles.user.name',
  };

  const key = keyByCode[role.code];

  return key ? t(key) : role.name;
}

export function translatedRoleDescription(role: Role, t: AdminT) {
  const keyByCode: Record<string, string> = {
    admin: 'roles.admin.description',
    user: 'roles.user.description',
  };

  const key = keyByCode[role.code];

  return key ? t(key) : role.description;
}

export function translatedAuthSource(source: string, t: AdminT) {
  const key = `authSources.${source.toLowerCase()}`;
  const translated = t(key);

  return translated === key ? source : translated;
}

export function translatedApiName(api: ApiPermission, t: AdminT) {
  const translated = t(`apiPermissionNames.${api.code}`);

  return translated === `apiPermissionNames.${api.code}` ? api.name : translated;
}

export function translatedApiGroup(group: string, t: AdminT) {
  if (!group) {
    return '-';
  }

  const translated = t(`apiGroups.${group.toLowerCase()}`);

  return translated === `apiGroups.${group.toLowerCase()}` ? group : translated;
}
