'use client';

import type { CaptchaConfig, CaptchaLabels, TurnstilePublicConfig } from '../model/types';

import Alert from '@mui/material/Alert';

import { useTranslate } from 'src/shared/i18n';

import { CapCaptcha } from './cap-captcha';
import { CloudflareTurnstile } from './cloudflare-turnstile';
import { CAPTCHA_PROVIDER_CAP, CAPTCHA_PROVIDER_CLOUDFLARE_TURNSTILE } from '../model/types';

type CaptchaWidgetProps = {
  config?: CaptchaConfig;
  resetKey: number;
  onTokenChange: (token: string | null) => void;
  labels?: CaptchaLabels;
};

export function CaptchaWidget({ config, resetKey, onTokenChange, labels }: CaptchaWidgetProps) {
  const { t } = useTranslate('messages');
  const captchaLabels = labels ?? defaultLabels(t);
  if (!config?.enabled) {
    return null;
  }

  if (config.provider === CAPTCHA_PROVIDER_CAP) {
    return (
      <CapCaptcha
        resetKey={resetKey}
        onTokenChange={onTokenChange}
        labels={captchaLabels}
      />
    );
  }

  if (config.provider === CAPTCHA_PROVIDER_CLOUDFLARE_TURNSTILE) {
    return (
      <CloudflareTurnstile
        config={turnstileConfig(config)}
        resetKey={resetKey}
        onTokenChange={onTokenChange}
        siteKeyRequiredMessage={t('auth.captcha.turnstileSiteKeyRequired')}
      />
    );
  }

  return <Alert severity="error">{t('auth.captcha.unsupportedProvider', { provider: config.provider })}</Alert>;
}

export function isCaptchaReady(config: CaptchaConfig | undefined, token: string | null) {
  return !config?.enabled || !!token;
}

function turnstileConfig(config: CaptchaConfig): TurnstilePublicConfig {
  return config.public_config as TurnstilePublicConfig;
}

function defaultLabels(t: ReturnType<typeof useTranslate>['t']): CaptchaLabels {
  return {
    initial: t('auth.captcha.initial'),
    verifying: t('auth.captcha.verifying'),
    solved: t('auth.captcha.solved'),
    error: t('auth.captcha.error'),
  };
}
