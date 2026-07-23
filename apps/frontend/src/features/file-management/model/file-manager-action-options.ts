import type { fileCapabilities } from './permissions';
import type { useTranslate } from 'src/shared/i18n/use-locales';
import type { FileDirectoryTrailEntry } from 'src/entities/file';
import type { FileManagerState } from './use-file-manager-state';
import type { usePendingMutation } from 'src/shared/api/use-pending-mutation';

export type FileManagerActionOptions = Readonly<{
  state: FileManagerState;
  mutation: ReturnType<typeof usePendingMutation>;
  permissions: ReturnType<typeof fileCapabilities>;
  spaceId?: string;
  directoryTrail: readonly FileDirectoryTrailEntry[];
  refreshMoveFolders: () => Promise<void>;
  t: ReturnType<typeof useTranslate>['t'];
}>;
