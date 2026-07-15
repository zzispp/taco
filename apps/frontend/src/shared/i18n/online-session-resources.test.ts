import { it, expect, describe } from 'vitest';

import { staticAdminResources } from './admin-static-resources';

const resources = [
  ['en', staticAdminResources.en.onlineSessions],
  ['tw', staticAdminResources.tw.onlineSessions],
] as const;

describe('online-session translations', () => {
  it.each(resources)('%s has the same keys as cn', (_, resource) => {
    expect(Object.keys(resource).sort()).toEqual(
      Object.keys(staticAdminResources.cn.onlineSessions).sort()
    );
  });
});
