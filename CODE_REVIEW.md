# Git 工作区代码审查报告（第三次复审）

## 1. 审查结论

- 审查时间：2026-07-17 12:12（Asia/Shanghai）。
- 基线：`main@b98006f4487d2fae764470021ca7ee09b8b01215`。
- 审查范围：121 个已跟踪修改文件，141 个业务未跟踪文件，另含本报告；已跟踪差异为 1,807 行新增、658 行删除。
- 规范依据：根目录 `AGENTS.md`、`apps/backend/AGENTS.md`、`docs/system-log-design.md`、`docs/glossary.md`、`docs/adr/0001` 至 `0039`。
- 审查方法：完整 Git 差异与未跟踪文件核对、前后端和跨上下文调用链静态审查、硬指标 AST 扫描、依赖漏洞审计、前后端自动化门禁、锁文件版本生产构建、真实 PostgreSQL migration/repository 验证及故障路径独立复现。
- 结论：**当前仍不应合并**。本轮确认 6 项 P1、12 项 P2、8 组 P3。新增阻断项包括生产锁文件的高危依赖漏洞和 `error_with_fields!` 原始错误泄漏；异步清理虽然关闭了旧的 HTTP 超时问题，但前端跟踪、终态刷新和跨上下文契约仍未闭合。
- 本次复审只更新本报告，没有修改被审业务代码。

严重级别：

- **P1**：合并阻断；存在已知高危漏洞、敏感信息泄漏、扩大误删、持续故障放大、核心查询漏行或主体需求缺失。
- **P2**：发布前必须修复；存在明确的数据可见性、可用性、性能、迁移兼容或硬架构问题。
- **P3**：质量与治理问题；违反项目硬指标、所有权规则或缺少必要边界验证。

## 2. P1：合并阻断问题

### P1-01 生产锁文件仍解析到 8 个 high 级依赖漏洞

- 证据：`pnpm-lock.yaml:127-129,178-180,3907-3910,8011-8017,8951-8958,9127-9131`。
- `pnpm audit --prod` 失败并报告 16 个漏洞：8 high、5 moderate、3 low。锁文件固定的 Next.js `16.2.4` 命中 7 个 high advisory，其中包括 App Router/Server Components DoS、SSRF 和路由绕过；Axios 依赖的 `form-data 4.0.5` 命中 CRLF injection。另有 `postcss 8.4.31` 的 moderate XSS 和 `@babel/core 7.29.0` 的 low 任意文件读取公告。
- 生产构建使用 frozen lock 时确实解析到 Next.js `16.2.4`。该问题不是本次 system-log 差异引入，但属于当前可发布依赖图，不能因本机 `node_modules` 已漂移到较新版本而忽略。
- 至少应把锁文件刷新到 Next.js `>=16.2.6`、`form-data >=4.0.6` 及其余公告的修复版本，并以 `pnpm install --frozen-lockfile` 后 `pnpm audit --prod` 通过作为验收条件。

### P1-02 system-log writer 写入失败会产生永久自递归

- 证据：`crates/tracing/src/system_log/emitter.rs:236-245`、`crates/tracing/src/macros.rs:16-22`、`crates/tracing/src/system_log/layer.rs:30-53`。
- `flush()` 写库失败后调用带 admission marker 的 `error_with_fields!`，该诊断事件重新进入同一个持久化队列。数据库持续失败时，每次失败又生成下一条待写事件，即使业务事件已经停止也不会收敛。
- 隔离 `FailingSink` 复现中，单条输入在 550 ms 内形成 6 次 sink 调用和 6 条 dropped，随后约每 100 ms 继续增长。
- sink 自身诊断必须绕开 system-log admission，同时保留 stdout、metrics 和 health 可观测性；需要增加“持续失败不产生新持久化事件”的回归测试。

### P1-03 `error_with_fields!` 的 error 参数绕过所有脱敏

- 证据：`crates/tracing/src/macros.rs:20-21`、`crates/tracing/src/system_log/layer.rs:64-79,111-120`、`crates/kernel/src/redaction.rs:27-45`。
- 宏把 `%$error` 原文交给 `tracing`，因此 stdout layer 在任何脱敏前已经收到原文；持久化 layer 又把它保存为普通 `error` 字段，而 `kernel::redaction` 不把 `error` key 视为敏感。
- 隔离实测中，错误文本里的 password、URL userinfo、query token 和 fragment 同时原样进入 stdout 与持久化字段。生产代码有 31 个文件调用该宏分支，暴露面不是单一调用点。
- 这直接违反 ADR 0037 的“进入任何 sink 前脱敏”。错误必须先转换为安全诊断结构或经过内容级脱敏，再构造 tracing event；测试必须同时断言格式化 stdout 和数据库事件。

