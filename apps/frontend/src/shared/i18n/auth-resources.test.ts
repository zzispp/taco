import { it, expect, describe } from 'vitest';

import enMessages from './langs/en/messages.json';
import cnMessages from './langs/cn/messages.json';
import twMessages from './langs/tw/messages.json';

describe('authentication translations', () => {
  it.each([
    ['en', enMessages.auth],
    ['tw', twMessages.auth],
  ] as const)('%s has the same scalar keys as cn', (_, resource) => {
    expect(scalarPaths(resource)).toEqual(scalarPaths(cnMessages.auth));
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
