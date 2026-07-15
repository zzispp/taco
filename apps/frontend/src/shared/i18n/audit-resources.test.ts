import { it, expect, describe } from 'vitest';

import enAudit from './langs/en/audit.json';
import cnAudit from './langs/cn/audit.json';
import twAudit from './langs/tw/audit.json';

describe('audit translations', () => {
  it.each([
    ['en', enAudit],
    ['tw', twAudit],
  ] as const)('%s has the same scalar keys as cn', (_, resource) => {
    expect(scalarPaths(resource)).toEqual(scalarPaths(cnAudit));
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
