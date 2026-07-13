import type React from 'react';
import type { IconifyName } from 'src/shared/ui/iconify';
import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { LocalDateTimeFilterError } from 'src/shared/lib/local-date-time-filter';

export type CrudField<T> = {
  key: keyof T;
  label: string;
  type?: 'text' | 'number' | 'select' | 'textarea' | 'switch' | 'boolean';
  format?: 'dateTime';
  width?: TableHeadCellProps['width'];
  ellipsis?: boolean;
  options?: { value: string; label: string }[];
  disabled?: (context: {
    form: Record<string, unknown>;
    editing: Record<string, unknown> | null;
  }) => boolean;
  hiddenInTable?: boolean;
  hiddenInForm?: boolean;
};

export type CrudFilter = {
  key: string;
  label: string;
  type?: 'text' | 'select' | 'dateTime';
  options?: { value: string; label: string }[];
};

export type CrudPanelProps<T extends Record<string, unknown>, I extends Record<string, unknown>> = {
  title: string;
  addLabel: string;
  idKey: keyof T;
  nameKey: keyof T;
  fields: CrudField<T>[];
  defaultInput: I;
  resource: { items: T[]; total: number; isLoading: boolean };
  page: number;
  rowsPerPage: number;
  filters?: CrudFilter[];
  filterValues?: Record<string, string>;
  filterError?: LocalDateTimeFilterError | null;
  permissionPrefix: string;
  extraActions?: (row: T) => React.ReactNode;
  toolbarAction?: React.ReactNode;
  batchDeleteItems?: (ids: string[]) => Promise<void>;
  isRowSelectable?: (row: T) => boolean;
  onFilterChange?: (filters: Record<string, string>) => void;
  onPageChange: (event: unknown, page: number) => void;
  onRowsPerPageChange: (event: React.ChangeEvent<HTMLInputElement>) => void;
  createItem: (input: I) => Promise<T>;
  updateItem: (id: string, input: I) => Promise<T>;
  deleteItem: (id: string) => Promise<void>;
  onAfterSave?: () => void;
};

export type ActionIconProps = {
  title: string;
  icon: IconifyName;
  disabled?: boolean;
  color?: 'primary' | 'error' | 'warning' | 'info' | 'success';
  onClick: () => void;
};