### P1-04 界面草稿条件与实际清理条件仍可不同，可能扩大误删

- 证据：`apps/frontend/src/widgets/admin-system-logs-panel/ui/filters.tsx:27-53`、`apps/frontend/src/features/system-log-management/model/use-system-log-controller.ts:162-175,205-244`、`apps/frontend/src/widgets/admin-system-logs-panel/ui/dialogs.tsx:86-107`。
- 筛选栏显示并编辑 `filterDraft`，只有点击搜索才更新 `filterQuery`；清理预览、最终执行和导出继续使用旧 `filterQuery`。确认框只显示数量，不展示实际不可变条件。
- 可复现路径：应用宽条件 A，输入窄条件 B 但不点击搜索，然后清理。界面显示 B，count 和 delete 实际按 A 执行。count/confirm 之间的 revision 与冻结 snapshot 已修复，但操作最初选择了哪份条件仍错误。
- 破坏性操作必须与界面明确展示的同一份已验证 snapshot 绑定；需要覆盖 A/B 未应用草稿场景的 controller/component 测试。

### P1-05 异步延迟落库破坏普通列表的游标快照

- 证据：`crates/observability/src/infra/query.rs:24-40,93-102`、`crates/observability/src/infra/query/keyset.rs:9-38`、`crates/tracing/src/system_log/emitter.rs:93-109,184-207`。
- 首屏只以当时已经落库的最大 `(occurred_at,id)` 建立 snapshot。队列中已发生但尚未写入的事件，可能在首屏后以更早的 `occurred_at` 插入。
- 例如首屏返回时间 10、9，boundary 为 9；随后写入时间 9.5；下一页条件 `<9`，该行永久漏掉。XLSX 的 repeatable-read session 不受此问题影响，普通列表每页独立查询没有相同保证。
- 需要以 ingestion sequence/watermark 定义快照，或采用覆盖完整浏览会话的一致性机制，并用真实 PostgreSQL 并发写入测试证明不漏行。

### P1-06 PostgreSQL 基础设施观测的主体仍未交付

- 证据：`crates/tracing/src/infrastructure.rs:23-73`、`apps/backend/src/composition/tracing_config_listener.rs:100-152`；代表性未接入点见 `crates/observability/src/infra/query.rs:19-102`、`crates/user/src/infra/user_repository/queries.rs`、`crates/scheduler/src/infra/persistence/runtime_store.rs`。
- 全仓生产读点中，`InfrastructureDependency::Postgres` 只用于 tracing config listen/read。user、RBAC、system、scheduler、audit、observability 的常规 SQLx repository 均未接 observer。
- 因此 `slow_operation_ms.postgres` 对主体业务 SQL 没有行为影响，数据库失败和慢查询也不会按 ADR 0022/0023 进入系统日志。
- 应在共享 SQLx 执行边界或各 repository adapter 注入 observer，记录静态 operation 与耗时但不记录 SQL 参数；需要覆盖失败、低于阈值、超过阈值和热更新阈值。

## 3. P2：发布前必须修复

### P2-01 生产退出没有 graceful signal 链，内部 drain 也不是总时限

- 证据：`apps/backend/src/startup.rs:8-23`、`crates/tracing/src/system_log/emitter.rs:135-140,184-226,236-245`。
- 生产使用裸 `axum::serve(...).await`，全仓没有 `with_graceful_shutdown`、`ctrl_c` 或 SIGTERM 处理；第 21 行 drain 只有 server 自行结束后才可达，正常容器终止会直接丢弃队列。
- writer 若已进入普通 `insert_batch().await`，5 秒 drain timeout 尚未开始，`shutdown()` 可永久等待。drain timeout 取消已移出 buffer 的 in-flight batch 时，dropped 统计又最多少计 100 条。独立复现中普通 in-flight shutdown 5.5 秒仍未返回，drain timeout 丢 1 条却记录 `dropped=0`。
- 需要一条明确的 signal -> Axum graceful stop -> producer close -> 有总截止时间的 writer drain 链，并准确保留 in-flight 数量和失败原因。

### P2-02 writer 最近失败状态没有生产出口且丢失原因

