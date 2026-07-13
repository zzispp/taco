DELETE FROM sys_role_menu WHERE menu_id IN ('110', '1100', '1101', '1102', '1103');
DELETE FROM sys_menu WHERE menu_id IN ('110', '1100', '1101', '1102', '1103');
DELETE FROM sys_dict_data WHERE dict_type IN ('sys_notice_type', 'sys_notice_status');
DELETE FROM sys_dict_type WHERE dict_id IN ('notice-type', 'notice-status');
DROP TABLE sys_notice_read;
DROP TABLE sys_notice;
