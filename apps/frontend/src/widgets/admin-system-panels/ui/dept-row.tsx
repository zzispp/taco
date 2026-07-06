import type { Dept } from 'src/entities/system';
import type { DeptRowView } from './dept-helpers';
import type { IconifyProps } from 'src/shared/ui/iconify';

import Box from '@mui/material/Box';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TextField from '@mui/material/TextField';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';

import { Iconify } from 'src/shared/ui/iconify';
import { StatusLabel } from 'src/shared/ui/admin';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { fAdminDateTime } from 'src/shared/lib/admin-time';

import { useHasPermission } from 'src/entities/session';

import { DATE_TIME_CELL_SX } from './dept-helpers';

export function DeptRow({
  row,
  expanded,
  orderValue,
  onToggle,
  onSort,
  onCreateChild,
  onEdit,
  onDelete,
}: {
  row: DeptRowView;
  expanded: boolean;
  orderValue: number;
  onToggle: () => void;
  onSort: (value: number) => void;
  onCreateChild: (dept: Dept) => void;
  onEdit: (dept: Dept) => void;
  onDelete: (dept: Dept) => void;
}) {
  const { t } = useTranslate('admin');
  const canAdd = useHasPermission('system:dept:add');
  const canEdit = useHasPermission('system:dept:edit');
  const canDelete = useHasPermission('system:dept:remove');
  return (
    <TableRow hover>
      <TableCell>
        <Box sx={{ display: 'flex', alignItems: 'center', pl: row.level * 2 }}>
          {row.childCount > 0 ? (
            <IconButton size="small" onClick={onToggle}>
              <Iconify
                icon={expanded ? 'eva:arrow-ios-downward-fill' : 'eva:arrow-ios-forward-fill'}
              />
            </IconButton>
          ) : (
            <Box sx={{ width: 34 }} />
          )}
          {row.dept.dept_name}
        </Box>
      </TableCell>
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
      <TableCell>{row.dept.leader || '-'}</TableCell>
      <TableCell>{row.dept.phone || '-'}</TableCell>
      <TableCell>{row.dept.email || '-'}</TableCell>
      <TableCell>
        <StatusLabel status={row.dept.status} />
      </TableCell>
      <TableCell sx={DATE_TIME_CELL_SX}>{fAdminDateTime(row.dept.create_time) || '-'}</TableCell>
      <TableCell align="right">
        <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
          <DeptAction
            title={t('common.add')}
            disabled={!canAdd}
            icon="mingcute:add-line"
            onClick={() => onCreateChild(row.dept)}
          />
          <DeptAction
            title={t('common.edit')}
            disabled={!canEdit}
            icon="solar:pen-bold"
            onClick={() => onEdit(row.dept)}
          />
          <DeptAction
            title={t('common.delete')}
            disabled={!canDelete}
            color="error"
            icon="solar:trash-bin-trash-bold"
            onClick={() => onDelete(row.dept)}
          />
        </Box>
      </TableCell>
    </TableRow>
  );
}

function DeptAction({
  title,
  disabled,
  icon,
  color,
  onClick,
}: {
  title: string;
  disabled: boolean;
  icon: IconifyProps['icon'];
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
