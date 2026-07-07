'use client';

import type { Namespace } from 'i18next';
import type { LangCode } from './locales-config';

import dayjs from 'dayjs';
import { useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';

import { toast } from 'src/shared/ui/snackbar';
import { useSettingsContext } from 'src/shared/ui/settings';

import { fallbackLng, storageConfig, getCurrentLang } from './locales-config';

// ----------------------------------------------------------------------

export function useTranslate(namespace?: Namespace) {
  const { t, i18n } = useTranslation(namespace);
  const currentLang = getCurrentLang(i18n.resolvedLanguage);
  const changeLang = useLanguageChanger(i18n);

  return {
    t,
    i18n,
    currentLang,
    onChangeLang: changeLang,
    onResetLang: () => changeLang(fallbackLng),
  };
}

function useLanguageChanger(i18n: ReturnType<typeof useTranslation>['i18n']) {
  const settings = useSettingsContext();
  const { t: tMessages } = useTranslation('messages');

  const updateDirection = useCallback(
    (lang: LangCode) => {
      settings.setState({ direction: i18n.dir(lang) });
    },
    [i18n, settings]
  );

  const updateDayjsLocale = useCallback((lang: LangCode) => {
    const updatedLang = getCurrentLang(lang);
    dayjs.locale(updatedLang.adapterLocale);
  }, []);

  const persistLanguage = useCallback((lang: LangCode) => {
    localStorage.setItem(storageConfig.localStorage.key, lang);
  }, []);

  return useCallback(
    async (lang: LangCode) => {
      try {
        const changeLangPromise = i18n.changeLanguage(lang);

        toast.promise(changeLangPromise, {
          loading: tMessages('languageSwitch.loading'),
          success: () => tMessages('languageSwitch.success'),
          error: () => tMessages('languageSwitch.error'),
        });

        await changeLangPromise;

        persistLanguage(lang);
        updateDirection(lang);
        updateDayjsLocale(lang);
      } catch (error) {
        console.error(error);
      }
    },
    [i18n, persistLanguage, tMessages, updateDayjsLocale, updateDirection]
  );
}

// ----------------------------------------------------------------------

export function useLocaleDirectionSync() {
  const { i18n, currentLang } = useTranslate();
  const { state, setState } = useSettingsContext();

  const handleSync = useCallback(async () => {
    const selectedLang = currentLang.value;
    const i18nDir = i18n.dir(selectedLang);

    if (document.dir !== i18nDir) {
      document.dir = i18nDir;
    }

    if (state.direction !== i18nDir) {
      setState({ direction: i18nDir });
    }

    if (i18n.resolvedLanguage !== selectedLang) {
      await i18n.changeLanguage(selectedLang);
    }
  }, [currentLang.value, i18n, setState, state.direction]);

  useEffect(() => {
    handleSync();
  }, [handleSync]);

  return null;
}
