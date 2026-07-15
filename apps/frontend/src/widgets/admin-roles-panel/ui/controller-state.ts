import type { Role } from 'src/entities/role';
import type { TreeSelectNode } from 'src/entities/system';

import { useState } from 'react';

import { DEFAULT_FORM } from './constants';

export function useRoleDialogState() {
  const [form, setForm] = useState(DEFAULT_FORM);
  const [editing, setEditing] = useState<Role | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Role | null>(null);
  const [batchDeleteOpen, setBatchDeleteOpen] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);
  const [usersTarget, setUsersTarget] = useState<Role | null>(null);

  return {
    form,
    setForm,
    editing,
    setEditing,
    creating,
    setCreating,
    submitting,
    setSubmitting,
    deleteTarget,
    setDeleteTarget,
    batchDeleteOpen,
    setBatchDeleteOpen,
    selected,
    setSelected,
    usersTarget,
    setUsersTarget,
  };
}

export function useRoleBindingState() {
  const [target, setTarget] = useState<Role | null>(null);
  const [type, setType] = useState<'menus' | 'depts'>('menus');
  const [selected, setSelected] = useState<string[]>([]);
  const [resolvedDeptBindings, setResolvedDeptBindings] = useState<string[]>([]);
  const [nodes, setNodes] = useState<TreeSelectNode[]>([]);
  const [strict, setStrict] = useState(true);
  const [dataScope, setDataScope] = useState('5');
  const [loading, setLoading] = useState(false);

  return {
    target,
    setTarget,
    type,
    setType,
    selected,
    setSelected,
    resolvedDeptBindings,
    setResolvedDeptBindings,
    nodes,
    setNodes,
    strict,
    setStrict,
    dataScope,
    setDataScope,
    loading,
    setLoading,
  };
}
