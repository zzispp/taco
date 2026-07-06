import type { UserFiltersValue } from './constants';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { DEFAULT_FILTERS } from './constants';

export function UserFilters({
  filters,
  onChange,
}: {
  filters: UserFiltersValue;
  onChange: (filters: UserFiltersValue) => void;
}) {
  const { t } = useTranslate('admin');
  const write = (key: keyof UserFiltersValue, value: string) =>
    onChange({ ...filters, [key]: value });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2 }}>
      <TextField
        size="small"
        label={t('common.username')}
        value={filters.username}
        onChange={(event) => write('username', event.target.value)}
      />
      <TextField
        size="small"
        label={t('fields.phone')}
        value={filters.phonenumber}
        onChange={(event) => write('phonenumber', event.target.value)}
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
      <Button variant="outlined" onClick={() => onChange(DEFAULT_FILTERS)}>
        {t('common.reset')}
      </Button>
    </Stack>
  );
}
