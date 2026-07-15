import type { FlatNode } from './helpers';
import type { Post } from 'src/entities/system';
import type { RoleOption } from 'src/entities/role';
import type { SystemUser } from 'src/entities/user';
import type { IconifyName } from 'src/shared/ui/iconify';

import Box from '@mui/material/Box';
import Switch from '@mui/material/Switch';
import Tooltip from '@mui/material/Tooltip';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { fAdminDateTime } from 'src/shared/lib/admin-time';

import { useHasPermission } from 'src/entities/session';

import {
  nameById,
  sexLabel,
  namesByIds,
  displayRoles,
  USER_CELL_SX,
  USER_ELLIPSIS_CELL_SX,
} from './helpers';

type UserRowProps = Readonly<{
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
}>;

export function UserRow({
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
}: UserRowProps) {
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
          <Checkbox checked={selected} onChange={() => onToggleSelected(row.user_id)} />
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
          disabled={!canEdit}
          onChange={(event) => onStatusChange(event.target.checked ? '0' : '1')}
        />
      </TableCell>
      <TableCell sx={USER_ELLIPSIS_CELL_SX}>{postNames}</TableCell>
      <TableCell sx={USER_ELLIPSIS_CELL_SX}>{roleNames}</TableCell>
      <TableCell sx={USER_CELL_SX}>{fAdminDateTime(row.create_time) || '-'}</TableCell>
      <UserActions
        {...{ row, canEdit, canDelete, canReset, onEdit, onDelete, onRoles, onResetPassword }}
      />
    </TableRow>
  );
}

type UserActionsProps = Pick<
  UserRowProps,
  'row' | 'onEdit' | 'onDelete' | 'onRoles' | 'onResetPassword'
> &
  Readonly<{ canEdit: boolean; canDelete: boolean; canReset: boolean }>;

function UserActions({
  row,
  canEdit,
  canDelete,
  canReset,
  onEdit,
  onDelete,
  onRoles,
  onResetPassword,
}: UserActionsProps) {
  const { t } = useTranslate('admin');
  return (
    <TableCell align="right" sx={USER_CELL_SX}>
      <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
        <ActionIcon
          title={t('common.edit')}
          disabled={!canEdit}
          icon="solar:pen-bold"
          onClick={() => onEdit(row)}
        />
        <ActionIcon
          title={t('actions.resetPassword')}
          disabled={!canReset}
          icon="solar:restart-bold"
          onClick={() => onResetPassword(row)}
        />
        <ActionIcon
          title={t('actions.assignRoles')}
          disabled={!canEdit}
          icon="solar:user-id-bold"
          onClick={() => onRoles(row)}
        />
        <ActionIcon
          title={t('common.delete')}
          disabled={!canDelete}
          color="error"
          icon="solar:trash-bin-trash-bold"
          onClick={() => onDelete(row)}
        />
      </Box>
    </TableCell>
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
