import type { UseTableReturn } from 'src/shared/ui/table';

type AuditTableSort = Pick<UseTableReturn, 'setOrderBy' | 'setOrder'>;
type AuditMutationTable = Pick<UseTableReturn, 'onResetCursor' | 'setSelected'>;

export function resetAuditTableSort(table: AuditTableSort, defaultOrderBy: string) {
  table.setOrderBy(defaultOrderBy);
  table.setOrder('desc');
}

export function resetAuditMutationCursor(table: AuditMutationTable) {
  table.onResetCursor();
  table.setSelected([]);
}
