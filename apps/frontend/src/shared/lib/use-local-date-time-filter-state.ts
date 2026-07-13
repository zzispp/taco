'use client';

import type { LocalDateTimeFilterDraft, LocalDateTimeFilterState } from './local-date-time-filter';

import { useRef, useState, useEffect, useCallback } from 'react';

import {
  createLocalDateTimeFilterState,
  transitionLocalDateTimeFilterState,
} from './local-date-time-filter';

type FilterStateOptions = Readonly<{
  onValidQuery?: () => void;
}>;

export function useLocalDateTimeFilterState<T extends LocalDateTimeFilterDraft>(
  initialDraft: T,
  options: FilterStateOptions = {}
) {
  const [state, setState] = useState<LocalDateTimeFilterState<T>>(() =>
    createLocalDateTimeFilterState(initialDraft)
  );
  const stateRef = useRef(state);
  const onValidQueryRef = useRef(options.onValidQuery);
  useEffect(() => {
    onValidQueryRef.current = options.onValidQuery;
  }, [options.onValidQuery]);
  const change = useCallback((nextDraft: T) => {
    const transition = transitionLocalDateTimeFilterState(stateRef.current, nextDraft);
    stateRef.current = transition.state;
    setState(transition.state);
    if (transition.resetTable) onValidQueryRef.current?.();
  }, []);

  return { ...state, change, isValid: state.error === null } as const;
}
