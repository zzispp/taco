import type { useTranslate } from './use-locales';

export type TranslateFn = ReturnType<typeof useTranslate>['t'];
