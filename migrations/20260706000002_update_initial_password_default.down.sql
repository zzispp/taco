UPDATE sys_config
SET config_value = '123456',
    remark = '初始化密码 123456'
WHERE config_key = 'sys.user.initPassword'
  AND config_value = '12345678';
