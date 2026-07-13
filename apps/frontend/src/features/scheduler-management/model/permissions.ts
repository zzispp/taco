export type SchedulerEditorPermissions = Readonly<{
  canImport: boolean;
  canEdit: boolean;
}>;

export function canPreviewCron(permissions: SchedulerEditorPermissions): boolean {
  return permissions.canImport || permissions.canEdit;
}
