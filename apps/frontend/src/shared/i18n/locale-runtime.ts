import type { Theme, Components } from '@mui/material/styles';
import type { LangCode } from './locales-config';

import { getLocaleContractEntry } from './locale-contract';

type MuiLocaleModule = Readonly<Record<string, unknown>>;

export type LocaleSystemValue = Readonly<{
  components: Components<Theme>;
}>;

const localeRuntimeLoads = new Map<LangCode, Promise<void>>();
const localeSystemValues = new Map<LangCode, LocaleSystemValue>();

export function preloadLocaleRuntime(lang: LangCode): Promise<void> {
  const existingLoad = localeRuntimeLoads.get(lang);
  if (existingLoad) return existingLoad;

  const load = loadLocaleRuntime(lang);
  localeRuntimeLoads.set(lang, load);
  return load;
}

export function requireLocaleSystemValue(lang: LangCode): LocaleSystemValue {
  const value = localeSystemValues.get(lang);
  if (value) return value;
  throw new Error(`Locale runtime resources are not loaded: ${lang}`);
}

async function loadLocaleRuntime(lang: LangCode): Promise<void> {
  const locale = getLocaleContractEntry(lang);
  const [materialLocales, datePickerLocales, dataGridLocales] = await Promise.all([
    import('@mui/material/locale'),
    import('@mui/x-date-pickers/locales'),
    import('@mui/x-data-grid/locales'),
    import(`dayjs/locale/${locale.dayjsLocale}.js`),
  ]);

  localeSystemValues.set(lang, {
    components: {
      ...readMuiComponents(materialLocales, locale.muiLocale, '@mui/material'),
      ...readMuiComponents(datePickerLocales, locale.muiLocale, '@mui/x-date-pickers'),
      ...readMuiComponents(dataGridLocales, locale.muiLocale, '@mui/x-data-grid'),
    },
  });
}

function readMuiComponents(
  localeModule: MuiLocaleModule,
  localeKey: string,
  packageName: string
): Components<Theme> {
  const locale = localeModule[localeKey];
  if (!isRecord(locale)) throw new Error(`MUI locale is missing: ${packageName}/${localeKey}`);
  if (locale.components === undefined) return {};
  if (!isRecord(locale.components)) {
    throw new Error(`MUI locale components are invalid: ${packageName}/${localeKey}`);
  }
  return locale.components as Components<Theme>;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}
