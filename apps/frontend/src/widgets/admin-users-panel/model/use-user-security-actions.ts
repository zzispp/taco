import type { TranslateFn } from 'src/shared/i18n';
import type { SystemUser } from 'src/entities/user';
import type { PasswordPolicy } from 'src/entities/system';

import { useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';

import { getUserRoles, updateUserRoles, resetUserPassword } from 'src/features/user-management';

import { validateAdminPassword, adminUserValidationMessages } from './user-form-validation';

type UserSecurityState = {
  setSubmitting: (value: boolean) => void;
  roleTarget: SystemUser | null;
  setRoleTarget: (value: SystemUser | null) => void;
  assignedRoles: string[];
  setAssignedRoles: (value: string[]) => void;
  passwordTarget: SystemUser | null;
  setPasswordTarget: (value: SystemUser | null) => void;
  newPassword: string;
  setNewPassword: (value: string) => void;
};

type UserRoleActionOptions = {
  state: UserSecurityState;
  t: TranslateFn;
  resetList: () => void;
};

type UserSecurityActionOptions = UserRoleActionOptions & {
  passwordPolicy: PasswordPolicy | undefined;
};

export function useUserRoleActions({ state, t, resetList }: UserRoleActionOptions) {
  const openRoles = useCallback(
    async (user: SystemUser) => {
      state.setRoleTarget(user);
      try {
        const payload = await getUserRoles(user.user_id);
        state.setAssignedRoles(payload.role_ids);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.loadBindingsFailed'));
      }
    },
    [state, t]
  );
  const submitRoles = useCallback(async () => {
    if (!state.roleTarget) return;
    state.setSubmitting(true);
    try {
      await updateUserRoles(state.roleTarget.user_id, state.assignedRoles);
      toast.success(t('messages.rolePermissionsUpdated'));
      resetList();
      state.setRoleTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveBindingsFailed'));
    } finally {
      state.setSubmitting(false);
    }
  }, [resetList, state, t]);

  return { openRoles, submitRoles };
}

export function useUserPasswordAction(options: UserSecurityActionOptions) {
  const { state, t, passwordPolicy, resetList } = options;
  const submitPassword = useCallback(async () => {
    if (!state.passwordTarget) return;
    const error = validateAdminPassword({
      password: state.newPassword,
      username: state.passwordTarget.username,
      policy: passwordPolicy,
      messages: adminUserValidationMessages(t),
    });
    if (error) {
      toast.error(error);
      return;
    }
    await submitPasswordReset({
      state,
      t,
      userId: state.passwordTarget.user_id,
      resetList,
    });
  }, [passwordPolicy, resetList, state, t]);

  return { submitPassword };
}

async function submitPasswordReset({ state, t, userId, resetList }: UserPasswordResetOptions) {
  state.setSubmitting(true);
  try {
    await resetUserPassword(userId, state.newPassword);
    toast.success(t('messages.saved'));
    resetList();
    state.setPasswordTarget(null);
    state.setNewPassword('');
  } catch (error) {
    toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
  } finally {
    state.setSubmitting(false);
  }
}

type UserPasswordResetOptions = UserRoleActionOptions & { userId: string };
