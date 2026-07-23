import { it, vi, expect, describe } from 'vitest';

import { persistCroppedAvatar } from './avatar-persistence';

describe('avatar persistence', () => {
  it('always uploads the cropped result as a new avatar file', async () => {
    const cropped = new Blob(['cropped'], { type: 'image/png' });
    const crop = vi.fn().mockResolvedValue(cropped);
    const upload = vi.fn().mockResolvedValue(undefined);
    const croppedArea = { x: 1, y: 2, width: 80, height: 80 };

    await persistCroppedAvatar(
      { imageSrc: 'blob:source-asset', croppedArea, rotation: 90 },
      { crop, upload }
    );

    expect(crop).toHaveBeenCalledExactlyOnceWith('blob:source-asset', croppedArea, 90);
    expect(upload).toHaveBeenCalledExactlyOnceWith(cropped, 'avatar.png');
  });
});
