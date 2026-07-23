import type { FileDirectoryTrailEntry } from 'src/entities/file';

export type FileDirectoryBreadcrumb = Readonly<{
  id: string | null;
  name: string;
  directoryTrail: readonly string[];
}>;

export function fileDirectoryBreadcrumbs(
  rootName: string,
  directoryTrail: readonly FileDirectoryTrailEntry[]
): readonly FileDirectoryBreadcrumb[] {
  return [
    { id: null, name: rootName, directoryTrail: [] },
    ...directoryTrail.map((entry, index) => ({
      id: entry.id,
      name: entry.name,
      directoryTrail: directoryTrail.slice(0, index + 1).map((item) => item.id),
    })),
  ];
}

export function fileDirectoryParentTrail(
  directoryTrail: readonly FileDirectoryTrailEntry[]
): readonly string[] {
  return directoryTrail.slice(0, -1).map((entry) => entry.id);
}

export function isCurrentDirectoryTrailResolved(
  directoryId: string | null,
  directoryTrail: readonly FileDirectoryTrailEntry[]
): boolean {
  return Boolean(directoryId && directoryTrail.at(-1)?.id === directoryId);
}
