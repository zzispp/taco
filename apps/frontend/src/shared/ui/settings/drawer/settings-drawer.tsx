'use client';

import type { SettingsDrawerProps } from '../types';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Drawer from '@mui/material/Drawer';

import { Scrollbar } from '../../scrollbar';
import { useSettingsContext } from '../context/use-settings-context';
import {
  settingsVisibility,
  SettingsDrawerHead,
  SettingsFontOptions,
  SettingsToggleOptions,
  SettingsPresetOptions,
  SettingsNavigationOptions,
} from './settings-drawer-sections';

// ----------------------------------------------------------------------

export function SettingsDrawer({ sx, defaultSettings: propDefaults }: SettingsDrawerProps) {
  const settings = useSettingsContext();
  const defaultSettings = propDefaults ?? settings.defaultSettings;
  const visibility = settingsVisibility(defaultSettings);

  return (
    <Drawer
      anchor="right"
      open={settings.openDrawer}
      onClose={settings.onCloseDrawer}
      slotProps={{
        backdrop: { invisible: true },
        paper: {
          sx: [
            (theme) => ({
              ...theme.mixins.paperStyles(theme, {
                color: varAlpha(theme.vars.palette.background.defaultChannel, 0.9),
              }),
              width: 360,
            }),
            ...(Array.isArray(sx) ? sx : [sx]),
          ],
        },
      }}
    >
      <SettingsDrawerHead settings={settings} />
      <Scrollbar>
        <Box sx={{ pb: 5, gap: 6, px: 2.5, display: 'flex', flexDirection: 'column' }}>
          <SettingsToggleOptions
            settings={settings}
            visibility={visibility}
            defaultSettings={defaultSettings}
          />
          {(visibility.navColor || visibility.navLayout) && (
            <SettingsNavigationOptions
              settings={settings}
              visibility={visibility}
              defaultSettings={defaultSettings}
            />
          )}
          {visibility.primaryColor && (
            <SettingsPresetOptions
              settings={settings}
              visibility={visibility}
              defaultSettings={defaultSettings}
            />
          )}
          {(visibility.fontFamily || visibility.fontSize) && (
            <SettingsFontOptions
              settings={settings}
              visibility={visibility}
              defaultSettings={defaultSettings}
            />
          )}
        </Box>
      </Scrollbar>
    </Drawer>
  );
}
