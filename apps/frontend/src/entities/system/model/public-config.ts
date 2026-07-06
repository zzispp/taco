import type { SettingsState } from 'src/shared/ui/settings';

export const PUBLIC_CONFIG_KEYS = {
  captchaEnabled: 'sys.account.captchaEnabled',
  captchaProvider: 'sys.account.captchaProvider',
  captchaPublicConfig: 'sys.account.captchaPublicConfig',
  registerUser: 'sys.account.registerUser',
  skinName: 'sys.index.skinName',
  modeTheme: 'sys.index.modeTheme',
} as const;

export type PublicConfigKey = (typeof PUBLIC_CONFIG_KEYS)[keyof typeof PUBLIC_CONFIG_KEYS];
export type PublicConfigMap = Record<string, string>;

const SKIN_PRIMARY_COLOR: Record<string, SettingsState['primaryColor']> = {
  'skin-blue': 'preset1',
  'skin-green': 'default',
  'skin-purple': 'preset2',
  'skin-red': 'preset5',
  'skin-yellow': 'preset4',
};

const MODE_THEME: Record<
  string,
  Pick<SettingsState, 'mode' | 'navColor'>
> = {
  'theme-dark': { mode: 'dark', navColor: 'apparent' },
  'theme-light': { mode: 'light', navColor: 'integrate' },
};

export function publicConfigKeys() {
  return Object.values(PUBLIC_CONFIG_KEYS);
}

export function isRegisterEnabled(configs?: PublicConfigMap) {
  return configs?.[PUBLIC_CONFIG_KEYS.registerUser]?.trim().toLowerCase() === 'true';
}

export function isCaptchaEnabled(configs?: PublicConfigMap) {
  return configs?.[PUBLIC_CONFIG_KEYS.captchaEnabled]?.trim().toLowerCase() === 'true';
}

export function settingsFromPublicConfigs(configs?: PublicConfigMap): Partial<SettingsState> {
  const primaryColor = SKIN_PRIMARY_COLOR[configs?.[PUBLIC_CONFIG_KEYS.skinName] ?? ''];
  const modeTheme = MODE_THEME[configs?.[PUBLIC_CONFIG_KEYS.modeTheme] ?? ''];

  return {
    ...(primaryColor ? { primaryColor } : {}),
    ...(modeTheme ?? {}),
  };
}
