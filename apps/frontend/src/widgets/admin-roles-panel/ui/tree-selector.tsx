'use client';

import { useMemo, useState, useEffect, useCallback } from 'react';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import List from '@mui/material/List';
import Paper from '@mui/material/Paper';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Switch from '@mui/material/Switch';
import Collapse from '@mui/material/Collapse';
import Checkbox from '@mui/material/Checkbox';
import IconButton from '@mui/material/IconButton';
import ButtonGroup from '@mui/material/ButtonGroup';
import ListItemText from '@mui/material/ListItemText';
import ListItemButton from '@mui/material/ListItemButton';
import FormControlLabel from '@mui/material/FormControlLabel';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

export type TreeOption = { id: string; parentId: string; label: string };

type TreeNode = TreeOption & { children: TreeNode[] };
type TreeNodeState = { checked: boolean; indeterminate: boolean };

type TreeSelectorProps = {
  items: TreeOption[];
  selected: string[];
  strict: boolean;
  onChange: (selected: string[]) => void;
  onStrictChange: (value: boolean) => void;
  onResolvedSelectionChange?: (selected: string[]) => void;
};

export function TreeSelector({
  items,
  selected,
  strict,
  onChange,
  onStrictChange,
  onResolvedSelectionChange,
}: TreeSelectorProps) {
  const tree = useMemo(() => buildTree(items), [items]);
  const allIds = useMemo(() => items.map((item) => item.id), [items]);
  const selectedSet = useMemo(() => new Set(selected), [selected]);
  const [expanded, setExpanded] = useState<string[]>(() => tree.map((node) => node.id));
  const resolvedSelection = useMemo(
    () => selectedWithAncestors(selected, items),
    [items, selected]
  );

  useEffect(() => {
    onResolvedSelectionChange?.(resolvedSelection);
  }, [onResolvedSelectionChange, resolvedSelection]);

  const toggleExpanded = useCallback((id: string) => {
    setExpanded((current) =>
      current.includes(id) ? current.filter((item) => item !== id) : [...current, id]
    );
  }, []);

  return (
    <Box>
      <SelectorToolbar
        selectedCount={selected.length}
        strict={strict}
        onExpandAll={() => setExpanded(allIds)}
        onCollapseAll={() => setExpanded([])}
        onSelectAll={() => onChange(allIds)}
        onUnselectAll={() => onChange([])}
        onStrictChange={onStrictChange}
      />
      <List disablePadding>
        {tree.map((node) => (
          <TreeNodeRow
            key={node.id}
            node={node}
            level={0}
            selected={selected}
            selectedSet={selectedSet}
            strict={strict}
            expanded={expanded}
            onToggleExpanded={toggleExpanded}
            onChange={onChange}
          />
        ))}
      </List>
    </Box>
  );
}

export function selectedWithAncestors(selected: string[], items: TreeOption[]) {
  const selectedSet = new Set(selected);
  const parentById = new Map(items.map((item) => [item.id, item.parentId]));
  selected.forEach((id) => addAncestors(id, parentById, selectedSet));
  return items.map((item) => item.id).filter((id) => selectedSet.has(id));
}

function addAncestors(id: string, parentById: Map<string, string>, selected: Set<string>) {
  const parentId = parentById.get(id);
  if (!parentId || !parentById.has(parentId) || selected.has(parentId)) return;
  selected.add(parentId);
  addAncestors(parentId, parentById, selected);
}

