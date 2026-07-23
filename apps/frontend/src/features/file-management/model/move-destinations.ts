import type { FileEntry, FileDirectoryTrailEntry } from 'src/entities/file';

import { ROOT_DIRECTORY_ID } from './constants';

export type MoveFolderCandidate = Pick<
  FileEntry,
  'id' | 'space_id' | 'parent_id' | 'name' | 'type'
>;

export type MoveDirectoryBrowser = Readonly<{
  currentId: string;
  parentId: string;
  trail: readonly FileDirectoryTrailEntry[];
  children: readonly MoveFolderCandidate[];
}>;

type MoveDirectoryBrowserOptions = Readonly<{
  currentId: string;
  trail: readonly FileDirectoryTrailEntry[];
  children: readonly MoveFolderCandidate[];
  target: Pick<FileEntry, 'id' | 'parent_id' | 'type'>;
}>;

export function canMoveToDestination(
  destinationId: string,
  target: Pick<FileEntry, 'id' | 'parent_id' | 'type'>
): boolean {
  if (destinationId === (target.parent_id ?? ROOT_DIRECTORY_ID)) return false;
  return target.type !== 'folder' || destinationId !== target.id;
}

export function buildMoveDirectoryBrowser({
  currentId,
  trail,
  children,
  target,
}: MoveDirectoryBrowserOptions): MoveDirectoryBrowser | null {
  if (!isMoveDirectoryValid(currentId, trail, target)) return null;
  const current = trail.at(-1);
  return {
    currentId,
    parentId: current?.parent_id ?? ROOT_DIRECTORY_ID,
    trail,
    children: children.filter((entry) => entry.id !== target.id),
  };
}

function isMoveDirectoryValid(
  currentId: string,
  trail: readonly FileDirectoryTrailEntry[],
  target: Pick<FileEntry, 'id' | 'type'>
): boolean {
  if (currentId === ROOT_DIRECTORY_ID) return trail.length === 0;
  if (trail.at(-1)?.id !== currentId) return false;
  return target.type !== 'folder' || !trail.some((entry) => entry.id === target.id);
}
