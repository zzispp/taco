'use client';

import { useMemo } from 'react';

import { defaultSettings, SettingsProvider } from 'src/shared/ui/settings';
import {
  type SiteDisplay,
  SiteDisplayContext,
  defaultSiteDisplay,
} from 'src/shared/config/site-display-context';

import {
  usePublicConfigs,
  PUBLIC_CONFIG_KEYS,
  settingsFromPublicConfigs,
  siteDisplayFromPublicConfigs,
} from 'src/entities/system';

type AppSettingsProviderProps = {
  children: React.ReactNode;
};

export function AppSettingsProvider({ children }: AppSettingsProviderProps) {
  const { data, error, isLoading } = usePublicConfigs();
  const mergedDefaults = useMemo(() => {
    const remote = settingsFromPublicConfigs(data);
    return { ...defaultSettings, ...remote };
  }, [data]);
  const siteDisplay = useMemo(
    () => siteDisplayFromConfigs({ data, error, isLoading }),
    [data, error, isLoading]
  );

  return (
    <SiteDisplayContext value={siteDisplay}>
      <SettingsProvider defaultSettings={mergedDefaults}>{children}</SettingsProvider>
    </SiteDisplayContext>
  );
}

type SiteDisplayFromConfigsOptions = {
  data: Parameters<typeof siteDisplayFromPublicConfigs>[0];
  error: unknown;
  isLoading: boolean;
};

function siteDisplayFromConfigs({
  data,
  error,
  isLoading,
}: SiteDisplayFromConfigsOptions): SiteDisplay {
  if (error) {
    throw error;
  }

  if (isLoading) {
    return defaultSiteDisplay;
  }
  if (!data) {
    throw new Error(`Missing public system config: ${PUBLIC_CONFIG_KEYS.siteDisplayConfig}`);
  }

  const remote = siteDisplayFromPublicConfigs(data);
  if (!remote) {
    throw new Error(`Missing public system config: ${PUBLIC_CONFIG_KEYS.siteDisplayConfig}`);
  }

  return {
    siteName: remote.site_name.trim(),
    logoUrl: remote.logo_url.trim(),
    footerText: remote.footer_text.trim(),
  };
}
