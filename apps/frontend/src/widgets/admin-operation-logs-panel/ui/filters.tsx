import type { OperationLogController } from 'src/features/audit-log-management';
import type {
  AuditStatus,
  OperationLogFilters,
  OperationBusinessType,
} from 'src/entities/audit-log';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { FilterDateTimePicker } from 'src/shared/ui/filter-date-time-picker';

import {
  AUDIT_STATUS,
  auditStatusKeys,
  OPERATION_BUSINESS_TYPE,
  operationBusinessTypeKeys,
} from 'src/entities/audit-log';

const FIELD_MIN_WIDTH = 150;
type FilterWriter = <Key extends keyof OperationLogFilters>(
  key: Key,
  value: OperationLogFilters[Key]
) => void;

export function OperationLogFiltersBar({ controller }: { controller: OperationLogController }) {
  const { t } = useTranslate('audit');
  const { state, resources, actions } = controller;
  const write = <Key extends keyof OperationLogFilters>(
    key: Key,
    value: OperationLogFilters[Key]
  ) => actions.changeFilters({ ...state.filterDraft, [key]: value });
  return (
    <Box sx={{ p: 2 }}>
      <Stack
        useFlexGap
        direction={{ xs: 'column', md: 'row' }}
        spacing={1}
        sx={{ flexWrap: { md: 'wrap' }, alignItems: { md: 'center' } }}
      >
        <OperationTextFilters draft={state.filterDraft} write={write} />
        <BusinessTypeFilter
          value={state.filterDraft.business_type}
          onChange={(value) => write('business_type', value)}
        />
        <StatusFilter
          value={state.filterDraft.status}
          onChange={(value) => write('status', value)}
        />
        <OperationDateFilters draft={state.filterDraft} write={write} />
        <Button
          variant="outlined"
          startIcon={<Iconify icon="solar:restart-bold" />}
          onClick={actions.resetFilters}
        >
          {t('actions.reset')}
        </Button>
      </Stack>
      {resources.filterErrorMessage && (
        <Alert severity="error" role="alert" sx={{ mt: 2 }}>
          {resources.filterErrorMessage}
        </Alert>
      )}
    </Box>
  );
}

function OperationTextFilters({
  draft,
  write,
}: {
  draft: OperationLogFilters;
  write: FilterWriter;
}) {
  const { t } = useTranslate('audit');
  return (
    <>
      {(
        [
          ['title', 'fields.title'],
          ['oper_name', 'fields.operator'],
          ['oper_ip', 'fields.operationIp'],
        ] as const
      ).map(([key, label]) => (
        <TextField
          key={key}
          size="small"
          label={t(label)}
          value={draft[key]}
          sx={{ minWidth: FIELD_MIN_WIDTH }}
          onChange={(event) => write(key, event.target.value)}
        />
      ))}
    </>
  );
}

function OperationDateFilters({
  draft,
  write,
}: {
  draft: OperationLogFilters;
  write: FilterWriter;
}) {
  const { t } = useTranslate('audit');
  return (
    <>
      <FilterDateTimePicker
        label={t('fields.beginTime')}
        value={draft.begin_time}
        onChange={(value) => write('begin_time', value)}
      />
      <FilterDateTimePicker
        label={t('fields.endTime')}
        value={draft.end_time}
        onChange={(value) => write('end_time', value)}
      />
    </>
  );
}

function BusinessTypeFilter({
  value,
  onChange,
}: {
  value: OperationBusinessType | '';
  onChange: (value: OperationBusinessType | '') => void;
}) {
  const { t } = useTranslate('audit');
  return (
    <TextField
      select
      size="small"
      label={t('fields.businessType')}
      value={value}
      sx={{ minWidth: FIELD_MIN_WIDTH }}
      onChange={(event) =>
        onChange(
          event.target.value === '' ? '' : (Number(event.target.value) as OperationBusinessType)
        )
      }
    >
      <MenuItem value="">{t('all')}</MenuItem>
      {Object.values(OPERATION_BUSINESS_TYPE).map((type) => (
        <MenuItem key={type} value={type}>
          {t(operationBusinessTypeKeys[type])}
        </MenuItem>
      ))}
    </TextField>
  );
}

function StatusFilter({
  value,
  onChange,
}: {
  value: AuditStatus | '';
  onChange: (value: AuditStatus | '') => void;
}) {
  const { t } = useTranslate('audit');
  return (
    <TextField
      select
      size="small"
      label={t('fields.operationStatus')}
      value={value}
      sx={{ minWidth: FIELD_MIN_WIDTH }}
      onChange={(event) =>
        onChange(event.target.value === '' ? '' : (Number(event.target.value) as AuditStatus))
      }
    >
      <MenuItem value="">{t('all')}</MenuItem>
      {Object.values(AUDIT_STATUS).map((status) => (
        <MenuItem key={status} value={status}>
          {t(auditStatusKeys[status])}
        </MenuItem>
      ))}
    </TextField>
  );
}
