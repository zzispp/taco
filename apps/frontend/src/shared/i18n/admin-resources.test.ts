import { it, expect, describe } from 'vitest';

import { mergeAdminResources } from './admin-resources';
import { staticAdminResources } from './admin-static-resources';

const resources = [
  ['en', staticAdminResources.en],
  ['tw', staticAdminResources.tw],
] as const;

describe('admin translations', () => {
  it.each(resources)('%s has the same scalar keys as cn', (_, resource) => {
    expect(scalarPaths(resource)).toEqual(scalarPaths(staticAdminResources.cn));
  });

  it('rejects duplicate top-level keys instead of silently overwriting them', () => {
    expect(() => mergeAdminResources({ common: { save: 'Save' } }, { common: {} })).toThrowError(
      'Duplicate admin resource key: common'
    );
  });
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
