import type { Role } from 'src/entities/role';

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

import { translatedRoleName } from 'src/entities/role';
import { useHasPermission } from 'src/entities/session';

import { BooleanLabel } from 'src/widgets/admin-common';

import { dataScopeLabel } from './role-dialog';

const DATE_TIME_CELL_SX = { whiteSpace: 'nowrap' } as const;

type RoleRowProps = {
  row: Role;
  selected: boolean;
  onToggleSelected: (id: string) => void;
  onEdit: (role: Role) => void;
  onDelete: (role: Role) => void;
  onBind: (role: Role, type: 'menus' | 'depts') => void;
  onUsers: (role: Role) => void;
  onStatusChange: (status: string) => void;
};

export function RoleRow({
  row,
  selected,
  onToggleSelected,
  onEdit,
  onDelete,
  onBind,
  onUsers,
  onStatusChange,
}: RoleRowProps) {
  const { t } = useTranslate('admin');
  const canEdit = useHasPermission('system:role:edit');
  const canDelete = useHasPermission('system:role:remove');
  return (
    <TableRow hover>
      {canDelete && (
        <TableCell padding="checkbox">
          <Checkbox
            disabled={row.system}
            checked={selected}
            onChange={() => onToggleSelected(row.role_id)}
          />
        </TableCell>
      )}
      <TableCell>{translatedRoleName(row, t)}</TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.role_key}</TableCell>
      <TableCell>{row.role_sort}</TableCell>
      <TableCell>{dataScopeLabel(row.data_scope, t)}</TableCell>
      <TableCell>
        <Switch
          size="small"
          checked={row.status === '0'}
          disabled={row.system || !canEdit}
          onChange={(event) => onStatusChange(event.target.checked ? '0' : '1')}
        />
      </TableCell>
      <TableCell>
        <BooleanLabel
          enabled={row.system}
          trueText={t('common.system')}
          falseText={t('common.custom')}
        />
      </TableCell>
      <TableCell sx={DATE_TIME_CELL_SX}>{fAdminDateTime(row.create_time) || '-'}</TableCell>
      <RoleActionsCell {...{ row, canEdit, canDelete, onBind, onUsers, onEdit, onDelete }} />
    </TableRow>
  );
}

type RoleActionsCellProps = Pick<
  RoleRowProps,
  'row' | 'onBind' | 'onUsers' | 'onEdit' | 'onDelete'
> & {
  canEdit: boolean;
  canDelete: boolean;
};

function RoleActionsCell({
  row,
  canEdit,
  canDelete,
  onBind,
  onUsers,
  onEdit,
  onDelete,
}: RoleActionsCellProps) {
  return (
    <TableCell align="right">
      <RoleActions
        system={row.system}
        canEdit={canEdit}
        canDelete={canDelete}
        onMenu={() => onBind(row, 'menus')}
        onDept={() => onBind(row, 'depts')}
        onUsers={() => onUsers(row)}
        onEdit={() => onEdit(row)}
        onDelete={() => onDelete(row)}
      />
    </TableCell>
  );
}

type RoleActionsProps = {
  system: boolean;
  canEdit: boolean;
  canDelete: boolean;
  onMenu: () => void;
  onDept: () => void;
  onUsers: () => void;
  onEdit: () => void;
  onDelete: () => void;
};

function RoleActions({
  system,
  canEdit,
  canDelete,
  onMenu,
  onDept,
  onUsers,
  onEdit,
  onDelete,
}: RoleActionsProps) {
  const { t } = useTranslate('admin');
  return (
    <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
      <Tooltip title={t('actions.menuPermissions')}>
        <IconButton disabled={!canEdit} onClick={onMenu}>
          <Iconify icon="solar:shield-keyhole-bold-duotone" />
        </IconButton>
      </Tooltip>
      <Tooltip title={t('actions.dataPermissions')}>
        <IconButton disabled={!canEdit} onClick={onDept}>
          <Iconify icon="solar:notes-bold-duotone" />
        </IconButton>
      </Tooltip>
      <Tooltip title={t('actions.authorizedUsers')}>
        <IconButton disabled={!canEdit} onClick={onUsers}>
          <Iconify icon="solar:user-id-bold" />
        </IconButton>
      </Tooltip>
      <Tooltip title={t('common.edit')}>
        <span>
          <IconButton disabled={system || !canEdit} onClick={onEdit}>
            <Iconify icon="solar:pen-bold" />
          </IconButton>
        </span>
      </Tooltip>
      <Tooltip title={t('common.delete')}>
        <span>
          <IconButton color="error" disabled={system || !canDelete} onClick={onDelete}>
            <Iconify icon="solar:trash-bin-trash-bold" />
          </IconButton>
        </span>
      </Tooltip>
    </Box>
  );
}
