import { it, expect, describe } from 'vitest';

import {
  requireNoticeDeleteTarget,
  NOTICE_DELETE_TARGET_REQUIRED_ERROR,
} from './mutation-preconditions';

describe('notice mutation preconditions', () => {
  it('rejects a notice delete without a selected target', () => {
    expect(() => requireNoticeDeleteTarget(null)).toThrowError(NOTICE_DELETE_TARGET_REQUIRED_ERROR);
  });

  it('preserves the selected notice delete target', () => {
    const target = { notice_id: 'notice-1' };

    expect(requireNoticeDeleteTarget(target)).toBe(target);
  });

  it('does not confuse a non-null target with a missing target', () => {
    expect(requireNoticeDeleteTarget(0)).toBe(0);
  });
});
