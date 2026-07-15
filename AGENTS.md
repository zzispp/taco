# Repository Guidelines

## Project Structure & Module Organization

This is a Rust and pnpm monorepo.

- `apps/backend` is the backend composition root and runtime entry.
- `apps/frontend` is the Next.js admin frontend.
- Rust workspace members live in `apps/backend` and `crates/*`.
- Runtime YAML configuration lives in `config/`.
- SQLx migrations live in `migrations/`.
- Static assets belong under each app's `public/` directory.

Current backend business crates:

- `crates/audit`: operation and login audit bounded context
- `crates/user`: user bounded context
- `crates/rbac`: RBAC bounded context
- `crates/system`: system administration bounded context
- `crates/scheduler`: scheduled job bounded context
- `crates/captcha`: captcha capability

Current backend shared crates:

- `crates/audit_contract`: cross-context audit event and endpoint contracts
- `crates/client_info`: client address, user-agent, and IP location infrastructure
- `crates/config`: typed config loading and validation
- `crates/storage`: database and storage foundation
- `crates/types`: shared transport DTOs
- `crates/constants`, `crates/kernel`, `crates/tracing`: shared support crates

Current frontend source layout:

- `apps/frontend/src/app`
- `apps/frontend/src/pages-layer`
- `apps/frontend/src/widgets`
- `apps/frontend/src/features`
- `apps/frontend/src/entities`
- `apps/frontend/src/shared`

`apps/frontend/src/pages-layer` is the FSD pages layer. It is named `pages-layer` because Next.js reserves top-level `src/pages`.

## Hard Architecture Guardrails

Treat the current architecture as a hard constraint, not a suggestion.

- Do not introduce a new parallel architecture to avoid touching the right module.
- Do not place code in a convenient layer if it belongs to a different bounded context or FSD layer.
- Do not restore deleted template-era structures for speed.
- Before editing, identify the owning backend crate or frontend FSD layer first. Then place the change there.

If a requested change does not fit the current boundaries, refactor the boundary deliberately. Do not bypass it.

## Backend Architecture Enforcement

The backend must remain DDD + Clean Architecture.

- `apps/backend` is composition root only: bootstrap, wiring, router assembly, startup, migration commands.
- Business logic belongs in the owning bounded context crate: `crates/audit`, `crates/user`, `crates/rbac`, `crates/system`, `crates/scheduler`, or `crates/captcha`.
- Inside a bounded context, keep responsibilities separated across `domain`, `application`, `infra`, and `api`.
- Generic crates such as `audit_contract`, `client_info`, `config`, `storage`, `types`, `constants`, `kernel`, and `tracing` must not absorb bounded-context business rules.
- Do not move domain decisions into transport DTOs, config loaders, persistence helpers, or HTTP entrypoints.

Backend placement rule:

1. Identify the bounded context.
2. Identify whether the change is `domain`, `application`, `infra`, or `api`.
3. Implement it there.
4. Wire it from `apps/backend` only after the owning crate change exists.

## Constants, Runtime Parameters, And Configuration Boundaries

Constants and runtime parameters must have a single clear owner. Do not duplicate the same semantic value across crates, modules, `config.yaml`, and `sys_config`.

Constant ownership rules:

- Cross-context stable constants belong in `crates/constants`, split by responsibility.
- `crates/constants` must not absorb bounded-context business rules. It may expose stable keys, shared names, and cross-context primitives only.
- Module-private constants stay in the owning bounded context or implementation module.
- Implementation-specific constants stay in the owning implementation module unless they are truly shared across contexts.
- Do not introduce magic numbers or magic strings. Extract named constants at the nearest correct ownership boundary.

Runtime parameter rules:

- Parameters that may be changed during operations without a code release belong in `sys_config`.
- Startup infrastructure configuration belongs in `config.yaml` or environment variables.
- A semantic value must not have two active runtime sources. Do not read the same behavior from both `config.yaml` and `sys_config`.
- Before changing any constant, runtime parameter, business threshold, limit, or policy value, locate all read sites and state the parameter's semantic meaning in one line.

Parsing and validation ownership:

- `sys_config` key constants belong in `crates/constants/src/system_config.rs`.
- JSON structure, parsing, and validation belong to the owning bounded context or feature crate, not `crates/types`.
- `apps/backend` is only the composition root: it may inject config providers and adapters, but must not own business parsing or validation logic.
- Frontend public config types and parsing belong under `apps/frontend/src/entities/system`; shared UI must receive values through props or app-level providers and must not import entity config directly.

Failure and migration rules:

