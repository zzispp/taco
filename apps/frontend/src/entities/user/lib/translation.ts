import type { TranslateFn } from 'src/shared/i18n';

export function translatedAuthSource(source: string, t: TranslateFn) {
  const key = `authSources.${source.toLowerCase()}`;
  const translated = t(key);

  return translated === key ? source : translated;
}
