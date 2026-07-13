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

import { DEFAULT_FILTERS } from './constants';

const SELECT_FILTER_MIN_WIDTH = 140;

type MenuFiltersProps = {
  filters: typeof DEFAULT_FILTERS;
  error: LocalDateTimeFilterError | null;
  onChange: (filters: typeof DEFAULT_FILTERS) => void;
};

export function MenuFilters({ filters, error, onChange }: MenuFiltersProps) {
  const { t } = useTranslate('admin');
  const write = (key: keyof typeof DEFAULT_FILTERS, value: string) =>
    onChange({ ...filters, [key]: value });

  return (
    <Box sx={{ p: 2 }}>
      <Stack direction="row" useFlexGap flexWrap="wrap" spacing={1} sx={{ alignItems: 'center' }}>
        <TextField
          size="small"
          label={t('fields.menuName')}
          value={filters.menu_name}
          onChange={(event) => write('menu_name', event.target.value)}
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
        <Button variant="outlined" onClick={() => onChange(DEFAULT_FILTERS)}>
          {t('common.reset')}
        </Button>
      </Stack>
      <FilterDateTimeErrorAlert error={error} />
    </Box>
  );
}
