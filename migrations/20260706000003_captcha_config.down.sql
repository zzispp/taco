DELETE FROM sys_config
WHERE config_key IN (
    'sys.account.captchaProvider',
    'sys.account.captchaPublicConfig',
    'sys.account.captchaPrivateConfig'
);

UPDATE sys_config
SET public_read = FALSE
WHERE config_key = 'sys.account.captchaEnabled';