- Missing or invalid required `sys_config` values must fail explicitly. Do not add silent fallback, defensive defaulting, mock success paths, or swallowed parse errors.
- Migrations or seed data must provide valid defaults so normal deployments have required parameters.
- Do not edit already-applied migration files for compatibility work. Add a new migration for data conversion, seed changes, or key consolidation.
- When changing `sys_config` seeds, update migration tests that assert key existence, public readability, counts, and default JSON shape.
- Frontend may keep build-time UI defaults only for initial display before runtime parameters load; it must not use defaults to hide invalid runtime parameter data.

## Frontend Architecture Enforcement

The frontend must stay inside the current FSD structure under `apps/frontend/src`:

- `app`: Next.js route entry, providers, guards, app initialization
- `pages-layer`: page composition
- `widgets`: page blocks and business/layout containers
- `features`: user actions and use-case behavior
- `entities`: business entities, entity queries/types, entity UI
- `shared`: pure shared infrastructure, generic UI, config, routes, theme, assets, utils

Mandatory dependency direction:

- `app -> pages-layer/widgets/features/entities/shared`
- `pages-layer -> widgets/features/entities/shared`
- `widgets -> features/entities/shared`
- `features -> entities/shared`
- `entities -> shared`
- `shared -> shared`

Mandatory frontend rules:

- `src/app/**/page.tsx` must stay thin. It should assemble metadata, guards, and render `src/pages-layer/**`.
- Do not put page composition back into `app` or `widgets` when it belongs in `pages-layer`.
- Do not put business logic into `shared`.
- Do not put entity-specific behavior into generic UI folders.
- New slices must expose a public `index.ts`; prefer imports through the slice public API instead of cross-slice deep imports.
- Keep the existing absolute import style `src/...`.

Forbidden drift under `apps/frontend/src`:

- Do not reintroduce top-level `auth`, `layouts`, `sections`, `actions`, `types`, or `components` as business structure.
- Do not create new top-level parallel folders that bypass `app/pages-layer/widgets/features/entities/shared`.
- Do not move `routes`, `i18n`, `theme`, `assets`, `lib`, or shared config concerns out of `shared`, and do not recreate legacy top-level names such as `locales` or `global-config`.

## Internationalization Guidelines

Internationalization is a cross-cutting transport/UI concern, but business error ownership stays inside the owning bounded context.

Supported backend wire locales:

- `zh-CN`: Simplified Chinese, default fallback
- `en`: English
- `zh-TW`: Traditional Chinese

Supported frontend language codes:

- `cn`: maps to backend `zh-CN`
- `en`: maps to backend `en`
- `tw`: maps to backend `zh-TW`

### Backend Error I18n

- Backend API error responses must localize both `message` and `details`.
- Use `rust-i18n` with YAML catalogs under `crates/types/locales/`.
- Keep `rust_i18n::i18n!("locales", fallback = "zh-CN")` in `crates/types`; do not initialize locale catalogs from `apps/backend`.
- Do not call global `rust_i18n::set_locale` for request handling. Resolve the locale per request and pass/use it explicitly when building the response.
- HTTP language parsing, locale normalization, request locale middleware, and `ApiErrorResponse` construction helpers belong in `crates/types/src/http`.
- Shared localizable error payload primitives belong in `crates/kernel`; they must not depend on HTTP, Axum, or a bounded context.
- Business error keys belong to the owning bounded context. Do not move user/RBAC/system/captcha business rules into `types`, `kernel`, `config`, `storage`, or `apps/backend`.
- API error mappers belong in each bounded context `api` layer, for example `crates/user/src/api/error.rs`.
- `apps/backend` may only wire shared middleware and routers after the owning crate provides the behavior.
- Do not store user-facing English or Chinese sentences in `AppError`, `RbacError`, `SystemError`, `CaptchaError`, or equivalent application errors. Store stable localization keys plus explicit parameters.
- Parameterized details must use named parameters, for example `errors.user.import_account_exists` with `{ username }`.
- Infrastructure errors must not expose raw database, Redis, JWT, IO, or provider error text in API responses. Log/trace the raw error internally, and return stable localized `message/details`.
- JSON/content-type/body extraction errors must use the same localized API error shape as business errors.

Backend `Accept-Language` normalization rules:

- `zh-CN`, `zh-Hans`, `zh`, `cn` -> `zh-CN`
- `zh-TW`, `zh-Hant`, `zh-HK`, `zh-MO`, `tw` -> `zh-TW`
- `en`, `en-US`, `en-GB`, other `en-*` -> `en`
- Unknown or missing language -> `zh-CN`

