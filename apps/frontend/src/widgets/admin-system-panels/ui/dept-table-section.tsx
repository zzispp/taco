import type { DeptManagementController } from './dept-controller';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import TableBody from '@mui/material/TableBody';

import { TableNoData } from 'src/shared/ui/table';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { ManagementTableHead } from 'src/shared/ui/admin';

import { DeptRow } from './dept-row';
import { toggle } from './dept-helpers';
import { DeptFilters } from './dept-filters';

export function DeptTableSection({ resources, state, actions }: DeptManagementController) {
  const { t, rows, resource } = resources;

  return (
    <Card>
      <DeptFilters filters={state.filters} onChange={state.setFilters} />
      <DeptSortToolbar resources={resources} state={state} actions={actions} />
      <Scrollbar>
        <Table sx={{ minWidth: 1080 }}>
          <ManagementTableHead head={resources.head} />
          <TableBody>
            {rows.map((row) => (
              <DeptRow
                key={row.dept.dept_id}
                row={row}
                expanded={state.expanded.includes(row.dept.dept_id)}
                orderValue={state.sortEdits[row.dept.dept_id] ?? row.dept.order_num}
                onToggle={() => state.setExpanded(toggle(state.expanded, row.dept.dept_id))}
                onSort={(orderNum) =>
                  state.setSortEdits((current) => ({ ...current, [row.dept.dept_id]: orderNum }))
                }
                onCreateChild={actions.openCreateChild}
                onEdit={actions.openEdit}
                onDelete={state.setDeleteTarget}
              />
            ))}
            <TableNoData title={t('common.noData')} notFound={!resource.isLoading && rows.length === 0} />
          </TableBody>
        </Table>
      </Scrollbar>
    </Card>
  );
}

type DeptSortToolbarProps = Pick<DeptManagementController, 'resources' | 'state' | 'actions'>;

function DeptSortToolbar({ resources, state, actions }: DeptSortToolbarProps) {
  const { t } = resources;
  const hasSortEdits = Object.keys(state.sortEdits).length > 0;

  return (
    <Stack direction="row" spacing={1} sx={{ px: 2, pb: 2 }}>
      <Button size="small" onClick={() => state.setExpanded(resources.allIds)}>
        {t('actions.expandAll')}
      </Button>
      <Button size="small" onClick={() => state.setExpanded([])}>
        {t('actions.collapseAll')}
      </Button>
      <Button size="small" variant="contained" disabled={!hasSortEdits || state.submitting} onClick={actions.saveSorts}>
        {t('actions.saveSort')}
      </Button>
    </Stack>
  );
}
