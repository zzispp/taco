import type { CrudFilter } from './types';
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

const SELECT_FILTER_MIN_WIDTH = 140;
const FILTER_SPACING = 1;

export function CrudFilters({ filters, values, error, onChange }: CrudFiltersProps) {
  const { t } = useTranslate('admin');
  if (filters.length === 0 || !onChange) return null;
  const write = (key: string, value: string) => onChange({ ...values, [key]: value });

  return (
    <Box sx={{ p: 2 }}>
      <Stack
        direction="row"
        useFlexGap
        flexWrap="wrap"
        spacing={FILTER_SPACING}
        sx={{ alignItems: 'center' }}
      >
        {filters.map((filter) => (
          <FilterControl
            key={filter.key}
            filter={filter}
            value={values[filter.key] ?? ''}
            onChange={(value) => write(filter.key, value)}
          />
        ))}
        <Button
          variant="outlined"
          onClick={() => onChange(Object.fromEntries(filters.map((filter) => [filter.key, ''])))}
        >
          {t('common.reset')}
        </Button>
      </Stack>
      <FilterDateTimeErrorAlert error={error} />
    </Box>
  );
}

function FilterControl({ filter, value, onChange }: FilterControlProps) {
  const type: NonNullable<CrudFilter['type']> = filter.type ?? 'text';
  switch (type) {
    case 'dateTime':
      return <FilterDateTimePicker label={filter.label} value={value} onChange={onChange} />;
    case 'select':
      return <SelectFilter filter={filter} value={value} onChange={onChange} />;
    case 'text':
      return (
        <TextField
          size="small"
          label={filter.label}
          value={value}
          onChange={(event) => onChange(event.target.value)}
        />
      );
    default:
      return unsupportedFilterType(type);
  }
}

function SelectFilter({ filter, value, onChange }: FilterControlProps) {
  return (
    <TextField
      select
      size="small"
      label={filter.label}
      value={value}
      sx={{ minWidth: SELECT_FILTER_MIN_WIDTH }}
      onChange={(event) => onChange(event.target.value)}
    >
      {filter.options?.map((option) => (
        <MenuItem key={option.value} value={option.value}>
          {option.label}
        </MenuItem>
      ))}
    </TextField>
  );
}

function unsupportedFilterType(type: never): never {
  throw new Error(`Unsupported CRUD filter type: ${type}`);
}

type CrudFiltersProps = Readonly<{
  filters: CrudFilter[];
  values: Record<string, string>;
  error: LocalDateTimeFilterError | null;
  onChange?: (filters: Record<string, string>) => void;
}>;

type FilterControlProps = Readonly<{
  filter: CrudFilter;
  value: string;
  onChange: (value: string) => void;
}>;
