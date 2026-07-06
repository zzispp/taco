ALTER TABLE sys_config ALTER COLUMN config_value TYPE TEXT;

WITH old_values AS (
    SELECT
        COALESCE((SELECT lower(trim(config_value)) = 'true' FROM sys_config WHERE config_key = 'sys.account.captchaEnabled'), TRUE) AS captcha_enabled,
        COALESCE(NULLIF((SELECT trim(config_value) FROM sys_config WHERE config_key = 'sys.account.captchaProvider'), ''), 'cap') AS captcha_provider,
        COALESCE((SELECT config_value::jsonb FROM sys_config WHERE config_key = 'sys.account.captchaPublicConfig'), '{}'::jsonb) AS captcha_public,
        COALESCE((SELECT config_value::jsonb FROM sys_config WHERE config_key = 'sys.account.captchaPrivateConfig'), '{}'::jsonb) AS captcha_private
), captcha_config AS (
    SELECT jsonb_build_object(
        'enabled', captcha_enabled,
        'provider', captcha_provider,
        'providers', jsonb_build_object(
            'cap', jsonb_build_object(
                'challenge_count', COALESCE((captcha_private #>> '{cap,challenge_count}')::int, 50),
                'challenge_size', COALESCE((captcha_private #>> '{cap,challenge_size}')::int, 32),
                'challenge_difficulty', COALESCE((captcha_private #>> '{cap,challenge_difficulty}')::int, 4),
                'challenge_ttl_seconds', COALESCE((captcha_private #>> '{cap,challenge_ttl_seconds}')::int, 600),
                'redeemed_token_ttl_seconds', COALESCE((captcha_private #>> '{cap,redeemed_token_ttl_seconds}')::int, 1200)
            ),
            'cloudflare_turnstile', jsonb_build_object(
                'site_key', COALESCE(captcha_public #>> '{cloudflare_turnstile,site_key}', ''),
                'secret_key', COALESCE(captcha_private #>> '{cloudflare_turnstile,secret_key}', ''),
                'theme', COALESCE(captcha_public #>> '{cloudflare_turnstile,theme}', 'auto'),
                'size', COALESCE(captcha_public #>> '{cloudflare_turnstile,size}', 'normal')
            )
        )
    )::text AS value
    FROM old_values
)
INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '9', '账号自助-验证码配置', 'sys.account.captchaConfig', value, 'Y', FALSE, 'admin', CURRENT_TIMESTAMP,
       '验证码完整配置 JSON。包含 enabled、provider、providers.cap 和 providers.cloudflare_turnstile；仅后端读取，公开内容由 /api/captcha/config 投影返回。'
FROM captcha_config
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.account.captchaConfig');

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '10', '认证-JWT Token 配置', 'sys.auth.tokenConfig', '{"access_token_ttl_seconds":1440,"refresh_token_ttl_seconds":604800}', 'Y', FALSE, 'admin', CURRENT_TIMESTAMP,
       'JWT Token 过期时间 JSON。secret 仍在 config.yaml。'
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.auth.tokenConfig');

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '11', '用户管理-密码策略', 'sys.user.passwordPolicy', '{"min_length":8,"max_length":128,"require_letter":false,"require_number":false,"require_symbol":false,"forbid_username_contains":false}', 'Y', TRUE, 'admin', CURRENT_TIMESTAMP,
       '用户密码策略 JSON。后端最终校验，前端用于提示和早期校验。'
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.user.passwordPolicy');

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '12', '上传-头像配置', 'sys.upload.avatarConfig', '{"max_bytes":2097152}', 'Y', FALSE, 'admin', CURRENT_TIMESTAMP,
       '头像上传运行期配置 JSON。上传目录仍在 config.yaml。'
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.upload.avatarConfig');

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '13', '导出-批量配置', 'sys.export.batchConfig', '{"page_size":100}', 'Y', FALSE, 'admin', CURRENT_TIMESTAMP,
       '后端导出批量查询配置 JSON。'
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.export.batchConfig');

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '14', '站点-展示配置', 'sys.site.displayConfig', '{"site_name":"taco","logo_url":"/logo/logo.svg","footer_text":"taco backend control plane."}', 'Y', TRUE, 'admin', CURRENT_TIMESTAMP,
       '站点展示公开配置 JSON。包含站点名、Logo URL、页脚文案。'
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.site.displayConfig');

UPDATE sys_config
SET public_read = FALSE
WHERE config_key IN (
    'sys.account.captchaEnabled',
    'sys.account.captchaProvider',
    'sys.account.captchaPublicConfig',
    'sys.account.captchaPrivateConfig'
);
