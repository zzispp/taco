import { readdirSync } from 'node:fs';
import { join, resolve } from 'node:path';

const DOTENV_FILE_PATTERN = /^\.env(?:\..*)?$/;
const DOTENV_ERROR_MESSAGE =
  'Environment files are not supported. Pass configuration through process environment variables instead:';

export function assertNoEnvironmentFiles(directories: readonly string[]): void {
  const forbiddenFiles = directories.flatMap(findEnvironmentFiles);

  if (forbiddenFiles.length === 0) {
    return;
  }

  throw new Error(`${DOTENV_ERROR_MESSAGE}\n${forbiddenFiles.join('\n')}`);
}

function findEnvironmentFiles(directory: string): readonly string[] {
  const absoluteDirectory = resolve(directory);
  const fileNames = readdirSync(absoluteDirectory)
    .filter((fileName) => DOTENV_FILE_PATTERN.test(fileName))
    .sort((left, right) => left.localeCompare(right));

  return fileNames.map((fileName) => join(absoluteDirectory, fileName));
}
