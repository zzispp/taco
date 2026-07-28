DO $$
BEGIN
    UPDATE sys_config
    SET update_by = 'admin',
        update_time = CURRENT_TIMESTAMP,
        remark = '客户端 IP 归属地解析配置 JSON。enabled 控制是否调用 pconline 解析公网 IP；关闭时地点显示本地化未知文案，内网 IP 显示本地化内网文案；在线会话与审计日志共用此配置。'
    WHERE config_key = 'sys.client.ipLocationConfig';

    IF NOT FOUND THEN
        RAISE EXCEPTION 'cannot restore missing sys.client.ipLocationConfig provider description';
    END IF;
END $$;
