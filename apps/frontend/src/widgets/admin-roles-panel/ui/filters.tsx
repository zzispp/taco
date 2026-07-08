import type React from 'react';
import type { TranslateFn } from 'src/shared/i18n';
import type { RoleFiltersValue } from './constants';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { DEFAULT_FILTERS } from './constants';

const SELECT_FILTER_MIN_WIDTH = 140;

type RoleFilterKey = keyof RoleFiltersValue;
type FilterWriter = (key: RoleFilterKey, value: string) => void;

type RoleFiltersProps = {
  filters: RoleFiltersValue;
  onChange: (filters: RoleFiltersValue) => void;
};

export function RoleFilters({ filters, onChange }: RoleFiltersProps) {
  const { t } = useTranslate('admin');
  const write: FilterWriter = (key, value) => onChange({ ...filters, [key]: value });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2 }}>
      <RoleTextFilters filters={filters} write={write} t={t} />
      <RoleSelectFilters filters={filters} write={write} t={t} />
      <RoleDateFilters filters={filters} write={write} t={t} />
      <Button variant="outlined" onClick={() => onChange(DEFAULT_FILTERS)}>
        {t('common.reset')}
      </Button>
    </Stack>
  );
}

function RoleTextFilters({ filters, write, t }: RoleFilterSectionProps) {
  return (
    <>
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
    </>
  );
}

function RoleSelectFilters({ filters, write, t }: RoleFilterSectionProps) {
  return (
    <>
      <SelectFilter
        label={t('common.status')}
        value={filters.status}
        onChange={(value) => write('status', value)}
      >
        <MenuItem value="">{t('common.all')}</MenuItem>
        <MenuItem value="0">{t('common.enabled')}</MenuItem>
        <MenuItem value="1">{t('common.disabled')}</MenuItem>
      </SelectFilter>
      <SelectFilter
        label={t('common.type')}
        value={filters.system}
        onChange={(value) => write('system', value)}
      >
        <MenuItem value="">{t('common.all')}</MenuItem>
        <MenuItem value="true">{t('common.system')}</MenuItem>
        <MenuItem value="false">{t('common.custom')}</MenuItem>
      </SelectFilter>
    </>
  );
}

function RoleDateFilters({ filters, write, t }: RoleFilterSectionProps) {
  return (
    <>
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
    </>
  );
}

function SelectFilter({ label, value, children, onChange }: SelectFilterProps) {
  return (
    <TextField
      select
      size="small"
      label={label}
      value={value}
      sx={{ minWidth: SELECT_FILTER_MIN_WIDTH }}
      onChange={(event) => onChange(event.target.value)}
    >
      {children}
    </TextField>
  );
}

type RoleFilterSectionProps = {
  filters: RoleFiltersValue;
  write: FilterWriter;
  t: TranslateFn;
};

type SelectFilterProps = {
  label: string;
  value: string;
  children: React.ReactNode;
  onChange: (value: string) => void;
};
