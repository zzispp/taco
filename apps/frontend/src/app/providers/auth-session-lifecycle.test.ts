import { it, expect, describe } from 'vitest';

import { endSessionAfterTerminalRejection } from './auth-session-lifecycle';

describe('terminal auth session lifecycle', () => {
  it('clears the session, refreshes auth state, then redirects to sign-in', async () => {
    const events: string[] = [];

    await endSessionAfterTerminalRejection({
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
