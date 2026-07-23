import { it, vi, expect, describe } from 'vitest';

import { createFileObjectUrlLifecycle } from './object-url-lifecycle';

describe('file object URL lifecycle', () => {
  it('revokes replaced and cleared object URLs exactly once', () => {
    const revokeObjectURL = vi.fn();
    const api = {
      createObjectURL: vi.fn().mockReturnValueOnce('blob:first').mockReturnValueOnce('blob:second'),
      revokeObjectURL,
    };
    const lifecycle = createFileObjectUrlLifecycle(api);

    expect(lifecycle.replace(new Blob(['first']))).toBe('blob:first');
    expect(lifecycle.replace(new Blob(['second']))).toBe('blob:second');
    lifecycle.clear();
    lifecycle.clear();

    expect(revokeObjectURL.mock.calls).toEqual([['blob:first'], ['blob:second']]);
  });
});
