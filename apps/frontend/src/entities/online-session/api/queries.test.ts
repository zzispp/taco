import { it, expect, describe } from 'vitest';

import { onlineSessionsKey } from './queries';
import { onlineSessionEndpoints } from './endpoints';

describe('online-session cursor query key', () => {
  it('passes the opaque cursor and filters without offset fields', () => {
    expect(
      onlineSessionsKey(
        { limit: 50, cursor: 'next-session' },
        { userName: 'alice', browser: 'Firefox' }
      )
    ).toEqual([
      onlineSessionEndpoints.list,
      {
        params: {
          limit: 50,
          cursor: 'next-session',
          userName: 'alice',
          browser: 'Firefox',
        },
      },
    ]);
  });

  it('omits empty filter values without dropping the limit', () => {
    expect(onlineSessionsKey({ limit: 20 }, { ipaddr: '' })).toEqual([
      onlineSessionEndpoints.list,
      { params: { limit: 20 } },
    ]);
  });
});
