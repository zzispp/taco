'use client';

import dayjs from 'dayjs';

import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import { LocalizationProvider as Provider } from '@mui/x-date-pickers/LocalizationProvider';

import { useTranslate } from './use-locales';
import { requireLocaleSystemValue } from './locale-runtime';

// ----------------------------------------------------------------------

type Props = {
  children: React.ReactNode;
};

export function LocalizationProvider({ children }: Props) {
  const { currentLang } = useTranslate();

  requireLocaleSystemValue(currentLang.value);
  dayjs.locale(currentLang.dayjsLocale);

  return (
    <Provider dateAdapter={AdapterDayjs} adapterLocale={currentLang.dayjsLocale}>
      {children}
    </Provider>
  );
}
