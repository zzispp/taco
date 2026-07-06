import type React from 'react';
import type { SelectOptionItem } from './helpers';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Checkbox from '@mui/material/Checkbox';
import TextField from '@mui/material/TextField';
import Autocomplete from '@mui/material/Autocomplete';

import { MAX_VISIBLE_SELECT_TAGS } from './constants';

export function SearchMultiSelect({
  label,
  values,
  options,
  onChange,
}: {
  label: string;
  values: string[];
  options: SelectOptionItem[];
  onChange: (values: string[]) => void;
}) {
  const selectedOptions = options.filter((option) => values.includes(option.id));
  return (
    <Autocomplete
      multiple
      disableCloseOnSelect
      size="small"
      options={options}
      value={selectedOptions}
      isOptionEqualToValue={(option, value) => option.id === value.id}
      getOptionLabel={(option) => option.label}
      onChange={(_, next) => onChange(next.map((option) => option.id))}
      renderOption={(props, option, state) => (
        <OptionRow key={option.id} props={props} option={option} selected={state.selected} />
      )}
      renderValue={(selected, getItemProps) =>
        renderSelectedOptions(selected as SelectOptionItem[], getItemProps)
      }
      renderInput={(params) => <TextField {...params} label={label} />}
      slotProps={{ listbox: { sx: { maxHeight: 280 } } }}
    />
  );
}

function OptionRow({
  props,
  option,
  selected,
}: {
  props: React.HTMLAttributes<HTMLLIElement> & { key: React.Key };
  option: SelectOptionItem;
  selected: boolean;
}) {
  const { key, ...itemProps } = props;
  return (
    <li key={key} {...itemProps}>
      <Checkbox size="small" checked={selected} sx={{ mr: 1 }} />
      {option.label}
    </li>
  );
}

function renderSelectedOptions(
  selected: SelectOptionItem[],
  getItemProps: (args: { index: number }) => Record<string, unknown>
) {
  const visible = selected.slice(0, MAX_VISIBLE_SELECT_TAGS);
  const hiddenCount = selected.length - visible.length;
  return (
    <Box sx={{ display: 'flex', minWidth: 0, gap: 0.5, overflow: 'hidden' }}>
      {visible.map((option, index) => (
        <Chip
          {...getItemProps({ index })}
          key={option.id}
          size="small"
          label={option.label}
          sx={{ maxWidth: 150 }}
        />
      ))}
      {hiddenCount > 0 && <Chip size="small" label={`+${hiddenCount}`} />}
    </Box>
  );
}
