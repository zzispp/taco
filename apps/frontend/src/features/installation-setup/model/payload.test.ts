import { it, expect, describe } from 'vitest';

import { buildInstallationRequest } from './payload';
import { createSetupFormValues } from './form-values';

const defaults = {
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
} as const;

describe('installation request payload', () => {
  it('omits empty Redis options and all advanced overrides until an operator changes one', () => {
    const values = createSetupFormValues(defaults);
    values.postgres = {
      host: ' postgres.internal ',
      port: 5_432,
      username: ' taco ',
      password: 'postgres-secret',
      database: ' taco ',
      use_tls: true,
    };
    values.redis.host = ' redis.internal ';
    values.administrator = {
      username: ' owner ',
      email: ' owner@example.test ',
      password: ' owner-secret ',
      password_confirmation: ' owner-secret ',
    };

    expect(buildInstallationRequest(values, { advancedKeys: [] })).toEqual({
      postgres: {
        host: 'postgres.internal',
        port: 5_432,
        username: 'taco',
        password: 'postgres-secret',
        database: 'taco',
        use_tls: true,
      },
      redis: { host: 'redis.internal', port: 6_379, use_tls: true },
      administrator: {
        username: 'owner',
        email: 'owner@example.test',
        password: 'owner-secret',
      },
      advanced: {},
    });
  });

  it('submits only selected advanced override fields and preserves Redis database zero', () => {
    const values = createSetupFormValues(defaults);
    values.postgres.host = 'postgres.internal';
    values.postgres.username = 'taco';
    values.postgres.password = 'secret';
    values.postgres.database = 'taco';
    values.redis.host = 'redis.internal';
    values.redis.database = '0';
    values.administrator.username = 'owner';
    values.administrator.email = 'owner@example.test';
    values.administrator.password = 'owner-secret';

    const payload = buildInstallationRequest(values, {
      advancedKeys: ['http_request_timeout_ms', 'redis_key_prefix'],
    });

    expect(payload.redis.database).toBe(0);
    expect(payload.advanced).toEqual({
      http_request_timeout_ms: 30_000,
      redis_key_prefix: 'taco:',
    });
  });
});
