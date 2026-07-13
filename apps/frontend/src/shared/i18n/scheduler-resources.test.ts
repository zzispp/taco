import { it, expect, describe } from 'vitest';

import enScheduler from './langs/en/scheduler.json';
import cnScheduler from './langs/cn/scheduler.json';
import twScheduler from './langs/tw/scheduler.json';

const resources = [
  ['en', enScheduler],
  ['tw', twScheduler],
] as const;

describe('scheduler translations', () => {
  it.each(resources)('%s has the same keys as cn', (_, resource) => {
    expect(scalarPaths(resource)).toEqual(scalarPaths(cnScheduler));
  });
});

function scalarPaths(value: unknown, prefix = ''): string[] {
  if (!isRecord(value)) {
    return [prefix];
  }

  return Object.entries(value)
    .flatMap(([key, child]) => scalarPaths(child, prefix ? `${prefix}.${key}` : key))
    .sort();
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
