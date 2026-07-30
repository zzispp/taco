UPDATE sys_config
SET config_value = CASE
        WHEN config_value::jsonb ? 'site_subtitle' THEN config_value
        ELSE jsonb_set(
            config_value::jsonb,
            '{site_subtitle}',
            to_jsonb('Backend Control Plane'::text),
            TRUE
        )::text
    END,
    remark = '站点展示公开配置 JSON。site_name 是站点名称，site_subtitle 是 Dashboard 品牌区副标题，logo_url 是 Logo 地址，footer_text 是页脚文案。'
WHERE config_key = 'sys.site.displayConfig';
