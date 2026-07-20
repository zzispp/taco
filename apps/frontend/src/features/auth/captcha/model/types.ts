export const CAPTCHA_PROVIDER_CAP = 'cap';

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
