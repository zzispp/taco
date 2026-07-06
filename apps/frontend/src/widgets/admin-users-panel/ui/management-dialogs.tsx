import type React from 'react';
import type { RoleOption } from 'src/entities/role';
import type { UserInput, SystemUser } from 'src/entities/user';
import type { Post, TreeSelectNode } from 'src/entities/system';

import { UserDialog, PasswordDialog, RoleAssignDialog, UserImportDialog } from './dialogs';

export function UserManagementDialogs({
  form,
  roles,
  posts,
  deptTree,
  editing,
  creating,
  submitting,
  roleTarget,
  assignedRoles,
  passwordTarget,
  newPassword,
  importOpen,
  importFile,
  updateSupport,
  setForm,
  onDialogClose,
  onUserSubmit,
  onAssignedRolesChange,
  onRoleClose,
  onRolesSubmit,
  onPasswordChange,
  onPasswordClose,
  onPasswordSubmit,
  onImportFileChange,
  onUpdateSupportChange,
  onImportTemplate,
  onImportClose,
  onImportSubmit,
}: UserManagementDialogsProps) {
  return (
    <>
      <UserDialog
        open={creating || !!editing}
        editing={!!editing}
        submitting={submitting}
        form={form}
        roles={roles}
        depts={deptTree}
        posts={posts}
        setForm={setForm}
        onClose={onDialogClose}
        onSubmit={onUserSubmit}
      />
      <RoleAssignDialog
        user={roleTarget}
        roles={roles}
        selected={assignedRoles}
        submitting={submitting}
        onSelectedChange={onAssignedRolesChange}
        onClose={onRoleClose}
        onSubmit={onRolesSubmit}
      />
      <PasswordDialog
        user={passwordTarget}
        password={newPassword}
        submitting={submitting}
        onPasswordChange={onPasswordChange}
        onClose={onPasswordClose}
        onSubmit={onPasswordSubmit}
      />
      <UserImportDialog
        open={importOpen}
        file={importFile}
        updateSupport={updateSupport}
        submitting={submitting}
        onFileChange={onImportFileChange}
        onUpdateSupportChange={onUpdateSupportChange}
        onTemplate={onImportTemplate}
        onClose={onImportClose}
        onSubmit={onImportSubmit}
      />
    </>
  );
}

type UserManagementDialogsProps = {
  form: UserInput;
  roles: RoleOption[];
  posts: Post[];
  deptTree: TreeSelectNode[];
  editing: SystemUser | null;
  creating: boolean;
  submitting: boolean;
  roleTarget: SystemUser | null;
  assignedRoles: string[];
  passwordTarget: SystemUser | null;
  newPassword: string;
  importOpen: boolean;
  importFile: File | null;
  updateSupport: boolean;
  setForm: React.Dispatch<React.SetStateAction<UserInput>>;
  onDialogClose: () => void;
  onUserSubmit: () => void;
  onAssignedRolesChange: (roles: string[]) => void;
  onRoleClose: () => void;
  onRolesSubmit: () => void;
  onPasswordChange: (password: string) => void;
  onPasswordClose: () => void;
  onPasswordSubmit: () => void;
  onImportFileChange: (file: File | null) => void;
  onUpdateSupportChange: (value: boolean) => void;
  onImportTemplate: () => Promise<void>;
  onImportClose: () => void;
  onImportSubmit: () => void;
};
