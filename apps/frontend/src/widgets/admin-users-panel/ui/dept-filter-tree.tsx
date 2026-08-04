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

type DeptFilterTreeProps = {
  nodes: TreeSelectNode[];
  selected: string;
  onSelect: (id: string) => void;
};

type DeptFilterNodeProps = {
  node: TreeSelectNode;
  level: number;
  selected: string;
  expanded: string[];
  onToggle: (id: string) => void;
  onSelect: (id: string) => void;
};

export function DeptFilterTree(props: DeptFilterTreeProps) {
  const { t } = useTranslate('admin');
  const [keyword, setKeyword] = useState('');
  const [expanded, setExpanded] = useState<string[]>([]);
  const visibleNodes = useMemo(() => filterDeptTree(props.nodes, keyword), [keyword, props.nodes]);
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
          selected={props.selected === ''}
          sx={{ mb: 0.5 }}
          onClick={() => props.onSelect('')}
        >
          <Box sx={{ width: 34 }} />
          <ListItemText primary={t('common.all')} />
        </ListItemButton>
        {visibleNodes.map((node) => (
          <DeptFilterNode
            key={node.id}
            node={node}
            level={0}
            selected={props.selected}
            expanded={expandedIds}
            onToggle={(id) => setExpanded(toggle(expandedIds, id))}
            onSelect={props.onSelect}
          />
        ))}
      </List>
    </Box>
  );
}

function DeptFilterNode(props: DeptFilterNodeProps) {
  const { open, hasChildren } = getDeptNodeState(props.node, props.expanded);
  return (
    <>
      <ListItemButton
        dense
        selected={props.selected === props.node.id}
        sx={{ pl: 1 + props.level * 2 }}
        onClick={() => props.onSelect(props.node.id)}
      >
        {hasChildren ? (
          <IconButton
            size="small"
            onClick={(event) => {
              event.stopPropagation();
              props.onToggle(props.node.id);
            }}
          >
            <Iconify icon={open ? 'eva:arrow-ios-downward-fill' : 'eva:arrow-ios-forward-fill'} />
          </IconButton>
        ) : (
          <Box sx={{ width: 34 }} />
        )}
        <ListItemText primary={props.node.label} />
      </ListItemButton>
      {hasChildren && (
        <Collapse in={open}>
          {props.node.children.map((child) => (
            <DeptFilterNode
              key={child.id}
              node={child}
              level={props.level + 1}
              selected={props.selected}
              expanded={props.expanded}
              onToggle={props.onToggle}
              onSelect={props.onSelect}
            />
          ))}
        </Collapse>
      )}
    </>
  );
}

function getDeptNodeState(node: TreeSelectNode, expanded: string[]) {
  return { open: expanded.includes(node.id), hasChildren: node.children.length > 0 };
}
