'use client';

import type { SetupDefaults } from 'src/entities/installation';

import { useState, useEffect, useCallback } from 'react';

import { getSetupDefaults } from 'src/entities/installation';

type SetupDefaultsState =
  | Readonly<{ kind: 'loading' }>
  | Readonly<{ kind: 'ready'; defaults: SetupDefaults }>
  | Readonly<{ kind: 'failure'; error: unknown }>;

export function useSetupDefaults(): SetupDefaultsState & { retry: () => void } {
  const [attempt, setAttempt] = useState(0);
  const [state, setState] = useState<SetupDefaultsState>({ kind: 'loading' });
  const retry = useCallback(() => setAttempt((currentAttempt) => currentAttempt + 1), []);

  useEffect(() => {
    let current = true;
    setState({ kind: 'loading' });

    getSetupDefaults()
      .then((defaults) => {
        if (current) setState({ kind: 'ready', defaults });
      })
      .catch((error: unknown) => {
        if (current) setState({ kind: 'failure', error });
      });

    return () => {
      current = false;
    };
  }, [attempt]);

  return { ...state, retry };
}
