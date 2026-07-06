import type { DictDataFiltersValue, DictTypeFiltersValue } from './dict-constants';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { DEFAULT_DATA_FILTERS, DEFAULT_TYPE_FILTERS } from './dict-constants';

export function DictTypeFilters({
  filters,
  onChange,
}: {
  filters: DictTypeFiltersValue;
  onChange: (filters: DictTypeFiltersValue) => void;
}) {
  const { t } = useTranslate('admin');
  const write = (key: keyof DictTypeFiltersValue, value: string) =>
    onChange({ ...filters, [key]: value });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2 }}>
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
        sx={{ minWidth: 140 }}
        onChange={(event) => write('status', event.target.value)}
      >
        <MenuItem value="">{t('common.all')}</MenuItem>
        <MenuItem value="0">{t('common.enabled')}</MenuItem>
        <MenuItem value="1">{t('common.disabled')}</MenuItem>
      </TextField>
      <TextField
        size="small"
        type="date"
        label={t('fields.beginTime')}
        value={filters.begin_time}
        InputLabelProps={{ shrink: true }}
        onChange={(event) => write('begin_time', event.target.value)}
      />
      <TextField
        size="small"
        type="date"
        label={t('fields.endTime')}
        value={filters.end_time}
        InputLabelProps={{ shrink: true }}
        onChange={(event) => write('end_time', event.target.value)}
      />
      <Button variant="outlined" onClick={() => onChange(DEFAULT_TYPE_FILTERS)}>
        {t('common.reset')}
      </Button>
    </Stack>
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
        sx={{ minWidth: 140 }}
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
