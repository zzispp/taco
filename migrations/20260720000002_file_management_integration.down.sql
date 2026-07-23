DELETE FROM sys_job_execution WHERE job_id IN ('file-purge-trash', 'file-cleanup-upload-sessions');
DELETE FROM sys_job WHERE job_id IN ('file-purge-trash', 'file-cleanup-upload-sessions');
DELETE FROM sys_role_menu WHERE menu_id IN ('5','115','116','117','1150','1160','1161','1162','1163','1164','1165','1166','1167','1168','1170');
DELETE FROM sys_menu WHERE menu_id IN ('5','115','116','117','1150','1160','1161','1162','1163','1164','1165','1166','1167','1168','1170');
DELETE FROM sys_config WHERE config_key = 'sys.file.managementConfig';

DROP TRIGGER trg_prevent_referenced_avatar_trash ON file_entry;
DROP TRIGGER trg_retire_replaced_avatar_asset ON sys_user;
DROP TRIGGER trg_validate_user_avatar_update ON sys_user;
DROP TRIGGER trg_validate_user_avatar_insert ON sys_user;
DROP FUNCTION retire_replaced_avatar_asset();
DROP FUNCTION prevent_referenced_avatar_trash();
DROP FUNCTION validate_user_avatar_asset();

ALTER TABLE sys_user
    DROP CONSTRAINT fk_sys_user_avatar_file,
    DROP CONSTRAINT chk_sys_user_avatar_version,
    DROP COLUMN avatar_file_id,
    DROP COLUMN avatar_version,
    ADD COLUMN avatar VARCHAR(255) NULL;