function SelectorToolbar({
  selectedCount,
  strict,
  onExpandAll,
  onCollapseAll,
  onSelectAll,
  onUnselectAll,
  onStrictChange,
}: {
  selectedCount: number;
  strict: boolean;
  onExpandAll: () => void;
  onCollapseAll: () => void;
  onSelectAll: () => void;
  onUnselectAll: () => void;
  onStrictChange: (value: boolean) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Paper
      variant="outlined"
      sx={{ position: 'sticky', top: 0, zIndex: 1, p: 1.25, mb: 1.5, bgcolor: 'background.paper' }}
    >
      <Stack
        direction={{ xs: 'column', md: 'row' }}
        spacing={1.25}
        alignItems={{ xs: 'stretch', md: 'center' }}
        justifyContent="space-between"
      >
        <ButtonGroup variant="outlined" size="small" sx={{ flexWrap: 'wrap' }}>
          <Button startIcon={<Iconify icon="eva:expand-fill" />} onClick={onExpandAll}>
            {t('actions.expandAll')}
          </Button>
          <Button startIcon={<Iconify icon="eva:collapse-fill" />} onClick={onCollapseAll}>
            {t('actions.collapseAll')}
          </Button>
          <Button startIcon={<Iconify icon="eva:done-all-fill" />} onClick={onSelectAll}>
            {t('actions.selectAll')}
          </Button>
          <Button startIcon={<Iconify icon="eva:minus-circle-fill" />} onClick={onUnselectAll}>
            {t('actions.unselectAll')}
          </Button>
        </ButtonGroup>
        <Stack
          direction="row"
          spacing={1.25}
          alignItems="center"
          justifyContent={{ xs: 'space-between', md: 'flex-end' }}
        >
          <FormControlLabel
            label={t('actions.parentChildLinkage')}
            control={
              <Switch checked={strict} onChange={(event) => onStrictChange(event.target.checked)} />
            }
            sx={{ m: 0, whiteSpace: 'nowrap' }}
          />
          <Chip
            color="primary"
            variant="outlined"
            label={t('messages.selectedCount', { count: selectedCount })}
          />
        </Stack>
      </Stack>
    </Paper>
  );
}

function TreeNodeRow({
  node,
  level,
  selected,
  selectedSet,
  strict,
  expanded,
  onToggleExpanded,
  onChange,
}: {
  node: TreeNode;
  level: number;
  selected: string[];
  selectedSet: Set<string>;
  strict: boolean;
  expanded: string[];
  onToggleExpanded: (id: string) => void;
  onChange: (selected: string[]) => void;
}) {
  const { checked, indeterminate } = nodeState(node, selectedSet);
  const open = expanded.includes(node.id);
  const hasChildren = node.children.length > 0;
  const toggle = () => onChange(nextSelected(node, selected, strict));

  return (
    <>
      <ListItemButton dense sx={{ pl: 1 + level * 2 }} onClick={toggle}>
        {hasChildren ? (
          <IconButton
            size="small"
            onClick={(event) => {
              event.stopPropagation();
              onToggleExpanded(node.id);
            }}
          >
            <Iconify icon={open ? 'eva:arrow-ios-downward-fill' : 'eva:arrow-ios-forward-fill'} />
          </IconButton>
        ) : (
          <Box sx={{ width: 34 }} />
        )}
        <Checkbox edge="start" checked={checked} indeterminate={indeterminate} tabIndex={-1} />
        <ListItemText primary={node.label} />
      </ListItemButton>
      {hasChildren && (
        <Collapse in={open}>
          {node.children.map((child) => (
            <TreeNodeRow
              key={child.id}
              node={child}
              level={level + 1}
              selected={selected}
              selectedSet={selectedSet}
              strict={strict}
              expanded={expanded}
              onToggleExpanded={onToggleExpanded}
              onChange={onChange}
            />
          ))}
        </Collapse>
      )}
    </>
  );
}

function nodeState(node: TreeNode, selected: Set<string>): TreeNodeState {
  if (node.children.length === 0) return { checked: selected.has(node.id), indeterminate: false };
  const childStates = node.children.map((child) => nodeState(child, selected));
  const checked = selected.has(node.id) || childStates.every((state) => state.checked);
  const indeterminate =
    !checked && childStates.some((state) => state.checked || state.indeterminate);
  return { checked, indeterminate };
}

function nextSelected(node: TreeNode, selected: string[], strict: boolean) {
  const ids = strict ? [node.id, ...descendants(node)] : [node.id];
  const next = new Set(selected);
  const shouldAdd = ids.some((id) => !next.has(id));
  ids.forEach((id) => (shouldAdd ? next.add(id) : next.delete(id)));
  return Array.from(next);
}

function descendants(node: TreeNode): string[] {
  return node.children.flatMap((child) => [child.id, ...descendants(child)]);
}

function buildTree(items: TreeOption[]) {
  const nodes = new Map(items.map((item) => [item.id, { ...item, children: [] as TreeNode[] }]));
  const roots: TreeNode[] = [];
  nodes.forEach((node) => {
    const parent = nodes.get(node.parentId);
    if (parent && node.id !== node.parentId) parent.children.push(node);
    else roots.push(node);
  });
  return roots;
}
