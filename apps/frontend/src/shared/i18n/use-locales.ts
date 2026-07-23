'use client';

import type { Namespace } from 'i18next';
import type { LangCode } from './locales-config';

import { useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';

import { usePathname } from 'src/shared/routes/hooks';
import { useSettingsContext } from 'src/shared/ui/settings';
import { localizePath, requireLangCode, localeFromPathname } from 'src/shared/routes/locale-path';

import { fallbackLng, getCurrentLang } from './locales-config';
import { replaceDocumentLocation } from './document-navigation';

// ----------------------------------------------------------------------

export function useTranslate(namespace?: Namespace) {
  const { t, i18n } = useTranslation(namespace);
  const currentLang = getCurrentLang(i18n.resolvedLanguage);
  const changeLang = useLanguageChanger();

  return {
    t,
    i18n,
    currentLang,
    onChangeLang: changeLang,
    onResetLang: () => changeLang(fallbackLng),
  };
}

function useLanguageChanger() {
  const pathname = usePathname();

  return useCallback(
    (lang: LangCode) => {
      const locale = requireLangCode(localeFromPathname(pathname));
      if (lang === locale) return;

      const currentPath = `${pathname}${window.location.search}${window.location.hash}`;
      replaceDocumentLocation(window.location, localizePath(lang, currentPath));
    },
    [pathname]
  );
}

// ----------------------------------------------------------------------

export function useLocaleDirectionSync() {
  const { i18n, currentLang } = useTranslate();
  const { state, setState } = useSettingsContext();

  const handleSync = useCallback(() => {
    const selectedLang = currentLang.value;
    const i18nDir = i18n.dir(selectedLang);

    if (document.dir !== i18nDir) {
      document.dir = i18nDir;
    }

    if (state.direction !== i18nDir) {
      setState({ direction: i18nDir });
    }
  }, [currentLang.value, i18n, setState, state.direction]);

  useEffect(() => {
    handleSync();
  }, [handleSync]);

  return null;
}
