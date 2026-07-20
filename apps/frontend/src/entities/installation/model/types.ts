import * as z from 'zod';

const installationStateSchema = z.enum(['setup', 'installed']);

const connectionDefaultsSchema = z.object({
  port: z.number().int().positive(),
  use_tls: z.boolean(),
});

const advancedSetupDefaultsSchema = z.object({
  http_request_timeout_ms: z.number().int().positive(),
  compression_enabled: z.boolean(),
  metrics_enabled: z.boolean(),
  online_session_cleanup_interval_ms: z.number().int().positive(),
  online_session_cleanup_batch_size: z.number().int().positive(),
  audit_outbox_worker_count: z.number().int().positive(),
  audit_outbox_claim_batch_size: z.number().int().positive(),
  audit_outbox_poll_interval_ms: z.number().int().positive(),
  audit_outbox_lease_duration_ms: z.number().int().positive(),
  audit_outbox_retry_delay_ms: z.number().int().positive(),
  audit_outbox_cleanup_interval_ms: z.number().int().positive(),
  audit_outbox_cleanup_batch_size: z.number().int().positive(),
  audit_outbox_processed_retention_days: z.number().int().positive(),
  client_ip_location_timeout_ms: z.number().int().positive(),
  scheduler_http_timeout_ms: z.number().int().positive(),
  scheduler_reconcile_interval_ms: z.number().int().positive(),
  redis_key_prefix: z.string().min(1),
});

const setupDefaultsSchema = z.object({
  postgres: connectionDefaultsSchema,
  redis: connectionDefaultsSchema,
  advanced: advancedSetupDefaultsSchema,
});

const installationStatusSchema = z.object({ state: installationStateSchema });
const connectionTestSchema = z.object({ status: z.literal('ok') });
const installationCompleteSchema = z.object({
  state: z.literal('installed'),
  restart_required: z.literal(true),
});

export type InstallationState = z.infer<typeof installationStateSchema>;
export type SetupDefaults = z.infer<typeof setupDefaultsSchema>;
export type AdvancedSetupDefaults = z.infer<typeof advancedSetupDefaultsSchema>;
export type AdvancedSetupOverrides = Partial<AdvancedSetupDefaults>;

export type PostgresConnectionInput = Readonly<{
  host: string;
  port: number;
  username: string;
  password: string;
  database: string;
  use_tls: boolean;
}>;

export type RedisConnectionInput = Readonly<{
  host: string;
  port: number;
  username?: string;
  password?: string;
  database?: number;
  use_tls: boolean;
}>;

export type InitialAdministratorInput = Readonly<{
  username: string;
  email: string;
  password: string;
}>;

export type InstallationRequest = Readonly<{
  postgres: PostgresConnectionInput;
  redis: RedisConnectionInput;
  administrator: InitialAdministratorInput;
  advanced: AdvancedSetupOverrides;
}>;

export function parseInstallationState(value: unknown): InstallationState {
  return installationStatusSchema.parse(value).state;
}

export function parseSetupDefaults(value: unknown): SetupDefaults {
  return setupDefaultsSchema.parse(value);
}

export function parseConnectionTest(value: unknown): void {
  connectionTestSchema.parse(value);
}

export function parseInstallationComplete(value: unknown): void {
  installationCompleteSchema.parse(value);
}
