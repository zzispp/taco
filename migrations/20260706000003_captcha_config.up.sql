UPDATE sys_config
SET public_read = TRUE
WHERE config_key = 'sys.account.captchaEnabled';

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '6', '账号自助-验证码类型', 'sys.account.captchaProvider', 'cap', 'Y', TRUE, 'admin', CURRENT_TIMESTAMP, '当前验证码提供方。可选值：cap（内置 PoW 验证码）、cloudflare_turnstile（Cloudflare Turnstile）。切换 provider 后需同步填写对应 JSON 配置。'
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.account.captchaProvider');

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '7', '账号自助-验证码公开配置', 'sys.account.captchaPublicConfig', '{"cap":{},"cloudflare_turnstile":{"site_key":"","theme":"auto","size":"normal"}}', 'Y', TRUE, 'admin', CURRENT_TIMESTAMP, '验证码公开配置 JSON，前端可读取。cap 使用 {}；cloudflare_turnstile 填写 {"site_key":"Cloudflare Turnstile Site Key","theme":"auto|light|dark","size":"normal|compact|flexible"}。'
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.account.captchaPublicConfig');

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '8', '账号自助-验证码私有配置', 'sys.account.captchaPrivateConfig', '{"cap":{},"cloudflare_turnstile":{"secret_key":""}}', 'Y', FALSE, 'admin', CURRENT_TIMESTAMP, '验证码私有配置 JSON，仅后端读取，不会公开。cap 使用 {}；cloudflare_turnstile 填写 {"secret_key":"Cloudflare Turnstile Secret Key"}。'
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.account.captchaPrivateConfig');
