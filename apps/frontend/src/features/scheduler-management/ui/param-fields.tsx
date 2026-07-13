'use client';

import type { ParamFieldSpec } from 'src/entities/scheduler';
import type { ParamDraft, KeyValueDraftRow } from '../model/param-draft';

import { useState } from 'react';

import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Switch from '@mui/material/Switch';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import FormControlLabel from '@mui/material/FormControlLabel';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { PARAM_WIDGET } from 'src/entities/scheduler';

import {
  updateJsonDraft,
  updateKeyValueDraft,
  isParamFieldDisabled,
  updateParamDraftValue,
} from '../model/param-draft';

type ParamFieldProps = {
  field: ParamFieldSpec;
  draft: ParamDraft;
  onChange: (draft: ParamDraft) => void;
};

type KeyValueRowProps = {
  row: KeyValueDraftRow;
  disabled: boolean;
  onChange: (row: KeyValueDraftRow) => void;
  onRemove: () => void;
};

export function ParamFields(props: {
  fields: ParamFieldSpec[];
  draft: ParamDraft;
  onChange: (draft: ParamDraft) => void;
}) {
  return (
    <Stack spacing={2}>
      {props.fields.map((field) => (
        <ParamField key={field.path} field={field} draft={props.draft} onChange={props.onChange} />
      ))}
    </Stack>
  );
}

function ParamField(props: ParamFieldProps) {
  switch (props.field.widget) {
    case PARAM_WIDGET.KEY_VALUE:
      return <KeyValueParamField {...props} />;
    case PARAM_WIDGET.JSON_EDITOR:
      return <JsonEditorParamField {...props} />;
    case PARAM_WIDGET.SWITCH:
      return <SwitchParamField {...props} />;
    case PARAM_WIDGET.TEXT:
    case PARAM_WIDGET.NUMBER:
    case PARAM_WIDGET.SELECT:
    case PARAM_WIDGET.TEXTAREA:
      return <ScalarParamField {...props} />;
    default:
      return unsupportedWidget(props.field.widget);
  }
}

function unsupportedWidget(widget: never): never {
  throw new Error(`unsupported param widget: ${String(widget)}`);
}

function ScalarParamField({ field, draft, onChange }: ParamFieldProps) {
  const multiline = field.widget === PARAM_WIDGET.TEXTAREA;
  const select = field.widget === PARAM_WIDGET.SELECT;
  const number = field.widget === PARAM_WIDGET.NUMBER;
  const value = draft.values[field.path];
  return (
    <TextField
      fullWidth
      select={select}
      multiline={multiline}
      minRows={multiline ? 3 : undefined}
      type={number ? 'number' : undefined}
      disabled={isDisabled(field, draft)}
      label={field.label}
      placeholder={field.placeholder ?? undefined}
      helperText={field.help}
      value={number ? numberInputValue(value) : String(value ?? '')}
      onChange={(event) => {
        const nextValue = number ? parseNumberInput(event.target.value) : event.target.value;
        onChange(updateParamDraftValue(draft, field.path, nextValue));
      }}
    >
      {field.options.map((option) => (
        <MenuItem key={option} value={option}>
          {option}
        </MenuItem>
      ))}
    </TextField>
  );
}

function SwitchParamField({ field, draft, onChange }: ParamFieldProps) {
  return (
    <FormControlLabel
      label={field.label}
      control={
        <Switch
          checked={draft.values[field.path] === true}
          disabled={isDisabled(field, draft)}
          onChange={(_, checked) => {
            onChange(updateParamDraftValue(draft, field.path, checked));
          }}
        />
      }
    />
  );
}

function JsonEditorParamField({ field, draft, onChange }: ParamFieldProps) {
  const { t } = useTranslate('scheduler');
  const [invalid, setInvalid] = useState(false);
  const value = draft.json[field.path] ?? '';
  return (
    <TextField
      fullWidth
      multiline
      minRows={4}
      error={invalid}
      disabled={isDisabled(field, draft)}
      label={field.label}
      value={value}
      helperText={invalid ? t('paramErrors.invalidJson') : field.help}
      onBlur={() => setInvalid(!isJson(value))}
      onChange={(event) => {
        setInvalid(false);
        onChange(updateJsonDraft(draft, field.path, event.target.value));
      }}
    />
  );
}

function KeyValueParamField({ field, draft, onChange }: ParamFieldProps) {
  const { t } = useTranslate('scheduler');
  const rows = draft.keyValues[field.path] ?? [];
  const disabled = isDisabled(field, draft);
  const write = (next: readonly KeyValueDraftRow[]) => {
    onChange(updateKeyValueDraft(draft, field.path, next));
  };
  return (
    <Stack spacing={1}>
      {draft.invalidKeyValues.has(field.path) && (
        <Alert severity="error">{t('paramErrors.invalidKeyValue')}</Alert>
      )}
      {rows.map((row, index) => (
        <KeyValueRow
          key={row.id}
          row={row}
          disabled={disabled}
          onChange={(next) => write(replaceRow(rows, index, next))}
          onRemove={() => write(rows.filter((item) => item.id !== row.id))}
        />
      ))}
      <Button
        variant="outlined"
        disabled={disabled}
        onClick={() => write([...rows, { id: crypto.randomUUID(), key: '', value: '' }])}
      >
        {t('addParamRow')}
      </Button>
    </Stack>
  );
}

function KeyValueRow({ row, disabled, onChange, onRemove }: KeyValueRowProps) {
  const { t } = useTranslate('scheduler');
  return (
    <Stack direction="row" spacing={1}>
      <TextField
        disabled={disabled}
        label={t('paramKey')}
        value={row.key}
        onChange={(event) => onChange({ ...row, key: event.target.value })}
      />
      <TextField
        fullWidth
        disabled={disabled}
        label={t('paramValue')}
        value={row.value}
        onChange={(event) => onChange({ ...row, value: event.target.value })}
      />
      <IconButton color="error" disabled={disabled} onClick={onRemove}>
        <Iconify icon="solar:trash-bin-trash-bold" />
      </IconButton>
    </Stack>
  );
}

function isDisabled(field: ParamFieldSpec, draft: ParamDraft) {
  return isParamFieldDisabled(field, draft.values);
}

function numberInputValue(value: unknown) {
  return typeof value === 'number' && Number.isFinite(value) ? value : '';
}

function parseNumberInput(value: string): number | '' {
  if (value === '') return '';
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : '';
}

function isJson(value: string): boolean {
  try {
    JSON.parse(value);
    return true;
  } catch {
    return false;
  }
}

function replaceRow(rows: readonly KeyValueDraftRow[], index: number, value: KeyValueDraftRow) {
  return rows.map((row, rowIndex) => (rowIndex === index ? value : row));
}
