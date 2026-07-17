import { it, expect, describe } from 'vitest';

import enSystemLog from './langs/en/systemLog.json';
import cnSystemLog from './langs/cn/systemLog.json';
import twSystemLog from './langs/tw/systemLog.json';

describe('system log translations', () => {
  it.each([
    ['en', enSystemLog],
    ['tw', twSystemLog],
  ] as const)('%s has the same scalar keys as cn', (_, resource) => {
    expect(scalarPaths(resource)).toEqual(scalarPaths(cnSystemLog));
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
