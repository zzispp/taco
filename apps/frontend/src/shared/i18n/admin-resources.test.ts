import { it, expect, describe } from 'vitest';

import enAdmin from './langs/en/admin.json';
import cnAdmin from './langs/cn/admin.json';
import twAdmin from './langs/tw/admin.json';

const resources = [
  ['en', enAdmin],
  ['tw', twAdmin],
] as const;

describe('admin translations', () => {
  it.each(resources)('%s has the same scalar keys as cn', (_, resource) => {
    expect(scalarPaths(resource)).toEqual(scalarPaths(cnAdmin));
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
