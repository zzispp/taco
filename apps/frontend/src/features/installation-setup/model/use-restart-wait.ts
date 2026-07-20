'use client';

import { useState, useEffect, useCallback } from 'react';

import { probeInstallationStatus } from 'src/entities/installation';

const RESTART_PROBE_INTERVAL_MS = 2_000;

export type RestartWaitState = 'idle' | 'waiting' | 'unavailable';

export function useRestartWait(onInstalled: () => void) {
  const [state, setState] = useState<RestartWaitState>('idle');
  const probe = useCallback(async () => {
    try {
      const installationState = await probeInstallationStatus();
      if (installationState === 'installed') {
        onInstalled();
        return;
      }
      setState('waiting');
    } catch {
      setState('unavailable');
    }
  }, [onInstalled]);

  useEffect(() => {
    if (state === 'idle') return undefined;
    void probe();
    const interval = window.setInterval(() => void probe(), RESTART_PROBE_INTERVAL_MS);
    return () => window.clearInterval(interval);
  }, [probe, state]);

  const start = useCallback(() => setState('waiting'), []);
  return { state, start, retry: probe };
}
