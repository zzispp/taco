UPDATE sys_config
SET config_value = '12345678',
    remark = '初始化密码 12345678'
WHERE config_key = 'sys.user.initPassword'
  AND config_value = '123456';
