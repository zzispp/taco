import { vi, it, expect, describe, beforeEach } from 'vitest';

import {
  changeNoticePage,
  resetNoticeQuery,
  changeNoticeRowsPerPage,
  updatePageAfterNoticeDelete,
  updatePageAfterNoticeBatchDelete,
} from './table-actions';

const table = {
  onChangePage: vi.fn(),
  onChangeRowsPerPage: vi.fn(),
  onResetPage: vi.fn(),
  onUpdatePageDeleteRow: vi.fn(),
  onUpdatePageDeleteRows: vi.fn(),
  setSelected: vi.fn(),
};

beforeEach(() => vi.clearAllMocks());

describe('notice table actions', () => {
  it('clears selection after page changes', () => {
    changeNoticePage({ table, event: null, page: 2 });
    expect(table.onChangePage).toHaveBeenCalledExactlyOnceWith(null, 2);
    expect(table.setSelected).toHaveBeenCalledExactlyOnceWith([]);
  });

  it('clears selection after page-size changes', () => {
    const event = { target: { value: '25' } } as React.ChangeEvent<HTMLInputElement>;
    changeNoticeRowsPerPage({ table, event });
    expect(table.onChangeRowsPerPage).toHaveBeenCalledExactlyOnceWith(event);
    expect(table.setSelected).toHaveBeenCalledExactlyOnceWith([]);
  });

  it('resets page and selection when the query changes', () => {
    resetNoticeQuery(table);
    expect(table.onResetPage).toHaveBeenCalledOnce();
    expect(table.setSelected).toHaveBeenCalledExactlyOnceWith([]);
  });

  it('delegates single and batch deletion page correction', () => {
    updatePageAfterNoticeDelete(table, 1);
    updatePageAfterNoticeBatchDelete({
      table,
      totalRowsInPage: 10,
      totalRowsFiltered: 21,
    });
    expect(table.onUpdatePageDeleteRow).toHaveBeenCalledExactlyOnceWith(1);
    expect(table.onUpdatePageDeleteRows).toHaveBeenCalledExactlyOnceWith(10, 21);
  });
});
