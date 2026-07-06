'use client';

import { useContext, createContext } from 'react';

import { CONFIG } from './index';

export type SiteDisplay = {
  siteName: string;
  logoUrl: string;
  footerText: string;
};

export const defaultSiteDisplay: SiteDisplay = {
  siteName: CONFIG.appName,
  logoUrl: `${CONFIG.assetsDir}/logo/logo.svg`,
  footerText: `${CONFIG.appName} backend control plane.`,
};

export const SiteDisplayContext = createContext<SiteDisplay>(defaultSiteDisplay);

export function useSiteDisplay() {
  return useContext(SiteDisplayContext);
}
