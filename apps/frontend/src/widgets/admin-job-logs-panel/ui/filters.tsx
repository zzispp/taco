import type React from 'react';
import type { JobLogController } from 'src/features/scheduler-management';
import type { JobLogStatus, SchedulerTriggerType } from 'src/entities/scheduler';

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
  JOB_LOG_STATUS,
  SCHEDULER_TRIGGER_TYPE,
  jobLogStatusTranslationKeys,
  schedulerTriggerTranslationKeys,
} from 'src/entities/scheduler';

const TEXT_FILTER_MIN_WIDTH = 170;
const SELECT_FILTER_MIN_WIDTH = 160;
const FILTER_SPACING = 1;

type FilterDraft = JobLogController['state']['filterDraft'];
type FilterWriter = <Key extends keyof FilterDraft>(key: Key, value: FilterDraft[Key]) => void;

export function JobLogFilters({ controller }: { controller: JobLogController }) {
  const { t } = useTranslate('scheduler');
  const { t: tAdmin } = useTranslate('admin');
  const { state, resources, actions } = controller;
  const write: FilterWriter = (key, value) =>
    actions.changeFilters({ ...state.filterDraft, [key]: value });

  return (
    <Box sx={{ p: 2 }}>
      <Stack
        direction="row"
        useFlexGap
        flexWrap="wrap"
        spacing={FILTER_SPACING}
        sx={{ alignItems: 'center' }}
      >
        <TextFilters draft={state.filterDraft} write={write} t={t} />
        <SelectFilters draft={state.filterDraft} write={write} t={t} tAdmin={tAdmin} />
        <DateTimeFilters draft={state.filterDraft} write={write} t={t} />
        <Button
          variant="outlined"
          startIcon={<Iconify icon="solar:restart-bold" />}
          onClick={actions.resetFilters}
        >
          {tAdmin('common.reset')}
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

function TextFilters({ draft, write, t }: BaseFilterProps) {
  return (
    <>
      <TextField
        size="small"
        label={t('jobName')}
        value={draft.job_name}
        sx={{ minWidth: TEXT_FILTER_MIN_WIDTH }}
        onChange={(event) => write('job_name', event.target.value)}
      />
      <TextField
        size="small"
        label={t('jobGroup')}
        value={draft.job_group}
        sx={{ minWidth: TEXT_FILTER_MIN_WIDTH }}
        onChange={(event) => write('job_group', event.target.value)}
      />
    </>
  );
}

function SelectFilters({ draft, write, t, tAdmin }: SelectFilterProps) {
  return (
    <>
      <SelectField
        label={tAdmin('common.status')}
        value={draft.status}
        onChange={(value) => write('status', value as JobLogStatus | '')}
      >
        <MenuItem value="">{tAdmin('common.all')}</MenuItem>
        {Object.values(JOB_LOG_STATUS).map((status) => (
          <MenuItem key={status} value={status}>
            {t(jobLogStatusTranslationKeys[status])}
          </MenuItem>
        ))}
      </SelectField>
      <SelectField
        label={t('triggerType')}
        value={draft.trigger_type}
        onChange={(value) => write('trigger_type', value as SchedulerTriggerType | '')}
      >
        <MenuItem value="">{tAdmin('common.all')}</MenuItem>
        {Object.values(SCHEDULER_TRIGGER_TYPE).map((trigger) => (
          <MenuItem key={trigger} value={trigger}>
            {t(schedulerTriggerTranslationKeys[trigger])}
          </MenuItem>
        ))}
      </SelectField>
    </>
  );
}

function DateTimeFilters({ draft, write, t }: BaseFilterProps) {
  return (
    <>
      <FilterDateTimePicker
        label={t('filters.createdFrom')}
        value={draft.begin_time}
        onChange={(value) => write('begin_time', value)}
      />
      <FilterDateTimePicker
        label={t('filters.createdTo')}
        value={draft.end_time}
        onChange={(value) => write('end_time', value)}
      />
    </>
  );
}

function SelectField(props: SelectFieldProps) {
  return (
    <TextField
      select
      size="small"
      label={props.label}
      value={props.value}
      sx={{ minWidth: SELECT_FILTER_MIN_WIDTH }}
      onChange={(event) => props.onChange(event.target.value)}
    >
      {props.children}
    </TextField>
  );
}

type Translate = ReturnType<typeof useTranslate>['t'];
type BaseFilterProps = { draft: FilterDraft; write: FilterWriter; t: Translate };
type SelectFilterProps = BaseFilterProps & { tAdmin: Translate };
type SelectFieldProps = {
  label: string;
  value: string;
  onChange: (value: string) => void;
  children: React.ReactNode;
};
