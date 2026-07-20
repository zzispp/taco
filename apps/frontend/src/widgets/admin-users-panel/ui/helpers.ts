import type { TranslateFn } from 'src/shared/i18n';
import type { RoleOption } from 'src/entities/role';
import type { SystemUser } from 'src/entities/user';
import type { TreeSelectNode } from 'src/entities/system';
import type { TableHeadCellProps } from 'src/shared/ui/table';

import { toast } from 'src/shared/ui/snackbar';

import { translatedRoleName } from 'src/entities/role';

export const USER_CELL_SX = { whiteSpace: 'nowrap' } as const;
export const USER_ELLIPSIS_CELL_SX = {
  whiteSpace: 'nowrap',
  maxWidth: 220,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
} as const;

const USER_HEAD_SX = { whiteSpace: 'nowrap' } as const;

export type FlatNode = { id: string; label: string };
export type SelectOptionItem = { id: string; label: string };

export function toInput(user: SystemUser) {
  return {
    username: user.username,
    password: '',
    nick_name: user.nick_name,
    dept_id: user.dept_id,
    email: user.email,
    phonenumber: user.phonenumber,
    sex: user.sex,
    status: user.status,
    remark: user.remark,
    role_ids: user.role_ids,
    post_ids: user.post_ids,
  };
}

export function userHead(t: TranslateFn): TableHeadCellProps[] {
  return [
    { id: 'username', label: t('common.username'), width: 120, sx: USER_HEAD_SX },
    { id: 'nick_name', label: t('fields.nickName'), width: 120, sx: USER_HEAD_SX },
    { id: 'dept_id', label: t('fields.deptName'), width: 140, sx: USER_HEAD_SX },
    { id: 'phonenumber', label: t('fields.phone'), width: 140, sx: USER_HEAD_SX },
    { id: 'email', label: t('common.email'), width: 220, sx: USER_HEAD_SX },
    { id: 'sex', label: t('fields.sex'), width: 80, sx: USER_HEAD_SX },
    { id: 'status', label: t('common.status'), width: 90, sx: USER_HEAD_SX },
    { id: 'post_ids', label: t('fields.postName'), width: 140, sx: USER_HEAD_SX },
    { id: 'role_ids', label: t('common.role'), width: 160, sx: USER_HEAD_SX },
    { id: 'create_time', label: t('fields.createTime'), width: 190, sx: USER_HEAD_SX },
    { id: 'actions', label: t('common.actions'), align: 'right', width: 176, sx: USER_HEAD_SX },
  ];
}

export function toggle(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}

export function sexLabel(value: string, t: TranslateFn) {
  return value === '0'
    ? t('common.male')
    : value === '1'
      ? t('common.female')
      : t('common.unknown');
}

export function displayRoles(ids: string[], roles: RoleOption[], t: TranslateFn) {
  return (
    ids
      .map((id) => roles.find((role) => role.role_id === id))
      .filter(Boolean)
      .map((role) => translatedRoleName(role as RoleOption))
      .join(', ') || '-'
  );
}

export function namesByIds<T>(items: T[], ids: string[], idKey: keyof T, nameKey: keyof T) {
  return (
    ids.map((id) => String(items.find((item) => item[idKey] === id)?.[nameKey] ?? id)).join(', ') ||
    '-'
  );
}

export function nameById(items: FlatNode[], id: string | null) {
  return id ? (items.find((item) => item.id === id)?.label ?? id) : '-';
}

export function showError(t: TranslateFn) {
  return (error: unknown) =>
    toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
}

export function flattenTree(nodes: TreeSelectNode[], level = 0): FlatNode[] {
  return nodes.flatMap((node) => [
    { id: node.id, label: `${'　'.repeat(level)}${node.label}` },
    ...flattenTree(node.children, level + 1),
  ]);
}

export function flattenDeptNames(nodes: TreeSelectNode[]): FlatNode[] {
  return nodes.flatMap((node) => [
    { id: node.id, label: node.label },
    ...flattenDeptNames(node.children),
  ]);
}

export function filterDeptTree(nodes: TreeSelectNode[], keyword: string): TreeSelectNode[] {
  const term = keyword.trim().toLowerCase();
  if (!term) return nodes;
  return nodes.flatMap((node) => {
    const children = filterDeptTree(node.children, term);
    if (node.label.toLowerCase().includes(term) || children.length > 0)
      return [{ ...node, children }];
    return [];
  });
}