- 证据：`crates/tracing/src/system_log/emitter.rs:35-71,131-132,242-244`、`apps/backend/src/system.rs:7-32`。
- `SystemLogRuntime::status()` 只有测试读取点；`/health` 仅暴露 tracing listener，metrics 只有 dropped counter。ADR 0010 要求的 latest write-failure state 无法被运维消费。
- `.is_err()` 还直接丢弃 sink 错误文本，状态只保留数量和时间，无法区分连接、分区、约束或权限故障。

### P2-03 listener 的断线与 reconciliation 失败语义仍不闭合

- 证据：`apps/backend/src/composition/tracing_config_listener.rs:100-131`、`docs/adr/0038-reconciled-cluster-tracing-reload.md:14-24`。
- SQLx `PgListener::try_recv()` 用 `Ok(None)` 表示连接丢失后自动重连，当前 `Ok(_)` 却直接 reconciliation 并在成功时标记 recovered，从不记录这次 listener failure。
- reconciliation 读取或解析失败后只标记 unhealthy，随后立即回到 `try_recv().await` 等待下一条通知；如果没有新 `NOTIFY`，实例会无限期保留旧配置，而不是主动重试。

### P2-04 target 筛选仍是精确相等，不符合 ADR 0039

- 证据：`crates/observability/src/infra/query.rs:84-90`、`docs/adr/0039-system-log-target-module-semantics.md:13-16`。
- SQL 仍为 `target=$1`。真实 PostgreSQL 中两条 `user::api::handlers` 记录使用 `target=user` 查询得到 0 条，前缀语义应得到 2 条。
- 需要实现 prefix 或明确的 keyword filtering，并用与语义匹配的索引和执行计划验证。

### P2-05 XLSX 在 async handler 内同步执行 CPU 和文件 IO

- 证据：`crates/observability/src/api/handlers.rs:202-210`、`crates/observability/src/api/export.rs:30-51`、`crates/observability/src/api/export/export_xlsx.rs:41-75`、`crates/kernel/src/excel.rs:23-25,106-114`。
- Tokio worker 同步完成 worksheet 写入、JSON 序列化、ZIP 压缩、临时文件创建和 flush，没有 `spawn_blocking` 或专用线程；期间还长期持有 repeatable-read transaction。
- 这违反 `apps/backend/AGENTS.md:45-51`，会阻塞 runtime worker并延长 MVCC snapshot/vacuum 压力。

### P2-06 短关键词查询仍存在数量级性能退化

- 证据：`crates/observability/src/infra/query.rs:105-114`、`migrations/20260716000001_system_observability.up.sql:18-22`。
- 查询始终组合 FTS 与 `%keyword%` ILIKE。PostgreSQL 17、50,001 行实测中，单字 `%中%` 从 trigram index 取回并 recheck 全部 50,001 个候选，约 5.26 秒；`%abc%` 仅约 348 个候选，约 0.559 ms。
- ADR 0027 明确要求中文和子串搜索，短词不能依赖当前 trigram 路径；需要独立的可索引策略和高基数 `EXPLAIN ANALYZE` 门禁。

### P2-07 定时 retention 部分成功后丢失已提交进度

- 证据：`crates/observability/src/application/retention.rs:11-34`、`apps/backend/src/composition/scheduler_wiring.rs:188-200`。
- 前几批已经独立提交后，后续 `delete_expired_batch(...).await?` 失败会直接丢弃累计 report。failure adapter 只为 `PartialManualCleanup` 写 `deleted/batches` detail，因此定时任务显示 failed，却无法得知实际已删除数量和批次。

### P2-08 异步手工清理 execution 跟踪可以被用户操作丢失

- 证据：`apps/frontend/src/features/system-log-management/model/use-system-log-controller.ts:76,112-115,157,221-235`、`apps/frontend/src/widgets/admin-system-logs-panel/ui/dialogs.tsx:21-40`、`apps/frontend/src/widgets/admin-system-logs-panel/ui/toolbar.tsx:26-36`。
- execution ID 只存在组件 state。关闭状态框、刷新或离开页面都会清空跟踪并停止 SWR 轮询；没有恢复入口。active execution 期间清理按钮又重新启用，可发起第二次请求或覆盖正在跟踪的 ID。
- 需要明确 single-active execution 状态、可恢复的 ID/查询入口和关闭语义，不能让“关闭对话框”等价于放弃后台任务生命周期。

