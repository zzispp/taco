import type { OnlineSessionFilters } from 'src/entities/online-session';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { DEFAULT_FILTERS } from './constants';

export function OnlineSessionFiltersBar({
  filters,
  onChange,
}: {
  filters: OnlineSessionFilters;
  onChange: (filters: OnlineSessionFilters) => void;
}) {
  const { t } = useTranslate('admin');
  const write = (key: keyof OnlineSessionFilters, value: string) =>
    onChange({ ...filters, [key]: value });

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2 }}>
      <TextField
        size="small"
        label={t('onlineSessions.ipaddr')}
        value={filters.ipaddr}
        onChange={(event) => write('ipaddr', event.target.value)}
      />
      <TextField
        size="small"
        label={t('onlineSessions.userName')}
        value={filters.userName}
        onChange={(event) => write('userName', event.target.value)}
      />
      <Button variant="outlined" onClick={() => onChange(DEFAULT_FILTERS)}>
        {t('common.reset')}
      </Button>
    </Stack>
  );
}
