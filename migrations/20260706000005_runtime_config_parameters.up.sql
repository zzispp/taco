ALTER TABLE sys_config ALTER COLUMN config_value TYPE TEXT;

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '10', '认证-JWT Token 配置', 'sys.auth.tokenConfig', '{"access_token_ttl_seconds":1440,"refresh_token_ttl_seconds":604800}', 'Y', FALSE, 'admin', CURRENT_TIMESTAMP,
       'JWT Token 过期时间 JSON。access_token_ttl_seconds 是访问令牌有效秒数，refresh_token_ttl_seconds 是刷新令牌有效秒数；secret 仍在 config.yaml。'
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.auth.tokenConfig');

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '11', '用户管理-密码策略', 'sys.user.passwordPolicy', '{"min_length":8,"max_length":128,"require_letter":false,"require_number":false,"require_symbol":false,"forbid_username_contains":true}', 'Y', TRUE, 'admin', CURRENT_TIMESTAMP,
       '用户密码策略 JSON。min_length 最小长度，max_length 最大长度，require_letter 是否必须含字母，require_number 是否必须含数字，require_symbol 是否必须含符号，forbid_username_contains 是否禁止包含用户名。'
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.user.passwordPolicy');

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '12', '上传-头像配置', 'sys.upload.avatarConfig', '{"max_bytes":2097152}', 'Y', FALSE, 'admin', CURRENT_TIMESTAMP,
       '头像上传运行期配置 JSON。max_bytes 是单个头像文件允许的最大字节数；上传目录仍在 config.yaml。'
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.upload.avatarConfig');

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '13', '导出-批量配置', 'sys.export.batchConfig', '{"page_size":100}', 'Y', FALSE, 'admin', CURRENT_TIMESTAMP,
       '后端导出批量查询配置 JSON。page_size 是导出时每批从数据库读取的记录数，有效范围为 1..=10000。'
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.export.batchConfig');

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '14', '站点-展示配置', 'sys.site.displayConfig', '{"site_name":"taco","logo_url":"/logo/logo.svg","footer_text":"taco backend control plane."}', 'Y', TRUE, 'admin', CURRENT_TIMESTAMP,
       '站点展示公开配置 JSON。site_name 是站点名称，logo_url 是 Logo 地址，footer_text 是页脚文案。'
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.site.displayConfig');
