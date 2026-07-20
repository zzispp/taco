UPDATE sys_config
SET
    config_value = jsonb_set(
        jsonb_set(
            config_value::jsonb,
            '{providers}',
            COALESCE(config_value::jsonb -> 'providers', '{}'::jsonb) - 'cloudflare_turnstile',
            TRUE
        ),
        '{provider}',
        '"cap"'::jsonb,
        TRUE
    )::TEXT,
    remark = '验证码非敏感运行时配置 JSON。enabled 控制开关，provider 固定为 cap，providers.cap 配置 PoW 难度和 TTL。'
WHERE config_key = 'sys.account.captchaConfig';
