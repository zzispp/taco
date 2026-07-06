import type React from 'react';
import type { CrudField } from './types';

import Switch from '@mui/material/Switch';
import MenuItem from '@mui/material/MenuItem';
import FormControlLabel from '@mui/material/FormControlLabel';

import { TextFieldRow } from 'src/shared/ui/admin';

import { normalizeValue } from './helpers';

export function CrudFieldControl<I extends Record<string, unknown>>({
  field,
  editing,
  form,
  setForm,
}: {
  field: CrudField<I>;
  editing: Record<string, unknown> | null;
  form: I;
  setForm: React.Dispatch<React.SetStateAction<I>>;
}) {
  const value = form[field.key] ?? '';
  const disabled = field.disabled?.({ form, editing }) ?? false;
  const writeValue = (next: string | boolean) =>
    setForm((current) => ({ ...current, [field.key]: normalizeValue(next, field.type) }));
  if (field.type === 'switch') {
    return (
      <Switch
        checked={String(value) === '0'}
        onChange={(event) => writeValue(event.target.checked ? '0' : '1')}
      />
    );
  }
  if (field.type === 'boolean') {
    return (
      <FormControlLabel
        control={
          <Switch
            checked={Boolean(value)}
            disabled={disabled}
            onChange={(event) => writeValue(event.target.checked)}
          />
        }
        label={field.label}
      />
    );
  }
  return (
    <TextFieldRow
      disabled={disabled}
      label={field.label}
      type={field.type === 'number' ? 'number' : 'text'}
      select={field.type === 'select'}
      multiline={field.type === 'textarea'}
      value={String(value)}
      onChange={writeValue}
    >
      {field.options?.map((option) => (
        <MenuItem key={option.value} value={option.value}>
          {option.label}
        </MenuItem>
      ))}
    </TextFieldRow>
  );
}
