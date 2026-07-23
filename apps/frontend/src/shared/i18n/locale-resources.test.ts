import { it, expect, describe } from 'vitest';

import { loadTranslationResource } from './locales-config';
import { I18N_NAMESPACES, type I18nNamespace } from './types';
import { defaultLocaleCode, supportedLocaleCodes } from './locale-contract';

type ResourcesByLocale = Record<string, Record<string, unknown>>;

const UPLOAD_CANCELED_MESSAGES: Readonly<Record<string, string>> = {
  cn: '上传已取消',
  en: 'Upload canceled',
  tw: '上傳已取消',
};

describe('locale resources', () => {
  it.each(I18N_NAMESPACES)('%s loads every locale declared by the contract', async (namespace) => {
    const resources = await loadResources(namespace);

    expect(Object.keys(resources).sort()).toEqual([...supportedLocaleCodes].sort());
    for (const resource of Object.values(resources)) {
      expect(Object.keys(resource).length).toBeGreaterThan(0);
    }
  });

  it.each(I18N_NAMESPACES)(
    '%s keeps every locale aligned with the default locale',
    async (namespace) => {
      const resources = await loadResources(namespace);
      const defaultPaths = scalarPaths(resources[defaultLocaleCode]);

      for (const locale of supportedLocaleCodes) {
        expect(scalarPaths(resources[locale])).toEqual(defaultPaths);
      }
    }
  );

  it('provides a non-empty localized upload-canceled message for every locale', async () => {
    const resources = await loadResources('admin');

    for (const [locale, resource] of Object.entries(resources)) {
      const message = readText(resource, ['file', 'messages', 'uploadCanceled']);
      expect(message.length).toBeGreaterThan(0);
      if (UPLOAD_CANCELED_MESSAGES[locale]) {
        expect(message).toBe(UPLOAD_CANCELED_MESSAGES[locale]);
      }
    }
  });
});

async function loadResources(namespace: I18nNamespace): Promise<ResourcesByLocale> {
  const entries = await Promise.all(
    supportedLocaleCodes.map(async (locale) => [
      locale,
      await loadTranslationResource(locale, namespace),
    ])
  );
  return Object.fromEntries(entries);
}

function scalarPaths(value: unknown, prefix = ''): string[] {
  if (!isRecord(value)) return [prefix];
  return Object.entries(value)
    .flatMap(([key, child]) => scalarPaths(child, prefix ? `${prefix}.${key}` : key))
    .sort();
}

function readText(value: unknown, path: readonly string[]): string {
  const text = path.reduce<unknown>(
    (current, key) => (isRecord(current) ? current[key] : undefined),
    value
  );
  if (typeof text !== 'string') throw new Error(`Expected translation at ${path.join('.')}`);
  return text;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
