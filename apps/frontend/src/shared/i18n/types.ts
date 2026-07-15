import type { useTranslate } from './use-locales';

export const I18N_NAMESPACES = [
  'common',
  'messages',
  'admin',
  'navbar',
  'scheduler',
  'audit',
] as const;

export type I18nNamespace = (typeof I18N_NAMESPACES)[number];

export type TranslateFn = ReturnType<typeof useTranslate>['t'];
