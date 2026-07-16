import { tmpdir } from 'node:os';
import { join, resolve, relative } from 'node:path';
import { it, expect, describe, afterEach } from 'vitest';
import { rmSync, mkdtempSync, writeFileSync } from 'node:fs';

import { assertNoEnvironmentFiles } from './environment-files';

const temporaryDirectories: string[] = [];

afterEach(() => {
  temporaryDirectories.splice(0).forEach((directory) => {
    rmSync(directory, { recursive: true, force: true });
  });
});

describe('environment file guard', () => {
  it('rejects Next dotenv filenames in every guarded directory', () => {
    const workspaceRoot = createTemporaryDirectory();
    const frontendRoot = createTemporaryDirectory();
    const forbiddenFiles = [
      join(workspaceRoot, '.env'),
      join(workspaceRoot, '.env.'),
      join(workspaceRoot, '.env.example'),
      join(frontendRoot, '.env.local'),
      join(frontendRoot, '.env.production.local'),
    ];

    forbiddenFiles.forEach((filePath) => writeFileSync(filePath, 'SECRET=value'));

    const guardedDirectories = [workspaceRoot, frontendRoot].map((directory) =>
      relative(process.cwd(), directory)
    );
    const expectedAbsolutePaths = forbiddenFiles.map((filePath) => resolve(filePath)).join('\n');

    expect(() => assertNoEnvironmentFiles(guardedDirectories)).toThrowError(expectedAbsolutePaths);
  });

  it('allows process environment usage and similarly named non-dotenv files', () => {
    const workspaceRoot = createTemporaryDirectory();
    const frontendRoot = createTemporaryDirectory();

    ['.envrc', '.environment', 'env.local'].forEach((fileName) => {
      writeFileSync(join(workspaceRoot, fileName), 'export VALUE=allowed');
    });

    expect(() => assertNoEnvironmentFiles([workspaceRoot, frontendRoot])).not.toThrow();
  });
});

function createTemporaryDirectory(): string {
  const directory = mkdtempSync(join(tmpdir(), 'taco-next-env-'));
  temporaryDirectories.push(directory);
  return directory;
}
