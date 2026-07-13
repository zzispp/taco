import type { UseTableReturn } from 'src/shared/ui/table';

type NoticeTable = Pick<
  UseTableReturn,
  | 'onChangePage'
  | 'onChangeRowsPerPage'
  | 'onResetPage'
  | 'onUpdatePageDeleteRow'
  | 'onUpdatePageDeleteRows'
  | 'setSelected'
>;

export function changeNoticePage(options: { table: NoticeTable; event: unknown; page: number }) {
  options.table.onChangePage(options.event, options.page);
  options.table.setSelected([]);
}

export function changeNoticeRowsPerPage(options: {
  table: NoticeTable;
  event: React.ChangeEvent<HTMLInputElement>;
}) {
  options.table.onChangeRowsPerPage(options.event);
  options.table.setSelected([]);
}

export function resetNoticeQuery(table: NoticeTable) {
  table.onResetPage();
  table.setSelected([]);
}

export function updatePageAfterNoticeDelete(table: NoticeTable, totalRowsInPage: number) {
  table.onUpdatePageDeleteRow(totalRowsInPage);
}

export function updatePageAfterNoticeBatchDelete(options: {
  table: NoticeTable;
  totalRowsInPage: number;
  totalRowsFiltered: number;
}) {
  options.table.onUpdatePageDeleteRows(options.totalRowsInPage, options.totalRowsFiltered);
}
