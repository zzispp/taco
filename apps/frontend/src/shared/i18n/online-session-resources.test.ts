import { it, expect, describe } from 'vitest';

import enAdmin from './langs/en/admin.json';
import cnAdmin from './langs/cn/admin.json';
import twAdmin from './langs/tw/admin.json';

const resources = [
  ['en', enAdmin.onlineSessions],
  ['tw', twAdmin.onlineSessions],
] as const;

describe('online-session translations', () => {
  it.each(resources)('%s has the same keys as cn', (_, resource) => {
    expect(Object.keys(resource).sort()).toEqual(Object.keys(cnAdmin.onlineSessions).sort());
  });
});
