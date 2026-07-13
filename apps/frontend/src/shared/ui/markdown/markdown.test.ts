import { it, expect, describe } from 'vitest';

import { normalizeMarkdownContent } from './markdown';

describe('normalizeMarkdownContent', () => {
  it('preserves first-line indentation in Markdown source mode', () => {
    const content = '    first line\nsecond line\n';
    expect(normalizeMarkdownContent(content, 'markdown')).toBe(content);
  });

  it('keeps the existing trim behavior in auto mode', () => {
    expect(normalizeMarkdownContent('  # Heading  ', 'auto')).toBe('# Heading');
  });

  it('keeps the existing HTML conversion behavior in auto mode', () => {
    expect(normalizeMarkdownContent('  <code>Hello</code>  ', 'auto')).toBe('`Hello`');
  });
});
