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

function useSettingsStorage(storageKey: string, defaultSettings: SettingsState) {
  const useDefaultState = useRef(!getStorage<SettingsState>(storageKey));
  const { state, setState, resetState, setField } = useLocalStorage<SettingsState>(
    storageKey,
    defaultSettings
  );

  useEffect(() => {
    const storedValue = getStorage<SettingsState>(storageKey);

    if (!storedValue) {
      return;
    }

    if (storedValue.version !== defaultSettings.version) {
      resetState(defaultSettings);
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

  return { state, setState, resetState, setField };
}

function useDrawerControls() {
  const [openDrawer, setOpenDrawer] = useState(false);
  const onToggleDrawer = useCallback(() => setOpenDrawer((previous) => !previous), []);
  const onCloseDrawer = useCallback(() => setOpenDrawer(false), []);

  return { openDrawer, onToggleDrawer, onCloseDrawer };
}

export function SettingsProvider({
  children,
  defaultSettings,
  storageKey = SETTINGS_STORAGE_KEY,
}: SettingsProviderProps) {
  const { state, setState, resetState, setField } = useSettingsStorage(storageKey, defaultSettings);
  const { openDrawer, onToggleDrawer, onCloseDrawer } = useDrawerControls();
  const canReset = !isEqual(state, defaultSettings);
  const onReset = useCallback(() => resetState(defaultSettings), [defaultSettings, resetState]);
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
