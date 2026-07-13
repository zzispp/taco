import { it, expect, describe } from 'vitest';

import { reloginAfterSessionExpired } from './relogin';

describe('relogin after an expired session', () => {
  it('clears local state, refreshes auth context, then navigates to sign-in', async () => {
    const events: string[] = [];

    await reloginAfterSessionExpired({
      clearSession: async () => {
        events.push('clear-session');
      },
      refreshAuthState: async () => {
        events.push('refresh-auth-state');
      },
      redirectToSignIn: () => events.push('redirect-to-sign-in'),
    });

    expect(events).toEqual(['clear-session', 'refresh-auth-state', 'redirect-to-sign-in']);
  });
});
