import type { DictDataFiltersValue, DictTypeFiltersValue } from './dict-constants';
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

import { DEFAULT_DATA_FILTERS, DEFAULT_TYPE_FILTERS } from './dict-constants';

const SELECT_FILTER_MIN_WIDTH = 140;

export function DictTypeFilters({
  filters,
  error,
  onChange,
}: {
  filters: DictTypeFiltersValue;
  error: LocalDateTimeFilterError | null;
  onChange: (filters: DictTypeFiltersValue) => void;
}) {
  const { t } = useTranslate('admin');
  const write = (key: keyof DictTypeFiltersValue, value: string) =>
    onChange({ ...filters, [key]: value });
  return (
    <Box sx={{ p: 2 }}>
      <Stack direction="row" useFlexGap flexWrap="wrap" spacing={1} sx={{ alignItems: 'center' }}>
        <DictTypeFilterFields filters={filters} write={write} t={t} />
        <Button variant="outlined" onClick={() => onChange(DEFAULT_TYPE_FILTERS)}>
          {t('common.reset')}
        </Button>
      </Stack>
      <FilterDateTimeErrorAlert error={error} />
    </Box>
  );
}

function DictTypeFilterFields({ filters, write, t }: DictTypeFilterFieldsProps) {
  return (
    <>
      <TextField
        size="small"
        label={t('fields.dictName')}
        value={filters.dict_name}
        onChange={(event) => write('dict_name', event.target.value)}
      />
      <TextField
        size="small"
        label={t('fields.dictType')}
        value={filters.dict_type}
        onChange={(event) => write('dict_type', event.target.value)}
      />
      <TextField
        select
        size="small"
        label={t('common.status')}
        value={filters.status}
        sx={{ minWidth: SELECT_FILTER_MIN_WIDTH }}
        onChange={(event) => write('status', event.target.value)}
      >
        <MenuItem value="">{t('common.all')}</MenuItem>
        <MenuItem value="0">{t('common.enabled')}</MenuItem>
        <MenuItem value="1">{t('common.disabled')}</MenuItem>
      </TextField>
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

export function DictDataFilters({
  filters,
  onChange,
}: {
  filters: DictDataFiltersValue;
  onChange: (filters: DictDataFiltersValue) => void;
}) {
  const { t } = useTranslate('admin');
  const write = (key: keyof DictDataFiltersValue, value: string) =>
    onChange({ ...filters, [key]: value });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ px: 2, pb: 2 }}>
      <TextField
        size="small"
        label={t('fields.dictLabel')}
        value={filters.dict_label}
        onChange={(event) => write('dict_label', event.target.value)}
      />
      <TextField
        select
        size="small"
        label={t('common.status')}
        value={filters.status}
        sx={{ minWidth: SELECT_FILTER_MIN_WIDTH }}
        onChange={(event) => write('status', event.target.value)}
      >
        <MenuItem value="">{t('common.all')}</MenuItem>
        <MenuItem value="0">{t('common.enabled')}</MenuItem>
        <MenuItem value="1">{t('common.disabled')}</MenuItem>
      </TextField>
      <Button variant="outlined" onClick={() => onChange(DEFAULT_DATA_FILTERS)}>
        {t('common.reset')}
      </Button>
    </Stack>
  );
}

type DictTypeFilterFieldsProps = Readonly<{
  filters: DictTypeFiltersValue;
  write: (key: keyof DictTypeFiltersValue, value: string) => void;
  t: ReturnType<typeof useTranslate>['t'];
}>;
