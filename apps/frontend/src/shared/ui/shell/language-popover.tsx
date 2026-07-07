'use client';

import type { IconButtonProps } from '@mui/material/IconButton';
import type { LangCode, LangOption } from 'src/shared/i18n';
import type { CustomPopoverProps } from 'src/shared/ui/custom-popover';

import { m } from 'framer-motion';
import { useCallback } from 'react';
import { usePopover } from 'minimal-shared/hooks';

import MenuList from '@mui/material/MenuList';
import MenuItem from '@mui/material/MenuItem';
import IconButton from '@mui/material/IconButton';

import { useTranslate } from 'src/shared/i18n';
import { FlagIcon } from 'src/shared/ui/flag-icon';
import { CustomPopover } from 'src/shared/ui/custom-popover';
import { varTap, varHover, transitionTap } from 'src/shared/ui/animate';

// ----------------------------------------------------------------------

const LANGUAGE_MENU_MIN_WIDTH = 220;
const LANGUAGE_MENU_MIN_HEIGHT = 72;
const LANGUAGE_BUTTON_SIZE = 40;
const LANGUAGE_BUTTON_TAP_SCALE = 0.96;
const LANGUAGE_BUTTON_HOVER_SCALE = 1.04;

type LanguageOption = Pick<LangOption, 'value' | 'label' | 'countryCode'>;

export type LanguagePopoverProps = IconButtonProps & {
  data?: LanguageOption[];
};

type LanguageOptionItemProps = {
  option: LanguageOption;
  selected: boolean;
  onSelect: (lang: LangCode) => void;
};

function LanguageOptionItem({ option, selected, onSelect }: LanguageOptionItemProps) {
  return (
    <MenuItem selected={selected} onClick={() => onSelect(option.value)}>
      <FlagIcon code={option.countryCode} />
      {option.label}
    </MenuItem>
  );
}

type LanguageMenuProps = {
  anchorEl: CustomPopoverProps['anchorEl'];
  currentValue: LangCode;
  onClose: () => void;
  onSelect: (lang: LangCode) => void;
  open: boolean;
  options: LanguageOption[];
};

function LanguageMenu({
  anchorEl,
  currentValue,
  onClose,
  onSelect,
  open,
  options,
}: LanguageMenuProps) {
  return (
    <CustomPopover open={open} anchorEl={anchorEl} onClose={onClose}>
      <MenuList
        sx={{
          width: 'max-content',
          minWidth: LANGUAGE_MENU_MIN_WIDTH,
          minHeight: LANGUAGE_MENU_MIN_HEIGHT,
        }}
      >
        {options.map((option) => (
          <LanguageOptionItem
            key={option.value}
            option={option}
            selected={option.value === currentValue}
            onSelect={onSelect}
          />
        ))}
      </MenuList>
    </CustomPopover>
  );
}

export function LanguagePopover({ data = [], sx, ...other }: LanguagePopoverProps) {
  const { open, anchorEl, onClose, onOpen } = usePopover();

  const { onChangeLang, currentLang } = useTranslate();

  const handleChangeLang = useCallback(
    (lang: LangCode) => {
      onChangeLang(lang);
      onClose();
    },
    [onChangeLang, onClose]
  );

  return (
    <>
      <IconButton
        component={m.button}
        whileTap={varTap(LANGUAGE_BUTTON_TAP_SCALE)}
        whileHover={varHover(LANGUAGE_BUTTON_HOVER_SCALE)}
        transition={transitionTap()}
        aria-label="Languages button"
        onClick={onOpen}
        sx={[
          (theme) => ({
            p: 0,
            width: LANGUAGE_BUTTON_SIZE,
            height: LANGUAGE_BUTTON_SIZE,
            ...(open && { bgcolor: theme.vars.palette.action.selected }),
          }),
          ...(Array.isArray(sx) ? sx : [sx]),
        ]}
        {...other}
      >
        <FlagIcon code={currentLang.countryCode} />
      </IconButton>

      <LanguageMenu
        open={open}
        anchorEl={anchorEl}
        options={data}
        currentValue={currentLang.value}
        onClose={onClose}
        onSelect={handleChangeLang}
      />
    </>
  );
}