### P2-09 清理终态后第一页列表不会可靠刷新

- 证据：`apps/frontend/src/features/system-log-management/model/use-system-log-controller.ts:51-55`、`apps/frontend/src/features/system-log-management/model/cleanup-execution.ts:16-30`、`apps/frontend/src/shared/lib/use-cursor-navigation.ts:30-49,71-76`、`apps/frontend/src/features/system-log-management/api/index.ts:15-23,47-49`。
- success 只调用 `table.onResetCursor()`。若当前已在第一页，cursor/limit/query 均不变，SWR key 不变，不会重新请求；列表继续显示已删除记录。已有 `refreshSystemLogs()` 只在单删/批删使用。
- failed/partial failure 完全不刷新，还忽略后端返回的 `deleted/batches`，因此已提交删除更容易长期显示为旧数据。

### P2-10 scheduler 表头全选仍选入不可删除 required task

- 证据：`apps/frontend/src/widgets/admin-scheduler-panel/ui/job-row.tsx:52-72`、`apps/frontend/src/widgets/admin-scheduler-panel/ui/table-section.tsx:39-48`、`apps/frontend/src/features/scheduler-management/model/use-scheduler-controller.ts:185-201`。
- 单行 checkbox 已按 `can_delete=false` 禁用，但表头仍把当前页所有 `job_id` 放入 selection。批量删除会因混入 required task 整批 403，使本可删除任务也无法删除。

### P2-11 菜单迁移会让已有自定义角色丢失调度日志导航

- 证据：`migrations/20260717000002_log_menu_hierarchy.up.sql:3-20`、`crates/rbac/src/infra/menu_queries.rs:177-190`、`crates/rbac/src/infra/mapping.rs:146-193`。
- migration 只把菜单 109 从父 3 移到 111，没有为已经拥有 109 的非 admin 角色补 `sys_role_menu(role_id,111)`。navbar 只读取显式 role binding，再从显式父节点建树；这些角色仍有 109/109x 和 3，却缺少新祖先 111，调度日志入口会消失。
- 现有 seed 测试只检查 role 2 与 parent/order，没有覆盖升级前自定义角色。应按既有 ancestor migration 模式迁移关系并增加真实 navbar 回归测试。

### P2-12 异步清理跨上下文业务契约落入 composition root

- 证据：`apps/backend/src/composition/system_log_cleanup_execution.rs:60-150`、`apps/backend/src/composition/scheduler_wiring.rs:184-239`、`crates/scheduler/src/application/tasks/system_log_cleanup.rs:36-45`。
- composition root 判断 execution outcome、验证 job/task/manual 参数、解析 scheduler detail JSON、解析 RFC3339 和 level，并在三处重复 `system_log_cleanup`、schema `1`、`deleted/batches`。
- 这违反 `AGENTS.md:59-70,94-95` 的 composition-only 和 JSON/校验所有权硬规则。scheduler 应提供 typed cleanup execution/detail contract，或使用明确的 cross-context contract；composition 只做端口装配。
- reader `system_log_cleanup_execution.rs:106-120` 还不校验 `detail.schema_version()`，未来同 kind 的 v2 payload 会被静默按 v1 解释，使版本字段失效。

## 4. P3：规范与维护性问题

