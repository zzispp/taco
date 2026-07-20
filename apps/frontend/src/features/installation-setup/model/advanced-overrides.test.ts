import { it, expect, describe } from 'vitest';

import { getDirtyAdvancedKeys } from './advanced-overrides';

describe('dirty advanced setup keys', () => {
  it('returns no overrides while advanced settings are collapsed', () => {
    expect(
      getDirtyAdvancedKeys(
        {
          http_request_timeout_ms: true,
          redis_key_prefix: true,
        },
        false
      )
    ).toEqual([]);
  });

  it('returns only dirty keys in their submission order when advanced settings are open', () => {
    expect(
      getDirtyAdvancedKeys(
        {
          compression_enabled: false,
          audit_outbox_worker_count: true,
          redis_key_prefix: true,
          http_request_timeout_ms: true,
        },
        true
      )
    ).toEqual(['http_request_timeout_ms', 'audit_outbox_worker_count', 'redis_key_prefix']);
  });
});
