UPDATE sys_config
SET config_value = (config_value::jsonb - 'site_subtitle')::text,
    remark = '站点展示公开配置 JSON。site_name 是站点名称，logo_url 是 Logo 地址，footer_text 是页脚文案。'
WHERE config_key = 'sys.site.displayConfig';
