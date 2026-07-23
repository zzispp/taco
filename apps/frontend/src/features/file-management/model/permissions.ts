import type { PermissionChecker } from 'src/entities/session';

export const FILE_PERMISSION = {
  list: 'file:asset:list',
  query: 'file:asset:query',
  download: 'file:asset:download',
  upload: 'file:asset:upload',
  folderAdd: 'file:folder:add',
  edit: 'file:asset:edit',
  remove: 'file:asset:remove',
  restore: 'file:asset:restore',
  purge: 'file:asset:purge',
  spaceList: 'file:space:list',
  spaceQuota: 'file:space:quota',
  providerQuery: 'file:provider:query',
} as const;

export function fileCapabilities(check: PermissionChecker) {
  return {
    canList: check(FILE_PERMISSION.list),
    canQuery: check(FILE_PERMISSION.query),
    canDownload: check(FILE_PERMISSION.download),
    canUpload: check(FILE_PERMISSION.upload),
    canAddFolder: check(FILE_PERMISSION.folderAdd),
    canEdit: check(FILE_PERMISSION.edit),
    canRemove: check(FILE_PERMISSION.remove),
    canRestore: check(FILE_PERMISSION.restore),
    canPurge: check(FILE_PERMISSION.purge),
    canListSpaces: check(FILE_PERMISSION.spaceList),
    canEditQuota: check(FILE_PERMISSION.spaceQuota),
    canQueryProvider: check(FILE_PERMISSION.providerQuery),
  };
}
