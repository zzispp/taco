import type { Metadata } from 'next';

import { cache } from 'react';

import { loadTranslationResource } from 'src/shared/i18n/locales-config';
import {
  defaultLocaleCode,
  defaultLocaleHomePath,
  defaultDocumentLanguage,
} from 'src/shared/i18n/locale-contract';

export async function generateMetadata(): Promise<Metadata> {
  const error404 = await loadDefaultError404();
  return { title: error404.title };
}

export default async function GlobalNotFound() {
  const error404 = await loadDefaultError404();

  return (
    <html lang={defaultDocumentLanguage} dir="ltr">
      <body>
        <main
          data-template-error-page="true"
          data-home-href={defaultLocaleHomePath}
          style={pageStyles}
        >
          <p style={codeStyles}>404</p>
          <h1 style={titleStyles}>{error404.title}</h1>
          <p style={descriptionStyles}>{error404.description}</p>
          <a href={defaultLocaleHomePath} style={homeLinkStyles}>
            {error404.home}
          </a>
        </main>
      </body>
    </html>
  );
}

const loadDefaultError404 = cache(async (): Promise<Error404Resource> => {
  const common = await loadTranslationResource(defaultLocaleCode, 'common');
  const error404 = common.error404;
  if (!isError404Resource(error404)) {
    throw new Error(`Default locale is missing common.error404: ${defaultLocaleCode}`);
  }
  return error404;
});

type Error404Resource = Readonly<{
  title: string;
  description: string;
  home: string;
}>;

function isError404Resource(value: unknown): value is Error404Resource {
  if (!isRecord(value)) return false;
  return ['title', 'description', 'home'].every((key) => typeof value[key] === 'string');
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

const pageStyles = {
  display: 'flex',
  flex: '1 1 auto',
  alignItems: 'center',
  flexDirection: 'column',
  justifyContent: 'center',
  padding: '48px 24px',
  textAlign: 'center',
  fontFamily: 'Public Sans Variable, Arial, sans-serif',
} as const;

const codeStyles = {
  margin: 0,
  color: '#00A76F',
  fontSize: '72px',
  fontWeight: 800,
  lineHeight: 1,
} as const;

const titleStyles = { margin: '24px 0 16px', fontSize: '32px', lineHeight: 1.25 } as const;

const descriptionStyles = {
  maxWidth: '520px',
  margin: '0 0 32px',
  color: '#637381',
  fontSize: '16px',
  lineHeight: 1.6,
} as const;

const homeLinkStyles = {
  padding: '12px 22px',
  borderRadius: '8px',
  color: '#FFFFFF',
  background: '#00A76F',
  fontWeight: 700,
  textDecoration: 'none',
} as const;
