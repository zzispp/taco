import type { TranslateFn } from 'src/shared/i18n';
import type { UserInput } from 'src/entities/user';
import type { PasswordPolicy } from 'src/entities/system';
import type { SessionValidationMessages } from 'src/entities/session';

import {
  createEmailSchema,
  createPasswordSchema,
  createUsernameSchema,
} from 'src/entities/session';

const PHONE_PATTERN = /^1[3-9]\d{9}$/;

export type AdminUserValidationMessages = SessionValidationMessages & {
  requiredUserFields: string;
  invalidPhone: string;
  passwordPolicyUnavailable: string;
};

type UserFormValidationOptions = Readonly<{
  mode: 'create' | 'edit';
  policy: PasswordPolicy | undefined;
  messages: AdminUserValidationMessages;
}>;

type AdminPasswordValidationOptions = Readonly<{
  password: string;
  username: string;
  policy: PasswordPolicy | undefined;
  messages: AdminUserValidationMessages;
}>;

export function validateAdminUserForm(form: UserInput, options: UserFormValidationOptions) {
  const fieldError = validateAdminUserFields(form, options.messages);
  if (fieldError) return fieldError;
  const phoneError = validateAdminPhone(form.phonenumber, options.messages);
  if (phoneError) return phoneError;
  if (options.mode === 'edit') return null;
  return validateAdminPassword({
    password: form.password ?? '',
    username: form.username,
    policy: options.policy,
    messages: options.messages,
  });
}

function validateAdminUserFields(form: UserInput, messages: AdminUserValidationMessages) {
  const username = createUsernameSchema(messages).safeParse(form.username);
  if (!username.success) {
    return username.error.issues[0]?.message ?? messages.requiredUserFields;
  }
  const email = createEmailSchema(messages).safeParse(form.email);
  if (!email.success) return email.error.issues[0]?.message ?? messages.emailInvalid;
  if (!form.nick_name.trim() || !form.status.trim()) return messages.requiredUserFields;
  return null;
}

function validateAdminPhone(
  phone: string | null | undefined,
  messages: AdminUserValidationMessages
) {
  const normalized = phone?.trim();
  return normalized && !PHONE_PATTERN.test(normalized) ? messages.invalidPhone : null;
}

export function validateAdminPassword({
  password,
  username,
  policy,
  messages,
}: AdminPasswordValidationOptions) {
  if (!policy) return messages.passwordPolicyUnavailable;
  const parsed = createPasswordSchema(messages, policy, username).safeParse(password);
  if (parsed.success) return null;
  return (
    parsed.error.issues[0]?.message ?? messages.passwordLength(policy.min_length, policy.max_length)
  );
}

export function adminUserValidationMessages(t: TranslateFn): AdminUserValidationMessages {
  return {
    usernameLength: (min, max) => t('profile.usernameRuleDynamic', { min, max }),
    usernamePattern: t('profile.usernamePattern'),
    passwordLength: (min, max) => t('profile.passwordRuleDynamic', { min, max }),
    passwordLetterRequired: t('profile.passwordLetterRequired'),
    passwordNumberRequired: t('profile.passwordNumberRequired'),
    passwordSymbolRequired: t('profile.passwordSymbolRequired'),
    passwordContainsUsername: t('profile.passwordContainsUsername'),
    emailRequired: t('profile.emailRequired'),
    emailInvalid: t('profile.invalidEmail'),
    identifierRequired: t('profile.identifierRequired'),
    identifierInvalid: t('profile.identifierInvalid'),
    requiredUserFields: t('profile.requiredProfileFields'),
    invalidPhone: t('profile.invalidPhone'),
    passwordPolicyUnavailable: t('profile.passwordPolicyUnavailable'),
  };
}