When adding or changing backend error keys:

1. Add/update the key in all three catalogs: `zh-CN.yml`, `en.yml`, and `zh-TW.yml`.
2. Update the owning context error construction or mapper.
3. Add or update tests that assert stable `code/status` and localized `message/details`.

### Frontend I18n

- Frontend i18n infrastructure belongs in `apps/frontend/src/shared/i18n`.
- Frontend HTTP language/error handling belongs in `apps/frontend/src/shared/api`.
- Do not create top-level `locales`, `i18n`, `global-config`, or other parallel language folders outside the FSD `shared` layer.
- Language resources must stay under `apps/frontend/src/shared/i18n/langs/{cn,en,tw}/`.
- Each supported language must provide the same namespace files: `common.json`, `messages.json`, `admin.json`, and `navbar.json`.
- Add UI copy to the namespace and slice that already owns the feature. Do not hardcode visible user-facing text in components when the surrounding feature uses i18n.
- `tw` resources should be Traditional Chinese equivalents of the existing Simplified Chinese copy unless the feature explicitly requires different wording.
- Frontend language normalization must map `zh-TW`, `zh-HK`, `zh-Hant`, and `tw` to `tw`; `zh-CN`, `zh-Hans`, `zh`, and `cn` to `cn`; and `en-*` to `en`.
- If `localStorage.i18nextLng` exists, the API client must send `Accept-Language` mapped as `cn -> zh-CN`, `en -> en`, `tw -> zh-TW`.
- If `localStorage.i18nextLng` does not exist, the API client must not override the browser's default `Accept-Language`.
- API error normalization must prefer backend localized details: `data.details ?? data.message ?? axios message`.
- Normalized frontend errors must preserve `status`, `code`, and `details` for auth guards, forms, and toast logic.
- Auth/session rejection must not compare localized or English error text. Use `status === 401` or `code === 'unauthorized'`.

## Build, Test, and Development Commands

- `pnpm install`: install JavaScript workspace dependencies.
- `pnpm dev:frontend`: run the frontend on port `8082`.
- `pnpm build:frontend`: build the Next.js frontend.
- `pnpm lint:frontend`: run ESLint for the frontend.
- `just check`: run `cargo check` for the Rust workspace.
- `just build`: build all Rust crates.
- `just test`: run Rust tests with the repository's 60-second timeout wrapper.
- `just quality-precommit`: run the mandatory Rust pre-commit gate.
- `just quality-complete`: run the mandatory Rust completion gate.
- `just install-git-hooks`: install the repository native Git pre-commit hook.

## Rust Quality Gates

Treat these gates as the highest-priority Rust quality rules. Do not skip checks, mock success, ignore errors, delete checks, add silent fallbacks, or downgrade failures to warnings just to make a run pass. Missing local quality tools must be installed by the gate before execution; installation failure must fail visibly.

Before every commit, run `just quality-precommit`. This gate must install missing `rustfmt` and `clippy` components first, then execute `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check --workspace --all-targets`, and `just test`. The installed Git hook must call the same gate and block the commit on failure.

Before marking any Rust task complete, run `just quality-complete`. This gate must run the pre-commit gate first, then install any missing required tools (`cargo-audit` and `cargo-deny`) and execute `cargo audit` and `cargo deny check`.

## Coding Style & Naming Conventions

TypeScript uses Prettier with 2-space indentation, semicolons, single quotes, `printWidth: 100`, and trailing commas where valid in ES5.

Frontend import rules already encode the FSD layers in `apps/frontend/eslint.config.mjs`. Keep imports compatible with that config.

Rust uses edition 2024 and `rustfmt.toml` with `max_width = 160`; keep crate names lowercase and module names snake_case.

## Testing Guidelines

No JavaScript test runner is configured yet; rely on linting and Next.js builds for frontend validation.

Rust tests should be colocated with the crate they validate using normal `#[cfg(test)]` modules or integration tests when a public API boundary is required.

Run `just test` before submitting Rust changes, and keep tests deterministic and under the configured timeout.

## Commit & Pull Request Guidelines

The current history uses Conventional Commit style, for example `chore: init monorepo`.

Continue with concise messages such as `feat: add user profile route` or `fix: validate config path`.

Pull requests should describe the change, list validation commands run, link related issues, and include screenshots or screen recordings for visible frontend changes.

## Security & Configuration Tips

Do not commit secrets or local credentials.

Keep runtime configuration in `config/` or environment variables, and document any new required setting in the relevant app or crate README.
