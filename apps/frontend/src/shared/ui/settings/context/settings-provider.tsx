'use client';

import type { SettingsState, SettingsProviderProps } from '../types';

import { isEqual } from 'es-toolkit';
import { getStorage } from 'minimal-shared/utils';
import { useLocalStorage } from 'minimal-shared/hooks';
import { useRef, useMemo, useState, useEffect, useCallback } from 'react';

import { SettingsContext } from './settings-context';
import { SETTINGS_STORAGE_KEY } from '../settings-config';

// ----------------------------------------------------------------------

function shouldMigrateCompactLayout(storedValue: SettingsState, defaultSettings: SettingsState) {
  return isEqual(storedValue, { ...defaultSettings, compactLayout: true });
}

export function SettingsProvider({
  children,
  defaultSettings,
  storageKey = SETTINGS_STORAGE_KEY,
}: SettingsProviderProps) {
  const useDefaultState = useRef(!getStorage<SettingsState>(storageKey));
  const { state, setState, resetState, setField } = useLocalStorage<SettingsState>(
    storageKey,
    defaultSettings
  );

  const [openDrawer, setOpenDrawer] = useState(false);

  const onToggleDrawer = useCallback(() => {
    setOpenDrawer((prev) => !prev);
  }, []);

  const onCloseDrawer = useCallback(() => {
    setOpenDrawer(false);
  }, []);

  const canReset = !isEqual(state, defaultSettings);

  const onReset = useCallback(() => {
    resetState(defaultSettings);
  }, [defaultSettings, resetState]);

  // Version mismatch reset handling
  useEffect(() => {
    const storedValue = getStorage<SettingsState>(storageKey);

    if (!storedValue) {
      return;
    }

    if (storedValue.version !== defaultSettings.version) {
      onReset();
      return;
    }

    if (shouldMigrateCompactLayout(storedValue, defaultSettings)) {
      setState({ compactLayout: defaultSettings.compactLayout });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (useDefaultState.current) {
      resetState(defaultSettings);
    }
  }, [defaultSettings, resetState]);

  const memoizedValue = useMemo(
    () => ({
      defaultSettings,
      canReset,
      onReset,
      openDrawer,
      onCloseDrawer,
      onToggleDrawer,
      state,
      setState,
      setField,
    }),
    [
      canReset,
      defaultSettings,
      onReset,
      openDrawer,
      onCloseDrawer,
      onToggleDrawer,
      state,
      setField,
      setState,
    ]
  );

  return <SettingsContext value={memoizedValue}>{children}</SettingsContext>;
}
