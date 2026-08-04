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

type RoleBindingDialogProps = {
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
};

export function RoleBindingDialog(props: RoleBindingDialogProps) {
  const { t } = useTranslate('admin');
  const options = useMemo(() => flattenTreeNodes(props.nodes), [props.nodes]);
  const showDeptTree = props.type !== 'depts' || props.dataScope === '2';

  return (
    <Dialog fullWidth maxWidth="md" open={Boolean(props.role)} onClose={props.onClose}>
      <RoleBindingDialogTitle role={props.role} type={props.type} t={t} />
      <RoleBindingDialogContent props={props} options={options} showDeptTree={showDeptTree} t={t} />
      <RoleBindingDialogActions
        submitting={props.submitting}
        t={t}
        onClose={props.onClose}
        onSubmit={props.onSubmit}
      />
    </Dialog>
  );
}

function RoleBindingDialogContent({
  props,
  options,
  showDeptTree,
  t,
}: {
  props: RoleBindingDialogProps;
  options: ReturnType<typeof flattenTreeNodes>;
  showDeptTree: boolean;
  t: ReturnType<typeof useTranslate>['t'];
}) {
  return (
    <DialogContent>
      {props.type === 'depts' && <RoleBindingDataScope props={props} t={t} />}
      {props.loading ? (
        <Box sx={{ py: 4, color: 'text.secondary' }}>{t('messages.loadingPermissions')}</Box>
      ) : showDeptTree ? (
        <Scrollbar sx={{ maxHeight: 520 }}>
          <TreeSelector
            items={options}
            selected={props.selected}
            strict={props.strict}
            onChange={props.onSelectedChange}
            onStrictChange={props.onStrictChange}
            onResolvedSelectionChange={props.onResolvedSelectionChange}
          />
        </Scrollbar>
      ) : (
        <Box sx={{ py: 3, color: 'text.secondary' }}>
          <Typography variant="body2">{t('messages.dataScopePresetNoDeptTree')}</Typography>
        </Box>
      )}
    </DialogContent>
  );
}

function RoleBindingDataScope({
  props,
  t,
}: {
  props: Pick<RoleBindingDialogProps, 'dataScope' | 'onDataScopeChange'>;
  t: ReturnType<typeof useTranslate>['t'];
}) {
  return (
    <TextField
      fullWidth
      select
      size="small"
      label={t('fields.dataScope')}
      value={props.dataScope}
      sx={{ mt: 1, mb: 2 }}
      onChange={(event) => props.onDataScopeChange(event.target.value)}
    >
      {['1', '2', '3', '4', '5'].map((value) => (
        <MenuItem key={value} value={value}>
          {dataScopeLabel(value, t)}
        </MenuItem>
      ))}
    </TextField>
  );
}

function RoleBindingDialogTitle({
  role,
  type,
  t,
}: Pick<RoleBindingDialogProps, 'role' | 'type'> & { t: ReturnType<typeof useTranslate>['t'] }) {
  return (
    <DialogTitle>
      {t(type === 'menus' ? 'dialogs.roleMenuPermissions' : 'dialogs.roleDataScope', {
        name: role ? translatedRoleName(role) : '',
      })}
    </DialogTitle>
  );
}

function RoleBindingDialogActions({
  t,
  submitting,
  onClose,
  onSubmit,
}: Pick<RoleBindingDialogProps, 'submitting' | 'onClose' | 'onSubmit'> & {
  t: ReturnType<typeof useTranslate>['t'];
}) {
  return (
    <DialogActions>
      <Button variant="outlined" onClick={onClose}>
        {t('common.cancel')}
      </Button>
      <Button variant="contained" loading={submitting} onClick={onSubmit}>
        {t('actions.savePermissions')}
      </Button>
    </DialogActions>
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
