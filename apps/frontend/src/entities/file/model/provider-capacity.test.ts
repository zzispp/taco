import { it, expect, describe } from 'vitest';

import { fileProviderCapacityMetrics } from './provider-capacity';

describe('file provider capacity', () => {
  it('derives used bytes from a bounded provider capacity', () => {
    expect(
      fileProviderCapacityMetrics({
        Bounded: { total_bytes: 1_000, available_bytes: 400 },
      })
    ).toEqual({
      kind: 'bounded',
      usedBytes: 600,
      totalBytes: 1_000,
      availableBytes: 400,
    });
  });

  it('does not fabricate total or available bytes for usage-based capacity', () => {
    expect(fileProviderCapacityMetrics({ UsageBased: { used_bytes: 600 } })).toEqual({
      kind: 'usage-based',
      usedBytes: 600,
    });
  });
});
