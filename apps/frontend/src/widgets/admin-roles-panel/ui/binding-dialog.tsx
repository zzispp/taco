'use client';

import type { Role } from 'src/entities/role';
import type { TreeSelectNode } from 'src/entities/system';

import { useMemo } from 'react';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { translatedRoleName } from 'src/entities/role';

import { TreeSelector } from './tree-selector';
import { dataScopeLabel } from './role-dialog';

export function RoleBindingDialog({
  role,
  type,
  nodes,
  selected,
  strict,
  dataScope,
  loading,
  submitting,
  onSelectedChange,
  onStrictChange,
  onDataScopeChange,
  onResolvedSelectionChange,
  onClose,
  onSubmit,
}: {
  role: Role | null;
  type: 'menus' | 'depts';
  nodes: TreeSelectNode[];
  selected: string[];
  strict: boolean;
  dataScope: string;
  loading: boolean;
  submitting: boolean;
  onSelectedChange: (value: string[]) => void;
  onStrictChange: (value: boolean) => void;
  onDataScopeChange: (value: string) => void;
  onResolvedSelectionChange?: (value: string[]) => void;
  onClose: () => void;
  onSubmit: () => void;
}) {
  const { t } = useTranslate('admin');
  const options = useMemo(() => flattenTreeNodes(nodes), [nodes]);
  const showDeptTree = type !== 'depts' || dataScope === '2';

  return (
    <Dialog fullWidth maxWidth="md" open={!!role} onClose={onClose}>
      <DialogTitle>
        {t(type === 'menus' ? 'dialogs.roleMenuPermissions' : 'dialogs.roleDataScope', {
          name: role ? translatedRoleName(role) : '',
        })}
      </DialogTitle>
      <DialogContent>
        {type === 'depts' && (
          <TextField
            fullWidth
            select
            size="small"
            label={t('fields.dataScope')}
            value={dataScope}
            sx={{ mt: 1, mb: 2 }}
            onChange={(event) => onDataScopeChange(event.target.value)}
          >
            {['1', '2', '3', '4', '5'].map((value) => (
              <MenuItem key={value} value={value}>
                {dataScopeLabel(value, t)}
              </MenuItem>
            ))}
          </TextField>
        )}
        {loading ? (
          <Box sx={{ py: 4, color: 'text.secondary' }}>{t('messages.loadingPermissions')}</Box>
        ) : showDeptTree ? (
          <Scrollbar sx={{ maxHeight: 520 }}>
            <TreeSelector
              items={options}
              selected={selected}
              strict={strict}
              onChange={onSelectedChange}
              onStrictChange={onStrictChange}
              onResolvedSelectionChange={onResolvedSelectionChange}
            />
          </Scrollbar>
        ) : (
          <Box sx={{ py: 3, color: 'text.secondary' }}>
            <Typography variant="body2">{t('messages.dataScopePresetNoDeptTree')}</Typography>
          </Box>
        )}
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button variant="contained" loading={submitting} onClick={onSubmit}>
          {t('actions.savePermissions')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function flattenTreeNodes(
  nodes: TreeSelectNode[]
): { id: string; parentId: string; label: string }[] {
  return nodes.flatMap((node) => [
    { id: node.id, parentId: node.parent_id, label: node.label },
    ...flattenTreeNodes(node.children),
  ]);
}
