DELETE FROM sys_role_menu WHERE menu_id IN ('108','109','1080','1081','1082','1083','1084','1085','1086','1090','1091','1092');
DELETE FROM sys_menu WHERE menu_id IN ('108','109','1080','1081','1082','1083','1084','1085','1086','1090','1091','1092');
DELETE FROM sys_dict_data WHERE dict_code IN ('scheduler-job-group-default','scheduler-job-group-system','scheduler-job-status-normal','scheduler-job-status-paused');
DELETE FROM sys_dict_type WHERE dict_id IN ('scheduler-job-group','scheduler-job-status');
DROP TABLE sys_job_execution;
DROP TABLE sys_job;
