ALTER TABLE sys_user
    ADD COLUMN avatar_file_id VARCHAR(36) NULL,
    ADD COLUMN avatar_version BIGINT NOT NULL DEFAULT 0,
    ADD CONSTRAINT chk_sys_user_avatar_version CHECK (avatar_version >= 0),
    ADD CONSTRAINT fk_sys_user_avatar_file FOREIGN KEY (avatar_file_id) REFERENCES file_entry(entry_id) ON DELETE RESTRICT;

ALTER TABLE sys_user DROP COLUMN avatar;

CREATE FUNCTION validate_user_avatar_asset() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.avatar_file_id IS NULL THEN
        RETURN NEW;
    END IF;
    PERFORM e.entry_id
    FROM file_entry e
    JOIN file_space s ON s.space_id = e.space_id
    JOIN file_object o ON o.object_id = e.object_id
    WHERE e.entry_id = NEW.avatar_file_id
      AND s.owner_user_id = NEW.user_id
      AND e.kind = 'file'
      AND e.status = 'active'
      AND LOWER(BTRIM(o.content_type)) IN ('image/png', 'image/jpeg', 'image/webp')
    FOR UPDATE OF e;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE = '23514', MESSAGE = 'avatar asset must be an active supported image owned by the user';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION prevent_referenced_avatar_trash() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = 'trashed'
       AND OLD.status <> 'trashed'
       AND EXISTS (SELECT 1 FROM sys_user u WHERE u.avatar_file_id = OLD.entry_id)
    THEN
        RAISE EXCEPTION USING ERRCODE = '23503', MESSAGE = 'referenced avatar asset cannot be moved to trash';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION retire_replaced_avatar_asset() RETURNS TRIGGER AS $$
DECLARE
    previous_space_id VARCHAR(36);
BEGIN
    IF OLD.avatar_file_id IS NULL OR OLD.avatar_file_id IS NOT DISTINCT FROM NEW.avatar_file_id THEN
        RETURN NEW;
    END IF;
    IF EXISTS (SELECT 1 FROM file_business_reference r WHERE r.entry_id = OLD.avatar_file_id) THEN
        RAISE EXCEPTION USING ERRCODE = '23503', MESSAGE = 'referenced previous avatar asset cannot be moved to trash';
    END IF;
    UPDATE file_entry
    SET status = 'trashed', trashed_at = CURRENT_TIMESTAMP, updated_by = NEW.user_id, updated_at = CURRENT_TIMESTAMP
    WHERE entry_id = OLD.avatar_file_id AND status = 'active'
    RETURNING space_id INTO previous_space_id;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE = '23514', MESSAGE = 'previous avatar asset must remain active until replacement';
    END IF;
    UPDATE file_space
    SET active_bytes = COALESCE((SELECT SUM(o.size_bytes) FROM file_entry e JOIN file_object o ON o.object_id = e.object_id WHERE e.space_id = previous_space_id AND e.status = 'active'), 0),
        trashed_bytes = COALESCE((SELECT SUM(o.size_bytes) FROM file_entry e JOIN file_object o ON o.object_id = e.object_id WHERE e.space_id = previous_space_id AND e.status = 'trashed'), 0),
        updated_at = CURRENT_TIMESTAMP
    WHERE space_id = previous_space_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_validate_user_avatar_insert
    BEFORE INSERT ON sys_user
    FOR EACH ROW EXECUTE FUNCTION validate_user_avatar_asset();

CREATE TRIGGER trg_validate_user_avatar_update
    BEFORE UPDATE OF avatar_file_id ON sys_user
    FOR EACH ROW EXECUTE FUNCTION validate_user_avatar_asset();

CREATE TRIGGER trg_retire_replaced_avatar_asset
    AFTER UPDATE OF avatar_file_id ON sys_user
    FOR EACH ROW EXECUTE FUNCTION retire_replaced_avatar_asset();

