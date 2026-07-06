import type { TranslateFn } from 'src/shared/i18n';
import type { Role, RoleInput } from 'src/entities/role';
import type { TableHeadCellProps } from 'src/shared/ui/table';

import { toast } from 'src/shared/ui/snackbar';

const TABLE_HEAD_SX = { whiteSpace: 'nowrap' } as const;

export function deptBindingIds(selected: string[], resolved: string[], strict: boolean) {
  return strict ? resolved : selected;
}

export function toInput(role: Role): RoleInput {
  return {
    role_name: role.role_name,
    role_key: role.role_key,
    role_sort: role.role_sort,
    data_scope: role.data_scope,
    menu_check_strictly: role.menu_check_strictly,
    dept_check_strictly: role.dept_check_strictly,
    status: role.status,
    remark: role.remark,
  };
}

export function showError(t: TranslateFn) {
  return (error: unknown) =>
    toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
}

export function roleHead(t: TranslateFn): TableHeadCellProps[] {
  return [
    { id: 'role_name', label: t('fields.roleName') },
    { id: 'role_key', label: t('fields.roleKey') },
    { id: 'role_sort', label: t('fields.roleSort') },
    { id: 'data_scope', label: t('fields.dataScope') },
    { id: 'status', label: t('common.status') },
    { id: 'system', label: t('common.type') },
    { id: 'create_time', label: t('fields.createTime'), width: 190, sx: TABLE_HEAD_SX },
    { id: 'actions', label: t('common.actions'), align: 'right', width: 220, sx: TABLE_HEAD_SX },
  ];
}

export function toggle(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}
