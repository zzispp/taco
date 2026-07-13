import { it, expect, describe } from 'vitest';

import { normalizeApiError } from './http-client';

describe('API error normalization', () => {
  it('preserves status, code, and localized details for notice error handling', () => {
    const normalized = normalizeApiError({
      isAxiosError: true,
      message: 'Request failed',
      response: {
        status: 400,
        data: {
          code: 'invalid_input',
          message: 'Invalid notice',
          details: '公告 Markdown 包含不安全链接',
        },
      },
    });

    expect({
      message: normalized.message,
      status: normalized.status,
      code: normalized.code,
      details: normalized.details,
    }).toEqual({
      message: '公告 Markdown 包含不安全链接',
      status: 400,
      code: 'invalid_input',
      details: '公告 Markdown 包含不安全链接',
    });
  });
});
