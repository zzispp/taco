'use client';

import { useEffect } from 'react';

import { useSiteDisplay } from 'src/shared/config/site-display-context';

import { useTranslate } from './use-locales';
import { formatDashboardDocumentTitle } from './document-title-format';

type LocalizedDashboardDocumentTitleProps = {
  titleKey: string;
  sectionKey?: string;
};

export function LocalizedDashboardDocumentTitle({
  titleKey,
  sectionKey = 'nav.dashboard',
}: LocalizedDashboardDocumentTitleProps) {
  const { t, i18n } = useTranslate('admin');
  const { siteName } = useSiteDisplay();

  useEffect(() => {
    document.title = formatDashboardDocumentTitle(t(titleKey), t(sectionKey), siteName);
  }, [i18n.resolvedLanguage, sectionKey, siteName, t, titleKey]);

  return null;
}
