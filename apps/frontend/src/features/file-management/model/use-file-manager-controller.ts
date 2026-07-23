'use client';

import type { FileListQuery, FileDirectoryTrailEntry } from 'src/entities/file';

import { useMemo } from 'react';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { useTable, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';
import { usePendingMutation } from 'src/shared/api/use-pending-mutation';

import { useAuthContext, usePermissionChecker } from 'src/entities/session';
import {
  useFileEntry,
  useFileEntries,
  useFileOverview,
  fileEntrySortField,
  useFileDirectoryTrail,
} from 'src/entities/file';

import { fileCapabilities } from './permissions';
import { fileQuery, ROOT_DIRECTORY_ID } from './constants';
import { useFileManagerState } from './use-file-manager-state';
import { useFileSpaceSelector } from './use-file-space-selector';
import { useFileManagerActions } from './use-file-manager-actions';
import { isCurrentDirectoryTrailResolved } from './directory-navigation';
import { canMoveToDestination, buildMoveDirectoryBrowser } from './move-destinations';
import { type FileManagerRoute, EMPTY_FILE_MANAGER_ROUTE } from './file-manager-route';

export function useFileManagerController(route: FileManagerRoute = EMPTY_FILE_MANAGER_ROUTE) {
  const context = useFileManagerContext(route);
  const moveResources = useFileManagerMoveResources(
    context.state,
    context.permissions.canEdit,
    context.spaceId
  );
  const mutation = usePendingMutation();
  const actions = useFileManagerActions({
    state: context.state,
    mutation,
    permissions: context.permissions,
    spaceId: context.spaceId,
    directoryTrail: context.directoryTrail,
    refreshMoveFolders: moveResources.folders.refresh,
    t: context.t,
  });
  return {
    state: context.state,
    permissions: context.permissions,
    resources: { ...context.resources, ...moveResources, spaceId: context.spaceId },
    actions,
    pending: mutation.pending,
  };
}

function useFileManagerContext(route: FileManagerRoute) {
  const { t } = useTranslate('admin');
  const state = useFileManagerState(route);
  const { user } = useAuthContext();
  const permissions = fileCapabilities(usePermissionChecker());
  const resources = useFileManagerResources(state, permissions, user?.user_id);
  return { t, state, permissions, ...resources };
}

function useFileManagerResources(
  state: ReturnType<typeof useFileManagerState>,
  permissions: ReturnType<typeof fileCapabilities>,
  currentUserId: string | undefined
) {
  const overview = useFileOverview(state.spaceId, permissions.canQuery);
  const spaceId = resolveFileManagerSpaceId(state.spaceId, overview.data?.space_id, currentUserId);
  const query = useManagerFileQuery(state, spaceId);
  const entries = useFileEntries(
    state.table.cursorRequest,
    query,
    permissions.canList && Boolean(spaceId)
  );
  const directory = useManagerDirectoryTrail(state, permissions.canList);
  const detail = useFileEntry(state.detailId, permissions.canQuery);
  const spaceSelector = useFileSpaceSelector({
    selectedSpaceId: state.spaceId,
    enabled: permissions.canListSpaces,
  });
  return {
    spaceId,
    directoryTrail: directory.trail,
    resources: {
      entries,
      directoryTrail: directory.trail,
      directoryTrailError: directory.error,
      directoryTrailLoading: directory.loading,
      detail,
      overview,
      spaceSelector,
      currentUserId,
    },
  };
}

function useManagerFileQuery(
  state: ReturnType<typeof useFileManagerState>,
  spaceId: string | undefined
) {
  return useMemo(
    () =>
      fileQuery({
        filters: state.filters,
        mode: state.mode,
        parentId: state.parentId,
        spaceId,
        sortBy: fileEntrySortField(state.table.orderBy),
        sortOrder: state.table.order,
      }),
    [spaceId, state.filters, state.mode, state.parentId, state.table.order, state.table.orderBy]
  );
}

function useManagerDirectoryTrail(state: ReturnType<typeof useFileManagerState>, canList: boolean) {
  const resource = useFileDirectoryTrail(state.mode === 'active' ? state.parentId : null, canList);
  return {
    trail: currentDirectoryTrail(state.parentId, resource.data),
    error: resource.error,
    loading: resource.isLoading,
  };
}

function currentDirectoryTrail(
  directoryId: string | null,
  directoryTrail: readonly FileDirectoryTrailEntry[] | undefined
) {
  if (!directoryTrail || !isCurrentDirectoryTrailResolved(directoryId, directoryTrail)) return [];
  return directoryTrail;
}

function useFileManagerMoveResources(
  state: ReturnType<typeof useFileManagerState>,
  canEdit: boolean,
  spaceId: string | undefined
) {
  const moveSpaceId = state.moveTarget?.space_id ?? spaceId;
  const table = useTable({
    defaultLimit: DEFAULT_TABLE_LIMIT,
    defaultOrderBy: 'name',
    scopeKey: moveDirectoryScopeKey(moveSpaceId, state.moveDestinationId),
  });
  const enabled = canEdit && Boolean(state.moveTarget && moveSpaceId);
  const folders = useFileEntries(
    table.cursorRequest,
    moveFolderQuery(moveSpaceId, state.moveDestinationId),
    enabled
  );
  const trailResource = useFileDirectoryTrail(moveDirectoryTrailId(state, enabled), enabled);
  const trail = currentDirectoryTrail(moveDirectoryTrailId(state, enabled), trailResource.data);
  const moveBrowser = buildMoveBrowser(state, trail, folders.items);
  const canSubmitMove = Boolean(
    state.moveTarget &&
    moveBrowser &&
    canMoveToDestination(state.moveDestinationId, state.moveTarget)
  );
  return {
    folders,
    moveFolderTable: table,
    moveBrowser,
    canSubmitMove,
    moveTrailError: trailResource.error,
    moveTrailLoading: trailResource.isLoading,
  };
}

function moveFolderQuery(spaceId: string | undefined, parentId: string): FileListQuery {
  return {
    ...(spaceId ? { space_id: spaceId } : {}),
    parent_id: parentId,
    kind: 'folder',
    trashed: false,
    sort_by: 'name',
    sort_order: 'asc',
  };
}

function moveDirectoryScopeKey(spaceId: string | undefined, parentId: string) {
  return JSON.stringify([spaceId ?? null, parentId]);
}

function moveDirectoryTrailId(state: ReturnType<typeof useFileManagerState>, enabled: boolean) {
  if (!enabled || state.moveDestinationId === ROOT_DIRECTORY_ID) return null;
  return state.moveDestinationId;
}

function buildMoveBrowser(
  state: ReturnType<typeof useFileManagerState>,
  trail: readonly FileDirectoryTrailEntry[],
  children: ReturnType<typeof useFileEntries>['items']
) {
  if (!state.moveTarget) return null;
  return buildMoveDirectoryBrowser({
    currentId: state.moveDestinationId,
    trail,
    children,
    target: state.moveTarget,
  });
}

export type FileManagerController = ReturnType<typeof useFileManagerController>;

export function resolveFileManagerSpaceId(
  selectedSpaceId: string | undefined,
  overviewSpaceId: string | undefined,
  currentUserId: string | undefined
) {
  return selectedSpaceId ?? overviewSpaceId ?? currentUserId;
}
