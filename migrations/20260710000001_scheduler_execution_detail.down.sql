DELETE FROM sys_role_menu WHERE menu_id = '1093';
DELETE FROM sys_menu WHERE menu_id = '1093';

ALTER TABLE sys_job_execution
    DROP CONSTRAINT chk_sys_job_execution_detail_lifecycle,
    DROP CONSTRAINT chk_sys_job_execution_detail_payload,
    DROP CONSTRAINT chk_sys_job_execution_detail_schema_version,
    DROP CONSTRAINT chk_sys_job_execution_detail_kind,
    DROP CONSTRAINT chk_sys_job_execution_detail_bundle,
    DROP COLUMN detail_payload,
    DROP COLUMN detail_schema_version,
    DROP COLUMN detail_kind;
