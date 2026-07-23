import { it, vi, expect, describe } from 'vitest';

import { replaceDocumentLocation } from './document-navigation';

describe('replaceDocumentLocation', () => {
  it('uses browser document replacement for a localized destination', () => {
    const replace = vi.fn();
    const destination = '/en/auth/sign-in/?return_to=dashboard#security';

    replaceDocumentLocation({ replace }, destination);

    expect(replace).toHaveBeenCalledExactlyOnceWith(destination);
  });
});
