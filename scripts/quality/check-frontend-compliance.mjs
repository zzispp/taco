#!/usr/bin/env node
import { createRequire } from 'node:module';
import { readdir, readFile } from 'node:fs/promises';
import { dirname, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(new URL('../../apps/frontend/package.json', import.meta.url));
const parser = require('@typescript-eslint/parser');

const MAX_FUNCTION_LINES = 50;
const MAX_FILE_LINES = 300;
const MAX_NESTING_DEPTH = 3;
const MAX_POSITIONAL_PARAMETERS = 3;
const MAX_CYCLOMATIC_COMPLEXITY = 10;
const FRONTEND_SOURCE = 'apps/frontend/src';
const FSD_LAYERS = ['app', 'pages-layer', 'widgets', 'features', 'entities', 'shared'];
const FUNCTION_NODES = new Set(['FunctionDeclaration', 'FunctionExpression', 'ArrowFunctionExpression']);
const NESTED_BRANCH_NODES = new Set(['IfStatement', 'ForStatement', 'ForInStatement', 'ForOfStatement', 'WhileStatement', 'DoWhileStatement', 'SwitchStatement']);
const SKIPPED_KEYS = new Set(['loc', 'range', 'parent', 'comments', 'tokens']);

const ALLOWED_LAYER_DEPENDENCIES = {
  app: new Set(FSD_LAYERS),
  'pages-layer': new Set(['widgets', 'features', 'entities', 'shared']),
  widgets: new Set(['features', 'entities', 'shared']),
  features: new Set(['entities', 'shared']),
  entities: new Set(['shared']),
  shared: new Set(['shared']),
};

export async function scanFrontend(root) {
  const sourceRoot = resolve(root, FRONTEND_SOURCE);
  const [rootViolations, files] = await Promise.all([
    rootDirectoryViolations(root, sourceRoot),
    collectSourceFiles(sourceRoot),
  ]);
  const fileViolations = await Promise.all(files.map((file) => scanFile(root, file)));
  return [...rootViolations, ...fileViolations.flat()];
}

export function topLevelDirectoryViolations(path, directories) {
  return directories
    .filter((directory) => !FSD_LAYERS.includes(directory))
    .map((directory) => violation(`${path}/${directory}`, 1, 'fsd-top-level-directory', `frontend source must not define top-level ${directory} outside FSD layers`));
}

export function analyzeSource(path, source) {
  const ast = parseSource(path, source);
  if (ast instanceof Error) {
    return [violation(path, ast.lineNumber ?? 1, 'typescript-parse', ast.message)];
  }
  return [
    ...fileLineViolations(path, source),
    ...functionViolations(path, source, ast),
    ...fsdImportViolations(path, ast),
  ];
}

async function scanFile(root, file) {
  const source = await readFile(file, 'utf8');
  return analyzeSource(relative(root, file), source);
}

async function rootDirectoryViolations(root, sourceRoot) {
  const entries = await readdir(sourceRoot, { withFileTypes: true });
  const directories = entries.filter((entry) => entry.isDirectory()).map((entry) => entry.name);
  return topLevelDirectoryViolations(relative(root, sourceRoot), directories);
}

async function collectSourceFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(entries.map((entry) => collectEntry(directory, entry)));
  return nested.flat();
}

async function collectEntry(directory, entry) {
  const path = resolve(directory, entry.name);
  if (entry.isDirectory()) {
    return collectSourceFiles(path);
  }
  return isProductionTypeScript(entry.name) ? [path] : [];
}

function isProductionTypeScript(name) {
  const isSource = /\.(?:ts|tsx|js|jsx)$/.test(name);
  const isTest = /\.(?:test|spec)\.(?:ts|tsx|js|jsx)$/.test(name);
  return isSource && !isTest;
}

function parseSource(path, source) {
  try {
    return parser.parseForESLint(source, {
      ecmaVersion: 2024,
      ecmaFeatures: { jsx: true },
      filePath: path,
      loc: true,
      range: true,
      sourceType: 'module',
    }).ast;
  } catch (error) {
    const result = new Error(error.message);
    result.lineNumber = error.lineNumber;
    return result;
  }
}

function fileLineViolations(path, source) {
  const lines = source.split(/\r?\n/).filter((line) => line.trim().length > 0).length;
  if (lines <= MAX_FILE_LINES) {
    return [];
  }
  return [violation(path, 1, 'typescript-file-lines', `production file has ${lines} nonblank lines (max ${MAX_FILE_LINES})`)];
}

function functionViolations(path, source, ast) {
  const violations = [];
  walk(ast, (node) => {
    if (FUNCTION_NODES.has(node.type)) {
      violations.push(...measureFunction(path, source, node));
    }
    return true;
  });
  return violations;
}

function measureFunction(path, source, node) {
  const name = node.id?.name ?? '<anonymous>';
  const lines = nonblankLineCount(source, node.loc.start.line, node.loc.end.line);
  const metrics = functionMetrics(node);
  const violations = [];
  if (lines > MAX_FUNCTION_LINES) {
    violations.push(violation(path, node.loc.start.line, 'typescript-function-lines', `function ${name} has ${lines} nonblank lines (max ${MAX_FUNCTION_LINES})`));
  }
  if (node.params.length > MAX_POSITIONAL_PARAMETERS) {
    violations.push(violation(path, node.loc.start.line, 'typescript-positional-parameters', `function ${name} has ${node.params.length} positional parameters (max ${MAX_POSITIONAL_PARAMETERS})`));
  }
  return violations.concat(metricViolations(path, node, name, metrics));
}

