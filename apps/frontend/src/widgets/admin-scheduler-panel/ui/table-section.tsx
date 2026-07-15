import type { SchedulerController } from 'src/features/scheduler-management';

import Card from '@mui/material/Card';
import Alert from '@mui/material/Alert';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { TableNoData, CursorPagination } from 'src/shared/ui/table';

import { TableLoadingRows, withSelectionHead, ManagementTableHead } from 'src/widgets/admin-common';

import { SchedulerJobRow } from './job-row';

const HEAD = [
  { id: 'job_name', label: 'jobName', width: 180, sx: { minWidth: 180, whiteSpace: 'nowrap' } },
  { id: 'job_group', label: 'jobGroup' },
  { id: 'invoke_target', label: 'invokeTarget' },
  { id: 'cron_expression', label: 'cronExpression' },
  { id: 'status', label: 'admin:common.status', width: 110 },
  { id: 'registry_status', label: 'registryStatus', width: 150 },
  { id: 'runtime_error', label: 'runtimeError', width: 150 },
  { id: 'create_time', label: 'admin:fields.createTime', width: 190 },
  { id: 'actions', label: 'admin:common.actions', align: 'right', width: 208 },
] as const;

export function SchedulerTableSection({ controller }: { controller: SchedulerController }) {
  const { t } = useTranslate('scheduler');
  const { state, resources } = controller;
  const head = HEAD.map((cell) => ({ ...cell, label: t(cell.label) }));
  if (resources.jobs.error)
    return <Alert severity="error">{getErrorMessage(resources.jobs.error)}</Alert>;
  return (
    <Card>
      <Scrollbar>
        <Table sx={{ minWidth: 1300 }}>
          <ManagementTableHead
            head={head}
            rowCount={resources.jobs.items.length}
            numSelected={state.table.selected.length}
            onSelectAllRows={(checked) =>
              state.table.onSelectAllRows(
                checked,
                resources.jobs.items.map((job) => job.job_id)
              )
            }
          />
          <TableBody>
            {resources.jobs.isLoading ? (
              <TableLoadingRows head={withSelectionHead(head)} rows={state.table.limit} />
            ) : (
              resources.jobs.items.map((job) => (
                <SchedulerJobRow key={job.job_id} job={job} controller={controller} />
              ))
            )}
            <TableNoData
              colSpan={withSelectionHead(head).length}
              title={t('admin:common.noData')}
              notFound={!resources.jobs.isLoading && resources.jobs.items.length === 0}
            />
          </TableBody>
        </Table>
      </Scrollbar>
      <SchedulerCursorNavigation controller={controller} />
    </Card>
  );
}

function SchedulerCursorNavigation({ controller }: { controller: SchedulerController }) {
  const { state, resources } = controller;
  return (
    <CursorPagination
      limit={state.table.limit}
      itemCount={resources.jobs.itemCount}
      visitedBatchIndex={state.table.visitedBatchIndex}
      hasPrevious={resources.jobs.hasPrevious}
      hasNext={resources.jobs.hasNext}
      pending={resources.jobs.isValidating}
      onPrevious={() => state.table.onPreviousCursor(resources.jobs.previousCursor)}
      onNext={() => state.table.onNextCursor(resources.jobs.nextCursor)}
      onLimitChange={state.table.onChangeLimit}
    />
  );
}
