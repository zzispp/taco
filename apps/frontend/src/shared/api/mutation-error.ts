import { isNormalizedApiError } from './http-client';

export function apiMutationErrorMessage(error: unknown, fallback: string): string {
  if (!isNormalizedApiError(error)) return fallback;
  if (error.details) return error.details;
  return error.status ? error.message : fallback;
}