function metricViolations(path, node, name, metrics) {
  const violations = [];
  if (metrics.maxNesting > MAX_NESTING_DEPTH) {
    violations.push(violation(path, node.loc.start.line, 'typescript-nesting-depth', `function ${name} has nesting depth ${metrics.maxNesting} (max ${MAX_NESTING_DEPTH})`));
  }
  if (metrics.complexity > MAX_CYCLOMATIC_COMPLEXITY) {
    violations.push(violation(path, node.loc.start.line, 'typescript-cyclomatic-complexity', `function ${name} has cyclomatic complexity ${metrics.complexity} (max ${MAX_CYCLOMATIC_COMPLEXITY})`));
  }
  return violations;
}

function functionMetrics(functionNode) {
  const state = { complexity: 1, depth: 0, maxNesting: 0 };
  walkMetricNode(functionNode.body, state, true);
  return state;
}

function walkMetricNode(node, state, root) {
  if (!isAstNode(node) || (!root && FUNCTION_NODES.has(node.type))) {
    return;
  }
  const weight = complexityWeight(node);
  if (NESTED_BRANCH_NODES.has(node.type)) {
    state.complexity += weight;
    enterBranch(state, () => walkChildren(node, (child) => walkMetricNode(child, state, false)));
    return;
  }
  state.complexity += weight;
  walkChildren(node, (child) => walkMetricNode(child, state, false));
}

function complexityWeight(node) {
  if (node.type === 'SwitchStatement') {
    return Math.max(0, node.cases.length - 1);
  }
  if (node.type === 'LogicalExpression' && ['&&', '||'].includes(node.operator)) {
    return 1;
  }
  return NESTED_BRANCH_NODES.has(node.type) || node.type === 'ConditionalExpression' ? 1 : 0;
}

function enterBranch(state, callback) {
  state.depth += 1;
  state.maxNesting = Math.max(state.maxNesting, state.depth);
  callback();
  state.depth -= 1;
}

function fsdImportViolations(path, ast) {
  const violations = [];
  walk(ast, (node) => {
    if (node.type === 'ImportDeclaration' && typeof node.source.value === 'string') {
      violations.push(...checkImport(path, node));
    }
    return true;
  });
  return violations;
}

function checkImport(path, node) {
  const importer = fsdLocation(path);
  const imported = fsdLocation(resolveImportPath(path, node.source.value));
  if (!importer.layer || !imported.layer || sameSlice(importer, imported)) {
    return [];
  }
  const line = node.loc.start.line;
  const violations = [];
  if (!ALLOWED_LAYER_DEPENDENCIES[importer.layer].has(imported.layer)) {
    violations.push(violation(path, line, 'fsd-layer-dependency', `${importer.layer} must not import ${imported.layer}`));
  }
  if (importer.layer === 'entities' && imported.layer === 'entities' && importer.slice !== imported.slice) {
    violations.push(violation(path, line, 'fsd-entity-sibling-import', `entity slice ${importer.slice} must not import ${imported.slice}`));
  }
  return violations;
}

function resolveImportPath(importerPath, importPath) {
  if (importPath.startsWith('src/')) {
    return importPath;
  }
  if (!importPath.startsWith('.')) {
    return '';
  }
  return resolve(dirname(importerPath), importPath);
}

function fsdLocation(path) {
  const segments = path.replaceAll('\\', '/').split('/');
  const sourceIndex = segments.lastIndexOf('src');
  const layer = sourceIndex >= 0 ? segments[sourceIndex + 1] : undefined;
  return { layer: FSD_LAYERS.includes(layer) ? layer : undefined, slice: segments[sourceIndex + 2] };
}

function sameSlice(importer, imported) {
  return importer.layer === imported.layer && importer.slice && importer.slice === imported.slice;
}

function nonblankLineCount(source, start, end) {
  return source
    .split(/\r?\n/)
    .slice(start - 1, end)
    .filter((line) => line.trim().length > 0).length;
}

function walk(node, callback) {
  if (!isAstNode(node) || callback(node) === false) {
    return;
  }
  walkChildren(node, (child) => walk(child, callback));
}

function walkChildren(node, callback) {
  for (const [key, value] of Object.entries(node)) {
    if (!SKIPPED_KEYS.has(key)) {
      visitChild(value, callback);
    }
  }
}

function visitChild(value, callback) {
  if (Array.isArray(value)) {
    value.forEach((child) => visitChild(child, callback));
  } else if (isAstNode(value)) {
    callback(value);
  }
}

function isAstNode(value) {
  return value && typeof value === 'object' && typeof value.type === 'string' && value.loc && value.range;
}

function violation(path, line, rule, message) {
  return { path, line, rule, message };
}

function printViolations(violations) {
  violations.sort(compareViolations).forEach((item) => console.error(`${item.path}:${item.line}: ${item.rule}: ${item.message}`));
}

function compareViolations(left, right) {
  return left.path.localeCompare(right.path) || left.line - right.line || left.rule.localeCompare(right.rule);
}

async function main() {
  const root = process.cwd();
  const violations = await scanFrontend(root);
  if (violations.length > 0) {
    printViolations(violations);
    process.exitCode = 1;
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await main();
}
