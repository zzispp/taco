import type { SchedulerJobLog } from 'src/entities/scheduler';
import type { JobLogController } from 'src/features/scheduler-management';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';

import { Label } from 'src/shared/ui/label';
import { Iconify } from 'src/shared/ui/iconify';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { useTranslate } from 'src/shared/i18n/use-locales';

import {
  JOB_LOG_STATUS,
  jobLogStatusTranslationKeys,
  schedulerTriggerTranslationKeys,
} from 'src/entities/scheduler';

const STATUS_COLORS = {
  [JOB_LOG_STATUS.SUCCESS]: 'success',
  [JOB_LOG_STATUS.FAILED]: 'error',
  [JOB_LOG_STATUS.SKIPPED]: 'warning',
  [JOB_LOG_STATUS.INTERRUPTED]: 'info',
} as const;

const EXECUTION_ID_COLUMN_WIDTH = 320;
const ACTION_COLUMN_WIDTH = 120;

export function JobLogRow(props: { log: SchedulerJobLog; controller: JobLogController }) {
  const { t } = useTranslate('scheduler');
  const { log, controller } = props;
  const { state, resources } = controller;
  return (
    <TableRow hover>
      <TableCell padding="checkbox">
        <Checkbox
          checked={state.table.selected.includes(log.execution_id)}
          disabled={!resources.canRemove}
          onChange={() => state.table.onSelectRow(log.execution_id)}
        />
      </TableCell>
      <TableCell sx={{ minWidth: 180, whiteSpace: 'nowrap' }}>{log.job_name}</TableCell>
      <ExecutionIdCell executionId={log.execution_id} controller={controller} />
      <TableCell>{log.job_group}</TableCell>
      <TableCell>{t(schedulerTriggerTranslationKeys[log.trigger_type])}</TableCell>
      <LogMessageCell log={log} />
      <TableCell>
        <Label color={STATUS_COLORS[log.status]} variant="soft">
          {t(jobLogStatusTranslationKeys[log.status])}
        </Label>
      </TableCell>
      <TableCell>{fAdminDateTime(log.scheduled_at)}</TableCell>
      <TableCell>{log.start_time ? fAdminDateTime(log.start_time) : t('notStarted')}</TableCell>
      <TableCell>{fAdminDateTime(log.end_time)}</TableCell>
      <LogActions controller={controller} log={log} />
    </TableRow>
  );
}

function ExecutionIdCell(props: { executionId: string; controller: JobLogController }) {
  const { t } = useTranslate('scheduler');
  return (
    <TableCell
      sx={{
        width: EXECUTION_ID_COLUMN_WIDTH,
        minWidth: EXECUTION_ID_COLUMN_WIDTH,
        maxWidth: EXECUTION_ID_COLUMN_WIDTH,
      }}
    >
      <ExecutionIdContent {...props} copyLabel={t('copyExecutionId')} />
    </TableCell>
  );
}

function ExecutionIdContent(props: {
  executionId: string;
  copyLabel: string;
  controller: JobLogController;
}) {
  return (
    <Box
      sx={{
        display: 'flex',
        alignItems: 'center',
        minWidth: 0,
        '& .execution-copy-action': { opacity: 0 },
        '&:hover .execution-copy-action, &:focus-within .execution-copy-action': { opacity: 1 },
      }}
    >
      <Tooltip title={props.executionId}>
        <Box
          component="span"
          sx={{
            minWidth: 0,
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
            fontFamily: 'monospace',
          }}
        >
          {props.executionId}
        </Box>
      </Tooltip>
      <Tooltip title={props.copyLabel}>
        <IconButton
          className="execution-copy-action"
          size="small"
          aria-label={props.copyLabel}
          sx={{
            ml: 0.5,
            flex: '0 0 auto',
            transition: (theme) => theme.transitions.create('opacity'),
          }}
          onClick={() => props.controller.actions.copyExecutionId(props.executionId)}
        >
          <Iconify icon="solar:copy-bold" />
        </IconButton>
      </Tooltip>
    </Box>
  );
}

function LogActions(props: { log: SchedulerJobLog; controller: JobLogController }) {
  const { t } = useTranslate('scheduler');
  const { t: tAdmin } = useTranslate('admin');
  const { log, controller } = props;
  const { state, resources, actions, pending } = controller;
  const detailLabel = log.has_detail ? t('executionDetail.open') : t('executionDetail.openLegacy');
  return (
    <TableCell
      align="right"
      sx={{ minWidth: ACTION_COLUMN_WIDTH, width: ACTION_COLUMN_WIDTH, whiteSpace: 'nowrap' }}
    >
      <Stack direction="row" spacing={0.5} flexWrap="nowrap" justifyContent="flex-end">
        {resources.canViewDetail && (
          <Tooltip title={detailLabel}>
            <IconButton
              aria-label={detailLabel}
              color={log.has_detail ? 'primary' : 'default'}
              onClick={() => actions.openDetail(log)}
            >
              <Iconify icon="solar:eye-bold" />
            </IconButton>
          </Tooltip>
        )}
        <Tooltip title={tAdmin('common.delete')}>
          <span>
            <IconButton
              aria-label={tAdmin('common.delete')}
              color="error"
              disabled={!resources.canRemove || pending.has(`delete:${log.execution_id}`)}
              onClick={() => state.setDeleteTarget(log)}
            >
              <Iconify icon="solar:trash-bin-trash-bold" />
            </IconButton>
          </span>
        </Tooltip>
      </Stack>
    </TableCell>
  );
}

function LogMessageCell({ log }: { log: SchedulerJobLog }) {
  const message = (
    <TableCell sx={{ maxWidth: 280, overflow: 'hidden', textOverflow: 'ellipsis' }}>
      {log.job_message}
    </TableCell>
  );
  return log.exception_info ? <Tooltip title={log.exception_info}>{message}</Tooltip> : message;
}
