ALTER TABLE sys_config ADD COLUMN public_read BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE sys_config
SET public_read = TRUE
WHERE config_key IN ('sys.index.skinName', 'sys.index.modeTheme', 'sys.account.registerUser', 'sys.account.captchaEnabled', 'sys.account.captchaProvider', 'sys.account.captchaPublicConfig');
