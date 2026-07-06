export const CAPTCHA_PROVIDER_CAP = 'cap';
export const CAPTCHA_PROVIDER_CLOUDFLARE_TURNSTILE = 'cloudflare_turnstile';

export type CaptchaPublicConfig = Record<string, unknown>;

export type CaptchaConfig = {
  enabled: boolean;
  provider: string;
  public_config: CaptchaPublicConfig;
};

export type CaptchaLabels = {
  initial: string;
  verifying: string;
  solved: string;
  error: string;
};


export type TurnstilePublicConfig = {
  site_key: string;
  script_url: string;
  theme: string;
  size: string;
};
