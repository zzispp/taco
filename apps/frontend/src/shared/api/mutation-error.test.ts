import { it, expect, describe } from 'vitest';

import { apiMutationErrorMessage } from './mutation-error';

const FALLBACK = 'localized fallback';

describe('API mutation errors', () => {
  it('prefers localized API details', () => {
    const error = Object.assign(new Error('message'), {
      status: 409,
      code: 'conflict',
      details: 'localized details',
    });

    expect(apiMutationErrorMessage(error, FALLBACK)).toBe('localized details');
  });

  it('uses the localized fallback for transport and unknown errors', () => {
    const transportError = Object.assign(new Error('Network Error'), {
      status: undefined,
      code: undefined,
      details: undefined,
    });

    expect(apiMutationErrorMessage(transportError, FALLBACK)).toBe(FALLBACK);
    expect(apiMutationErrorMessage(new Error('raw failure'), FALLBACK)).toBe(FALLBACK);
  });
});
