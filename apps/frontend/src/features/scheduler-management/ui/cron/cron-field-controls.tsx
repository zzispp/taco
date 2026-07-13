import type { ReactNode } from 'react';

import Stack from '@mui/material/Stack';
import Radio from '@mui/material/Radio';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import RadioGroup from '@mui/material/RadioGroup';
import Autocomplete from '@mui/material/Autocomplete';
import FormControlLabel from '@mui/material/FormControlLabel';

import { clampNumber } from '../../model/cron/cron-builder-model';

export type SelectOption = {
  label: string;
  value: string;
  disabled?: boolean;
};

export function CronRadioGroup({
  value,
  onChange,
  children,
}: {
  value: string;
  onChange: (value: string) => void;
  children: ReactNode;
}) {
  return (
    <RadioGroup value={value} onChange={(event) => onChange(event.target.value)}>
      <Stack spacing={1.5}>{children}</Stack>
    </RadioGroup>
  );
}

export function CronRadioOption({ value, children }: { value: string; children: ReactNode }) {
  return (
    <FormControlLabel
      value={value}
      control={<Radio />}
      label={
        <Stack
          alignItems={{ xs: 'flex-start', sm: 'center' }}
          direction={{ xs: 'column', sm: 'row' }}
          spacing={1}
        >
          {children}
        </Stack>
      }
      sx={{ alignItems: 'flex-start', m: 0 }}
    />
  );
}

export function CronNumberInput({
  label,
  value,
  min,
  max,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
}) {
  return (
    <TextField
      size="small"
      type="number"
      label={label}
      value={value}
      onChange={(event) => onChange(clampNumber(Number(event.target.value), min, max))}
      slotProps={{ htmlInput: { min, max } }}
      sx={{ width: 112 }}
    />
  );
}

export function CronMultiSelect({
  label,
  options,
  value,
  onChange,
}: {
  label: string;
  options: SelectOption[];
  value: string[];
  onChange: (value: string[]) => void;
}) {
  return (
    <Autocomplete
      multiple
      size="small"
      options={options.map((item) => item.value)}
      value={value}
      getOptionLabel={(option) => options.find((item) => item.value === option)?.label ?? option}
      onChange={(_, nextValue) => onChange(nextValue)}
      renderInput={(params) => <TextField {...params} label={label} />}
      sx={{ minWidth: 240 }}
    />
  );
}

export function CronSelect({
  label,
  options,
  value,
  onChange,
}: {
  label: string;
  options: SelectOption[];
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <TextField
      select
      size="small"
      label={label}
      value={String(value)}
      onChange={(event) => onChange(Number(event.target.value))}
      sx={{ width: 148 }}
    >
      {options.map((option) => (
        <MenuItem key={option.value} value={option.value} disabled={option.disabled}>
          {option.label}
        </MenuItem>
      ))}
    </TextField>
  );
}
