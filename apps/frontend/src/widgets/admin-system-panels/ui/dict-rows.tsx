import type { DictData, DictType } from 'src/entities/system';

import Box from '@mui/material/Box';
import Tooltip from '@mui/material/Tooltip';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';

import { Label } from 'src/shared/ui/label';
import { Iconify } from 'src/shared/ui/iconify';
import { StatusLabel } from 'src/shared/ui/admin';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useHasPermission } from 'src/entities/session';

import { labelColor, DATE_TIME_CELL_SX } from './dict-helpers';

export function DictTypeRow({
  row,
  selected,
  checked,
  canRemove,
  onCheck,
  onSelect,
  onEdit,
  onDelete,
}: {
  row: DictType;
  selected: boolean;
  checked: boolean;
  canRemove: boolean;
  onCheck: (id: string) => void;
  onSelect: (row: DictType) => void;
  onEdit: (row: DictType) => void;
  onDelete: (row: DictType) => void;
}) {
  const canEdit = useHasPermission('system:dict:edit');
  const canDelete = useHasPermission('system:dict:remove');
  return (
    <TableRow hover selected={selected} onClick={() => onSelect(row)}>
      {canRemove && (
        <TableCell padding="checkbox">
          <Checkbox
            checked={checked}
            onClick={(event) => event.stopPropagation()}
            onChange={() => onCheck(row.dict_id)}
          />
        </TableCell>
      )}
      <TableCell>{row.dict_name}</TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.dict_type}</TableCell>
      <TableCell>
        <StatusLabel status={row.status} />
      </TableCell>
      <TableCell>{row.remark || '-'}</TableCell>
      <TableCell sx={DATE_TIME_CELL_SX}>{fAdminDateTime(row.create_time) || '-'}</TableCell>
      <TableCell align="right">
        <RowActions
          editDisabled={!canEdit}
          deleteDisabled={!canDelete}
          onEdit={() => onEdit(row)}
          onDelete={() => onDelete(row)}
        />
      </TableCell>
    </TableRow>
  );
}

export function DictDataRow({
  row,
  selected,
  canRemove,
  onCheck,
  onEdit,
  onDelete,
}: {
  row: DictData;
  selected: boolean;
  canRemove: boolean;
  onCheck: (id: string) => void;
  onEdit: (row: DictData) => void;
  onDelete: (row: DictData) => void;
}) {
  const canEdit = useHasPermission('system:dict:edit');
  const canDelete = useHasPermission('system:dict:remove');
  return (
    <TableRow hover>
      {canRemove && (
        <TableCell padding="checkbox">
          <Checkbox checked={selected} onChange={() => onCheck(row.dict_code)} />
        </TableCell>
      )}
      <TableCell>{row.dict_sort}</TableCell>
      <TableCell>
        <DictLabel value={row.dict_label} listClass={row.list_class} />
      </TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.dict_value}</TableCell>
      <TableCell>{row.is_default}</TableCell>
      <TableCell>
        <StatusLabel status={row.status} />
      </TableCell>
      <TableCell>{row.remark || '-'}</TableCell>
      <TableCell sx={DATE_TIME_CELL_SX}>{fAdminDateTime(row.create_time) || '-'}</TableCell>
      <TableCell align="right">
        <RowActions
          editDisabled={!canEdit}
          deleteDisabled={!canDelete}
          onEdit={() => onEdit(row)}
          onDelete={() => onDelete(row)}
        />
      </TableCell>
    </TableRow>
  );
}

function DictLabel({ value, listClass }: { value: string; listClass: string | null }) {
  return (
    <Label color={labelColor(listClass)} variant="soft">
      {value}
    </Label>
  );
}

function RowActions({
  onEdit,
  onDelete,
  editDisabled,
  deleteDisabled,
}: {
  onEdit: () => void;
  onDelete: () => void;
  editDisabled: boolean;
  deleteDisabled: boolean;
}) {
  const { t } = useTranslate('admin');
  return (
    <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
      <Tooltip title={t('common.edit')}>
        <span>
          <IconButton
            disabled={editDisabled}
            onClick={(event) => {
              event.stopPropagation();
              onEdit();
            }}
          >
            <Iconify icon="solar:pen-bold" />
          </IconButton>
        </span>
      </Tooltip>
      <Tooltip title={t('common.delete')}>
        <span>
          <IconButton
            color="error"
            disabled={deleteDisabled}
            onClick={(event) => {
              event.stopPropagation();
              onDelete();
            }}
          >
            <Iconify icon="solar:trash-bin-trash-bold" />
          </IconButton>
        </span>
      </Tooltip>
    </Box>
  );
}
