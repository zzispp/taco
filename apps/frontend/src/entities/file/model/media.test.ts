import { it, expect, describe } from 'vitest';

import { supportsFileThumbnail } from './media';

describe('file thumbnail policy', () => {
  it.each(['image/png', 'image/jpeg', 'image/webp', 'image/gif'])('accepts %s', (mimeType) => {
    expect(supportsFileThumbnail({ type: 'file', mime_type: mimeType })).toBe(true);
  });

  it.each(['image/svg+xml', 'image/avif', 'application/pdf', 'text/plain'])(
    'rejects %s',
    (mimeType) => {
      expect(supportsFileThumbnail({ type: 'file', mime_type: mimeType })).toBe(false);
    }
  );

  it('rejects folders and missing MIME metadata', () => {
    expect(supportsFileThumbnail({ type: 'folder', mime_type: 'image/png' })).toBe(false);
    expect(supportsFileThumbnail({ type: 'file', mime_type: null })).toBe(false);
  });
});
