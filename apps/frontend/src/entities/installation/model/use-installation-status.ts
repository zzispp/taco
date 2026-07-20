'use client';

import type { InstallationState } from './types';

import { useState, useEffect, useCallback } from 'react';

import { probeInstallationStatus } from '../api/requests';

type InstallationStatusProbe =
  | Readonly<{ kind: 'loading' }>
  | Readonly<{ kind: 'ready'; state: InstallationState }>
  | Readonly<{ kind: 'failure'; error: unknown }>;

export function useInstallationStatus(): InstallationStatusProbe & { retry: () => void } {
  const [attempt, setAttempt] = useState(0);
  const [probe, setProbe] = useState<InstallationStatusProbe>({ kind: 'loading' });
  const retry = useCallback(() => setAttempt((currentAttempt) => currentAttempt + 1), []);

  useEffect(() => {
    let current = true;
    setProbe({ kind: 'loading' });

    probeInstallationStatus()
      .then((state) => {
        if (current) setProbe({ kind: 'ready', state });
      })
      .catch((error: unknown) => {
        if (current) setProbe({ kind: 'failure', error });
      });

    return () => {
      current = false;
    };
  }, [attempt]);

  return { ...probe, retry };
}
