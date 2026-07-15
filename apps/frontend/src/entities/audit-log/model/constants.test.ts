import { it, expect, describe } from 'vitest';

import { LOGIN_EVENT_TYPES, loginEventTypeKeys } from './constants';

describe('login audit event types', () => {
  it('exposes every authentication lifecycle outcome with a translation key', () => {
    expect(LOGIN_EVENT_TYPES).toEqual([
      'login_success',
      'login_failure',
      'register_success',
      'register_failure',
      'logout_success',
      'logout_failure',
      'refresh_success',
      'refresh_failure',
    ]);
    expect(LOGIN_EVENT_TYPES.map((eventType) => loginEventTypeKeys[eventType])).toEqual([
      'eventTypes.loginSuccess',
      'eventTypes.loginFailure',
      'eventTypes.registerSuccess',
      'eventTypes.registerFailure',
      'eventTypes.logoutSuccess',
      'eventTypes.logoutFailure',
      'eventTypes.refreshSuccess',
      'eventTypes.refreshFailure',
    ]);
  });
});
