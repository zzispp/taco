import { it, expect, describe } from 'vitest';

import { uploadIntentKey, clearUploadIntent, uploadIntentFingerprint } from './upload-intent';

describe('resumable upload intent', () => {
  it('reuses an active key and rotates it after explicit cleanup', () => {
    const fingerprint = `intent-${crypto.randomUUID()}`;
    const first = uploadIntentKey(fingerprint);

    expect(uploadIntentKey(fingerprint)).toBe(first);
    clearUploadIntent(fingerprint);
    expect(uploadIntentKey(fingerprint)).not.toBe(first);
    clearUploadIntent(fingerprint);
  });

  it('includes the normalized content type in the resume identity', () => {
    const base = {
      spaceId: 'user-1',
      parentId: null,
      fileName: 'avatar.png',
      fileSize: 12,
      digest: 'a'.repeat(64),
    };
    expect(uploadIntentFingerprint({ ...base, contentType: 'image/png' })).not.toBe(
      uploadIntentFingerprint({ ...base, contentType: 'application/octet-stream' })
    );
  });
});
