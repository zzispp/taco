import type { IconifyName } from 'src/shared/ui/iconify';
import type { SchedulerJob } from 'src/entities/scheduler';
import type { SchedulerController } from 'src/features/scheduler-management';

import Box from '@mui/material/Box';
import Switch from '@mui/material/Switch';
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
  JOB_STATUS,
  runtimeErrorTranslationKeys,
  registryStatusTranslationKeys,
} from 'src/entities/scheduler';

import { schedulerJobCapabilities } from 'src/features/scheduler-management';

const JOB_NAME_CELL_SX = { minWidth: 180, whiteSpace: 'nowrap' } as const;

export function SchedulerJobRow(props: { job: SchedulerJob; controller: SchedulerController }) {
  const { job, controller } = props;
  return (
    <TableRow hover>
      <SelectionCell job={job} controller={controller} />
      <TableCell sx={JOB_NAME_CELL_SX}>{job.job_name}</TableCell>
      <TableCell>{job.job_group}</TableCell>
      <TableCell sx={{ maxWidth: 280, overflow: 'hidden', textOverflow: 'ellipsis' }}>
        {job.invoke_target}
      </TableCell>
      <TableCell>{job.cron_expression}</TableCell>
      <JobStatusCell job={job} controller={controller} />
      <RegistryStatusCell job={job} />
      <TableCell>
        <RuntimeErrorCell job={job} />
      </TableCell>
      <TableCell>{fAdminDateTime(job.create_time)}</TableCell>
      <JobActions job={job} controller={controller} />
    </TableRow>
  );
}

type JobCellProps = { job: SchedulerJob; controller: SchedulerController };

function SelectionCell({ job, controller }: JobCellProps) {
  const { canDelete } = schedulerJobCapabilities(job.registry_status, job.capabilities);
  return (
    <TableCell padding="checkbox">
      <Checkbox
        checked={controller.state.table.selected.includes(job.job_id)}
        disabled={!controller.resources.canRemove || !canDelete}
        onChange={() => controller.state.table.onSelectRow(job.job_id)}
      />
    </TableCell>
  );
}

function JobStatusCell({ job, controller }: JobCellProps) {
  const { canDisable, runnable } = schedulerJobCapabilities(job.registry_status, job.capabilities);
  return (
    <TableCell>
      <Switch
        size="small"
        checked={job.status === JOB_STATUS.NORMAL}
        disabled={
          !controller.resources.canStatus ||
          !runnable ||
          !canDisable ||
          controller.pending.has(`status:${job.job_id}`)
        }
        onChange={() => controller.actions.updateStatus(job)}
      />
    </TableCell>
  );
}

function RegistryStatusCell({ job }: { job: SchedulerJob }) {
  const { t } = useTranslate('scheduler');
  const { runnable } = schedulerJobCapabilities(job.registry_status, job.capabilities);
  return (
    <TableCell>
      <Label color={runnable ? 'success' : 'warning'} variant="soft">
        {t(registryStatusTranslationKeys[job.registry_status])}
      </Label>
    </TableCell>
  );
}

function JobActions({ job, controller }: JobCellProps) {
  const { t } = useTranslate('scheduler');
  const { t: tAdmin } = useTranslate('admin');
  const { canDelete, editable, runnable } = schedulerJobCapabilities(
    job.registry_status,
    job.capabilities
  );
  return (
    <TableCell align="right" sx={{ width: 208, minWidth: 208, whiteSpace: 'nowrap' }}>
      <Box sx={{ display: 'flex', flexWrap: 'nowrap', justifyContent: 'flex-end' }}>
        {controller.resources.canViewDetail && (
          <Action
            title={t('jobDetail.open')}
            icon="solar:eye-bold"
            onClick={() => controller.actions.openDetail(job)}
          />
        )}
        <Action
          title={t('runOnce')}
          icon="solar:play-circle-bold"
          disabled={
            !controller.resources.canRun || !runnable || controller.pending.has(`run:${job.job_id}`)
          }
          onClick={() => controller.actions.run(job)}
        />
        <Action
          title={tAdmin('common.edit')}
          icon="solar:pen-bold"
          disabled={!controller.resources.canEdit || !editable}
          onClick={() => controller.state.setEditing(job)}
        />
        <Action
          title={tAdmin('common.delete')}
          icon="solar:trash-bin-trash-bold"
          color="error"
          disabled={
            !controller.resources.canRemove ||
            !canDelete ||
            controller.pending.has(`delete:${job.job_id}`)
          }
          onClick={() => controller.state.setDeleteTarget(job)}
        />
      </Box>
    </TableCell>
  );
}

function RuntimeErrorCell({ job }: { job: SchedulerJob }) {
  const { t } = useTranslate('scheduler');
  if (!job.runtime_error) return <>—</>;
  return (
    <Tooltip title={job.runtime_error.message}>
      <Label color="error" variant="soft">
        {t(runtimeErrorTranslationKeys[job.runtime_error.code])}
      </Label>
    </Tooltip>
  );
}

type ActionProps = {
  title: string;
  icon: IconifyName;
  color?: 'error';
  disabled?: boolean;
  onClick: () => void;
};

function Action({ title, icon, color, disabled, onClick }: ActionProps) {
  return (
    <Tooltip title={title}>
      <span>
        <IconButton aria-label={title} color={color} disabled={disabled} onClick={onClick}>
          <Iconify icon={icon} />
        </IconButton>
      </span>
    </Tooltip>
  );
}
