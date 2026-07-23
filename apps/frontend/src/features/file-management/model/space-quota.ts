import { BYTES_PER_GIB } from './constants';

export const MIN_QUOTA_GIB = 0.001;
export const QUOTA_STEP_GIB = 0.001;

export function quotaGibToBytes(value: string): number | null {
  const gibibytes = Number(value);
  if (!Number.isFinite(gibibytes) || gibibytes <= 0) return null;
  const bytes = Math.round(gibibytes * BYTES_PER_GIB);
  return Number.isSafeInteger(bytes) ? bytes : null;
}

export function quotaBytesToGib(value: number): string {
  return String(Number((value / BYTES_PER_GIB).toFixed(3)));
}
