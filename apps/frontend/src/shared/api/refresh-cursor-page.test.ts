import { vi, it, expect, describe } from 'vitest';

import { refreshCursorPage } from './refresh-cursor-page';

describe('cursor page refresh', () => {
  it('revalidates the current request on the first page', async () => {
    const resetCursor = vi.fn();
    const refresh = vi.fn().mockResolvedValue(undefined);

    await refreshCursorPage({ cursor: null, resetCursor, refresh });

    expect(refresh).toHaveBeenCalledOnce();
    expect(resetCursor).not.toHaveBeenCalled();
  });

  it('drops a stale cursor instead of revalidating its old boundary', async () => {
    const resetCursor = vi.fn();
    const refresh = vi.fn().mockResolvedValue(undefined);

    await refreshCursorPage({ cursor: 'stale-cursor', resetCursor, refresh });

    expect(resetCursor).toHaveBeenCalledOnce();
    expect(refresh).not.toHaveBeenCalled();
  });
});
