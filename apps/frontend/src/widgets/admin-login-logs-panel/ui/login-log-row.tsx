import type { LabelColor } from 'src/shared/ui/label';
import type { LoginLog, AuditStatus } from 'src/entities/audit-log';
import type { LoginLogController } from 'src/features/audit-log-management';

import Tooltip from '@mui/material/Tooltip';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';

import { Label } from 'src/shared/ui/label';
import { Iconify } from 'src/shared/ui/iconify';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { AUDIT_STATUS, auditStatusKeys, loginEventTypeKeys } from 'src/entities/audit-log';

const ELLIPSIS = {
  maxWidth: 240,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
} as const;

const STATUS_COLORS: Record<AuditStatus, LabelColor> = {
  [AUDIT_STATUS.SUCCESS]: 'success',
  [AUDIT_STATUS.FAILURE]: 'error',
};

export function LoginLogRow({
  log,
  controller,
}: {
  log: LoginLog;
  controller: LoginLogController;
}) {
  const { t } = useTranslate('audit');
  return (
    <TableRow hover>
      <LoginSelectionCell log={log} controller={controller} />
      <Tooltip title={log.info_id}>
        <TableCell sx={ELLIPSIS}>{log.info_id}</TableCell>
      </Tooltip>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{log.user_name}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{log.ipaddr}</TableCell>
      <TableCell sx={ELLIPSIS}>{log.login_location}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{log.browser}</TableCell>
      <TableCell sx={ELLIPSIS}>{log.os}</TableCell>
      <TableCell>
        <Label variant="soft" color={STATUS_COLORS[log.status]}>
          {t(auditStatusKeys[log.status])}
        </Label>
      </TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{t(loginEventTypeKeys[log.event_type])}</TableCell>
      <Tooltip title={log.msg}>
        <TableCell sx={ELLIPSIS}>{log.msg}</TableCell>
      </Tooltip>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{fAdminDateTime(log.login_time)}</TableCell>
      <LoginDeleteCell log={log} controller={controller} />
    </TableRow>
  );
}

function LoginSelectionCell({
  log,
  controller,
}: {
  log: LoginLog;
  controller: LoginLogController;
}) {
  const { t } = useTranslate('audit');
  const { state, resources } = controller;
  if (!resources.canRemove && !resources.canUnlock) return null;
  return (
    <TableCell padding="checkbox">
      <Checkbox
        aria-label={t('table.selectRow', { id: log.info_id })}
        checked={state.table.selected.includes(log.info_id)}
        onChange={() => state.table.onSelectRow(log.info_id)}
      />
    </TableCell>
  );
}

function LoginDeleteCell({ log, controller }: { log: LoginLog; controller: LoginLogController }) {
  const { t } = useTranslate('audit');
  const { state, resources, pending } = controller;
  return (
    <TableCell align="right" sx={{ width: 72, whiteSpace: 'nowrap' }}>
      {resources.canRemove && (
        <Tooltip title={t('actions.delete')}>
          <span>
            <IconButton
              color="error"
              aria-label={t('actions.delete')}
              disabled={pending.has(`delete:${log.info_id}`)}
              onClick={() => state.setDeleteTarget(log)}
            >
              <Iconify icon="solar:trash-bin-trash-bold" />
            </IconButton>
          </span>
        </Tooltip>
      )}
    </TableCell>
  );
}
