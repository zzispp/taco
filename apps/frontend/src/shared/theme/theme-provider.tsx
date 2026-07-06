'use client';

import type { Theme, ThemeProviderProps as MuiThemeProviderProps } from '@mui/material/styles';
import type {} from './extend-theme-types';
import type { ThemeOptions } from './types';
import type { SettingsState } from 'src/shared/ui/settings';

import { useEffect } from 'react';

import CssBaseline from '@mui/material/CssBaseline';
import { useColorScheme, ThemeProvider as ThemeVarsProvider } from '@mui/material/styles';

import { useTranslate } from 'src/shared/i18n';
import { useSettingsContext } from 'src/shared/ui/settings';

import { createTheme } from './create-theme';
import { Rtl } from './with-settings/right-to-left';

// ----------------------------------------------------------------------

export type ThemeProviderProps = Partial<MuiThemeProviderProps<Theme>> & {
  themeOverrides?: ThemeOptions;
};

function ThemeModeSync({ mode }: { mode: SettingsState['mode'] }) {
  const { mode: muiMode, setMode } = useColorScheme();

  useEffect(() => {
    if (muiMode !== undefined && muiMode !== mode) {
      setMode(mode);
    }
  }, [mode, muiMode, setMode]);

  return null;
}

export function ThemeProvider({ themeOverrides, children, ...other }: ThemeProviderProps) {
  const settings = useSettingsContext();
  const { currentLang } = useTranslate();

  const theme = createTheme({
    settingsState: settings.state,
    localeComponents: currentLang?.systemValue,
    themeOverrides,
  });

  return (
    <ThemeVarsProvider disableTransitionOnChange theme={theme} {...other} defaultMode={settings.state.mode}>
      <ThemeModeSync mode={settings.state.mode} />
      <CssBaseline />
      <Rtl direction={settings.state.direction}>{children}</Rtl>
    </ThemeVarsProvider>
  );
}
