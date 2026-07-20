import type { TranslateFn } from 'src/shared/i18n';
import type { PasswordPolicy, SessionValidationMessages } from 'src/entities/session';

import * as z from 'zod';

import {
  createEmailSchema,
  PASSWORD_MAX_LENGTH,
  createPasswordSchema,
  createUsernameSchema,
  passwordContainsUsername,
} from 'src/entities/session';

const MIN_PORT = 1;
const MAX_PORT = 65_535;
const INSTALLATION_OWNER_PASSWORD_MIN_LENGTH = 8;

const INSTALLATION_OWNER_PASSWORD_POLICY: PasswordPolicy = Object.freeze({
  min_length: INSTALLATION_OWNER_PASSWORD_MIN_LENGTH,
  max_length: PASSWORD_MAX_LENGTH,
  require_letter: false,
  require_number: false,
  require_symbol: false,
  forbid_username_contains: true,
});

export function createSetupFormSchema(setupT: TranslateFn, messagesT: TranslateFn) {
  return z
    .object({
      postgres: postgresSchema(setupT),
      redis: redisSchema(setupT),
      administrator: administratorSchema(messagesT),
      advanced: advancedSchema(setupT),
    })
    .refine(
      (values) => values.administrator.password === values.administrator.password_confirmation,
      {
        path: ['administrator', 'password_confirmation'],
        error: setupT('validation.passwordConfirmation'),
      }
    );
}

function postgresSchema(t: TranslateFn) {
  return z.object({
    host: requiredText(t),
    port: portSchema(t),
    username: requiredText(t),
    password: requiredText(t),
    database: requiredText(t),
    use_tls: z.boolean(),
  });
}

function redisSchema(t: TranslateFn) {
  return z.object({
    host: requiredText(t),
    port: portSchema(t),
    username: z.string(),
    password: z.string(),
    database: optionalRedisDatabaseSchema(t),
    use_tls: z.boolean(),
  });
}

function administratorSchema(messagesT: TranslateFn) {
  const validationMessages = sessionValidationMessages(messagesT);
  return z
    .object({
      username: createUsernameSchema(validationMessages),
      email: createEmailSchema(validationMessages),
      password: createPasswordSchema(validationMessages, INSTALLATION_OWNER_PASSWORD_POLICY),
      password_confirmation: createPasswordSchema(
        validationMessages,
        INSTALLATION_OWNER_PASSWORD_POLICY
      ),
    })
    .superRefine((administrator, context) => {
      if (!passwordContainsUsername(administrator.password, administrator.username, INSTALLATION_OWNER_PASSWORD_POLICY)) {
        return;
      }
      context.addIssue({
        code: 'custom',
        path: ['password'],
        message: validationMessages.passwordContainsUsername,
      });
    });
}

function advancedSchema(t: TranslateFn) {
  return z.object({
    http_request_timeout_ms: positiveInteger(t),
    compression_enabled: z.boolean(),
    metrics_enabled: z.boolean(),
    online_session_cleanup_interval_ms: positiveInteger(t),
    online_session_cleanup_batch_size: positiveInteger(t),
    audit_outbox_worker_count: positiveInteger(t),
    audit_outbox_claim_batch_size: positiveInteger(t),
    audit_outbox_poll_interval_ms: positiveInteger(t),
    audit_outbox_lease_duration_ms: positiveInteger(t),
    audit_outbox_retry_delay_ms: positiveInteger(t),
    audit_outbox_cleanup_interval_ms: positiveInteger(t),
    audit_outbox_cleanup_batch_size: positiveInteger(t),
    audit_outbox_processed_retention_days: positiveInteger(t),
    client_ip_location_timeout_ms: positiveInteger(t),
    scheduler_http_timeout_ms: positiveInteger(t),
    scheduler_reconcile_interval_ms: positiveInteger(t),
    redis_key_prefix: z
      .string()
      .trim()
      .min(1, { error: t('validation.required') }),
  });
}

function requiredText(t: TranslateFn) {
  return z
    .string()
    .trim()
    .min(1, { error: t('validation.required') });
}

function portSchema(t: TranslateFn) {
  return z
    .number()
    .int({ error: t('validation.port') })
    .min(MIN_PORT, { error: t('validation.port') })
    .max(MAX_PORT, { error: t('validation.port') });
}

function optionalRedisDatabaseSchema(t: TranslateFn) {
  return z
    .string()
    .trim()
    .refine((value) => value === '' || /^\d+$/.test(value), {
      error: t('validation.redisDatabase'),
    });
}

function positiveInteger(t: TranslateFn) {
  return z
    .number()
    .int({ error: t('validation.positive') })
    .positive({ error: t('validation.positive') });
}

function sessionValidationMessages(t: TranslateFn): SessionValidationMessages {
  return {
    usernameLength: (min, max) => t('auth.validation.usernameLength', { min, max }),
    usernamePattern: t('auth.validation.usernamePattern'),
    passwordLength: (min, max) => t('auth.validation.passwordLength', { min, max }),
    passwordLetterRequired: t('auth.validation.passwordLetterRequired'),
    passwordNumberRequired: t('auth.validation.passwordNumberRequired'),
    passwordSymbolRequired: t('auth.validation.passwordSymbolRequired'),
    passwordContainsUsername: t('auth.validation.passwordContainsUsername'),
    emailRequired: t('auth.validation.emailRequired'),
    emailInvalid: t('auth.validation.emailInvalid'),
    identifierRequired: t('auth.validation.identifierRequired'),
    identifierInvalid: t('auth.validation.identifierInvalid'),
  };
}
