import type { OnlineSessionFilters } from 'src/entities/online-session';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { FilterDateTimePicker } from 'src/shared/ui/filter-date-time-picker';

import { DEFAULT_ONLINE_SESSION_FILTERS } from 'src/entities/online-session';

type OnlineSessionFilterKey = keyof OnlineSessionFilters;

type TextFilter = {
  key: OnlineSessionFilterKey;
  label: string;
};

const TEXT_FILTER_MIN_WIDTH = 150;
const FILTER_SPACING = 1;

export function OnlineSessionFiltersBar({
  filters,
  errorMessage,
  onChange,
}: OnlineSessionFiltersBarProps) {
  const { t } = useTranslate('admin');
  const write = (key: OnlineSessionFilterKey, value: string) =>
    onChange({ ...filters, [key]: value });

  return (
    <Box sx={{ p: 2 }}>
      <Stack
        useFlexGap
        spacing={FILTER_SPACING}
        direction={{ xs: 'column', md: 'row' }}
        sx={{ flexWrap: { md: 'wrap' }, alignItems: { md: 'center' } }}
      >
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
        <Button variant="outlined" onClick={() => onChange(DEFAULT_ONLINE_SESSION_FILTERS)}>
          {t('common.reset')}
        </Button>
      </Stack>
      {errorMessage && (
        <Alert severity="error" role="alert" sx={{ mt: 2 }}>
          {errorMessage}
        </Alert>
      )}
    </Box>
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

type OnlineSessionFiltersBarProps = Readonly<{
  filters: OnlineSessionFilters;
  errorMessage: string | null;
  onChange: (filters: OnlineSessionFilters) => void;
}>;
