'use client';

import { useRef, useState, useEffect, useCallback } from 'react';

const DEFAULT_MINIMUM_MS = 600;

export function useMinimumChecking(minimumMs = DEFAULT_MINIMUM_MS) {
  const startedAtRef = useRef<number | null>(null);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [isChecking, setIsChecking] = useState(true);

  useEffect(() => {
    startedAtRef.current = Date.now();
  }, []);

  const finishChecking = useCallback(() => {
    const startedAt = startedAtRef.current ?? Date.now();
    const elapsed = Date.now() - startedAt;
    const delay = Math.max(minimumMs - elapsed, 0);

    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }

    timeoutRef.current = setTimeout(() => {
      setIsChecking(false);
      timeoutRef.current = null;
    }, delay);
  }, [minimumMs]);

  useEffect(
    () => () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    },
    []
  );

  return { isChecking, finishChecking };
}
