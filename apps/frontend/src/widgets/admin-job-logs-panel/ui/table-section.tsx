import type { JobLogController } from 'src/features/scheduler-management';

import Card from '@mui/material/Card';
import Alert from '@mui/material/Alert';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { TableNoData, CursorPagination } from 'src/shared/ui/table';

import { TableLoadingRows, withSelectionHead, ManagementTableHead } from 'src/widgets/admin-common';

import { JobLogRow } from './job-log-row';
import { JobLogFilters } from './filters';

const TABLE_MIN_WIDTH = 1850;

const HEAD = [
  { id: 'job_name', label: 'jobName', width: 180, sx: { minWidth: 180, whiteSpace: 'nowrap' } },
  { id: 'execution_id', label: 'executionId', width: 320, sx: { minWidth: 320 } },
  { id: 'job_group', label: 'jobGroup' },
  { id: 'trigger_type', label: 'triggerType', width: 120 },
  { id: 'job_message', label: 'jobMessage' },
  { id: 'status', label: 'admin:common.status', width: 110 },
  { id: 'scheduled_at', label: 'scheduledAt', width: 190 },
  { id: 'start_time', label: 'startTime', width: 190 },
  { id: 'end_time', label: 'endTime', width: 190 },
  {
    id: 'actions',
    label: 'admin:common.actions',
    align: 'right',
    width: 120,
    sx: { minWidth: 120, whiteSpace: 'nowrap' },
  },
] as const;

export function JobLogTableSection({ controller }: { controller: JobLogController }) {
  return (
    <Card>
      <JobLogFilters controller={controller} />
      <JobLogTableContent controller={controller} />
    </Card>
  );
}

function JobLogTableContent({ controller }: { controller: JobLogController }) {
  const { t } = useTranslate('scheduler');
  const { state, resources } = controller;
  const head = HEAD.map((cell) => ({ ...cell, label: t(cell.label) }));
  if (resources.logs.error)
    return (
      <Alert severity="error" sx={{ mx: 2, mb: 2 }}>
        {getErrorMessage(resources.logs.error)}
      </Alert>
    );
  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: TABLE_MIN_WIDTH }}>
          <ManagementTableHead
            head={head}
            rowCount={resources.logs.items.length}
            numSelected={state.table.selected.length}
            onSelectAllRows={(checked) =>
              state.table.onSelectAllRows(
                checked,
                resources.logs.items.map((log) => log.execution_id)
              )
            }
          />
          <TableBody>
            {resources.logs.isLoading ? (
              <TableLoadingRows head={withSelectionHead(head)} rows={state.table.limit} />
            ) : (
              resources.logs.items.map((log) => (
                <JobLogRow key={log.execution_id} log={log} controller={controller} />
              ))
            )}
            <JobLogEmptyState controller={controller} head={head} />
          </TableBody>
        </Table>
      </Scrollbar>
      <JobLogCursorNavigation controller={controller} />
    </>
  );
}

function JobLogCursorNavigation({ controller }: { controller: JobLogController }) {
  const { state, resources } = controller;
  return (
    <CursorPagination
      limit={state.table.limit}
      itemCount={resources.logs.itemCount}
      visitedBatchIndex={state.table.visitedBatchIndex}
      hasPrevious={resources.logs.hasPrevious}
      hasNext={resources.logs.hasNext}
      pending={resources.logs.isValidating}
      onPrevious={() => state.table.onPreviousCursor(resources.logs.previousCursor)}
      onNext={() => state.table.onNextCursor(resources.logs.nextCursor)}
      onLimitChange={state.table.onChangeLimit}
    />
  );
}

function JobLogEmptyState({
  controller,
  head,
}: {
  controller: JobLogController;
  head: ReturnType<typeof withSelectionHead>;
}) {
  const { t } = useTranslate('scheduler');
  const { logs } = controller.resources;
  return (
    <TableNoData
      colSpan={withSelectionHead(head).length}
      title={t('admin:common.noData')}
      notFound={!logs.isLoading && logs.items.length === 0}
    />
  );
}
