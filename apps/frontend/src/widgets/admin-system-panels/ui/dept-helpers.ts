import type { TranslateFn } from 'src/shared/i18n';
import type { Dept, DeptInput } from 'src/entities/system';
import type { TableHeadCellProps } from 'src/shared/ui/table';

export type DeptRowView = { dept: Dept; level: number; childCount: number };

const TABLE_HEAD_SX = { whiteSpace: 'nowrap' } as const;

export const DATE_TIME_CELL_SX = { whiteSpace: 'nowrap' } as const;

export function toInput(dept: Dept): DeptInput {
  return {
    parent_id: dept.parent_id,
    dept_name: dept.dept_name,
    order_num: dept.order_num,
    leader: dept.leader,
    phone: dept.phone,
    email: dept.email,
    status: dept.status,
  };
}

export function deptHead(t: TranslateFn): TableHeadCellProps[] {
  return [
    { id: 'dept_name', label: t('fields.deptName') },
    { id: 'order_num', label: t('common.sort') },
    { id: 'leader', label: t('fields.leader') },
    { id: 'phone', label: t('fields.phone') },
    { id: 'email', label: t('common.email') },
    { id: 'status', label: t('common.status') },
    { id: 'create_time', label: t('fields.createTime'), width: 190, sx: TABLE_HEAD_SX },
    { id: 'actions', label: t('common.actions'), align: 'right', width: 132, sx: TABLE_HEAD_SX },
  ];
}

export function toggle(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}

export function flattenDeptRows(depts: Dept[], expanded: string[]) {
  const ids = new Set(depts.map((dept) => dept.dept_id));
  const roots = sortDepts(
    depts.filter((dept) => dept.parent_id === '0' || !ids.has(dept.parent_id))
  );
  return roots.flatMap((dept) => flattenDeptBranch(depts, dept, 0, expanded));
}

function flattenDeptBranch(
  depts: Dept[],
  dept: Dept,
  level: number,
  expanded: string[]
): DeptRowView[] {
  const children = sortDepts(depts.filter((child) => child.parent_id === dept.dept_id));
  const row = { dept, level, childCount: children.length };
  if (!expanded.includes(dept.dept_id)) return [row];
  return [
    row,
    ...children.flatMap((child) => flattenDeptBranch(depts, child, level + 1, expanded)),
  ];
}

function sortDepts(depts: Dept[]) {
  return [...depts].sort((a, b) => a.order_num - b.order_num);
}
