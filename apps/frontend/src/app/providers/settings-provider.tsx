'use client';

import { useMemo } from 'react';

import { defaultSettings, SettingsProvider } from 'src/shared/ui/settings';

import { usePublicConfigs, settingsFromPublicConfigs } from 'src/entities/system';

type AppSettingsProviderProps = {
  children: React.ReactNode;
};

export function AppSettingsProvider({ children }: AppSettingsProviderProps) {
  const { data, error } = usePublicConfigs();
  const mergedDefaults = useMemo(() => {
    const remote = settingsFromPublicConfigs(data);
    return { ...defaultSettings, ...remote };
  }, [data]);

  if (error) {
    throw error;
  }

  return <SettingsProvider defaultSettings={mergedDefaults}>{children}</SettingsProvider>;
}
