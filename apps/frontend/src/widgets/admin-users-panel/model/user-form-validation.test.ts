import type { UserInput } from 'src/entities/user';
import type { PasswordPolicy } from 'src/entities/system';

import { it, expect, describe } from 'vitest';

import {
  validateAdminPassword,
  validateAdminUserForm,
  type AdminUserValidationMessages,
} from './user-form-validation';

const policy: PasswordPolicy = {
  min_length: 8,
  max_length: 128,
  require_letter: false,
  require_number: false,
  require_symbol: false,
  forbid_username_contains: true,
};

const messages: AdminUserValidationMessages = {
  usernameLength: () => 'username-length',
  usernamePattern: 'username-pattern',
  passwordLength: () => 'password-length',
  passwordLetterRequired: 'password-letter',
  passwordNumberRequired: 'password-number',
  passwordSymbolRequired: 'password-symbol',
  passwordContainsUsername: 'password-username',
  emailRequired: 'email-required',
  emailInvalid: 'email-invalid',
  identifierRequired: 'identifier-required',
  identifierInvalid: 'identifier-invalid',
  requiredUserFields: 'required-user-fields',
  invalidPhone: 'invalid-phone',
  passwordPolicyUnavailable: 'password-policy-unavailable',
};

describe('admin user form validation', () => {
  it('accepts a new user with no assigned roles', () => {
    expect(validateAdminUserForm(validUser(), { mode: 'create', policy, messages })).toBeNull();
  });

  it('rejects a create password outside 8..128 characters', () => {
    expect(
      validateAdminUserForm(validUser({ password: '1234567' }), {
        mode: 'create',
        policy,
        messages,
      })
    ).toBe('password-length');
  });

  it('does not require a password when editing user profile fields', () => {
    expect(
      validateAdminUserForm(validUser({ password: undefined }), {
        mode: 'edit',
        policy: undefined,
        messages,
      })
    ).toBeNull();
  });
});

describe('admin password reset validation', () => {
  it('rejects passwords containing the username case-insensitively', () => {
    expect(
      validateAdminPassword({
        password: 'prefixALICEsuffix',
        username: 'Alice',
        policy,
        messages,
      })
    ).toBe('password-username');
  });

  it('fails explicitly when required password policy is unavailable', () => {
    expect(
      validateAdminPassword({
        password: 'valid-pass',
        username: 'alice',
        policy: undefined,
        messages,
      })
    ).toBe('password-policy-unavailable');
  });
});

function validUser(overrides: Partial<UserInput> = {}): UserInput {
  return {
    username: 'alice',
    password: 'valid-pass',
    nick_name: 'Alice',
    dept_id: null,
    email: 'alice@example.com',
    phonenumber: null,
    sex: '2',
    status: '0',
    remark: null,
    role_ids: [],
    post_ids: [],
    ...overrides,
  };
}
