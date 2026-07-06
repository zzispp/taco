DELETE FROM sys_config
WHERE config_key IN (
    'sys.account.captchaConfig',
    'sys.auth.tokenConfig',
    'sys.user.passwordPolicy',
    'sys.upload.avatarConfig',
    'sys.export.batchConfig',
    'sys.site.displayConfig'
);

UPDATE sys_config
SET public_read = TRUE
WHERE config_key IN (
    'sys.account.captchaEnabled',
    'sys.account.captchaProvider',
    'sys.account.captchaPublicConfig'
);
