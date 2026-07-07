import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { DEFAULT_FILTERS } from './constants';

type MenuFiltersProps = {
  filters: typeof DEFAULT_FILTERS;
  onChange: (filters: typeof DEFAULT_FILTERS) => void;
};

export function MenuFilters({ filters, onChange }: MenuFiltersProps) {
  const { t } = useTranslate('admin');
  const write = (key: keyof typeof DEFAULT_FILTERS, value: string) =>
    onChange({ ...filters, [key]: value });

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2 }}>
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
        sx={{ minWidth: 140 }}
        onChange={(event) => write('status', event.target.value)}
      >
        <MenuItem value="">{t('common.all')}</MenuItem>
        <MenuItem value="0">{t('common.enabled')}</MenuItem>
        <MenuItem value="1">{t('common.disabled')}</MenuItem>
      </TextField>
      <Button variant="outlined" onClick={() => onChange(DEFAULT_FILTERS)}>
        {t('common.reset')}
      </Button>
    </Stack>
  );
}
