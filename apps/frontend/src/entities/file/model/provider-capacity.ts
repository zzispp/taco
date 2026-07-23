import type { FileProviderCapacity } from './types';

export type FileProviderCapacityMetrics =
  | Readonly<{
      kind: 'bounded';
      usedBytes: number;
      totalBytes: number;
      availableBytes: number;
    }>
  | Readonly<{
      kind: 'usage-based';
      usedBytes: number;
    }>;

export function fileProviderCapacityMetrics(
  capacity: FileProviderCapacity
): FileProviderCapacityMetrics {
  if ('Bounded' in capacity) {
    return {
      kind: 'bounded',
      usedBytes: capacity.Bounded.total_bytes - capacity.Bounded.available_bytes,
      totalBytes: capacity.Bounded.total_bytes,
      availableBytes: capacity.Bounded.available_bytes,
    };
  }
  return { kind: 'usage-based', usedBytes: capacity.UsageBased.used_bytes };
}
