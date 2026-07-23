import { it, expect, describe } from 'vitest';
import { existsSync, readFileSync } from 'node:fs';

const ROOT_LAYOUT_URL = new URL('./(root)/layout.tsx', import.meta.url);
const ROOT_PAGE_URL = new URL('./(root)/page.tsx', import.meta.url);

describe('application root entry', () => {
  it('owns a complete document layout outside the locale root', () => {
    expect(existsSync(ROOT_LAYOUT_URL)).toBe(true);
    expect(readFileSync(ROOT_LAYOUT_URL, 'utf8')).toContain('<html');
    expect(readFileSync(ROOT_LAYOUT_URL, 'utf8')).toContain('<body>');
  });

  it('redirects the bare origin to the default localized root', () => {
    expect(existsSync(ROOT_PAGE_URL)).toBe(true);
    expect(readFileSync(ROOT_PAGE_URL, 'utf8')).toContain('redirect(defaultLocaleHomePath)');
  });
});
