import type { RoleFiltersValue } from './constants';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { DEFAULT_FILTERS } from './constants';

export function RoleFilters({
  filters,
  onChange,
}: {
  filters: RoleFiltersValue;
  onChange: (filters: RoleFiltersValue) => void;
}) {
  const { t } = useTranslate('admin');
  const write = (key: keyof RoleFiltersValue, value: string) =>
    onChange({ ...filters, [key]: value });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2 }}>
      <TextField
        size="small"
        label={t('fields.roleName')}
        value={filters.role_name}
        onChange={(event) => write('role_name', event.target.value)}
      />
      <TextField
        size="small"
        label={t('fields.roleKey')}
        value={filters.role_key}
        onChange={(event) => write('role_key', event.target.value)}
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
