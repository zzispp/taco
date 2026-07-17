import { vi, it, expect, describe, afterEach } from 'vitest';

import { loadCleanupExecutionId, storeCleanupExecutionId } from './cleanup-execution-storage';

describe('system log cleanup execution storage', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('restores an active execution and removes it only after terminal handling', () => {
    const storage = storageSpy();
    storage.getItem.mockReturnValue('execution-1');

    expect(loadCleanupExecutionId()).toBe('execution-1');

    storeCleanupExecutionId(null);

    expect(storage.removeItem).toHaveBeenCalledWith('system-log-cleanup-execution-id');
  });
});

function storageSpy() {
  const sessionStorage = {
    getItem: vi.fn(),
    setItem: vi.fn(),
    removeItem: vi.fn(),
  };
  vi.stubGlobal('window', { sessionStorage });
  return sessionStorage;
}
