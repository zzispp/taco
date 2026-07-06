import type { CrudFilter } from './types';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/shared/i18n/use-locales';

export function CrudFilters({
  filters,
  values,
  onChange,
}: {
  filters: CrudFilter[];
  values: Record<string, string>;
  onChange?: (filters: Record<string, string>) => void;
}) {
  const { t } = useTranslate('admin');
  if (filters.length === 0 || !onChange) return null;
  const write = (key: string, value: string) => onChange({ ...values, [key]: value });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2 }}>
      {filters.map((filter) => (
        <TextField
          key={filter.key}
          select={filter.type === 'select'}
          type={filter.type === 'date' ? 'date' : 'text'}
          size="small"
          label={filter.label}
          value={values[filter.key] ?? ''}
          InputLabelProps={filter.type === 'date' ? { shrink: true } : undefined}
          sx={{
            minWidth: filter.type === 'select' ? 140 : filter.type === 'date' ? 170 : undefined,
          }}
          onChange={(event) => write(filter.key, event.target.value)}
        >
          {filter.options?.map((option) => (
            <MenuItem key={option.value} value={option.value}>
              {option.label}
            </MenuItem>
          ))}
        </TextField>
      ))}
      <Button
        variant="outlined"
        onClick={() => onChange(Object.fromEntries(filters.map((filter) => [filter.key, ''])))}
      >
        {t('common.reset')}
      </Button>
    </Stack>
  );
}
