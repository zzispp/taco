UPDATE sys_config
SET config_value = '12345678',
    remark = '新建用户未填写密码时使用的初始密码；由后端读取，不对前端公开。'
WHERE config_key = 'sys.user.initPassword'
  AND config_value = '123456';
