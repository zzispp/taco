DELETE FROM sys_role_menu AS role_menu
USING sys_log_menu_hierarchy_role_grant AS grant_record
WHERE role_menu.role_id = grant_record.role_id
  AND role_menu.menu_id = '111';

DROP TABLE sys_log_menu_hierarchy_role_grant;
