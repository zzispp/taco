import type { SettingsState } from 'src/shared/ui/settings';

export const PUBLIC_CONFIG_KEYS = {
  registerUser: 'sys.account.registerUser',
  skinName: 'sys.index.skinName',
  modeTheme: 'sys.index.modeTheme',
  passwordPolicy: 'sys.user.passwordPolicy',
  siteDisplayConfig: 'sys.site.displayConfig',
} as const;

export type PublicConfigKey = (typeof PUBLIC_CONFIG_KEYS)[keyof typeof PUBLIC_CONFIG_KEYS];
export type PublicConfigMap = Record<string, string>;
export type PasswordPolicy = {
  min_length: number;
  max_length: number;
  require_letter: boolean;
  require_number: boolean;
  require_symbol: boolean;
  forbid_username_contains: boolean;
};
export type SiteDisplayConfig = {
  site_name: string;
  logo_url: string;
  footer_text: string;
};

const SKIN_PRIMARY_COLOR: Record<string, SettingsState['primaryColor']> = {
  'skin-blue': 'preset1',
  'skin-green': 'default',
  'skin-purple': 'preset2',
  'skin-red': 'preset5',
  'skin-yellow': 'preset4',
};

const MODE_THEME: Record<string, Pick<SettingsState, 'mode' | 'navColor'>> = {
  'theme-dark': { mode: 'dark', navColor: 'apparent' },
  'theme-light': { mode: 'light', navColor: 'integrate' },
};

export function publicConfigKeys() {
  return Object.values(PUBLIC_CONFIG_KEYS);
}

export function isRegisterEnabled(configs?: PublicConfigMap) {
  return configs?.[PUBLIC_CONFIG_KEYS.registerUser]?.trim().toLowerCase() === 'true';
}

export function settingsFromPublicConfigs(configs?: PublicConfigMap): Partial<SettingsState> {
  const primaryColor = SKIN_PRIMARY_COLOR[configs?.[PUBLIC_CONFIG_KEYS.skinName] ?? ''];
  const modeTheme = MODE_THEME[configs?.[PUBLIC_CONFIG_KEYS.modeTheme] ?? ''];

  return {
    ...(primaryColor ? { primaryColor } : {}),
    ...(modeTheme ?? {}),
  };
}

export function passwordPolicyFromPublicConfigs(configs?: PublicConfigMap) {
  return parseJsonConfig<PasswordPolicy>(
    configs?.[PUBLIC_CONFIG_KEYS.passwordPolicy],
    isPasswordPolicy
  );
}

export function siteDisplayFromPublicConfigs(configs?: PublicConfigMap) {
  return parseJsonConfig<SiteDisplayConfig>(
    configs?.[PUBLIC_CONFIG_KEYS.siteDisplayConfig],
    isSiteDisplayConfig
  );
}

function parseJsonConfig<T>(value: string | undefined, isValid: (parsed: unknown) => parsed is T) {
  if (!value) {
    return undefined;
  }
  const parsed: unknown = JSON.parse(value);
  if (!isValid(parsed)) {
    throw new Error('Invalid public system config');
  }
  return parsed;
}

function isPasswordPolicy(value: unknown): value is PasswordPolicy {
  if (!isRecord(value)) {
    return false;
  }
  const minLength = value.min_length;
  const maxLength = value.max_length;
  return (
    isPositiveInteger(minLength) &&
    isPositiveInteger(maxLength) &&
    maxLength >= minLength &&
    typeof value.require_letter === 'boolean' &&
    typeof value.require_number === 'boolean' &&
    typeof value.require_symbol === 'boolean' &&
    typeof value.forbid_username_contains === 'boolean'
  );
}

function isSiteDisplayConfig(value: unknown): value is SiteDisplayConfig {
  if (!isRecord(value)) {
    return false;
  }
  return (
    isNonEmptyString(value.site_name) &&
    isNonEmptyString(value.logo_url) &&
    isNonEmptyString(value.footer_text)
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isPositiveInteger(value: unknown): value is number {
  return typeof value === 'number' && Number.isInteger(value) && value > 0;
}

function isNonEmptyString(value: unknown): value is string {
  return typeof value === 'string' && value.trim().length > 0;
}
