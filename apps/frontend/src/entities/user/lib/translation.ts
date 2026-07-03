import type { AdminT } from 'src/shared/ui/admin/common';

export function translatedAuthSource(source: string, t: AdminT) {
  const key = `authSources.${source.toLowerCase()}`;
  const translated = t(key);

  return translated === key ? source : translated;
}
