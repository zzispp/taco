import { readFileSync } from 'node:fs';
import { it, expect, describe } from 'vitest';

const LOCALE_LAYOUT_URL = new URL('./[locale]/layout.tsx', import.meta.url);
const GLOBAL_STYLES_URL = new URL('../global.css', import.meta.url);

describe('application global styles contract', () => {
  it('loads the global stylesheet from the locale root layout', () => {
    const layoutSource = readFileSync(LOCALE_LAYOUT_URL, 'utf8');

    expect(layoutSource.match(/import ['"]\.\.\/\.\.\/global\.css['"];?/g)).toHaveLength(1);
  });

  it('keeps the full-height layout baseline in the imported stylesheet', () => {
    const globalStyles = readFileSync(GLOBAL_STYLES_URL, 'utf8');

    expect(globalStyles).toContain('body,\n#root,\n#root__layout');
    expect(globalStyles).toContain('min-height: 100%');
  });
});
