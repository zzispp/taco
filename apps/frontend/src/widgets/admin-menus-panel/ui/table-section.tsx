import type { MenuManagementController } from './controller';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import TableBody from '@mui/material/TableBody';

import { TableNoData } from 'src/shared/ui/table';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { ManagementTableHead } from 'src/shared/ui/admin';

import { MenuRow } from './row';
import { toggle } from './helpers';
import { MenuFilters } from './filters';

export function MenuTableSection({ resources, dialogs, actions }: MenuManagementController) {
  const { t, menus, expanded, setExpanded, treeRows, head } = resources;

  return (
    <Card>
      <MenuFilters
        filters={resources.filters}
        error={resources.filterError}
        onChange={resources.setFilters}
      />
      <MenuSortToolbar controller={{ resources, dialogs, actions }} />
      <Scrollbar>
        <Table sx={{ minWidth: 1240 }}>
          <ManagementTableHead head={head} />
          <TableBody>
            {treeRows.map((row) => (
              <MenuRow
                key={row.menu.menu_id}
                row={row}
                expanded={expanded.includes(row.menu.menu_id)}
                onToggle={() => setExpanded(toggle(expanded, row.menu.menu_id))}
                onEdit={actions.openEdit}
                onDelete={dialogs.setDeleteTarget}
                onCreateChild={actions.openCreateChild}
                orderValue={dialogs.sortEdits[row.menu.menu_id] ?? row.menu.order_num}
                onSort={(orderNum) =>
                  dialogs.setSortEdits((current) => ({ ...current, [row.menu.menu_id]: orderNum }))
                }
              />
            ))}
            <TableNoData
              title={t('common.noData')}
              notFound={!menus.isLoading && treeRows.length === 0}
            />
          </TableBody>
        </Table>
      </Scrollbar>
    </Card>
  );
}

type MenuSortToolbarProps = { controller: MenuManagementController };

function MenuSortToolbar({ controller }: MenuSortToolbarProps) {
  const { resources, dialogs, actions } = controller;
  const { t } = resources;
  const hasSortEdits = Object.keys(dialogs.sortEdits).length > 0;

  return (
    <Stack direction="row" spacing={1} sx={{ px: 2, pb: 2 }}>
      <Button size="small" onClick={() => resources.setExpanded(resources.allIds)}>
        {t('actions.expandAll')}
      </Button>
      <Button size="small" onClick={() => resources.setExpanded([])}>
        {t('actions.collapseAll')}
      </Button>
      <Button
        size="small"
        variant="contained"
        disabled={!hasSortEdits || dialogs.submitting}
        onClick={actions.saveSorts}
      >
        {t('actions.saveSort')}
      </Button>
    </Stack>
  );
}
