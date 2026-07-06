import type { TreeSelectNode } from 'src/entities/system';

import { useMemo, useState } from 'react';

import Box from '@mui/material/Box';
import List from '@mui/material/List';
import Collapse from '@mui/material/Collapse';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import ListItemText from '@mui/material/ListItemText';
import ListItemButton from '@mui/material/ListItemButton';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { toggle, flattenTree, filterDeptTree } from './helpers';

export function DeptFilterTree({
  nodes,
  selected,
  onSelect,
}: {
  nodes: TreeSelectNode[];
  selected: string;
  onSelect: (id: string) => void;
}) {
  const { t } = useTranslate('admin');
  const [keyword, setKeyword] = useState('');
  const [expanded, setExpanded] = useState<string[]>([]);
  const visibleNodes = useMemo(() => filterDeptTree(nodes, keyword), [keyword, nodes]);
  const expandedIds =
    expanded.length > 0 ? expanded : flattenTree(visibleNodes).map((dept) => dept.id);
  return (
    <Box sx={{ p: 2 }}>
      <Box sx={{ typography: 'subtitle2', mb: 1 }}>{t('fields.deptTree')}</Box>
      <TextField
        fullWidth
        size="small"
        value={keyword}
        label={t('fields.deptName')}
        sx={{ mb: 1 }}
        onChange={(event) => setKeyword(event.target.value)}
      />
      <List disablePadding>
        <ListItemButton
          dense
          selected={selected === ''}
          sx={{ mb: 0.5 }}
          onClick={() => onSelect('')}
        >
          <Box sx={{ width: 34 }} />
          <ListItemText primary={t('common.all')} />
        </ListItemButton>
        {visibleNodes.map((node) => (
          <DeptFilterNode
            key={node.id}
            node={node}
            level={0}
            selected={selected}
            expanded={expandedIds}
            onToggle={(id) => setExpanded(toggle(expandedIds, id))}
            onSelect={onSelect}
          />
        ))}
      </List>
    </Box>
  );
}

function DeptFilterNode({
  node,
  level,
  selected,
  expanded,
  onToggle,
  onSelect,
}: {
  node: TreeSelectNode;
  level: number;
  selected: string;
  expanded: string[];
  onToggle: (id: string) => void;
  onSelect: (id: string) => void;
}) {
  const open = expanded.includes(node.id);
  const hasChildren = node.children.length > 0;
  return (
    <>
      <ListItemButton
        dense
        selected={selected === node.id}
        sx={{ pl: 1 + level * 2 }}
        onClick={() => onSelect(node.id)}
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
            <DeptFilterNode
              key={child.id}
              node={child}
              level={level + 1}
              selected={selected}
              expanded={expanded}
              onToggle={onToggle}
              onSelect={onSelect}
            />
          ))}
        </Collapse>
      )}
    </>
  );
}
