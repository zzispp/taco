import { it, expect, describe } from 'vitest';

import { BYTES_PER_GIB } from './constants';
import { quotaBytesToGib, quotaGibToBytes } from './space-quota';

describe('file space quota conversion', () => {
  it('converts GiB input to an integer byte payload', () => {
    expect(quotaGibToBytes('20')).toBe(20 * BYTES_PER_GIB);
    expect(quotaGibToBytes('1.5')).toBe(1.5 * BYTES_PER_GIB);
  });

  it('rejects invalid or unsafe quota values', () => {
    expect(quotaGibToBytes('0')).toBeNull();
    expect(quotaGibToBytes('-1')).toBeNull();
    expect(quotaGibToBytes('not-a-number')).toBeNull();
    expect(quotaGibToBytes(String(Number.MAX_SAFE_INTEGER))).toBeNull();
  });

  it('formats the current byte quota for editing', () => {
    expect(quotaBytesToGib(20 * BYTES_PER_GIB)).toBe('20');
    expect(quotaBytesToGib(1.5 * BYTES_PER_GIB)).toBe('1.5');
  });
});
