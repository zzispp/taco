import type { FormEvent } from 'react';
import type { SystemLogController } from 'src/features/system-log-management';
import type { SystemLogLevel, SystemLogFilters } from 'src/entities/system-log';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import ListItemText from '@mui/material/ListItemText';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { FilterDateTimePicker } from 'src/shared/ui/filter-date-time-picker';

import { SYSTEM_LOG_LEVELS } from 'src/entities/system-log';

const FIELD_MIN_WIDTH = 160;

type FilterWriter = <Key extends keyof SystemLogFilters>(
  key: Key,
  value: SystemLogFilters[Key]
) => void;

export function SystemLogFiltersBar({ controller }: { controller: SystemLogController }) {
  const { state, resources, actions } = controller;
  const write: FilterWriter = (key, value) =>
    actions.changeFilters({ ...state.filterDraft, [key]: value });
  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    actions.applyFilters();
  };

  return (
    <Box component="form" noValidate sx={{ p: 2 }} onSubmit={submit}>
      <Stack
        useFlexGap
        direction={{ xs: 'column', md: 'row' }}
        spacing={1}
        sx={{ flexWrap: { md: 'wrap' }, alignItems: { md: 'center' } }}
      >
        <SystemLogTextFilters draft={state.filterDraft} write={write} />
        <LevelFilter
          value={state.filterDraft.levels}
          onChange={(levels) => write('levels', levels)}
        />
        <SystemLogDateFilters draft={state.filterDraft} write={write} />
        <SystemLogFilterActions onReset={actions.resetFilters} />
      </Stack>
      {resources.filterErrorMessage && <FilterError message={resources.filterErrorMessage} />}
    </Box>
  );
}

function SystemLogTextFilters({ draft, write }: { draft: SystemLogFilters; write: FilterWriter }) {
  const { t } = useTranslate('systemLog');
  return (
    <>
      <TextField
        size="small"
        label={t('fields.keyword')}
        value={draft.keyword}
        sx={{ minWidth: FIELD_MIN_WIDTH }}
        onChange={(event) => write('keyword', event.target.value)}
      />
      <TextField
        size="small"
        label={t('fields.target')}
        value={draft.target}
        sx={{ minWidth: FIELD_MIN_WIDTH }}
        onChange={(event) => write('target', event.target.value)}
      />
    </>
  );
}

function SystemLogDateFilters({ draft, write }: { draft: SystemLogFilters; write: FilterWriter }) {
  const { t } = useTranslate('systemLog');
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

function SystemLogFilterActions({ onReset }: { onReset: () => void }) {
  const { t } = useTranslate('systemLog');
  return (
    <>
      <Button type="submit" variant="outlined" startIcon={<Iconify icon="eva:search-fill" />}>
        {t('actions.search')}
      </Button>
      <Button
        variant="outlined"
        startIcon={<Iconify icon="solar:restart-bold" />}
        onClick={onReset}
      >
        {t('actions.reset')}
      </Button>
    </>
  );
}

function FilterError({ message }: { message: string }) {
  return (
    <Alert severity="error" role="alert" sx={{ mt: 2 }}>
      {message}
    </Alert>
  );
}

function LevelFilter({
  value,
  onChange,
}: {
  value: readonly SystemLogLevel[];
  onChange: (levels: readonly SystemLogLevel[]) => void;
}) {
  const { t } = useTranslate('systemLog');
  return (
    <TextField
      select
      SelectProps={{
        multiple: true,
        renderValue: (selected) =>
          (selected as string[]).map((level) => t(`levels.${level}`)).join(', '),
      }}
      size="small"
      label={t('fields.level')}
      value={value}
      sx={{ minWidth: FIELD_MIN_WIDTH }}
      onChange={(event) => onChange(readLevels(event.target.value))}
    >
      {SYSTEM_LOG_LEVELS.map((level) => (
        <MenuItem key={level} value={level}>
          <Checkbox checked={value.includes(level)} />
          <ListItemText primary={t(`levels.${level}`)} />
        </MenuItem>
      ))}
    </TextField>
  );
}

function readLevels(value: unknown): SystemLogLevel[] {
  const values = Array.isArray(value) ? value : String(value).split(',');
  return values.filter((item): item is SystemLogLevel =>
    SYSTEM_LOG_LEVELS.includes(item as SystemLogLevel)
  );
}
