import { vi, it, expect, describe, beforeEach } from 'vitest';

import { resetNoticeQuery } from './table-actions';

const table = {
  onResetCursor: vi.fn(),
  setSelected: vi.fn(),
};

beforeEach(() => vi.clearAllMocks());

describe('notice cursor actions', () => {
  it('resets the cursor and current-batch selection when the query changes', () => {
    resetNoticeQuery(table);

    expect(table.onResetCursor).toHaveBeenCalledExactlyOnceWith();
    expect(table.setSelected).toHaveBeenCalledExactlyOnceWith([]);
  });
});
