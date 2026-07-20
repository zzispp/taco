UPDATE sys_config
SET remark = 'JWT Token 过期时间 JSON。access_token_ttl_seconds 是访问令牌有效秒数，refresh_token_ttl_seconds 是刷新令牌有效秒数；签名密钥由安装引导生成并保存在本地加密安装状态中，不能通过系统配置修改。'
WHERE config_key = 'sys.auth.tokenConfig';

UPDATE sys_config
SET remark = '头像上传运行期配置 JSON。max_bytes 是单个头像文件允许的最大字节数；上传目录由安装数据目录管理，不能通过系统配置修改。'
WHERE config_key = 'sys.upload.avatarConfig';
