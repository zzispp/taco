import type { SetupFormValues, AdvancedSetupKey } from './form-values';
import type { InstallationRequest, AdvancedSetupOverrides } from 'src/entities/installation';

import { trimCredential } from 'src/entities/session';

export type InstallationPayloadOptions = Readonly<{
  advancedKeys: readonly AdvancedSetupKey[];
}>;

export function buildInstallationRequest(
  values: SetupFormValues,
  options: InstallationPayloadOptions
): InstallationRequest {
  return {
    postgres: {
      host: values.postgres.host.trim(),
      port: values.postgres.port,
      username: values.postgres.username.trim(),
      password: values.postgres.password,
      database: values.postgres.database.trim(),
      use_tls: values.postgres.use_tls,
    },
    redis: {
      host: values.redis.host.trim(),
      port: values.redis.port,
      username: optionalString(values.redis.username),
      password: optionalString(values.redis.password),
      database: optionalRedisDatabase(values.redis.database),
      use_tls: values.redis.use_tls,
    },
    administrator: {
      username: values.administrator.username.trim(),
      email: values.administrator.email.trim(),
      password: trimCredential(values.administrator.password),
    },
    advanced: selectAdvancedOverrides(values, options.advancedKeys),
  };
}

function selectAdvancedOverrides(
  values: SetupFormValues,
  keys: readonly AdvancedSetupKey[]
): AdvancedSetupOverrides {
  return Object.fromEntries(
    keys.map((key) => [key, values.advanced[key]])
  ) as AdvancedSetupOverrides;
}

function optionalString(value: string): string | undefined {
  const trimmed = value.trim();
  return trimmed || undefined;
}

function optionalRedisDatabase(value: string): number | undefined {
  const trimmed = value.trim();
  return trimmed ? Number(trimmed) : undefined;
}
