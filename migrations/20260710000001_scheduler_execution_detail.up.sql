DO $scheduler_execution_detail$
BEGIN
    IF EXISTS (
        SELECT 1 FROM sys_menu WHERE perms = 'system:job:log:detail'
    ) THEN
        RAISE EXCEPTION 'scheduler job log detail permission already exists';
    END IF;
END;
$scheduler_execution_detail$;

ALTER TABLE sys_job_execution
    ADD COLUMN detail_kind VARCHAR(64) NULL,
    ADD COLUMN detail_schema_version SMALLINT NULL,
    ADD COLUMN detail_payload JSONB NULL,
    ADD CONSTRAINT chk_sys_job_execution_detail_bundle CHECK (
        (detail_kind IS NULL AND detail_schema_version IS NULL AND detail_payload IS NULL)
        OR (detail_kind IS NOT NULL AND detail_schema_version IS NOT NULL AND detail_payload IS NOT NULL)
    ),
    ADD CONSTRAINT chk_sys_job_execution_detail_kind CHECK (
        detail_kind IS NULL
        OR btrim(detail_kind, ' ' || chr(9) || chr(10) || chr(11) || chr(12) || chr(13)) <> ''
    ),
    ADD CONSTRAINT chk_sys_job_execution_detail_schema_version CHECK (
        detail_schema_version IS NULL OR detail_schema_version > 0
    ),
    ADD CONSTRAINT chk_sys_job_execution_detail_payload CHECK (
        detail_payload IS NULL OR jsonb_typeof(detail_payload) = 'object'
    ),
    ADD CONSTRAINT chk_sys_job_execution_detail_lifecycle CHECK (
        state = 'T'
        OR (detail_kind IS NULL AND detail_schema_version IS NULL AND detail_payload IS NULL)
    );

INSERT INTO sys_menu (
    menu_id, menu_name, parent_id, order_num, path, component, query, route_name,
    is_frame, is_cache, menu_type, visible, status, perms, icon, create_by, create_time, remark
)
VALUES (
    '1093', '调度日志详情', '109', 4, '#', '', '', '',
    FALSE, FALSE, 'F', '0', '0', 'system:job:log:detail', '#', 'admin', CURRENT_TIMESTAMP, ''
);

INSERT INTO sys_role_menu (role_id, menu_id)
VALUES ('admin', '1093');
