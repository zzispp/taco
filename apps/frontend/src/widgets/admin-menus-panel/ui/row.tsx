import type { MenuRowView } from './helpers';
import type { Menu } from 'src/entities/menu';

import Box from '@mui/material/Box';
import TableRow from '@mui/material/TableRow';
import TextField from '@mui/material/TextField';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { translatedMenuItem } from 'src/entities/menu';
import { useHasPermission } from 'src/entities/session';

import { StatusLabel } from 'src/widgets/admin-common';

import { MenuActions } from './actions';
import { menuIcon, menuTypeLabel } from './helpers';

type MenuRowProps = {
  row: MenuRowView;
  expanded: boolean;
  orderValue: number;
  onToggle: () => void;
  onEdit: (menu: Menu) => void;
  onDelete: (menu: Menu) => void;
  onCreateChild: (menu: Menu) => void;
  onSort: (orderNum: number) => void;
};

export function MenuRow(props: MenuRowProps) {
  const { row, expanded, orderValue, onToggle, onEdit, onDelete, onCreateChild, onSort } = props;
  const { t } = useTranslate('admin');
  const canAdd = useHasPermission('system:menu:add');
  const canEdit = useHasPermission('system:menu:edit');
  const canDelete = useHasPermission('system:menu:remove');
  const hasChildren = row.childCount > 0;

  return (
    <TableRow hover>
      <TableCell>{renderNameCell({ row, expanded, hasChildren, onToggle, t })}</TableCell>
      <TableCell>{menuTypeLabel(row.menu.menu_type, t)}</TableCell>
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
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.menu.path || '-'}</TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.menu.component || '-'}</TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.menu.perms || '-'}</TableCell>
      <TableCell>{row.menu.visible === '0' ? t('common.show') : t('common.hide')}</TableCell>
      <TableCell>
        <StatusLabel status={row.menu.status} />
      </TableCell>
      <TableCell align="right">
        <MenuActions
          canAdd={canAdd && row.menu.menu_type !== 'F'}
          canEdit={canEdit}
          canDelete={canDelete}
          onCreateChild={() => onCreateChild(row.menu)}
          onEdit={() => onEdit(row.menu)}
          onDelete={() => onDelete(row.menu)}
        />
      </TableCell>
    </TableRow>
  );
}

type NameCellOptions = {
  row: MenuRowView;
  expanded: boolean;
  hasChildren: boolean;
  onToggle: () => void;
  t: ReturnType<typeof useTranslate>['t'];
};

function renderNameCell({ row, expanded, hasChildren, onToggle, t }: NameCellOptions) {
  return (
    <Box sx={{ display: 'flex', alignItems: 'center', pl: row.level * 2 }}>
      {hasChildren ? (
        <IconButton size="small" onClick={onToggle}>
          <Iconify icon={expanded ? 'eva:arrow-ios-downward-fill' : 'eva:arrow-ios-forward-fill'} />
        </IconButton>
      ) : (
        <Box sx={{ width: 34 }} />
      )}
      <Iconify icon={menuIcon(row.menu.icon)} width={18} sx={{ mr: 1 }} />
      {translatedMenuItem(row.menu, t)}
    </Box>
  );
}
