import type { TranslateFn } from 'src/shared/i18n';

import { it, expect, describe } from 'vitest';

import { createSetupFormSchema } from './schema';
import { createSetupFormValues } from './form-values';

const PASSWORD_LENGTH_ERROR = 'auth.validation.passwordLength:8:128';
const PASSWORD_USERNAME_ERROR = 'auth.validation.passwordContainsUsername';

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

describe('setup administrator validation', () => {
  it('trims the password before matching confirmation and accepts the 8-character baseline', () => {
    const parsed = schema().safeParse(setupValues(' safe-pass-8 ', 'safe-pass-8'));
    const numericPassword = schema().safeParse(setupValues('12345678', '12345678'));

    expect(parsed.success).toBe(true);
    expect(numericPassword.success).toBe(true);
    if (parsed.success) {
      expect(parsed.data.administrator.password).toBe('safe-pass-8');
      expect(parsed.data.administrator.password_confirmation).toBe('safe-pass-8');
    }
  });

  it('rejects passwords shorter than 8 Unicode characters', () => {
    const parsed = schema().safeParse(setupValues('😀'.repeat(7), '😀'.repeat(7)));

    expect(parsed.success).toBe(false);
    if (!parsed.success) {
      expect(parsed.error.issues).toContainEqual(
        expect.objectContaining({
          path: ['administrator', 'password'],
          message: PASSWORD_LENGTH_ERROR,
        })
      );
    }
  });

  it('rejects passwords containing the normalized administrator username', () => {
    const parsed = schema().safeParse(setupValues('owner-pass12', 'owner-pass12'));

    expect(parsed.success).toBe(false);
    if (!parsed.success) {
      expect(parsed.error.issues).toContainEqual(
        expect.objectContaining({
          path: ['administrator', 'password'],
          message: PASSWORD_USERNAME_ERROR,
        })
      );
    }
  });
});

function schema() {
  return createSetupFormSchema(translate, translate);
}

function setupValues(password: string, passwordConfirmation: string) {
  const values = createSetupFormValues(defaults);
  values.postgres = {
    host: 'postgres.internal',
    port: 5_432,
    username: 'taco',
    password: 'postgres-secret',
    database: 'taco',
    use_tls: true,
  };
  values.redis.host = 'redis.internal';
  values.administrator = {
    username: ' owner ',
    email: 'owner@example.test',
    password,
    password_confirmation: passwordConfirmation,
  };
  return values;
}

const translate = ((key: string, options?: Record<string, unknown>) => {
  if (key !== 'auth.validation.passwordLength') return key;
  const minimum = options?.min;
  const maximum = options?.max;
  return `${key}:${minimum}:${maximum}`;
}) as unknown as TranslateFn;
