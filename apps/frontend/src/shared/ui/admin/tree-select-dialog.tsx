'use client';

import { useMemo, useState } from 'react';

import Box from '@mui/material/Box';
import List from '@mui/material/List';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Collapse from '@mui/material/Collapse';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import DialogTitle from '@mui/material/DialogTitle';
import ListItemText from '@mui/material/ListItemText';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import ListItemButton from '@mui/material/ListItemButton';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

export type TreeSelectDialogNode = {
  id: string;
  label: string;
  disabled: boolean;
  children: TreeSelectDialogNode[];
};

type TreeSelectFieldProps = {
  label: string;
  value: string;
  nodes: TreeSelectDialogNode[];
  rootLabel?: string;
  rootValue?: string;
  allowRoot?: boolean;
  onChange: (value: string) => void;
};

export function TreeSelectField({
  label,
  value,
  nodes,
  rootLabel,
  rootValue = '0',
  allowRoot = true,
  onChange,
}: TreeSelectFieldProps) {
  const { t } = useTranslate('admin');
  const [open, setOpen] = useState(false);
  const rootText = rootLabel ?? t('common.root');
  const selectedLabel = useMemo(
    () => selectedNodeLabel(nodes, value) ?? (allowRoot && value === rootValue ? rootText : ''),
    [allowRoot, nodes, rootText, rootValue, value]
  );

  return (
    <>
      <TextField
        fullWidth
        label={label}
        value={selectedLabel}
        onClick={() => setOpen(true)}
        InputProps={{ readOnly: true }}
      />
      <TreeSelectDialog
        open={open}
        nodes={nodes}
        value={value}
        rootLabel={rootText}
        rootValue={rootValue}
        allowRoot={allowRoot}
        onClose={() => setOpen(false)}
        onChange={(next) => {
          onChange(next);
          setOpen(false);
        }}
      />
    </>
  );
}

function TreeSelectDialog({
  open,
  nodes,
  value,
  rootLabel,
  rootValue,
  allowRoot,
  onClose,
  onChange,
}: {
  open: boolean;
  nodes: TreeSelectDialogNode[];
  value: string;
  rootLabel: string;
  rootValue: string;
  allowRoot: boolean;
  onClose: () => void;
  onChange: (value: string) => void;
}) {
  const { t } = useTranslate('admin');
  const allIds = useMemo(() => flattenIds(nodes), [nodes]);
  const [expanded, setExpanded] = useState<string[]>([]);
  const expandedIds = expanded.length > 0 ? expanded : allIds;

  return (
    <Dialog fullWidth maxWidth="sm" open={open} onClose={onClose}>
      <DialogTitle>{t('common.select')}</DialogTitle>
      <DialogContent>
        <Box sx={{ display: 'flex', gap: 1, mb: 1 }}>
          <Button size="small" onClick={() => setExpanded(allIds)}>
            {t('actions.expandAll')}
          </Button>
          <Button size="small" onClick={() => setExpanded([])}>
            {t('actions.collapseAll')}
          </Button>
        </Box>
        <List disablePadding>
          {allowRoot && (
            <ListItemButton
              dense
              selected={value === rootValue}
              onClick={() => onChange(rootValue)}
            >
              <Box sx={{ width: 34 }} />
              <ListItemText primary={rootLabel} />
            </ListItemButton>
          )}
          {nodes.map((node) => (
            <TreeSelectRow
              key={node.id}
              node={node}
              level={0}
              value={value}
              expanded={expandedIds}
              onToggle={(id) => setExpanded(toggle(expandedIds, id))}
              onChange={onChange}
            />
          ))}
        </List>
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onClose}>
          {t('common.cancel')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function TreeSelectRow({
  node,
  level,
  value,
  expanded,
  onToggle,
  onChange,
}: {
  node: TreeSelectDialogNode;
  level: number;
  value: string;
  expanded: string[];
  onToggle: (id: string) => void;
  onChange: (value: string) => void;
}) {
  const open = expanded.includes(node.id);
  const hasChildren = node.children.length > 0;

  return (
    <>
      <ListItemButton
        dense
        disabled={node.disabled}
        selected={value === node.id}
        sx={{ pl: 1 + level * 2 }}
        onClick={() => onChange(node.id)}
      >
        {hasChildren ? (
          <IconButton
            size="small"
            onClick={(event) => {
              event.stopPropagation();
              onToggle(node.id);
            }}
          >
            <Iconify icon={open ? 'eva:arrow-ios-downward-fill' : 'eva:arrow-ios-forward-fill'} />
          </IconButton>
        ) : (
          <Box sx={{ width: 34 }} />
        )}
        <ListItemText primary={node.label} />
      </ListItemButton>
      {hasChildren && (
        <Collapse in={open}>
          {node.children.map((child) => (
            <TreeSelectRow
              key={child.id}
              node={child}
              level={level + 1}
              value={value}
              expanded={expanded}
              onToggle={onToggle}
              onChange={onChange}
            />
          ))}
        </Collapse>
      )}
    </>
  );
}

function selectedNodeLabel(nodes: TreeSelectDialogNode[], value: string): string | null {
  for (const node of nodes) {
    if (node.id === value) return node.label;
    const child = selectedNodeLabel(node.children, value);
    if (child) return child;
  }
  return null;
}

function flattenIds(nodes: TreeSelectDialogNode[]): string[] {
  return nodes.flatMap((node) => [node.id, ...flattenIds(node.children)]);
}

function toggle(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}
