import type { LabelColor } from 'src/shared/ui/label';
import type { OperationLogController } from 'src/features/audit-log-management';
import type {
  AuditStatus,
  OperationLogSummary,
  OperationBusinessType,
} from 'src/entities/audit-log';

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
  AUDIT_STATUS,
  auditStatusKeys,
  OPERATION_BUSINESS_TYPE,
  operationBusinessTypeKeys,
} from 'src/entities/audit-log';

import { MethodLabel } from 'src/widgets/admin-common';

const ELLIPSIS = {
  maxWidth: 220,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
} as const;

const STATUS_COLORS: Record<AuditStatus, LabelColor> = {
  [AUDIT_STATUS.SUCCESS]: 'success',
  [AUDIT_STATUS.FAILURE]: 'error',
};

const BUSINESS_TYPE_COLORS: Record<OperationBusinessType, LabelColor> = {
  [OPERATION_BUSINESS_TYPE.OTHER]: 'default',
  [OPERATION_BUSINESS_TYPE.INSERT]: 'success',
  [OPERATION_BUSINESS_TYPE.UPDATE]: 'info',
  [OPERATION_BUSINESS_TYPE.DELETE]: 'error',
  [OPERATION_BUSINESS_TYPE.GRANT]: 'default',
  [OPERATION_BUSINESS_TYPE.EXPORT]: 'warning',
  [OPERATION_BUSINESS_TYPE.IMPORT]: 'warning',
  [OPERATION_BUSINESS_TYPE.FORCE]: 'error',
  [OPERATION_BUSINESS_TYPE.GENERATE]: 'default',
  [OPERATION_BUSINESS_TYPE.CLEAN]: 'error',
};

export function OperationLogRow({
  log,
  controller,
}: {
  log: OperationLogSummary;
  controller: OperationLogController;
}) {
  const { t } = useTranslate('audit');
  return (
    <TableRow hover>
      <OperationSelectionCell log={log} controller={controller} />
      <Tooltip title={log.oper_id}>
        <TableCell sx={ELLIPSIS}>{log.oper_id}</TableCell>
      </Tooltip>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{log.title}</TableCell>
      <TableCell>
        <Label variant="soft" color={BUSINESS_TYPE_COLORS[log.business_type]}>
          {t(operationBusinessTypeKeys[log.business_type])}
        </Label>
      </TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{log.oper_name || '-'}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{log.dept_name || '-'}</TableCell>
      <TableCell>
        <MethodLabel method={log.request_method} />
      </TableCell>
      <Tooltip title={log.oper_url}>
        <TableCell sx={ELLIPSIS}>{log.oper_url}</TableCell>
      </Tooltip>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{log.oper_ip}</TableCell>
      <TableCell sx={ELLIPSIS}>{log.oper_location}</TableCell>
      <TableCell>
        <Label variant="soft" color={STATUS_COLORS[log.status]}>
          {t(auditStatusKeys[log.status])}
        </Label>
      </TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{fAdminDateTime(log.oper_time)}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>
        {t('table.milliseconds', { value: log.cost_time })}
      </TableCell>
      <OperationRowActions log={log} controller={controller} />
    </TableRow>
  );
}

function OperationSelectionCell({
  log,
  controller,
}: {
  log: OperationLogSummary;
  controller: OperationLogController;
}) {
  const { t } = useTranslate('audit');
  const { state, resources } = controller;
  if (!resources.canRemove) return null;
  return (
    <TableCell padding="checkbox">
      <Checkbox
        aria-label={t('table.selectRow', { id: log.oper_id })}
        checked={state.table.selected.includes(log.oper_id)}
        onChange={() => state.table.onSelectRow(log.oper_id)}
      />
    </TableCell>
  );
}

function OperationRowActions({
  log,
  controller,
}: {
  log: OperationLogSummary;
  controller: OperationLogController;
}) {
  const { t } = useTranslate('audit');
  const { state, resources, pending, actions } = controller;
  return (
    <TableCell align="right" sx={{ minWidth: 110, whiteSpace: 'nowrap' }}>
      <Stack direction="row" spacing={0.5} justifyContent="flex-end">
        {resources.canQuery && (
          <Tooltip title={t('actions.detail')}>
            <IconButton
              color="primary"
              aria-label={t('actions.detail')}
              onClick={() => actions.openDetail(log)}
            >
              <Iconify icon="solar:eye-bold" />
            </IconButton>
          </Tooltip>
        )}
        {resources.canRemove && (
          <Tooltip title={t('actions.delete')}>
            <span>
              <IconButton
                color="error"
                aria-label={t('actions.delete')}
                disabled={pending.has(`delete:${log.oper_id}`)}
                onClick={() => state.setDeleteTarget(log)}
              >
                <Iconify icon="solar:trash-bin-trash-bold" />
              </IconButton>
            </span>
          </Tooltip>
        )}
      </Stack>
    </TableCell>
  );
}
