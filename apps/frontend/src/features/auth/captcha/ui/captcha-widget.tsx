'use client';

import type { CaptchaConfig, CaptchaLabels, TurnstilePublicConfig } from '../model/types';

import Alert from '@mui/material/Alert';

import { CapCaptcha } from './cap-captcha';
import { CloudflareTurnstile } from './cloudflare-turnstile';
import { CAPTCHA_PROVIDER_CAP, CAPTCHA_PROVIDER_CLOUDFLARE_TURNSTILE } from '../model/types';

type CaptchaWidgetProps = {
  config?: CaptchaConfig;
  resetKey: number;
  onTokenChange: (token: string | null) => void;
  labels?: CaptchaLabels;
};

const DEFAULT_LABELS: CaptchaLabels = {
  initial: 'Verify you are human',
  verifying: 'Verifying...',
  solved: 'You are human',
  error: 'Error. Try again.',
};

export function CaptchaWidget({ config, resetKey, onTokenChange, labels }: CaptchaWidgetProps) {
  if (!config?.enabled) {
    return null;
  }

  if (config.provider === CAPTCHA_PROVIDER_CAP) {
    return (
      <CapCaptcha
        resetKey={resetKey}
        onTokenChange={onTokenChange}
        labels={labels ?? DEFAULT_LABELS}
      />
    );
  }

  if (config.provider === CAPTCHA_PROVIDER_CLOUDFLARE_TURNSTILE) {
    return (
      <CloudflareTurnstile
        config={turnstileConfig(config)}
        resetKey={resetKey}
        onTokenChange={onTokenChange}
      />
    );
  }

  return <Alert severity="error">Unsupported captcha provider: {config.provider}</Alert>;
}

export function isCaptchaReady(config: CaptchaConfig | undefined, token: string | null) {
  return !config?.enabled || !!token;
}

function turnstileConfig(config: CaptchaConfig): TurnstilePublicConfig {
  return config.public_config as TurnstilePublicConfig;
}
