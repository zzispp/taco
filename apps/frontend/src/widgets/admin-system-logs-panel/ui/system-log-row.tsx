import type { LabelColor } from 'src/shared/ui/label';
import type { SystemLogController } from 'src/features/system-log-management';
import type { SystemLogLevel, SystemLogSummary } from 'src/entities/system-log';

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

const LEVEL_COLOR: Record<SystemLogLevel, LabelColor> = {
  trace: 'default',
  debug: 'info',
  info: 'success',
  warn: 'warning',
  error: 'error',
};
const ELLIPSIS = {
  maxWidth: 400,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
} as const;

export function SystemLogRow({
  log,
  controller,
}: {
  log: SystemLogSummary;
  controller: SystemLogController;
}) {
  return (
    <TableRow hover>
      <SystemLogSelectionCell log={log} controller={controller} />
      <SystemLogSummaryCells log={log} />
      <SystemLogActions log={log} controller={controller} />
    </TableRow>
  );
}

function SystemLogSelectionCell({
  log,
  controller,
}: {
  log: SystemLogSummary;
  controller: SystemLogController;
}) {
  const { t } = useTranslate('systemLog');
  if (!controller.resources.canRemove) return null;
  return (
    <TableCell padding="checkbox">
      <Checkbox
        aria-label={t('table.selectRow', { id: log.log_id })}
        checked={controller.state.table.selected.includes(log.log_id)}
        onChange={() => controller.state.table.onSelectRow(log.log_id)}
      />
    </TableCell>
  );
}

function SystemLogSummaryCells({ log }: { log: SystemLogSummary }) {
  const { t } = useTranslate('systemLog');
  return (
    <>
      <Tooltip title={log.log_id}>
        <TableCell sx={ELLIPSIS}>{log.log_id}</TableCell>
      </Tooltip>
      <TableCell>
        <Label variant="soft" color={LEVEL_COLOR[log.level]}>
          {t(`levels.${log.level}`)}
        </Label>
      </TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{log.target}</TableCell>
      <Tooltip title={log.message}>
        <TableCell sx={ELLIPSIS}>{log.message}</TableCell>
      </Tooltip>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{fAdminDateTime(log.occurred_at)}</TableCell>
    </>
  );
}

function SystemLogActions({
  log,
  controller,
}: {
  log: SystemLogSummary;
  controller: SystemLogController;
}) {
  const { t } = useTranslate('systemLog');
  const { resources, pending, actions } = controller;
  return (
    <TableCell align="right">
      <Stack direction="row" spacing={0.5} justifyContent="flex-end">
        {resources.canQuery && (
          <SystemLogAction
            title={t('actions.detail')}
            icon="solar:eye-bold"
            onClick={() => actions.openDetail(log)}
          />
        )}
        {resources.canRemove && (
          <SystemLogAction
            color="error"
            title={t('actions.delete')}
            icon="solar:trash-bin-trash-bold"
            disabled={pending.has(`delete:${log.log_id}`)}
            onClick={() => actions.requestDelete(log)}
          />
        )}
      </Stack>
    </TableCell>
  );
}

function SystemLogAction({
  title,
  icon,
  color,
  disabled,
  onClick,
}: {
  title: string;
  icon: 'solar:eye-bold' | 'solar:trash-bin-trash-bold';
  color?: 'error';
  disabled?: boolean;
  onClick: () => void;
}) {
  return (
    <Tooltip title={title}>
      <span>
        <IconButton color={color} aria-label={title} disabled={disabled} onClick={onClick}>
          <Iconify icon={icon} />
        </IconButton>
      </span>
    </Tooltip>
  );
}
