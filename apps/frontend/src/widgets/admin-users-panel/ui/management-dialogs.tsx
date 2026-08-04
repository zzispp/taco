import type React from 'react';
import type { RoleOption } from 'src/entities/role';
import type { UserInput, SystemUser } from 'src/entities/user';
import type { Post, TreeSelectNode } from 'src/entities/system';

import { UserDialog, PasswordDialog, RoleAssignDialog, UserImportDialog } from './dialogs';

export function UserManagementDialogs(props: UserManagementDialogsProps) {
  return (
    <>
      <UserDialog
        open={props.creating || !!props.editing}
        editing={Boolean(props.editing)}
        submitting={props.submitting}
        form={props.form}
        roles={props.roles}
        depts={props.deptTree}
        posts={props.posts}
        setForm={props.setForm}
        onClose={props.onDialogClose}
        onSubmit={props.onUserSubmit}
      />
      <RoleAssignDialog
        user={props.roleTarget}
        roles={props.roles}
        selected={props.assignedRoles}
        submitting={props.submitting}
        onSelectedChange={props.onAssignedRolesChange}
        onClose={props.onRoleClose}
        onSubmit={props.onRolesSubmit}
      />
      <PasswordDialog
        user={props.passwordTarget}
        password={props.newPassword}
        submitting={props.submitting}
        onPasswordChange={props.onPasswordChange}
        onClose={props.onPasswordClose}
        onSubmit={props.onPasswordSubmit}
      />
      <UserImportDialog
        open={props.importOpen}
        file={props.importFile}
        updateSupport={props.updateSupport}
        submitting={props.submitting}
        onFileChange={props.onImportFileChange}
        onUpdateSupportChange={props.onUpdateSupportChange}
        onTemplate={props.onImportTemplate}
        onClose={props.onImportClose}
        onSubmit={props.onImportSubmit}
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
