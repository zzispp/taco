import * as z from 'zod';

export type PasswordPolicy = {
  min_length: number;
  max_length: number;
  require_letter: boolean;
  require_number: boolean;
  require_symbol: boolean;
  forbid_username_contains: boolean;
};

export type SessionValidationMessages = {
  usernameLength: (min: number, max: number) => string;
  usernamePattern: string;
  passwordLength: (min: number, max: number) => string;
  passwordLetterRequired: string;
  passwordNumberRequired: string;
  passwordSymbolRequired: string;
  passwordContainsUsername: string;
  emailRequired: string;
  emailInvalid: string;
  identifierRequired: string;
  identifierInvalid: string;
};

export const USERNAME_MIN_LENGTH = 3;
export const USERNAME_MAX_LENGTH = 30;
export const PASSWORD_MIN_LENGTH = 8;
export const PASSWORD_MAX_LENGTH = 128;

const USERNAME_PATTERN = /^[A-Za-z0-9](?:[A-Za-z0-9_-]*[A-Za-z0-9])?$/;

export const trimCredential = (value: string) => value.trim();

export function createUsernameSchema(messages: SessionValidationMessages) {
  return z
    .string()
    .transform(trimCredential)
    .pipe(
      z
        .string()
        .min(USERNAME_MIN_LENGTH, {
          error: () => messages.usernameLength(USERNAME_MIN_LENGTH, USERNAME_MAX_LENGTH),
        })
        .max(USERNAME_MAX_LENGTH, {
          error: () => messages.usernameLength(USERNAME_MIN_LENGTH, USERNAME_MAX_LENGTH),
        })
        .regex(USERNAME_PATTERN, { error: messages.usernamePattern })
    );
}

export function createBasicPasswordSchema(messages: SessionValidationMessages) {
  return z
    .string()
    .transform(trimCredential)
    .pipe(
      z
        .string()
        .min(PASSWORD_MIN_LENGTH, {
          error: () => messages.passwordLength(PASSWORD_MIN_LENGTH, PASSWORD_MAX_LENGTH),
        })
        .max(PASSWORD_MAX_LENGTH, {
          error: () => messages.passwordLength(PASSWORD_MIN_LENGTH, PASSWORD_MAX_LENGTH),
        })
    );
}

export function createPasswordSchema(
  messages: SessionValidationMessages,
  policy?: PasswordPolicy,
  username?: string
) {
  const activePolicy = policy ?? defaultPasswordPolicy();
  return z
    .string()
    .transform(trimCredential)
    .pipe(
      z
        .string()
        .min(activePolicy.min_length, {
          error: () => messages.passwordLength(activePolicy.min_length, activePolicy.max_length),
        })
        .max(activePolicy.max_length, {
          error: () => messages.passwordLength(activePolicy.min_length, activePolicy.max_length),
        })
        .refine((value) => !activePolicy.require_letter || hasLetter(value), {
          error: messages.passwordLetterRequired,
        })
        .refine((value) => !activePolicy.require_number || hasNumber(value), {
          error: messages.passwordNumberRequired,
        })
        .refine((value) => !activePolicy.require_symbol || hasSymbol(value), {
          error: messages.passwordSymbolRequired,
        })
        .refine((value) => !containsUsername(value, username, activePolicy), {
          error: messages.passwordContainsUsername,
        })
    );
}

export function createEmailSchema(messages: SessionValidationMessages) {
  return z
    .string()
    .transform(trimCredential)
    .pipe(
      z.email({
        error: ({ input }) => (input ? messages.emailInvalid : messages.emailRequired),
      })
    );
}

export function createIdentifierSchema(messages: SessionValidationMessages) {
  return z
    .string()
    .transform(trimCredential)
    .pipe(
      z
        .string()
        .min(1, { error: messages.identifierRequired })
        .refine((value) => isValidIdentifier(value, messages), {
          error: messages.identifierInvalid,
        })
    );
}

function isValidIdentifier(value: string, messages: SessionValidationMessages) {
  if (value.includes('@')) {
    return createEmailSchema(messages).safeParse(value).success;
  }

  return createUsernameSchema(messages).safeParse(value).success;
}

function defaultPasswordPolicy(): PasswordPolicy {
  return {
    min_length: PASSWORD_MIN_LENGTH,
    max_length: PASSWORD_MAX_LENGTH,
    require_letter: false,
    require_number: false,
    require_symbol: false,
    forbid_username_contains: true,
  };
}

function hasLetter(value: string) {
  return /[A-Za-z]/.test(value);
}

function hasNumber(value: string) {
  return /\d/.test(value);
}

function hasSymbol(value: string) {
  return /[!"#$%&'()*+,\-./:;<=>?@[\\\]^_`{|}~]/.test(value);
}

function containsUsername(value: string, username: string | undefined, policy: PasswordPolicy) {
  if (!policy.forbid_username_contains) {
    return false;
  }
  const normalizedUsername = username?.trim().toLowerCase();
  return !!normalizedUsername && value.toLowerCase().includes(normalizedUsername);
}
