import type { Editor } from '@tiptap/react';
import type { ButtonBaseProps } from '@mui/material/ButtonBase';

import { useMemo, useCallback } from 'react';
import { varAlpha } from 'minimal-shared/utils';
import { usePopover } from 'minimal-shared/hooks';

import Menu from '@mui/material/Menu';
import { listClasses } from '@mui/material/List';
import ButtonBase, { buttonBaseClasses } from '@mui/material/ButtonBase';

import { Iconify } from '../../iconify';
import { ToolbarItem } from './toolbar-item';

// ----------------------------------------------------------------------

const HEADING_OPTIONS = [
  { label: 'Paragraph', level: null },
  { label: 'Heading 1', level: 1 },
  { label: 'Heading 2', level: 2 },
  { label: 'Heading 3', level: 3 },
  { label: 'Heading 4', level: 4 },
  { label: 'Heading 5', level: 5 },
  { label: 'Heading 6', level: 6 },
] as const;

export type TextHeadingLevel = (typeof HEADING_OPTIONS)[number]['level'];

type HeadingBlock = {
  editor: Editor;
  isActive: (value: TextHeadingLevel) => boolean;
};

export function HeadingBlock({ editor, isActive }: HeadingBlock) {
  const { anchorEl, open, onOpen, onClose } = usePopover();

  const selectedOption = useMemo(
    () => HEADING_OPTIONS.find((option) => isActive(option.level)),
    [isActive]
  );

  const handleSelect = useCallback(
    (value: TextHeadingLevel) => {
      onClose();
      if (value) {
        editor.chain().focus().toggleHeading({ level: value }).run();
      } else {
        editor.chain().focus().setParagraph().run();
      }
    },
    [editor, onClose]
  );

  const buttonId = 'heading-menu-button';
  const menuId = 'heading-menu';

  const buttonProps: ButtonBaseProps = {
    id: buttonId,
    'aria-label': 'Heading menu',
    'aria-controls': open ? menuId : undefined,
    'aria-haspopup': 'true',
    'aria-expanded': open ? 'true' : undefined,
  };

  return (
    <>
      <ButtonBase
        {...buttonProps}
        onClick={onOpen}
        sx={(theme) => ({
          px: 1,
          width: 120,
          height: 32,
          borderRadius: 0.75,
          typography: 'body2',
          fontWeight: 'fontWeightMedium',
          justifyContent: 'space-between',
          border: `solid 1px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.2)}`,
        })}
      >
        {selectedOption?.label ?? 'Paragraph'}
        <Iconify
          width={16}
          icon={open ? 'eva:arrow-ios-upward-fill' : 'eva:arrow-ios-downward-fill'}
        />
      </ButtonBase>

      <Menu
        id={menuId}
        anchorEl={anchorEl}
        open={open}
        onClose={onClose}
        slotProps={{
          list: { 'aria-labelledby': buttonId },
          paper: {
            sx: {
              width: 120,
              [`& .${listClasses.root}`]: { gap: 0.5, display: 'flex', flexDirection: 'column' },
              [`& .${buttonBaseClasses.root}`]: {
                px: 1,
                width: 1,
                height: 34,
                borderRadius: 0.75,
                justifyContent: 'flex-start',
                '&:hover': { backgroundColor: 'action.hover' },
              },
            },
          },
        }}
      >
        {HEADING_OPTIONS.map((option) => (
          <ToolbarItem
            key={option.label}
            component="li"
            aria-label={option.label}
            label={option.label}
            active={isActive(option.level)}
            onClick={() => handleSelect(option.level)}
            sx={{
              ...(option.level && {
                fontSize: 18 - option.level,
                fontWeight: 'fontWeightBold',
              }),
            }}
          />
        ))}
      </Menu>
    </>
  );
}
