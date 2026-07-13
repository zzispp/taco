import type {
  CapturedBytes,
  CapturedHeader,
  HttpExecutionRequest,
  HttpExecutionResponse,
  ExecutionDetailEnvelope,
  HttpExecutionDetailPayload,
} from 'src/entities/scheduler';

import {
  EXECUTION_DETAIL_KIND,
  CAPTURED_BYTES_ENCODING,
  HTTP_EXECUTION_FAILURE_CODE,
  HTTP_EXECUTION_DETAIL_SCHEMA_VERSION,
} from 'src/entities/scheduler';

export const EXECUTION_DETAIL_VIEW_KIND = {
  LEGACY: 'legacy',
  HTTP: 'http',
  UNKNOWN: 'unknown',
} as const;

export const EXECUTION_DETAIL_TAB = {
  OVERVIEW: 'overview',
  PARAMETERS: 'parameters',
  REQUEST: 'request',
  RESPONSE: 'response',
} as const;

export type ExecutionDetailTab = (typeof EXECUTION_DETAIL_TAB)[keyof typeof EXECUTION_DETAIL_TAB];

export type ExecutionDetailView =
  | Readonly<{ kind: 'legacy' }>
  | Readonly<{ kind: 'http'; payload: HttpExecutionDetailPayload }>
  | Readonly<{ kind: 'unknown'; detail: ExecutionDetailEnvelope; raw: string }>;

export function executionDetailView(detail: ExecutionDetailEnvelope | null): ExecutionDetailView {
  if (detail === null) return { kind: EXECUTION_DETAIL_VIEW_KIND.LEGACY };
  if (isHttpExecutionDetail(detail)) {
    return { kind: EXECUTION_DETAIL_VIEW_KIND.HTTP, payload: detail.payload };
  }
  return { kind: EXECUTION_DETAIL_VIEW_KIND.UNKNOWN, detail, raw: formatRawJson(detail) };
}

export function formatRawJson(value: unknown): string {
  return JSON.stringify(value, null, 2) ?? String(value);
}

function isHttpExecutionDetail(
  detail: ExecutionDetailEnvelope
): detail is ExecutionDetailEnvelope & { payload: HttpExecutionDetailPayload } {
  return (
    detail.kind === EXECUTION_DETAIL_KIND.HTTP_EXCHANGE &&
    detail.schema_version === HTTP_EXECUTION_DETAIL_SCHEMA_VERSION &&
    isHttpExecutionPayload(detail.payload)
  );
}

function isHttpExecutionPayload(value: unknown): value is HttpExecutionDetailPayload {
  if (!isRecord(value)) return false;
  if (typeof value.duration_ms !== 'number' || !Number.isFinite(value.duration_ms)) return false;
  if (!isHttpRequest(value.request)) return false;
  if (value.response !== null && !isHttpResponse(value.response)) return false;
  return value.failure === null || isHttpFailure(value.failure);
}

function isHttpRequest(value: unknown): value is HttpExecutionRequest {
  if (!isRecord(value)) return false;
  return (
    typeof value.method === 'string' &&
    typeof value.url === 'string' &&
    isCapturedHeaders(value.headers) &&
    (value.body === null || isCapturedBytes(value.body))
  );
}

function isHttpResponse(value: unknown): value is HttpExecutionResponse {
  if (!isRecord(value)) return false;
  return (
    typeof value.status === 'number' &&
    typeof value.final_url === 'string' &&
    isCapturedHeaders(value.headers) &&
    (value.body === null || isCapturedBytes(value.body))
  );
}

function isCapturedHeaders(value: unknown): value is readonly CapturedHeader[] {
  return Array.isArray(value) && value.every(isCapturedHeader);
}

function isCapturedHeader(value: unknown): value is CapturedHeader {
  return isRecord(value) && typeof value.name === 'string' && isCapturedBytes(value.value);
}

function isCapturedBytes(value: unknown): value is CapturedBytes {
  if (!isRecord(value)) return false;
  const encoding = value.encoding;
  return (
    (encoding === CAPTURED_BYTES_ENCODING.UTF8 || encoding === CAPTURED_BYTES_ENCODING.BASE64) &&
    typeof value.content === 'string' &&
    typeof value.byte_length === 'number'
  );
}

function isHttpFailure(value: unknown) {
  if (!isRecord(value) || typeof value.code !== 'string') return false;
  return Object.values(HTTP_EXECUTION_FAILURE_CODE).some((code) => code === value.code);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
