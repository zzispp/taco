import { readFileSync } from 'node:fs';
import { it, expect, describe } from 'vitest';

const PANEL_URL = new URL('./panel.tsx', import.meta.url);
const TOOLBAR_URL = new URL('./toolbar.tsx', import.meta.url);
const TABLE_SECTION_URL = new URL('./table-section.tsx', import.meta.url);

describe('notice management layout contract', () => {
  it('keeps the toolbar and table inside one card module', () => {
    const panel = readFileSync(PANEL_URL, 'utf8');
    const toolbar = readFileSync(TOOLBAR_URL, 'utf8');
    const tableSection = readFileSync(TABLE_SECTION_URL, 'utf8');

    expect(panel).not.toContain('<NoticeToolbar controller={controller} />');
    expect(tableSection).toContain("import { NoticeToolbar } from './toolbar'");
    expect(tableSection).toMatch(/<Card>[\s\S]*?<NoticeToolbar controller=\{controller\} \/>/);
    expect(toolbar).toContain('<Box sx={{ p: 2 }}>');
    expect(toolbar).not.toContain('borderColor');
    expect(toolbar).not.toContain('mb: 2');
  });
});
