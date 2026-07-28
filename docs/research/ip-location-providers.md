# Public IP Location Provider Research

日期：2026-07-28

## 结论

新链路建议按以下顺序调用：

1. `ipwho.is`
2. `ipquery.io`
3. `GeoJS`

三者均支持 HTTPS、免注册、免 API key、指定 IPv4/IPv6 查询和 JSON 响应。`ipwho.is` 的免费端点明确允许商业使用，位置字段和错误契约最完整；`ipquery.io` 同样明确允许商业项目且位置字段完整，但缺少正式在线条款页；GeoJS 官方明确提供免费不限量的 production instance，并有正式服务条款，但部分地址只返回国家、不返回省州或城市。

FreeIPAPI、DB-IP Free、ReallyFreeGeoIP 可作为候选对照，不建议加入本次生产链路。`ip-api.com`、`ipapi.is` 明确不满足商业生产使用要求。

实现上不能只把 HTTP 2xx 当作成功。每个 adapter 还必须校验服务方错误字段及至少一个有效位置字段；`200 + error`、`200 + 全空位置` 都应视为该 provider 失败并继续下一家。

## 候选对比

| Provider | HTTPS / 无 key | IPv4 / IPv6 | 国家 / 省州 / 城市 | 免费限制 | 商业使用与署名 | 结论 |
| --- | --- | --- | --- | --- | --- | --- |
| ipwho.is | 是 | 是 / 是 | 是 / 是 / 是 | 1,000 次/日/客户端 IP | 明确允许商业使用；未发现署名要求 | 推荐第 1 |
| ipquery.io | 是 | 是 / 是 | 是 / 是 / 是 | 官网称 unlimited free tier，但仍定义 429 | 明确允许商业应用、SaaS 和内部工具；缺少正式条款页 | 推荐第 2 |
| GeoJS | 是 | 是 / 是 | 是 / 是 / 是，未知字段可能省略 | 官方称免费不限量 production instance；仍保留节流权利 | 条款覆盖公司使用且未限制非商业；未发现署名要求 | 推荐第 3 |
| FreeIPAPI | 是 | 是 / 是 | 是 / 是 / 是 | 60 次/分钟/服务器公网 IP | 明确允许商业和非商业使用；未发现署名要求 | 当前实测不稳定，淘汰 |
| DB-IP Free | 是（实测） | 是 / 是 | 是 / 是 / 是 | 500 次/日 | 免费 API 的商业许可及 GeoNames 署名义务表述不清 | 淘汰 |
| ReallyFreeGeoIP | 是 | 是 / 是 | 是 / 是 / 是，可能全空 | 官网称无固定限制，滥用会封禁 | 未明确商业许可、数据许可或署名规则 | 淘汰 |
| ipapi.is | 是 | 是 / 是 | 是 / 是 / 是 | 1,000 次/日 | 免费层明确仅限测试开发，禁止商业产品 | 淘汰 |
| ip-api.com | 免费端点不支持 HTTPS | 是 / 是 | 是 / 是 / 是 | 45 次/分钟/IP | 免费端点明确禁止商业使用 | 淘汰 |

## 推荐 Provider

### 1. ipwho.is

