import { it, expect, describe } from 'vitest';

import { createPasswordSchema, type SessionValidationMessages } from './validation';

const messages: SessionValidationMessages = {
  usernameLength: (min, max) => `username-${min}-${max}`,
  usernamePattern: 'username-pattern',
  passwordLength: (min, max) => `password-${min}-${max}`,
  passwordLetterRequired: 'password-letter',
  passwordNumberRequired: 'password-number',
  passwordSymbolRequired: 'password-symbol',
  passwordContainsUsername: 'password-username',
  emailRequired: 'email-required',
  emailInvalid: 'email-invalid',
  identifierRequired: 'identifier-required',
  identifierInvalid: 'identifier-invalid',
};

describe('default password policy', () => {
  it('rejects a password containing the username without requiring character groups', () => {
    const schema = createPasswordSchema(messages, undefined, 'Alice');

    expect(schema.safeParse('prefixALICEsuffix').error?.issues[0]?.message).toBe(
      'password-username'
    );
    expect(schema.safeParse('87654321').success).toBe(true);
  });
});
