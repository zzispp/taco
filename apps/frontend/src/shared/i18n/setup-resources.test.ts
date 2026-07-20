import { it, expect, describe } from 'vitest';

import enSetup from './langs/en/setup.json';
import cnSetup from './langs/cn/setup.json';
import twSetup from './langs/tw/setup.json';

describe('setup translations', () => {
  it.each([
    ['en', enSetup],
    ['tw', twSetup],
  ] as const)('%s has the same scalar keys as cn', (_, resource) => {
    expect(scalarPaths(resource)).toEqual(scalarPaths(cnSetup));
  });

  it.each([cnSetup, enSetup, twSetup])(
    'does not include the obsolete JWT disclosure',
    (resource) => {
      expect(resource.steps.confirmation).not.toHaveProperty('generatedJwt');
    }
  );

  it.each([cnSetup, enSetup, twSetup])(
    'does not include the removed installation-defaults preview',
    (resource) => {
      expect(resource.steps.confirmation).not.toHaveProperty('profileTitle');
      expect(resource.steps.confirmation).not.toHaveProperty('profileDescription');
    }
  );
});

function scalarPaths(value: unknown, prefix = ''): string[] {
  if (!isRecord(value)) return [prefix];
  return Object.entries(value)
    .flatMap(([key, child]) => scalarPaths(child, prefix ? `${prefix}.${key}` : key))
    .sort();
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
