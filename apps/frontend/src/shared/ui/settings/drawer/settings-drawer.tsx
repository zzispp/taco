'use client';

import type { SettingsVisibility } from './settings-drawer-sections';
import type { SettingsState, SettingsDrawerProps, SettingsContextValue } from '../types';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Drawer from '@mui/material/Drawer';

import { Scrollbar } from '../../scrollbar';
import { SettingsFontOptions } from './settings-font-section';
import { useSettingsContext } from '../context/use-settings-context';
import {
  settingsVisibility,
  SettingsDrawerHead,
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
      <SettingsSections
        settings={settings}
        visibility={visibility}
        defaultSettings={defaultSettings}
      />
    </Drawer>
  );
}

type SettingsSectionsProps = Readonly<{
  settings: SettingsContextValue;
  visibility: SettingsVisibility;
  defaultSettings: SettingsState;
}>;

function SettingsSections({ settings, visibility, defaultSettings }: SettingsSectionsProps) {
  return (
    <Scrollbar>
      <Box sx={{ pb: 5, gap: 6, px: 2.5, display: 'flex', flexDirection: 'column' }}>
        <SettingsToggleOptions {...{ settings, visibility, defaultSettings }} />
        {(visibility.navColor || visibility.navLayout) && (
          <SettingsNavigationOptions {...{ settings, visibility, defaultSettings }} />
        )}
        {visibility.primaryColor && (
          <SettingsPresetOptions {...{ settings, visibility, defaultSettings }} />
        )}
        {(visibility.fontFamily || visibility.fontSize) && (
          <SettingsFontOptions {...{ settings, visibility, defaultSettings }} />
        )}
      </Box>
    </Scrollbar>
  );
}
