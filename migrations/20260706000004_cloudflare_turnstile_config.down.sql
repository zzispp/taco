UPDATE sys_config
SET config_value = (config_value::jsonb - 'cloudflare_turnstile')::text
WHERE config_key IN ('sys.account.captchaPublicConfig', 'sys.account.captchaPrivateConfig');

UPDATE sys_config
SET remark = '当前验证码提供方。可选值：cap（内置 PoW 验证码）、cloudflare_turnstile（Cloudflare Turnstile）。切换 provider 后需同步填写对应 JSON 配置。'
WHERE config_key = 'sys.account.captchaProvider';
