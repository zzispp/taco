import { CONFIG } from 'src/shared/config';

const ABSOLUTE_URL_PATTERN = /^(https?:|data:|blob:)/i;

export function resolveServerAssetUrl(value?: string | null) {
  if (!value) {
    return '';
  }

  if (ABSOLUTE_URL_PATTERN.test(value)) {
    return value;
  }

  if (value.startsWith('/')) {
    return `${CONFIG.serverUrl}${value}`;
  }

  return value;
}
