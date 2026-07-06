UPDATE sys_config
SET config_value = (
    (config_value::jsonb - 'cloudflare_turnstile') || '{"cloudflare_turnstile":{"site_key":"","theme":"auto","size":"normal"}}'::jsonb
)::text,
public_read = TRUE,
remark = '验证码公开配置 JSON，前端可读取。cap 使用 {}；cloudflare_turnstile 填写 {"site_key":"Cloudflare Turnstile Site Key","theme":"auto|light|dark","size":"normal|compact|flexible"}。'
WHERE config_key = 'sys.account.captchaPublicConfig';

UPDATE sys_config
SET config_value = (
    (config_value::jsonb - 'cloudflare_turnstile') || '{"cloudflare_turnstile":{"secret_key":""}}'::jsonb
)::text,
public_read = FALSE,
remark = '验证码私有配置 JSON，仅后端读取，不会公开。cap 使用 {}；cloudflare_turnstile 填写 {"secret_key":"Cloudflare Turnstile Secret Key"}。'
WHERE config_key = 'sys.account.captchaPrivateConfig';

UPDATE sys_config
SET remark = '当前验证码提供方。可选值：cap（内置 PoW 验证码）、cloudflare_turnstile（Cloudflare Turnstile）。切换 provider 后需同步填写对应 JSON 配置。',
    public_read = TRUE
WHERE config_key = 'sys.account.captchaProvider';
