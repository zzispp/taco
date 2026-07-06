import type { TranslateFn } from 'src/shared/i18n';
import type { LabelColor } from 'src/shared/ui/label';
import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { DictData, DictType, DictDataInput, DictTypeInput } from 'src/entities/system';

import { DEFAULT_DATA_INPUT, DEFAULT_TYPE_INPUT } from './dict-constants';

const TABLE_HEAD_SX = { whiteSpace: 'nowrap' } as const;

export const DATE_TIME_CELL_SX = { whiteSpace: 'nowrap' } as const;

export function labelColor(value: string | null): LabelColor {
  if (value === 'primary' || value === 'success' || value === 'info' || value === 'warning') {
    return value;
  }
  if (value === 'danger') return 'error';
  return 'default';
}

export function listClassOptions() {
  return ['default', 'primary', 'success', 'info', 'warning', 'danger'];
}

export function openTypeEdit(
  item: DictType,
  setEditing: (item: DictType) => void,
  setForm: (item: DictTypeInput) => void
) {
  setEditing(item);
  setForm({
    dict_name: item.dict_name,
    dict_type: item.dict_type,
    status: item.status,
    remark: item.remark,
  });
}

export function openDataEdit(
  item: DictData,
  setEditing: (item: DictData) => void,
  setForm: (item: DictDataInput) => void
) {
  setEditing(item);
  setForm({
    dict_sort: item.dict_sort,
    dict_label: item.dict_label,
    dict_value: item.dict_value,
    dict_type: item.dict_type,
    css_class: item.css_class,
    list_class: item.list_class,
    is_default: item.is_default,
    status: item.status,
    remark: item.remark,
  });
}

export function closeTypeDialog(
  setEditing: (item: DictType | null) => void,
  setCreating: (value: boolean) => void,
  setForm: (item: DictTypeInput) => void
) {
  setEditing(null);
  setCreating(false);
  setForm(DEFAULT_TYPE_INPUT);
}

export function closeDataDialog(
  setEditing: (item: DictData | null) => void,
  setCreating: (value: boolean) => void,
  setForm: (item: DictDataInput) => void
) {
  setEditing(null);
  setCreating(false);
  setForm(DEFAULT_DATA_INPUT);
}

export function dictTypeHead(t: TranslateFn): TableHeadCellProps[] {
  return [
    { id: 'dict_name', label: t('fields.dictName') },
    { id: 'dict_type', label: t('fields.dictType') },
    { id: 'status', label: t('common.status') },
    { id: 'remark', label: t('common.remark') },
    { id: 'create_time', label: t('fields.createTime'), width: 190, sx: TABLE_HEAD_SX },
    { id: 'actions', label: t('common.actions'), align: 'right', width: 96, sx: TABLE_HEAD_SX },
  ];
}

export function dictDataHead(t: TranslateFn): TableHeadCellProps[] {
  return [
    { id: 'dict_sort', label: t('fields.dictSort') },
    { id: 'dict_label', label: t('fields.dictLabel') },
    { id: 'dict_value', label: t('fields.dictValue') },
    { id: 'is_default', label: t('fields.isDefault') },
    { id: 'status', label: t('common.status') },
    { id: 'remark', label: t('common.remark') },
    { id: 'create_time', label: t('fields.createTime'), width: 190, sx: TABLE_HEAD_SX },
    { id: 'actions', label: t('common.actions'), align: 'right', width: 96, sx: TABLE_HEAD_SX },
  ];
}

export function toggle(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}
