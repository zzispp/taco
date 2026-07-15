'use client';

import type { CursorPageRequest } from 'src/shared/api/pagination';

import { useMemo, useEffect, useReducer, useCallback } from 'react';

import { DEFAULT_CURSOR_LIMIT } from 'src/shared/api/pagination';

export type CursorNavigationState = Readonly<{
  cursor: string | null;
  limit: number;
  scopeKey: string;
  visitedBatchIndex: number;
}>;

export type CursorNavigationAction =
  | Readonly<{ type: 'next'; cursor: string | null }>
  | Readonly<{ type: 'previous'; cursor: string | null }>
  | Readonly<{ type: 'limit'; limit: number }>
  | Readonly<{ type: 'scope'; scopeKey: string }>
  | Readonly<{ type: 'reset' }>;

export function createCursorNavigationState(
  limit = DEFAULT_CURSOR_LIMIT,
  scopeKey = ''
): CursorNavigationState {
  return { cursor: null, limit, scopeKey, visitedBatchIndex: 0 };
}

export function cursorNavigationReducer(
  state: CursorNavigationState,
  action: CursorNavigationAction
): CursorNavigationState {
  if (action.type === 'reset') return createCursorNavigationState(state.limit, state.scopeKey);
  if (action.type === 'limit') return createCursorNavigationState(action.limit, state.scopeKey);
  if (action.type === 'scope') {
    return state.scopeKey === action.scopeKey
      ? state
      : createCursorNavigationState(state.limit, action.scopeKey);
  }
  if (!action.cursor || action.cursor === state.cursor) return state;
  if (action.type === 'next') {
    return { ...state, cursor: action.cursor, visitedBatchIndex: state.visitedBatchIndex + 1 };
  }
  return {
    ...state,
    cursor: action.cursor,
    visitedBatchIndex: Math.max(0, state.visitedBatchIndex - 1),
  };
}

export function isCursorNavigationDisabled(pending: boolean, available: boolean) {
  return pending || !available;
}

export function cursorNavigationForScope(
  state: CursorNavigationState,
  scopeKey: string
): CursorNavigationState {
  return state.scopeKey === scopeKey ? state : createCursorNavigationState(state.limit, scopeKey);
}

export function useCursorNavigation(initialLimit = DEFAULT_CURSOR_LIMIT, scopeKey = '') {
  const [state, dispatch] = useReducer(
    cursorNavigationReducer,
    { limit: initialLimit, scopeKey },
    ({ limit, scopeKey: initialScopeKey }) => createCursorNavigationState(limit, initialScopeKey)
  );
  const scopedState = cursorNavigationForScope(state, scopeKey);
  useEffect(() => dispatch({ type: 'scope', scopeKey }), [scopeKey]);
  const request = useMemo<CursorPageRequest>(
    () => ({
      limit: scopedState.limit,
      ...(scopedState.cursor ? { cursor: scopedState.cursor } : {}),
    }),
    [scopedState.cursor, scopedState.limit]
  );

  return {
    ...scopedState,
    request,
    next: useCallback((cursor: string | null) => dispatch({ type: 'next', cursor }), []),
    previous: useCallback((cursor: string | null) => dispatch({ type: 'previous', cursor }), []),
    changeLimit: useCallback((limit: number) => dispatch({ type: 'limit', limit }), []),
    reset: useCallback(() => dispatch({ type: 'reset' }), []),
  };
}
