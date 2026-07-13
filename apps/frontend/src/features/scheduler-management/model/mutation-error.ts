import { isNormalizedApiError } from 'src/shared/api/http-client';

export function schedulerMutationErrorMessage(error: unknown, fallback: string): string {
  if (isNormalizedApiError(error)) {
    return error.details || error.message || fallback;
  }
  return error instanceof Error && error.message ? error.message : fallback;
}
