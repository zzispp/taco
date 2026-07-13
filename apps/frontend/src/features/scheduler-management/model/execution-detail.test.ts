import type { CapturedBytes, ExecutionDetailEnvelope } from 'src/entities/scheduler';

import { it, expect, describe } from 'vitest';

import {
  EXECUTION_DETAIL_KIND,
  CAPTURED_BYTES_ENCODING,
  HTTP_EXECUTION_DETAIL_SCHEMA_VERSION,
} from 'src/entities/scheduler';

import { formatRawJson, executionDetailView, EXECUTION_DETAIL_VIEW_KIND } from './execution-detail';

const RAW_TEST_LENGTH = 512;

describe('scheduler execution detail projection', () => {
  it('projects legacy null explicitly', () => {
    expect(executionDetailView(null)).toEqual({ kind: EXECUTION_DETAIL_VIEW_KIND.LEGACY });
  });

  it('keeps a null HTTP response as a supported exchange', () => {
    const view = executionDetailView(httpDetail(null));

    expect(view.kind).toBe(EXECUTION_DETAIL_VIEW_KIND.HTTP);
    if (view.kind === EXECUTION_DETAIL_VIEW_KIND.HTTP) {
      expect(view.payload.response).toBeNull();
    }
  });

  it.each([
    [CAPTURED_BYTES_ENCODING.UTF8, 'plain text'],
    [CAPTURED_BYTES_ENCODING.BASE64, 'AAEC/w=='],
  ] as const)('preserves complete %s captured content', (encoding, content) => {
    const detail = httpDetail({
      status: 200,
      final_url: 'https://example.test/final',
      headers: [],
      body: capturedBytes(encoding, content),
    });
    const view = executionDetailView(detail);

    expect(view.kind).toBe(EXECUTION_DETAIL_VIEW_KIND.HTTP);
    if (view.kind === EXECUTION_DETAIL_VIEW_KIND.HTTP) {
      expect(view.payload.response?.body?.content).toBe(content);
    }
  });

  it('renders unknown detail kinds as their complete raw envelope', () => {
    const detail = {
      kind: 'future_exchange',
      schema_version: 7,
      payload: { raw: 'x'.repeat(RAW_TEST_LENGTH) },
    };
    const view = executionDetailView(detail);

    expect(view).toEqual({
      kind: EXECUTION_DETAIL_VIEW_KIND.UNKNOWN,
      detail,
      raw: formatRawJson(detail),
    });
    expect(view.kind === EXECUTION_DETAIL_VIEW_KIND.UNKNOWN && view.raw).toContain(
      'x'.repeat(RAW_TEST_LENGTH)
    );
  });
});

function httpDetail(response: unknown): ExecutionDetailEnvelope {
  return {
    kind: EXECUTION_DETAIL_KIND.HTTP_EXCHANGE,
    schema_version: HTTP_EXECUTION_DETAIL_SCHEMA_VERSION,
    payload: {
      duration_ms: 18,
      request: {
        method: 'POST',
        url: 'https://example.test/start',
        headers: [],
        body: null,
      },
      response,
      failure: null,
    },
  };
}

function capturedBytes(encoding: CapturedBytes['encoding'], content: string): CapturedBytes {
  return { encoding, content, byte_length: content.length };
}
