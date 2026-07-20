import type { SetupDefaults, AdvancedSetupDefaults } from 'src/entities/installation';

export type SetupFormValues = {
  postgres: {
    host: string;
    port: number;
    username: string;
    password: string;
    database: string;
    use_tls: boolean;
  };
  redis: {
    host: string;
    port: number;
    username: string;
    password: string;
    database: string;
    use_tls: boolean;
  };
  administrator: {
    username: string;
    email: string;
    password: string;
    password_confirmation: string;
  };
  advanced: AdvancedSetupDefaults;
};

export const ADVANCED_SETUP_KEYS = [
  'http_request_timeout_ms',
  'compression_enabled',
  'metrics_enabled',
  'online_session_cleanup_interval_ms',
  'online_session_cleanup_batch_size',
  'audit_outbox_worker_count',
  'audit_outbox_claim_batch_size',
  'audit_outbox_poll_interval_ms',
  'audit_outbox_lease_duration_ms',
  'audit_outbox_retry_delay_ms',
  'audit_outbox_cleanup_interval_ms',
  'audit_outbox_cleanup_batch_size',
  'audit_outbox_processed_retention_days',
  'client_ip_location_timeout_ms',
  'scheduler_http_timeout_ms',
  'scheduler_reconcile_interval_ms',
  'redis_key_prefix',
] as const satisfies readonly (keyof AdvancedSetupDefaults)[];

export type AdvancedSetupKey = (typeof ADVANCED_SETUP_KEYS)[number];

export function createSetupFormValues(defaults: SetupDefaults): SetupFormValues {
  return {
    postgres: createPostgresDefaults(defaults),
    redis: createRedisDefaults(defaults),
    administrator: {
      username: '',
      email: '',
      password: '',
      password_confirmation: '',
    },
    advanced: { ...defaults.advanced },
  };
}

function createPostgresDefaults(defaults: SetupDefaults): SetupFormValues['postgres'] {
  return {
    host: '',
    port: defaults.postgres.port,
    username: '',
    password: '',
    database: '',
    use_tls: defaults.postgres.use_tls,
  };
}

function createRedisDefaults(defaults: SetupDefaults): SetupFormValues['redis'] {
  return {
    host: '',
    port: defaults.redis.port,
    username: '',
    password: '',
    database: '',
    use_tls: defaults.redis.use_tls,
  };
}
