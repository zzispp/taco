'use client';

import type { Role } from 'src/entities/role';

import { useState, useEffect, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { useTable, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';

import { useRoleUsers } from 'src/entities/role';

import { deleteRoleUsers, assignRoleUsers } from 'src/features/role-management';

import { RoleUsersDialogContent } from './role-users-dialog-content';

export function RoleUsersDialog({ role, onClose }: { role: Role | null; onClose: () => void }) {
  const content = useRoleUsersDialog(role);
  return <RoleUsersDialogContent {...content} role={role} onClose={onClose} />;
}

function useRoleUsersDialog(role: Role | null) {
  const { t } = useTranslate('admin');
  const [allocated, setAllocated] = useState(true);
  const [username, setUsername] = useState('');
  const [phonenumber, setPhonenumber] = useState('');
  const [selected, setSelected] = useState<string[]>([]);
  const table = useTable({
    defaultLimit: DEFAULT_TABLE_LIMIT,
    scopeKey: [role?.role_id ?? '', allocated, username, phonenumber].join('\u0000'),
  });
  const resetCursor = table.onResetCursor;
  const users = useRoleUsers(role?.role_id ?? null, table.cursorRequest, {
    allocated,
    username,
    phonenumber,
  });
  useRoleUserSelectionReset({ role, allocated, username, phonenumber, table, setSelected });
  const toggleAll = useCallback(
    (checked: boolean) => setSelected(checked ? users.items.map((user) => user.user_id) : []),
    [users.items]
  );

  const submit = useCallback(async () => {
    if (!role || selected.length === 0) return;
    try {
      if (allocated) await deleteRoleUsers(role.role_id, selected);
      else await assignRoleUsers(role.role_id, selected);
      toast.success(t('messages.saved'));
      setSelected([]);
      resetCursor();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    }
  }, [allocated, resetCursor, role, selected, t]);

  return {
    allocated,
    username,
    phonenumber,
    selected,
    users,
    table,
    submit,
    toggleAll,
    t,
    setAllocated,
    setUsername,
    setPhonenumber,
    setSelected,
  };
}

type SelectionResetOptions = Readonly<{
  role: Role | null;
  allocated: boolean;
  username: string;
  phonenumber: string;
  table: ReturnType<typeof useTable>;
  setSelected: React.Dispatch<React.SetStateAction<string[]>>;
}>;

function useRoleUserSelectionReset(options: SelectionResetOptions) {
  const { role, allocated, username, phonenumber, table, setSelected } = options;
  useEffect(() => {
    setSelected([]);
  }, [allocated, phonenumber, role?.role_id, setSelected, table.cursor, table.limit, username]);
}
