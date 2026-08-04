import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

import { analyzeSource, topLevelDirectoryViolations } from './check-frontend-compliance.mjs';

const fixture = (name) => readFile(new URL(`fixtures/${name}`, import.meta.url), 'utf8');

test('accepts the valid frontend fixture', async () => {
  const source = await fixture('valid.ts');
  const violations = analyzeSource('apps/frontend/src/shared/valid.ts', source);

  assert.deepEqual(violations, []);
});

test('reports frontend metrics and entity boundary violations', async () => {
  const source = await fixture('invalid.ts');
  const violations = analyzeSource('apps/frontend/src/entities/user/invalid.ts', source);
  const rules = violations.map((violation) => violation.rule);

  assert.ok(rules.includes('typescript-positional-parameters'));
  assert.ok(rules.includes('typescript-nesting-depth'));
  assert.ok(rules.includes('fsd-entity-sibling-import'));
  assert.ok(violations.every((violation) => violation.line > 0));
});

test('reports relative imports that violate the FSD layer direction', () => {
  const violations = analyzeSource(
    'apps/frontend/src/widgets/example/ui.ts',
    "import { Page } from '../../app/page';\nexport const example = Page;",
  );

  assert.deepEqual(violations.map((violation) => violation.rule), ['fsd-layer-dependency']);
  assert.equal(violations[0].line, 1);
});

test('reports non-FSD frontend top-level directories', () => {
  const violations = topLevelDirectoryViolations('apps/frontend/src', ['app', 'entities', 'legacy']);

  assert.deepEqual(violations, [
    {
      path: 'apps/frontend/src/legacy',
      line: 1,
      rule: 'fsd-top-level-directory',
      message: 'frontend source must not define top-level legacy outside FSD layers',
    },
  ]);
});

test('reports frontend function, file, and complexity limits from fixtures', async () => {
  const source = await fixture('limits.ts');
  const violations = analyzeSource('apps/frontend/src/shared/limits.ts', source);

  assert.deepEqual(
    violations.map(({ rule, line }) => ({ rule, line })),
    [
      { rule: 'typescript-function-lines', line: 1 },
      { rule: 'typescript-cyclomatic-complexity', line: 55 },
    ],
  );

  const oversizedSource = `${source}\n${'// padding\n'.repeat(301)}`;
  const oversizedViolations = analyzeSource('apps/frontend/src/shared/oversized.ts', oversizedSource);
  assert.deepEqual(
    oversizedViolations.filter(({ rule }) => rule === 'typescript-file-lines').map(({ line }) => line),
    [1],
  );
});
