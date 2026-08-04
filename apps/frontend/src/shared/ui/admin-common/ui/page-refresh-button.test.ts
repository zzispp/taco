import { vi, it, expect, describe } from 'vitest';

import { runPageRefresh } from './page-refresh-button';

describe('page refresh button behavior', () => {
  it('keeps the pending state until the refresh callback completes', async () => {
    let resolveRefresh: (() => void) | undefined;
    const refresh = new Promise<void>((resolve) => {
      resolveRefresh = resolve;
    });
    const onPendingChange = vi.fn();
    const execution = runPageRefresh({
      onRefresh: () => refresh,
      onPendingChange,
      onError: vi.fn(),
    });

    expect(onPendingChange).toHaveBeenCalledTimes(1);
    expect(onPendingChange).toHaveBeenLastCalledWith(true);

    resolveRefresh?.();
    await execution;

    expect(onPendingChange).toHaveBeenLastCalledWith(false);
  });

  it('reports the exact failure and clears the pending state', async () => {
    const failure = new Error('refresh failed');
    const onError = vi.fn();
    const onPendingChange = vi.fn();

    await runPageRefresh({
      onRefresh: () => Promise.reject(failure),
      onPendingChange,
      onError,
    });

    expect(onError).toHaveBeenCalledWith(failure);
    expect(onPendingChange.mock.calls).toEqual([[true], [false]]);
  });
});
