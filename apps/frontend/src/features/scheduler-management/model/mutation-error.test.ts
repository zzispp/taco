import { it, expect, describe } from 'vitest';

import { schedulerMutationErrorMessage } from './mutation-error';

const FALLBACK_MESSAGE = 'fallback';

describe('scheduler mutation errors', () => {
  it('prefers localized details from a normalized API error', () => {
    const error = Object.assign(new Error('message'), {
      status: 409,
      code: 'scheduler_execution_active',
      details: 'localized details',
    });

    expect(schedulerMutationErrorMessage(error, FALLBACK_MESSAGE)).toBe('localized details');
  });

  it('keeps explicit non-API errors visible', () => {
    expect(schedulerMutationErrorMessage(new Error('download failed'), FALLBACK_MESSAGE)).toBe(
      'download failed'
    );
  });

  it('uses the localized fallback for an unknown thrown value', () => {
    expect(schedulerMutationErrorMessage({ reason: 'unknown' }, FALLBACK_MESSAGE)).toBe(
      FALLBACK_MESSAGE
    );
  });
});
