import type { ColorSystem } from '@mui/material/styles';
import type { SettingsState } from 'src/shared/ui/settings';
import type { ThemeOptions, ThemeColorScheme, ColorSchemeOptionsExtended } from '../types';

import { setFont, hexToRgbChannel, createPaletteChannel } from 'minimal-shared/utils';

import { primaryColorPresets } from './color-presets';
import { createShadowColor } from '../core/custom-shadows';

// ----------------------------------------------------------------------

/**
 * Updates the core theme with the provided settings state.
 * @param theme - The base theme options to update.
 * @param settingsState - The settings state containing direction, fontFamily, contrast, and primaryColor.
 * @returns Updated theme options with applied settings.
 */

export function applySettingsToTheme(
  theme: ThemeOptions,
  settingsState: SettingsState
): ThemeOptions {
  const options = themeUpdateOptions(theme, settingsState);

  return {
    ...theme,
    direction: options.direction,
    colorSchemes: {
      light: updateColorScheme(options, 'light'),
      dark: updateColorScheme(options, 'dark'),
    },
    typography: {
      ...theme.typography,
      fontFamily: setFont(options.fontFamily),
    },
  };
}

type ThemeUpdateOptions = {
  theme: ThemeOptions;
  direction: SettingsState['direction'];
  fontFamily: SettingsState['fontFamily'];
  isDefaultContrast: boolean;
  isDefaultPrimaryColor: boolean;
  lightPalette: ColorSystem['palette'];
  primaryColorPalette: ReturnType<typeof createPaletteChannel>;
};

function themeUpdateOptions(theme: ThemeOptions, settingsState: SettingsState): ThemeUpdateOptions {
  const { direction, fontFamily, contrast, primaryColor } = settingsState;

  return {
    theme,
    direction,
    fontFamily,
    isDefaultContrast: contrast === 'default',
    isDefaultPrimaryColor: primaryColor === 'default',
    lightPalette: theme.colorSchemes?.light?.palette as ColorSystem['palette'],
    primaryColorPalette: createPaletteChannel(primaryColorPresets[primaryColor]),
  };
}

function updateColorScheme(options: ThemeUpdateOptions, schemeName: ThemeColorScheme) {
  const currentScheme: ColorSchemeOptionsExtended = options.theme.colorSchemes?.[schemeName] ?? {};

  return {
    ...currentScheme,
    palette: updatedPalette(options, currentScheme, schemeName),
    customShadows: updatedCustomShadows(options, currentScheme),
  };
}

function updatedPalette(
  options: ThemeUpdateOptions,
  currentScheme: ColorSchemeOptionsExtended,
  schemeName: ThemeColorScheme
) {
  return {
    ...currentScheme?.palette,
    ...(!options.isDefaultPrimaryColor && { primary: options.primaryColorPalette }),
    ...(schemeName === 'light' && { background: lightBackground(options) }),
  };
}

function lightBackground(options: ThemeUpdateOptions) {
  return {
    ...options.lightPalette?.background,
    ...(!options.isDefaultContrast && {
      default: options.lightPalette.grey[200],
      defaultChannel: hexToRgbChannel(options.lightPalette.grey[200]),
    }),
  };
}

function updatedCustomShadows(
  options: ThemeUpdateOptions,
  currentScheme: ColorSchemeOptionsExtended
) {
  return {
    ...currentScheme?.customShadows,
    ...(!options.isDefaultPrimaryColor && {
      primary: createShadowColor(options.primaryColorPalette.mainChannel),
    }),
  };
}
