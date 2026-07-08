import type { TranslateFn } from 'src/shared/i18n';
import type { DeptFiltersValue } from './dept-constants';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { DEFAULT_FILTERS } from './dept-constants';

type DeptFilterKey = keyof DeptFiltersValue;
type FilterWriter = (key: DeptFilterKey, value: string) => void;

type TextFilter = {
  key: DeptFilterKey;
  label: string;
};

const TEXT_FILTER_MIN_WIDTH = 150;
const STATUS_FILTER_MIN_WIDTH = 140;
const DATE_FILTER_MIN_WIDTH = 170;

export function DeptFilters({ filters, onChange }: DeptFiltersProps) {
  const { t } = useTranslate('admin');
  const write: FilterWriter = (key, value) => onChange({ ...filters, [key]: value });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2 }}>
      <DeptTextFilters filters={filters} write={write} t={t} />
      <DeptStatusFilter value={filters.status} t={t} onChange={(value) => write('status', value)} />
      <DeptDateFilters filters={filters} write={write} t={t} />
      <Button variant="outlined" onClick={() => onChange(DEFAULT_FILTERS)}>
        {t('common.reset')}
      </Button>
    </Stack>
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
      <TextField
        size="small"
        type="date"
        label={t('fields.beginTime')}
        value={filters.begin_time}
        InputLabelProps={{ shrink: true }}
        sx={{ minWidth: DATE_FILTER_MIN_WIDTH }}
        onChange={(event) => write('begin_time', event.target.value)}
      />
      <TextField
        size="small"
        type="date"
        label={t('fields.endTime')}
        value={filters.end_time}
        InputLabelProps={{ shrink: true }}
        sx={{ minWidth: DATE_FILTER_MIN_WIDTH }}
        onChange={(event) => write('end_time', event.target.value)}
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
