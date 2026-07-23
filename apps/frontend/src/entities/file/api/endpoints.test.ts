import { it, expect, describe } from 'vitest';

import { fileEndpoints } from './endpoints';

describe('file media endpoints', () => {
  it('keeps download, preview, and thumbnail URLs separate', () => {
    const id = 'file/with spaces';

    expect(fileEndpoints.content(id)).toBe('/api/system/files/file%2Fwith%20spaces/content');
    expect(fileEndpoints.preview(id)).toBe('/api/system/files/file%2Fwith%20spaces/preview');
    expect(fileEndpoints.thumbnail(id)).toBe('/api/system/files/file%2Fwith%20spaces/thumbnail');
  });

  it('builds a list-authorized directory-trail URL', () => {
    expect(fileEndpoints.directoryTrail('folder/with spaces')).toBe(
      '/api/system/files/folder%2Fwith%20spaces/directory-trail'
    );
  });

  it('builds scoped overview and provider summary endpoints', () => {
    expect(fileEndpoints.overview()).toBe('/api/system/files/overview');
    expect(fileEndpoints.overview('space/with spaces')).toBe(
      '/api/system/files/overview?space_id=space%2Fwith%20spaces'
    );
    expect(fileEndpoints.providers).toBe('/api/system/file-providers');
  });
});
