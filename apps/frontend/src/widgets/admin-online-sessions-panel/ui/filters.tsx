import type { OnlineSessionFilters } from 'src/entities/online-session';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { DEFAULT_FILTERS } from './constants';

type OnlineSessionFilterKey = keyof OnlineSessionFilters;

type TextFilter = {
  key: OnlineSessionFilterKey;
  label: string;
};

const TEXT_FILTER_MIN_WIDTH = 150;
const DATE_FILTER_MIN_WIDTH = 170;

export function OnlineSessionFiltersBar({
  filters,
  onChange,
}: {
  filters: OnlineSessionFilters;
  onChange: (filters: OnlineSessionFilters) => void;
}) {
  const { t } = useTranslate('admin');
  const write = (key: OnlineSessionFilterKey, value: string) =>
    onChange({ ...filters, [key]: value });

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2 }}>
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
      <Button variant="outlined" onClick={() => onChange(DEFAULT_FILTERS)}>
        {t('common.reset')}
      </Button>
    </Stack>
  );
}

function textFilters(t: ReturnType<typeof useTranslate>['t']): TextFilter[] {
  return [
    { key: 'ipaddr', label: t('onlineSessions.ipaddr') },
    { key: 'userName', label: t('onlineSessions.userName') },
    { key: 'loginLocation', label: t('onlineSessions.loginLocation') },
    { key: 'browser', label: t('onlineSessions.browser') },
    { key: 'os', label: t('onlineSessions.os') },
  ];
}
