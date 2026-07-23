# 生产 Docker 部署

Taco 的生产镜像包含静态导出的前端和一个 `taco` 可执行文件。PostgreSQL 与 Redis 是外部依赖，不由生产 Compose 文件创建。启动配置由宿主机 YAML 文件提供，容器内不保存、生成或修改配置。

## 配置文件

从发布包或仓库中的 `config/config.example.yaml` 创建宿主机配置：

```bash
sudo install -d -m 0750 /etc/taco
sudo cp config/config.example.yaml /etc/taco/config.yaml
sudo chmod 0600 /etc/taco/config.yaml
```

生成 JWT 签名密钥时，先构建镜像，再运行不依赖 YAML 的密钥命令：

```bash
docker compose -f compose.production.yaml build taco
docker compose -f compose.production.yaml run --rm taco secret generate-jwt
```

将命令的唯一输出完整复制到 `/etc/taco/config.yaml` 的 `jwt.secret`。不要将密钥提交到仓库，也不要通过命令参数传递。使用检出的 Rust 工具链时，等价命令为：

```bash
cargo run -p backend --bin taco -- secret generate-jwt
```

使用受控的编辑方式替换文件中每个 `<...>` 占位符。生产配置至少应满足下列约束：

- `data_directory` 可以是绝对路径或相对路径；相对路径以 YAML 文件所在目录解析。保留模板的 `../local-data` 时，Compose 将 YAML 挂载到 `/app/config/config.yaml`，其运行时结果为 `/app/local-data`；命名 `taco-data` volume 持久化该目录，Local File Provider 固定使用 `/app/local-data/files`。
- `server.host` 使用容器可监听的地址，`server.port` 与 Compose 发布端口一致。
- `database`、`redis` 与 `jwt.secret` 必须填写真实的外部依赖和密钥；`jwt.secret` 至少为 32 UTF-8 字节；不要把真实配置或凭据提交到仓库。
- `database.auto_migrate` 在生产环境建议明确设为 `false`。

配置加载是严格的：所有字段必须存在，未知字段、未替换的 `<...>` 占位符或空的必填值都会使 Taco 失败退出。YAML 不支持环境变量插值或隐式默认值。修改 `/etc/taco/config.yaml` 后必须重启 Taco。

## 首次启动

`compose.production.yaml` 将宿主机 `/etc/taco/config.yaml` 以只读方式挂载到容器 `/app/config/config.yaml`，并以 `taco --config /app/config/config.yaml` 启动服务。示例配置默认关闭自动迁移；新数据库必须先准备 schema 和启用中的系统管理员，否则 Taco 不会绑定 HTTP 端口。

先构建 Compose 服务镜像，再使用同一只读配置检查并应用 migration：

```bash
docker compose -f compose.production.yaml build taco
docker compose -f compose.production.yaml run --rm taco --config /app/config/config.yaml migration status
docker compose -f compose.production.yaml run --rm taco --config /app/config/config.yaml migration up
```

随后在首次启动前显式创建管理员：

```bash
docker compose -f compose.production.yaml run --rm taco --config /app/config/config.yaml administrator bootstrap --username <username> --email <email> --password-stdin
```

`--password-stdin` 只读取标准输入的第一行密码；密码不能作为命令参数提供，也不会写入 YAML 或命令输出。该命令只在不存在绑定内置 `admin`（`system=true`）角色的启用用户时允许执行，并在同一数据库事务中创建用户和绑定该角色。

完成上述初始化后启动服务：

```bash
docker compose -f compose.production.yaml up -d
```

服务仅发布到 `127.0.0.1:3000`。Compose 存活探针访问 `/health`；`/ready` 在 HTTP 服务成功启动后返回 `200`。配置、schema 和依赖初始化均在监听端口前完成，因此 `/ready` 不是持续依赖健康探针。

## 反向代理契约

在主机侧反向代理终止 TLS，并将上游请求代理到 `http://127.0.0.1:3000`。代理必须移除客户端提供的转发头，并写入规范的 `X-Forwarded-For`、`X-Forwarded-Host` 与 `X-Forwarded-Proto`。Taco 接受这些标准头，无需配置受信任代理 CIDR。

浏览器流量与 `/api` 必须使用同一公开 origin。Taco 负责前端安全响应头和同源 API 行为；代理负责域名对应的 TLS 证书、HSTS 和网络策略。

必须让 `/metrics`、`/docs` 与 `/openapi.json` 保持内网可见，不得通过公开虚拟主机路由这些路径。指标抓取与 API 文档访问仅限运维人员或私有监控网络。

## Migration 与升级

生产配置建议关闭自动迁移。已部署或需保留数据的实例，每次 schema 变更都必须以新的前向 migration 表达。升级当前 Compose 服务镜像时，先构建镜像，再执行与首次启动相同的 `migration status` 和 `migration up` 命令，最后重启 Taco：

```bash
docker compose -f compose.production.yaml build taco
docker compose -f compose.production.yaml run --rm taco --config /app/config/config.yaml migration status
docker compose -f compose.production.yaml run --rm taco --config /app/config/config.yaml migration up
docker compose -f compose.production.yaml up -d --force-recreate taco
```

若所有绑定内置 `admin` 角色的启用用户均不存在，必须在重启前使用首次启动中的 `administrator bootstrap` 命令恢复管理员。Taco 不会从启动 YAML 创建管理员；不存在启用管理员时会拒绝启动。若将 `database.auto_migrate` 明确设为 `true`，服务会在接受请求前自行应用前向 migration；生产环境仍推荐上述显式步骤。

## 配置与数据迁移

迁移服务器时，一并迁移宿主机 `/etc/taco/config.yaml`、外部 PostgreSQL、Redis 和 `taco-data` volume。新主机应继续将配置以只读方式挂载到 `/app/config/config.yaml`；保留 `data_directory: ../local-data` 时，其运行时目录仍为 `/app/local-data`。数据库、Redis、监听地址或数据目录发生变化时，直接更新 YAML 后重启；启动 YAML 不支持在线重载。各 `sys_config` 参数是否可在线生效由所属功能定义，不能将其视为 YAML 的替代。

## 构建契约

仓库发布命令会先导出前端，再启用 Rust 嵌入 feature：

```bash
just build-release
```

本地开发时，独立运行 Next.js 前端，不要启用嵌入式前端 feature。
