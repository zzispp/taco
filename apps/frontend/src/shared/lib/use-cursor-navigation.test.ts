import { it, expect } from 'vitest';

import {
  cursorNavigationReducer,
  cursorNavigationForScope,
  isCursorNavigationDisabled,
  createCursorNavigationState,
} from './use-cursor-navigation';

it('starts with the explicit limit and no cursor', () => {
  expect(createCursorNavigationState(20)).toEqual({
    cursor: null,
    limit: 20,
    scopeKey: '',
    visitedBatchIndex: 0,
  });
});

it('moves forward and backward with opaque server cursors', () => {
  const second = cursorNavigationReducer(createCursorNavigationState(), {
    type: 'next',
    cursor: 'next-1',
  });
  const first = cursorNavigationReducer(second, {
    type: 'previous',
    cursor: 'server-previous',
  });

  expect(second).toEqual({
    cursor: 'next-1',
    limit: 20,
    scopeKey: '',
    visitedBatchIndex: 1,
  });
  expect(first).toEqual({
    cursor: 'server-previous',
    limit: 20,
    scopeKey: '',
    visitedBatchIndex: 0,
  });
});

it('rejects missing and stale navigation cursors', () => {
  const state = navigationState();

  expect(cursorNavigationReducer(state, { type: 'next', cursor: null })).toBe(state);
  expect(cursorNavigationReducer(state, { type: 'next', cursor: 'current' })).toBe(state);
});

it('resets the cursor for limit and query resets', () => {
  const state = navigationState();

  expect(cursorNavigationReducer(state, { type: 'limit', limit: 50 })).toEqual({
    cursor: null,
    limit: 50,
    scopeKey: 'role-1',
    visitedBatchIndex: 0,
  });
  expect(cursorNavigationReducer(state, { type: 'reset' })).toEqual({
    cursor: null,
    limit: 20,
    scopeKey: 'role-1',
    visitedBatchIndex: 0,
  });
});

it('derives the first batch immediately when the query scope changes', () => {
  expect(cursorNavigationForScope(navigationState(), 'role-2')).toEqual({
    cursor: null,
    limit: 20,
    scopeKey: 'role-2',
    visitedBatchIndex: 0,
  });
});
it('blocks stale navigation while the current request is validating', () => {
  expect(isCursorNavigationDisabled(true, true)).toBe(true);
  expect(isCursorNavigationDisabled(false, false)).toBe(true);
  expect(isCursorNavigationDisabled(false, true)).toBe(false);
});

function navigationState() {
  return {
    cursor: 'current',
    limit: 20,
    scopeKey: 'role-1',
    visitedBatchIndex: 2,
  } as const;
}