CREATE TRIGGER trg_prevent_referenced_avatar_trash
    BEFORE UPDATE OF status ON file_entry
    FOR EACH ROW EXECUTE FUNCTION prevent_referenced_avatar_trash();

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, create_by, create_time, remark)
VALUES (
    '18',
    '文件管理-运行配置',
    'sys.file.managementConfig',
    '{"max_file_bytes":10737418240,"default_space_quota_bytes":21474836480,"upload_part_bytes":16777216,"upload_session_inactivity_days":7}',
    'Y',
    'admin',
    CURRENT_TIMESTAMP,
    '文件管理运行配置 JSON。max_file_bytes 是单个受管文件大小上限；default_space_quota_bytes 是个人空间默认逻辑配额；upload_part_bytes 是上传分片大小；upload_session_inactivity_days 是上传会话无活动期限。'
);

INSERT INTO sys_menu (menu_id, menu_name, parent_id, order_num, path, component, query, route_name, is_frame, is_cache, menu_type, visible, status, perms, icon, create_by, create_time, remark)
VALUES
    ('5', '文件管理', '0', 3, '/dashboard/files', NULL, '', '', FALSE, FALSE, 'M', '0', '0', NULL, 'icon.file-manager', 'admin', CURRENT_TIMESTAMP, '文件管理目录'),
    ('115', '文件概览', '5', 1, '/dashboard/file', 'file/overview/index', '', 'FileOverview', FALSE, FALSE, 'C', '0', '0', 'file:asset:list', 'icon.file-overview', 'admin', CURRENT_TIMESTAMP, '文件概览菜单'),
    ('116', '资产管理', '5', 2, '/dashboard/file-manager', 'file/manager/index', '', 'FileManager', FALSE, FALSE, 'C', '0', '0', 'file:asset:list', 'icon.file-manager', 'admin', CURRENT_TIMESTAMP, '资产管理菜单'),
    ('117', '空间管理', '5', 3, '/dashboard/file-spaces', 'file/spaces/index', '', 'FileSpaces', FALSE, FALSE, 'C', '0', '0', 'file:space:list', 'icon.storage', 'admin', CURRENT_TIMESTAMP, '文件空间管理菜单'),
    ('1150', '存储提供方查询', '115', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'file:provider:query', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1160', '资产查询', '116', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'file:asset:query', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1161', '资产下载', '116', 2, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'file:asset:download', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1162', '资产上传', '116', 3, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'file:asset:upload', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1163', '文件夹新增', '116', 4, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'file:folder:add', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1164', '资产编辑', '116', 5, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'file:asset:edit', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1165', '资产移入回收站', '116', 6, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'file:asset:remove', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1166', '资产恢复', '116', 7, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'file:asset:restore', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1167', '资产永久删除', '116', 8, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'file:asset:purge', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1168', '上传会话管理', '116', 9, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'file:upload:manage', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1170', '空间配额管理', '117', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'file:space:quota', '#', 'admin', CURRENT_TIMESTAMP, '');

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 'admin', menu_id
FROM sys_menu
WHERE menu_id IN ('5', '115', '116', '117', '1150', '1160', '1161', '1162', '1163', '1164', '1165', '1166', '1167', '1168', '1170');

INSERT INTO sys_job (job_id, job_name, job_group, task_key, task_params, params_schema_version, repeatable, invoke_target, cron_expression, misfire_policy, concurrent, status, create_by, create_time, remark)
VALUES
    ('file-purge-trash', '文件回收站清理', 'SYSTEM', 'file.purgeTrash', '{"retention_days":30,"batch_size":1000}'::jsonb, 1, FALSE, 'file.purgeTrash(retention_days=30,batch_size=1000)', '0 0 20 * * *', '2', '1', '0', 'admin', CURRENT_TIMESTAMP, '文件管理必需清理任务；仅可暂停并调整执行策略。'),
    ('file-cleanup-upload-sessions', '文件上传会话清理', 'SYSTEM', 'file.cleanupUploadSessions', '{"batch_size":1000}'::jsonb, 1, FALSE, 'file.cleanupUploadSessions(batch_size=1000)', '0 0 21 * * *', '2', '1', '0', 'admin', CURRENT_TIMESTAMP, '文件管理必需清理任务；无活动期限由文件管理运行配置提供。');
