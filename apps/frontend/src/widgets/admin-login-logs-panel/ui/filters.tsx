import type { LoginLogController } from 'src/features/audit-log-management';
import type { AuditStatus, LoginEventType, LoginLogFilters } from 'src/entities/audit-log';

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
  LOGIN_EVENT_TYPES,
  loginEventTypeKeys,
} from 'src/entities/audit-log';

const FIELD_MIN_WIDTH = 160;
type FilterWriter = <Key extends keyof LoginLogFilters>(
  key: Key,
  value: LoginLogFilters[Key]
) => void;

export function LoginLogFiltersBar({ controller }: { controller: LoginLogController }) {
  const { t } = useTranslate('audit');
  const { state, resources, actions } = controller;
  const write = <Key extends keyof LoginLogFilters>(key: Key, value: LoginLogFilters[Key]) =>
    actions.changeFilters({ ...state.filterDraft, [key]: value });
  return (
    <Box sx={{ p: 2 }}>
      <Stack
        useFlexGap
        direction={{ xs: 'column', md: 'row' }}
        spacing={1}
        sx={{ flexWrap: { md: 'wrap' }, alignItems: { md: 'center' } }}
      >
        <LoginTextFilters draft={state.filterDraft} write={write} />
        <LoginStatusFilter
          value={state.filterDraft.status}
          onChange={(value) => write('status', value)}
        />
        <LoginEventTypeFilter
          value={state.filterDraft.event_type}
          onChange={(value) => write('event_type', value)}
        />
        <LoginDateFilters draft={state.filterDraft} write={write} />
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

function LoginTextFilters({ draft, write }: { draft: LoginLogFilters; write: FilterWriter }) {
  const { t } = useTranslate('audit');
  return (
    <>
      {(
        [
          ['ipaddr', 'fields.loginIp'],
          ['user_name', 'fields.username'],
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

function LoginDateFilters({ draft, write }: { draft: LoginLogFilters; write: FilterWriter }) {
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

function LoginStatusFilter({
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
      label={t('fields.loginStatus')}
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

function LoginEventTypeFilter({
  value,
  onChange,
}: {
  value: LoginEventType | '';
  onChange: (value: LoginEventType | '') => void;
}) {
  const { t } = useTranslate('audit');
  return (
    <TextField
      select
      size="small"
      label={t('fields.eventType')}
      value={value}
      sx={{ minWidth: FIELD_MIN_WIDTH }}
      onChange={(event) => onChange(event.target.value as LoginEventType | '')}
    >
      <MenuItem value="">{t('all')}</MenuItem>
      {LOGIN_EVENT_TYPES.map((eventType) => (
        <MenuItem key={eventType} value={eventType}>
          {t(loginEventTypeKeys[eventType])}
        </MenuItem>
      ))}
    </TextField>
  );
}
