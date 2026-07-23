import { it, vi, expect, describe } from 'vitest';

import { createObjectUrlLifecycle } from './object-url-lifecycle';

describe('avatar object URL lifecycle', () => {
  it('revokes the previous URL on replacement and the current URL on clear', () => {
    const revokeObjectURL = vi.fn();
    const lifecycle = createObjectUrlLifecycle({
      createObjectURL: vi.fn().mockReturnValueOnce('blob:one').mockReturnValueOnce('blob:two'),
      revokeObjectURL,
    });
    const first = new Blob(['one']);
    const second = new Blob(['two']);

    expect(lifecycle.replace(first)).toBe('blob:one');
    expect(lifecycle.replace(second)).toBe('blob:two');
    lifecycle.clear();
    lifecycle.clear();

    expect(revokeObjectURL.mock.calls).toEqual([['blob:one'], ['blob:two']]);
  });
});
