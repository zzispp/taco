import type { CrudField } from './types';
import type { TableHeadCellProps } from 'src/shared/ui/table';

import Box from '@mui/material/Box';
import Switch from '@mui/material/Switch';
import Tooltip from '@mui/material/Tooltip';

import { fAdminDateTime } from 'src/shared/lib/admin-time';

const TABLE_HEAD_SX = { whiteSpace: 'nowrap' } as const;
const DATE_TIME_CELL_SX = { whiteSpace: 'nowrap' } as const;

type TableHeadOptions<T> = {
  fields: CrudField<T>[];
  hasExtra: boolean;
  hasSelection: boolean;
  actionLabel: string;
};

type DisplayLabels = {
  yes: string;
  no: string;
};

const ELLIPSIS_CELL_SX = {
  display: 'inline-block',
  maxWidth: '100%',
  overflow: 'hidden',
  verticalAlign: 'bottom',
  whiteSpace: 'nowrap',
  textOverflow: 'ellipsis',
} as const;

export function tableHead<T>(options: TableHeadOptions<T>): TableHeadCellProps[] {
  const { fields, hasExtra, hasSelection, actionLabel } = options;
  return [
    ...fields
      .filter((field) => !field.hiddenInTable)
      .map((field) => ({
        id: String(field.key),
        label: field.label,
        width: field.width ?? (isDateTimeField(field) ? 190 : undefined),
        sx: TABLE_HEAD_SX,
      })),
    {
      id: 'actions',
      label: actionLabel,
      align: 'right',
      width: hasExtra || hasSelection ? 144 : 96,
      sx: TABLE_HEAD_SX,
    },
  ];
}

export function toggle(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}

export function formFromRow<T extends Record<string, unknown>, I extends Record<string, unknown>>(
  row: T,
  fields: CrudField<T>[]
) {
  return Object.fromEntries(
    fields.filter((field) => !field.hiddenInForm).map((field) => [field.key, row[field.key] ?? ''])
  ) as I;
}

export function displayField<T>(value: unknown, field: CrudField<T>, labels: DisplayLabels) {
  if (value === null || value === undefined || value === '') return '-';
  if (isDateTimeField(field)) return fAdminDateTime(String(value)) || '-';
  if (field.type === 'switch')
    return <Switch size="small" checked={String(value) === '0'} disabled />;
  if (typeof value === 'boolean') return value ? labels.yes : labels.no;

  const text = String(value);
  if (field.ellipsis) return <EllipsisCellText value={text} />;

  return text;
}

export function fieldCellSx<T>(field: CrudField<T>) {
  if (field.ellipsis) {
    return {
      overflow: 'hidden',
      whiteSpace: 'nowrap',
      width: field.width,
      maxWidth: field.width,
    };
  }

  return isDateTimeField(field) ? DATE_TIME_CELL_SX : undefined;
}

export function normalizeValue(
  value: string | boolean,
  type?: CrudField<Record<string, unknown>>['type']
) {
  if (type === 'number') return Number(value);
  if (type === 'boolean') return Boolean(value);
  if (typeof value === 'boolean') return value ? '0' : '1';
  return value;
}

function isDateTimeField<T>(field: CrudField<T>) {
  return field.format === 'dateTime' || String(field.key) === 'create_time';
}

function EllipsisCellText({ value }: { value: string }) {
  return (
    <Tooltip title={value} arrow>
      <Box component="span" sx={ELLIPSIS_CELL_SX}>
        {value}
      </Box>
    </Tooltip>
  );
}
