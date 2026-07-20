import { it, expect, describe } from 'vitest';

import { parseSetupDefaults, parseInstallationState } from './types';

const setupDefaults = {
  postgres: { port: 5_432, use_tls: true },
  redis: { port: 6_379, use_tls: true },
  advanced: {
    http_request_timeout_ms: 30_000,
    compression_enabled: true,
    metrics_enabled: true,
    online_session_cleanup_interval_ms: 60_000,
    online_session_cleanup_batch_size: 1_000,
    audit_outbox_worker_count: 4,
    audit_outbox_claim_batch_size: 64,
    audit_outbox_poll_interval_ms: 250,
    audit_outbox_lease_duration_ms: 30_000,
    audit_outbox_retry_delay_ms: 5_000,
    audit_outbox_cleanup_interval_ms: 3_600_000,
    audit_outbox_cleanup_batch_size: 1_000,
    audit_outbox_processed_retention_days: 7,
    client_ip_location_timeout_ms: 3_000,
    scheduler_http_timeout_ms: 30_000,
    scheduler_reconcile_interval_ms: 1_000,
    redis_key_prefix: 'taco:',
  },
};

describe('installation API contracts', () => {
  it('accepts the setup status states exposed before and after installation', () => {
    expect(parseInstallationState({ state: 'setup' })).toBe('setup');
    expect(parseInstallationState({ state: 'installed' })).toBe('installed');
  });

  it('requires complete backend-provided setup defaults instead of inventing frontend fallbacks', () => {
    expect(parseSetupDefaults(setupDefaults)).toEqual(setupDefaults);
    expect(() => parseSetupDefaults({ ...setupDefaults, advanced: {} })).toThrow();
  });
});
