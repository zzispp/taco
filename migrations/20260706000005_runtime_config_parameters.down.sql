DELETE FROM sys_config
WHERE config_key IN (
    'sys.auth.tokenConfig',
    'sys.user.passwordPolicy',
    'sys.upload.avatarConfig',
    'sys.export.batchConfig',
    'sys.site.displayConfig'
);
