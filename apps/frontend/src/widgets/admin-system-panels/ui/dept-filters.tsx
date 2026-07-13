import type { TranslateFn } from 'src/shared/i18n';
import type { DeptFiltersValue } from './dept-constants';
import type { LocalDateTimeFilterError } from 'src/shared/lib/local-date-time-filter';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/shared/i18n/use-locales';
import {
  FilterDateTimePicker,
  FilterDateTimeErrorAlert,
} from 'src/shared/ui/filter-date-time-picker';

import { DEFAULT_FILTERS } from './dept-constants';

type DeptFilterKey = keyof DeptFiltersValue;
type FilterWriter = (key: DeptFilterKey, value: string) => void;

type TextFilter = {
  key: DeptFilterKey;
  label: string;
};

const TEXT_FILTER_MIN_WIDTH = 150;
const STATUS_FILTER_MIN_WIDTH = 140;

export function DeptFilters({ filters, error, onChange }: DeptFiltersProps) {
  const { t } = useTranslate('admin');
  const write: FilterWriter = (key, value) => onChange({ ...filters, [key]: value });
  return (
    <Box sx={{ p: 2 }}>
      <Stack direction="row" useFlexGap flexWrap="wrap" spacing={1} sx={{ alignItems: 'center' }}>
        <DeptTextFilters filters={filters} write={write} t={t} />
        <DeptStatusFilter
          value={filters.status}
          t={t}
          onChange={(value) => write('status', value)}
        />
        <DeptDateFilters filters={filters} write={write} t={t} />
        <Button variant="outlined" onClick={() => onChange(DEFAULT_FILTERS)}>
          {t('common.reset')}
        </Button>
      </Stack>
      <FilterDateTimeErrorAlert error={error} />
    </Box>
  );
}

function DeptTextFilters({ filters, write, t }: DeptFilterSectionProps) {
  return (
    <>
      {textFilters(t).map((filter) => (
        <TextField
          key={filter.key}
          size="small"
          label={filter.label}
          value={filters[filter.key]}
          sx={{ minWidth: TEXT_FILTER_MIN_WIDTH }}
          onChange={(event) => write(filter.key, event.target.value)}
        />
      ))}
    </>
  );
}

function DeptStatusFilter({ value, t, onChange }: SelectFilterProps) {
  return (
    <TextField
      select
      size="small"
      label={t('common.status')}
      value={value}
      sx={{ minWidth: STATUS_FILTER_MIN_WIDTH }}
      onChange={(event) => onChange(event.target.value)}
    >
      <MenuItem value="">{t('common.all')}</MenuItem>
      <MenuItem value="0">{t('common.enabled')}</MenuItem>
      <MenuItem value="1">{t('common.disabled')}</MenuItem>
    </TextField>
  );
}

function DeptDateFilters({ filters, write, t }: DeptFilterSectionProps) {
  return (
    <>
      <FilterDateTimePicker
        label={t('fields.beginTime')}
        value={filters.begin_time}
        onChange={(value) => write('begin_time', value)}
      />
      <FilterDateTimePicker
        label={t('fields.endTime')}
        value={filters.end_time}
        onChange={(value) => write('end_time', value)}
      />
    </>
  );
}

function textFilters(t: TranslateFn): TextFilter[] {
  return [
    { key: 'dept_name', label: t('fields.deptName') },
    { key: 'leader', label: t('fields.leader') },
    { key: 'phone', label: t('fields.phone') },
    { key: 'email', label: t('common.email') },
  ];
}

type DeptFiltersProps = {
  filters: DeptFiltersValue;
  error: LocalDateTimeFilterError | null;
  onChange: (filters: DeptFiltersValue) => void;
};

type DeptFilterSectionProps = {
  filters: DeptFiltersValue;
  write: FilterWriter;
  t: TranslateFn;
};

type SelectFilterProps = {
  value: string;
  t: TranslateFn;
  onChange: (value: string) => void;
};
