export type NoticeManagementPermissions = Readonly<{
  canList: boolean;
  canQuery: boolean;
  canAdd: boolean;
  canEdit: boolean;
  canRemove: boolean;
}>;

export function noticeManagementCapabilities(permissions: NoticeManagementPermissions) {
  return {
    ...permissions,
    canOpenDetail: permissions.canQuery,
    canViewReaders: permissions.canList,
  };
}
