import * as z from 'zod';

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

export const emailSchema = z
  .string()
  .transform(trimCredential)
  .pipe(
    z.email({
      error: ({ input }) =>
        input ? 'Email must be a valid email address!' : 'Email is required!',
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
