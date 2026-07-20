import type React from 'react';
import type { Post } from 'src/entities/system';
import type { TranslateFn } from 'src/shared/i18n';
import type { RoleOption } from 'src/entities/role';
import type { UserFiltersValue } from './constants';
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

import { translatedRoleName } from 'src/entities/role';

import { DEFAULT_FILTERS } from './constants';
import { SearchMultiSelect } from './search-multi-select';

const TEXT_FILTER_WIDTH = 150;
const SELECT_FILTER_WIDTH = 140;
const MULTI_SELECT_FILTER_WIDTH = 220;
const ID_SEPARATOR = ',';

type FilterWriter = (key: keyof UserFiltersValue, value: string) => void;

type UserFiltersProps = {
  filters: UserFiltersValue;
  error: LocalDateTimeFilterError | null;
  roles: RoleOption[];
  posts: Post[];
  onChange: (filters: UserFiltersValue) => void;
};

export function UserFilters({ filters, error, roles, posts, onChange }: UserFiltersProps) {
  const { t } = useTranslate('admin');
  const write: FilterWriter = (key, value) => onChange({ ...filters, [key]: value });

  return (
    <Box sx={{ p: 2 }}>
      <Stack direction="row" useFlexGap flexWrap="wrap" spacing={2} sx={{ alignItems: 'center' }}>
        <TextFilterFields filters={filters} write={write} t={t} />
        <SelectFilterFields filters={filters} write={write} t={t} />
        <MultiSelectFilterFields
          filters={filters}
          write={write}
          roles={roles}
          posts={posts}
          t={t}
        />
        <DateFilterFields filters={filters} write={write} t={t} />
        <Button variant="outlined" onClick={() => onChange(DEFAULT_FILTERS)}>
          {t('common.reset')}
        </Button>
      </Stack>
      <FilterDateTimeErrorAlert error={error} />
    </Box>
  );
}

function TextFilterFields({ filters, write, t }: BaseFilterProps) {
  return (
    <>
      <TextFilter
        label={t('common.username')}
        value={filters.username}
        onChange={(value) => write('username', value)}
      />
      <TextFilter
        label={t('fields.nickName')}
        value={filters.nick_name}
        onChange={(value) => write('nick_name', value)}
      />
      <TextFilter
        label={t('fields.deptName')}
        value={filters.dept_name}
        onChange={(value) => write('dept_name', value)}
      />
      <TextFilter
        label={t('common.email')}
        value={filters.email}
        onChange={(value) => write('email', value)}
      />
      <TextFilter
        label={t('fields.phone')}
        value={filters.phonenumber}
        onChange={(value) => write('phonenumber', value)}
      />
    </>
  );
}

function SelectFilterFields({ filters, write, t }: BaseFilterProps) {
  return (
    <>
      <SelectFilter
        label={t('fields.sex')}
        value={filters.sex}
        onChange={(value) => write('sex', value)}
      >
        <MenuItem value="">{t('common.all')}</MenuItem>
        <MenuItem value="0">{t('common.male')}</MenuItem>
        <MenuItem value="1">{t('common.female')}</MenuItem>
        <MenuItem value="2">{t('common.unknown')}</MenuItem>
      </SelectFilter>
      <SelectFilter
        label={t('common.status')}
        value={filters.status}
        onChange={(value) => write('status', value)}
      >
        <MenuItem value="">{t('common.all')}</MenuItem>
        <MenuItem value="0">{t('common.enabled')}</MenuItem>
        <MenuItem value="1">{t('common.disabled')}</MenuItem>
      </SelectFilter>
    </>
  );
}

function MultiSelectFilterFields({ filters, write, roles, posts, t }: MultiSelectFilterProps) {
  const postOptions = posts.map((post) => ({ id: post.post_id, label: post.post_name }));
  const roleOptions = roles.map((role) => ({
    id: role.role_id,
    label: translatedRoleName(role),
  }));

  return (
    <>
      <Box sx={{ minWidth: MULTI_SELECT_FILTER_WIDTH }}>
        <SearchMultiSelect
          label={t('fields.postName')}
          values={splitIds(filters.post_ids)}
          options={postOptions}
          onChange={(values) => write('post_ids', joinIds(values))}
        />
      </Box>
      <Box sx={{ minWidth: MULTI_SELECT_FILTER_WIDTH }}>
        <SearchMultiSelect
          label={t('common.role')}
          values={splitIds(filters.role_ids)}
          options={roleOptions}
          onChange={(values) => write('role_ids', joinIds(values))}
        />
      </Box>
    </>
  );
}

function DateFilterFields({ filters, write, t }: BaseFilterProps) {
  return (
    <>
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
    </>
  );
}

function TextFilter({ label, value, onChange }: FieldProps) {
  return (
    <TextField
      size="small"
      label={label}
      value={value}
      sx={{ minWidth: TEXT_FILTER_WIDTH }}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}

function SelectFilter({ label, value, children, onChange }: SelectFilterProps) {
  return (
    <TextField
      select
      size="small"
      label={label}
      value={value}
      sx={{ minWidth: SELECT_FILTER_WIDTH }}
      onChange={(event) => onChange(event.target.value)}
    >
      {children}
    </TextField>
  );
}

function splitIds(value: string) {
  return value
    .split(ID_SEPARATOR)
    .map((item) => item.trim())
    .filter(Boolean);
}

function joinIds(values: string[]) {
  return values.join(ID_SEPARATOR);
}

type BaseFilterProps = {
  filters: UserFiltersValue;
  write: FilterWriter;
  t: TranslateFn;
};

type MultiSelectFilterProps = BaseFilterProps & {
  roles: RoleOption[];
  posts: Post[];
};

type FieldProps = {
  label: string;
  value: string;
  onChange: (value: string) => void;
};

type SelectFilterProps = FieldProps & {
  children: React.ReactNode;
};
