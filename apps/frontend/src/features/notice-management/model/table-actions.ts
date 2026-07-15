import type { UseTableReturn } from 'src/shared/ui/table';

type NoticeTable = Pick<UseTableReturn, 'onResetCursor' | 'setSelected'>;

export function resetNoticeQuery(table: NoticeTable) {
  table.onResetCursor();
  table.setSelected([]);
}
