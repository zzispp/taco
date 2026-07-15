import type React from 'react';
import type { CrudField } from './types';

import Switch from '@mui/material/Switch';
import MenuItem from '@mui/material/MenuItem';
import FormControlLabel from '@mui/material/FormControlLabel';

import { TextFieldRow } from 'src/widgets/admin-common';

import { normalizeValue } from './helpers';

export function CrudFieldControl<I extends Record<string, unknown>>({
  field,
  editing,
  form,
  setForm,
}: CrudControlProps<I>) {
  const value = form[field.key] ?? '';
  const disabled = field.disabled?.({ form, editing }) ?? false;
  const writeValue = (next: string | boolean) =>
    setForm((current) => ({ ...current, [field.key]: normalizeValue(next, field.type) }));
  if (field.type === 'switch') return <StatusSwitch value={value} onChange={writeValue} />;
  if (field.type === 'boolean') {
    return <BooleanField field={field} value={value} disabled={disabled} onChange={writeValue} />;
  }
  return <ValueField field={field} value={value} disabled={disabled} onChange={writeValue} />;
}

type CrudControlProps<I extends Record<string, unknown>> = Readonly<{
  field: CrudField<I>;
  editing: Record<string, unknown> | null;
  form: I;
  setForm: React.Dispatch<React.SetStateAction<I>>;
}>;

type FieldControlProps<I extends Record<string, unknown>> = Readonly<{
  field: CrudField<I>;
  value: unknown;
  disabled: boolean;
  onChange: (value: string | boolean) => void;
}>;

function StatusSwitch({ value, onChange }: Pick<FieldControlProps<never>, 'value' | 'onChange'>) {
  return (
    <Switch
      checked={String(value) === '0'}
      onChange={(event) => onChange(event.target.checked ? '0' : '1')}
    />
  );
}

function BooleanField<I extends Record<string, unknown>>({
  field,
  value,
  disabled,
  onChange,
}: FieldControlProps<I>) {
  return (
    <FormControlLabel
      control={
        <Switch
          checked={Boolean(value)}
          disabled={disabled}
          onChange={(event) => onChange(event.target.checked)}
        />
      }
      label={field.label}
    />
  );
}

function ValueField<I extends Record<string, unknown>>({
  field,
  value,
  disabled,
  onChange,
}: FieldControlProps<I>) {
  return (
    <TextFieldRow
      disabled={disabled}
      label={field.label}
      type={field.type === 'number' ? 'number' : 'text'}
      select={field.type === 'select'}
      multiline={field.type === 'textarea'}
      value={String(value)}
      onChange={onChange}
    >
      {field.options?.map((option) => (
        <MenuItem key={option.value} value={option.value}>
          {option.label}
        </MenuItem>
      ))}
    </TextFieldRow>
  );
}
