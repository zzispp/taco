import { it, expect, describe } from 'vitest';

import { noticeManagementCapabilities } from './permissions';

const BASE_PERMISSIONS = {
  canList: false,
  canQuery: false,
  canAdd: false,
  canEdit: false,
  canRemove: false,
};

describe('notice management capabilities', () => {
  it('uses query permission for managed detail', () => {
    const capabilities = noticeManagementCapabilities({
      ...BASE_PERMISSIONS,
      canQuery: true,
    });
    expect(capabilities.canOpenDetail).toBe(true);
    expect(capabilities.canViewReaders).toBe(false);
  });

  it('uses list permission for reader visibility', () => {
    const capabilities = noticeManagementCapabilities({
      ...BASE_PERMISSIONS,
      canList: true,
    });
    expect(capabilities.canOpenDetail).toBe(false);
    expect(capabilities.canViewReaders).toBe(true);
  });
});