1. **外部响应缺少运行时校验。** `apps/frontend/src/entities/system-log/api/queries.ts:19-55`、`apps/frontend/src/features/system-log-management/api/index.ts:25-36`、`apps/frontend/src/entities/scheduler/api/queries.ts:43-74` 仅依赖 TypeScript 泛型。非法 cleanup state 会被当成 terminal 停止轮询，`accepted`、level、时间、count、capabilities 也没有 schema 验证。
2. **3 个本次改动源码文件超过 300 行。** `crates/tracing/src/http_capture.rs` 354 行，`apps/backend/src/composition.rs` 306 行，`crates/system/src/application/service/use_cases.rs` 302 行。
3. **函数与复杂度硬限制仍超标。** `apps/backend/src/composition.rs:69-134` 的 `build_app_state` 为 65 个非空行；`apps/frontend/src/widgets/admin-system-logs-panel/ui/table-contracts.test.tsx:28-88` 的 describe callback 为 56 个非空行；`crates/scheduler_macros/src/scheduled_task.rs:63-85` 的 parser 本次新增分支后 cyclomatic complexity 约 11。
4. **13 个函数超过 3 个非 self 参数。** 包括 `core_wiring.rs:58 build_user_services`、`scheduler_wiring.rs:103 build_scheduler_services`、`tracing_config_listener.rs:100,119`、`pconline.rs:86 with_endpoint`、`observability/api/handlers.rs:142 clean_system_logs`、scheduler audited trait/impl、`http_log.rs:136 emit_response`、`infrastructure.rs:44 record` 及 `emitter.rs:184,210,229`。应使用明确的 parts/options 对象。
5. **新增常量与初始化语义未收敛。** `entities/system-log/api/queries.ts:44-46` 的 `1_000` 是轮询 magic number；`use-system-log-controller.ts:68-71` 两次调用当前时间工厂，初始 draft 与 applied query 可相差毫秒。
6. **废弃 tracing helper 仍形成平行 API。** `crates/tracing/src/lib.rs` 继续导出 `fields.rs` 的 legacy helpers；生产无调用点，error helper 同样绕过内容脱敏，并把 target 固定为 helper 模块而非调用方。
7. **文档与实现漂移。** `README.md:357` 仍写 file logging；`docs/system-log-design.md:64-79` 和 ADR 0014 未记录 `202 + execution_id`、scheduler snapshot、polling、partial failure 和 detail schema；`AGENTS.md:233` 仍声称未配置 JavaScript test runner，实际已有 63 个 Vitest 文件。
8. **关键回归测试仍缺失。** 未覆盖 writer 不自递归、raw error 双 sink 脱敏、生产 signal/drain、延迟落库快照、PostgreSQL repository observer、listener `Ok(None)`/主动重试、target 前缀、短关键词执行计划、draft A/B 清理、execution 关闭/恢复/partial failure/终态刷新、scheduler capability 全选、自定义角色 navbar。migration 测试也未锁住 UTC 日界线和实际 child partition indexes。

## 5. 已确认关闭或符合规范的事项

- 手工清理已改成 `202 Accepted + execution_id` 的 scheduler 独立执行，旧的单 HTTP 请求无界批次循环问题关闭；count/confirm 使用冻结 filter snapshot，并用 revision 拒绝过期 count 回写。
- system-log 表头与行选择列已对齐；登录日志时间列紧邻 actions，三语标签统一为“登录时间 / Login Time / 登入時間”，新增测试通过。
- 前端 Prettier、`next-env.d.ts` 开发生成差异和无效 Iconify 名称已修复。
- 新前端 route 保持薄入口；FSD 顶层目录、依赖方向及 slice `index.ts` public API 未发现新增漂移。
- 三语言 system-log namespace key 一致；未发现新增硬编码 secret、危险 HTML、未编码路径 ID 或明显 XSS 注入点。
- SQL 过滤使用 bind 参数；XLSX 使用 `write_string`，未发现公式注入；URL、HTTP body 和结构化敏感字段的既有脱敏问题已修复，本轮剩余的是 raw error 专用路径。
- UTC 分区实现和 child indexes 经真实 PostgreSQL 验证正确；新 migration 的 up/down/refresh/reset/fresh 生命周期通过，但上述边界和角色升级场景仍缺自动化覆盖。
- Rust 格式、Clippy、check、测试、RustSec audit 和 cargo-deny 均通过；这些门禁通过不抵消上述运行时、架构及 pnpm audit 问题。

## 6. 验证记录

| 检查 | 结果 |
| --- | --- |
| `git diff --check` | 通过 |
| `cargo fmt --all -- --check` | 通过 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 通过 |
| `cargo check --workspace --all-targets` | 通过 |
| `timeout 60 cargo test -p taco_tracing -p observability -p scheduler` | 通过：16、19、83 unit + 9 integration + trybuild |
| backend system-log migration tests | 通过：5 项；完整 up/down/refresh/reset/fresh 通过 |
| `cargo audit` | 通过：521 个依赖无 RustSec 漏洞 |
| `cargo deny check` | 通过；已配置允许的 duplicate warnings 不影响退出码 |
| `pnpm --filter frontend fm:check` | 通过 |
| `pnpm lint:frontend` | 通过 |
| `pnpm --filter frontend test` | 通过：63 files、278 tests |
| frozen-lock production build | 通过：Next.js 16.2.4，24 个 route，含 system-log route |
| `pnpm audit --prod` | **失败：16 vulnerabilities，8 high / 5 moderate / 3 low** |

最终判断：**NO-GO**。P1 与 P2 均未清零，且生产依赖安全门禁明确失败。
