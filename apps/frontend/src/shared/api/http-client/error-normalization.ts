import type { AxiosResponse } from 'axios';

import axios from 'axios';

type ApiErrorPayload = {
  code?: unknown;
  message?: unknown;
  details?: unknown;
};

export type NormalizedApiError = Error & {
  status?: number;
  code?: string;
  details?: string;
};

export function normalizeApiError(error: unknown): NormalizedApiError {
  const response = apiErrorResponse(error);
  return normalizedApiError(error, response, apiErrorPayload(response?.data));
}

export async function normalizeApiErrorAsync(error: unknown): Promise<NormalizedApiError> {
  const response = apiErrorResponse(error);
  const data = await responseApiErrorPayload(response);
  return normalizedApiError(error, response, data);
}

export function isNormalizedApiError(error: unknown): error is NormalizedApiError {
  return (
    error instanceof Error &&
    Object.hasOwn(error, 'status') &&
    Object.hasOwn(error, 'code') &&
    Object.hasOwn(error, 'details')
  );
}

export function isAuthSessionRejected(error: unknown): boolean {
  if (typeof error !== 'object' || error === null) return false;

  const { status, code } = error as { status?: number; code?: string };
  return status === UNAUTHORIZED_STATUS || code === UNAUTHORIZED_CODE;
}

const UNAUTHORIZED_STATUS = 401;
const UNAUTHORIZED_CODE = 'unauthorized';

function normalizedApiError(
  error: unknown,
  response: AxiosResponse | undefined,
  data: ApiErrorPayload | undefined
): NormalizedApiError {
  const details = stringValue(data?.details);
  const message = details ?? stringValue(data?.message) ?? axiosErrorMessage(error);
  const normalizedError = new Error(message) as NormalizedApiError;

  Object.assign(normalizedError, {
    status: response?.status,
    code: stringValue(data?.code),
    details,
  });

  return normalizedError;
}

function apiErrorResponse(error: unknown) {
  return axios.isAxiosError(error) ? error.response : undefined;
}

async function responseApiErrorPayload(response: AxiosResponse | undefined) {
  const data = response?.data;
  return isBlob(data) ? blobApiErrorPayload(data) : apiErrorPayload(data);
}

async function blobApiErrorPayload(blob: Blob): Promise<ApiErrorPayload | undefined> {
  try {
    return apiErrorPayload(JSON.parse(await blob.text()));
  } catch {
    return undefined;
  }
}

function apiErrorPayload(value: unknown): ApiErrorPayload | undefined {
  return isRecord(value) ? (value as ApiErrorPayload) : undefined;
}

function isBlob(value: unknown): value is Blob {
  return typeof Blob !== 'undefined' && value instanceof Blob;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function axiosErrorMessage(error: unknown): string {
  return error instanceof Error && error.message ? error.message : 'Something went wrong!';
}

function stringValue(value: unknown): string | undefined {
  return typeof value === 'string' && value.length > 0 ? value : undefined;
}