- Endpoint：`GET https://ipwho.is/{ip}`。官方说明 `{ip}` 可为 IPv4 或 IPv6，也可省略；HTTPS 受支持且不需要 API key。[官方文档](https://ipwhois.io/documentation#quickstart)
- 成功字段：`success`、`type`、`country`、`country_code`、`region`、`region_code`、`city` 等。[响应字段](https://ipwhois.io/documentation#returned-data)
- 错误契约：普通 HTTP 错误使用 4xx JSON；非法或保留地址也可能返回 HTTP 200，但 body 为 `success:false` 和 `message`。超限返回 429，并带 `Retry-After`。[错误码](https://ipwhois.io/documentation#error_codes)
- 限流：免费端点每个客户端 IP 每日 1,000 次，超限后 24 小时恢复；免费端点无 SLA。[限流与方案说明](https://ipwhois.io/documentation#usage-limits)
- 使用约束：官方免费方案明确写明 `Commercial use allowed`。服务条款授予个人或内部业务用途，禁止转售、再许可或重分发服务。[官方条款](https://ipwhois.io/terms)
- 署名：文档和条款未发现要求 API 调用方展示署名的条款。
- 实测：[`8.8.8.8`](https://ipwho.is/8.8.8.8) 与 [`2001:4860:4860::8888`](https://ipwho.is/2001:4860:4860::8888) 均返回 HTTP 200 JSON，包含国家、省州和城市。

这是当前证据最完整的第一 provider。调用方必须检查 `success == true`，不能仅检查 HTTP 200。

### 2. ipquery.io

- Endpoint：`GET https://api.ipquery.io/{ip}`，无需 API key。官网提供单 IP、批量和多种格式入口，并宣称 `Unlimited Free Tier`。[官方文档](https://ipquery.io/#endpoints)
- 成功字段：`location.country`、`country_code`、`state`、`city`、`zipcode`、经纬度和时区。[数据字典](https://ipquery.io/#data-dictionary)
- IPv4/IPv6：[`8.8.8.8`](https://api.ipquery.io/8.8.8.8) 和 [`2001:4860:4860::8888`](https://api.ipquery.io/2001:4860:4860::8888) 均实测返回 HTTP 200 JSON 和完整位置字段。
- 错误契约：官网列出 400、404、429 和 500；非法地址实测返回 HTTP 404、`text/plain`，不是 JSON。[官方错误表](https://ipquery.io/#errors)
- 限流：官网虽称 unlimited free tier，但错误表仍定义 429，因此不能假设永不节流。
- 使用约束：官方 FAQ 明确允许用于商业应用、SaaS 和内部工具，表述为 `without restriction`；还声明数据由其自有基础设施聚合处理。[官方 FAQ](https://ipquery.io/#faq)
- 法律缺口：页脚的 Terms/Privacy 实际链接到 `mailto:contact@ipquery.io`，没有可审阅的正式条款页。这使其法律文本完备性弱于 `ipwho.is`。
- 署名：官网未声明调用方署名要求。

适合作为第二 provider。adapter 必须先检查 HTTP status，再解析 JSON，避免把其 404 文本当成 JSON 解码错误。

### 3. GeoJS

- Endpoint：`GET https://get.geojs.io/v1/ip/geo/{ip}.json`。官方只提供 HTTPS，并列出 IPv4-only、IPv6-only 域名。[通用说明](https://www.geojs.io/docs/general/)
- 成功字段：`country`、`country_code`、`region`、`city`、经纬度、时区和 ASN。省州或城市未知时字段可能直接省略。[Geo endpoint](https://www.geojs.io/docs/v1/endpoints/geo/)
- IPv4/IPv6：[`8.8.8.8`](https://get.geojs.io/v1/ip/geo/8.8.8.8.json) 和 [`2001:4860:4860::8888`](https://get.geojs.io/v1/ip/geo/2001:4860:4860::8888.json) 均实测返回 HTTP 200 JSON。
- 错误契约：非法地址实测返回 HTTP 404 和 HTML body；官方没有定义结构化 JSON error envelope。
- 限流：通用文档称目前没有 rate limit；官方源码 README 将 `get.geojs.io` 描述为 `free unlimited production instance`。服务条款仍保留认定 excessive usage、节流或封禁来源 IP 的权利。[官方源码 README](https://github.com/jloh/geojs#readme) [服务条款](https://www.geojs.io/tos/)
- 使用约束：服务条款明确覆盖代表公司实体使用的情形，只禁止违法、滥用和过量查询，未将服务限制为非商业用途。
- 署名：API 服务条款未要求调用方署名。其服务端源码使用 MIT 许可证；只有复制软件本身时才触发 MIT copyright notice 条件。[官方源码](https://github.com/jloh/geojs)

适合作为最终 provider。成功解析时应允许 `region`、`city` 缺失，但至少要求 `country` 非空；否则继续返回最终 `Unknown`。

## 未采用 Provider

### FreeIPAPI

- 正确免费 endpoint 是 `GET https://free.freeipapi.com/api/json/{ip}`，免费版无需认证。[API 介绍](https://freeipapi.com/docs/api-reference/api-introduction)
- 返回 `countryName`、`regionName`、`cityName` 等字段，文档给出指定 IP 的 JSON 结构。[IP 查询文档](https://freeipapi.com/docs/api-reference/get-ip-info)
- 免费后端限流为每个服务器公网 IP 60 次/分钟。[限流说明](https://freeipapi.com/rate-limit)
- 官网明确称可用于商业和非商业用途，未发现调用方署名要求。[官方介绍](https://freeipapi.com/docs/get-started/introduction)
- 非法地址实测为 HTTP 200 且关键位置字段全部为 `null`，没有明确 error envelope。
- 2026-07-28 实测多个 IPv4/IPv6 请求期间出现 Cloudflare HTTP 520 文本响应；成功响应也曾明显慢于其他候选。它会增加而不是降低登录链路的不确定性，因此不进入本次生产链路。

### DB-IP Free

- Endpoint：`GET https://api.db-ip.com/v2/free/{ip}`，无需 key，返回国家、省州和城市；免费层限制为每天 500 次。[Free API](https://db-ip.com/api/free)
- IPv4 和 IPv6 endpoint 当前均实测可通过 HTTPS 返回 JSON；非法地址返回 HTTP 200 和 `errorCode: INVALID_ADDRESS`。[IPv4 示例](https://api.db-ip.com/v2/free/8.8.8.8) [IPv6 示例](https://api.db-ip.com/v2/free/2001:4860:4860::8888)
- 官方页面仅将免费 API 定位为 prototype 或 small website，未明确免费 API 的商业使用权限。
- 通用条款说明部分数据来自采用 Creative Commons Attribution 的 GeoNames；同时仅明确免费数据库下载使用 CC BY 4.0，没有清楚说明免费在线 API 调用方是否必须署名。[官方条款](https://db-ip.com/tos.php)

限额偏低且许可/署名边界不够清楚，不进入链路。

### ReallyFreeGeoIP

- Endpoint：`GET https://reallyfreegeoip.org/json/{ip}`；无需账号或 key，官网称没有固定限制，但滥用来源会被封禁。[官方首页](https://reallyfreegeoip.org/)
- 返回国家、省州、城市、邮编、时区和经纬度；IPv4/IPv6 实测可用。非法地址返回 404 JSON `{"error":"Not found."}`。
- 对部分公网地址，实测 HTTP 200 但所有位置字段为空，因此必须把空 `country_name` 当失败。
- 官网没有正式服务条款、数据许可、商业使用声明或署名说明。

技术上可调用，但生产许可和成功语义不够明确，不进入链路。

### ipapi.is

- 技术上满足 HTTPS、免 key、IPv4/IPv6 和完整位置字段，匿名免费额度为 1,000 次/日。[开发文档](https://ipapi.is/developers.html)
- 但服务条款明确规定免费层仅用于 testing/development，不能用于 commercial products。[官方条款](https://ipapi.is/terms.html)

因此不符合生产候选要求。

### ip-api.com

- 免费 JSON endpoint 无需 key，支持 IPv4/IPv6，限流 45 次/分钟/IP。[官方文档](https://ip-api.com/docs/api:json)
- 官方明确说明免费端点不提供 HTTPS，并禁止商业使用；HTTPS 和商业使用属于 Pro 服务。[SSL 与使用限制](https://ip-api.com/docs/api:json#ssl)

它同时违反 HTTPS 和商业生产使用要求，直接淘汰。

## Adapter 契约建议

每个 provider adapter 应返回统一的结构化结果，至少区分：

- `Resolved { country, region?, city? }`
- `ProviderFailed { provider, category, status?, message }`

统一成功判定：

1. HTTP 为 2xx。
2. body 是该 provider 声明的 JSON 格式。
3. provider 自身没有错误标志，例如 `success:false` 或 `errorCode`。
4. `country` 非空；省州和城市允许为空。

连续失败应逐个记录 provider 名称、失败类别、HTTP 状态和耗时，但不得记录完整响应 body 或把客户端 IP 放入高基数 metrics label。所有 provider 均失败时，由上层将在线会话位置写为本地化“未知”，不能让 enrichment 失败阻断认证。
