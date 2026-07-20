import { it, expect, describe } from 'vitest';

import {
  nextStep,
  previousStep,
  administratorValidationFields,
} from './use-setup-wizard';

describe('setup wizard next steps', () => {
  it('advances through the connection, administrator, and confirmation flow', () => {
    expect(nextStep('postgres')).toBe('redis');
    expect(nextStep('redis')).toBe('administrator');
    expect(nextStep('administrator')).toBe('confirmation');
  });

  it('does not advance from the confirmation or restart screen', () => {
    expect(() => nextStep('confirmation')).toThrow('Setup step "confirmation" has no next step');
    expect(() => nextStep('restart')).toThrow('Setup step "restart" has no next step');
  });
});

describe('setup wizard previous steps', () => {
  it('maps only steps that expose a back action', () => {
    expect(previousStep('redis')).toBe('postgres');
    expect(previousStep('administrator')).toBe('redis');
    expect(previousStep('confirmation')).toBe('administrator');
  });

  it('rejects steps without a previous step instead of routing to Redis', () => {
    expect(() => previousStep('postgres')).toThrow('Setup step "postgres" has no previous step');
    expect(() => previousStep('restart')).toThrow('Setup step "restart" has no previous step');
  });
});

describe('administrator validation fields', () => {
  it('validates only administrator fields while advanced settings are collapsed', () => {
    expect(administratorValidationFields(false)).toEqual(['administrator']);
  });

  it('adds advanced fields only while advanced settings are expanded', () => {
    expect(administratorValidationFields(true)).toEqual(['administrator', 'advanced']);
  });
});
