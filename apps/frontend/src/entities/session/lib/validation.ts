import * as z from 'zod';

export type PasswordPolicy = {
  min_length: number;
  max_length: number;
  require_letter: boolean;
  require_number: boolean;
  require_symbol: boolean;
  forbid_username_contains: boolean;
};

export const USERNAME_MIN_LENGTH = 3;
export const USERNAME_MAX_LENGTH = 30;
export const PASSWORD_MIN_LENGTH = 8;
export const PASSWORD_MAX_LENGTH = 128;

const USERNAME_PATTERN = /^[A-Za-z0-9](?:[A-Za-z0-9_-]*[A-Za-z0-9])?$/;

export const trimCredential = (value: string) => value.trim();

export const usernameSchema = z
  .string()
  .transform(trimCredential)
  .pipe(
    z
      .string()
      .min(USERNAME_MIN_LENGTH, { error: usernameLengthMessage })
      .max(USERNAME_MAX_LENGTH, { error: usernameLengthMessage })
      .regex(USERNAME_PATTERN, { error: usernamePatternMessage })
  );

export const passwordSchema = z
  .string()
  .transform(trimCredential)
  .pipe(
    z
      .string()
      .min(PASSWORD_MIN_LENGTH, { error: passwordLengthMessage })
      .max(PASSWORD_MAX_LENGTH, { error: passwordLengthMessage })
  );

export function createPasswordSchema(policy?: PasswordPolicy, username?: string) {
  const activePolicy = policy ?? defaultPasswordPolicy();
  return z
    .string()
    .transform(trimCredential)
    .pipe(
      z
        .string()
        .min(activePolicy.min_length, { error: () => passwordPolicyLengthMessage(activePolicy) })
        .max(activePolicy.max_length, { error: () => passwordPolicyLengthMessage(activePolicy) })
        .refine((value) => !activePolicy.require_letter || hasLetter(value), {
          error: 'Password must contain a letter',
        })
        .refine((value) => !activePolicy.require_number || hasNumber(value), {
          error: 'Password must contain a number',
        })
        .refine((value) => !activePolicy.require_symbol || hasSymbol(value), {
          error: 'Password must contain a symbol',
        })
        .refine((value) => !containsUsername(value, username, activePolicy), {
          error: 'Password cannot contain username',
        })
    );
}

export const emailSchema = z
  .string()
  .transform(trimCredential)
  .pipe(
    z.email({
      error: ({ input }) => (input ? 'Email must be a valid email address!' : 'Email is required!'),
    })
  );

export const identifierSchema = z
  .string()
  .transform(trimCredential)
  .pipe(
    z
      .string()
      .min(1, { error: 'Username or email is required!' })
      .refine(isValidIdentifier, { error: 'Enter a valid username or email address' })
  );

function isValidIdentifier(value: string) {
  if (value.includes('@')) {
    return emailSchema.safeParse(value).success;
  }

  return usernameSchema.safeParse(value).success;
}

function usernameLengthMessage() {
  return `Username must be between ${USERNAME_MIN_LENGTH} and ${USERNAME_MAX_LENGTH} characters`;
}

function passwordLengthMessage() {
  return `Password must be between ${PASSWORD_MIN_LENGTH} and ${PASSWORD_MAX_LENGTH} characters`;
}

function usernamePatternMessage() {
  return 'Username can only contain letters, numbers, underscores, and hyphens, and must start and end with a letter or number';
}

function defaultPasswordPolicy(): PasswordPolicy {
  return {
    min_length: PASSWORD_MIN_LENGTH,
    max_length: PASSWORD_MAX_LENGTH,
    require_letter: false,
    require_number: false,
    require_symbol: false,
    forbid_username_contains: false,
  };
}

function passwordPolicyLengthMessage(policy: PasswordPolicy) {
  return `Password must be between ${policy.min_length} and ${policy.max_length} characters`;
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
