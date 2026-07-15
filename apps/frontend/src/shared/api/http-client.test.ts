import { it, expect, describe } from 'vitest';

import { normalizeApiError, normalizeApiErrorAsync, type NormalizedApiError } from './http-client';

type ApiErrorData = {
  code: string;
  message: string;
  details: string;
};

type ExpectedNormalizedError = {
  message: string;
  status: number;
  code: string;
  details: string;
};

function axiosErrorFixture(status: number, data: ApiErrorData | Blob) {
  return {
    isAxiosError: true,
    message: 'Request failed',
    response: { status, data },
  };
}

function expectNormalizedError(error: NormalizedApiError, expected: ExpectedNormalizedError) {
  expect({
    message: error.message,
    status: error.status,
    code: error.code,
    details: error.details,
  }).toEqual(expected);
}

describe('API error normalization', () => {
  it('preserves status, code, and localized details for notice error handling', () => {
    const normalized = normalizeApiError(
      axiosErrorFixture(400, {
        code: 'invalid_input',
        message: 'Invalid notice',
        details: '公告 Markdown 包含不安全链接',
      })
    );

    expectNormalizedError(normalized, {
      message: '公告 Markdown 包含不安全链接',
      status: 400,
      code: 'invalid_input',
      details: '公告 Markdown 包含不安全链接',
    });
  });

  it('decodes a JSON Blob error response without losing status, code, or details', async () => {
    const data = {
      code: 'invalid_export_filter',
      message: 'Invalid export filter',
      details: '结束时间不能早于开始时间',
    };
    const normalized = await normalizeApiErrorAsync(
      axiosErrorFixture(422, new Blob([JSON.stringify(data)], { type: 'application/json' }))
    );

    expectNormalizedError(normalized, {
      message: '结束时间不能早于开始时间',
      status: 422,
      code: 'invalid_export_filter',
      details: '结束时间不能早于开始时间',
    });
  });
});
