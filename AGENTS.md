# Repository Guidelines

## Project Structure & Module Organization

This is a Rust and pnpm monorepo.

- `apps/backend` is the backend composition root and runtime entry.
- `apps/frontend` is the Next.js admin frontend.
- Rust workspace members live in `apps/backend` and `crates/*`.
- Runtime YAML configuration lives in `config/`.
- SQLx migrations live in `migrations/`.
- Static assets belong under each app's `public/` directory.

Current backend crates:

- `crates/user`: user bounded context
- `crates/rbac`: RBAC bounded context
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
- Business logic belongs in the owning bounded context crate, currently `crates/user` or `crates/rbac`.
- Inside a bounded context, keep responsibilities separated across `domain`, `application`, `infra`, and `api`.
- Generic crates such as `config`, `storage`, `types`, `constants`, `kernel`, and `tracing` must not absorb user or RBAC business rules.
- Do not move domain decisions into transport DTOs, config loaders, persistence helpers, or HTTP entrypoints.

Backend placement rule:

1. Identify the bounded context.
2. Identify whether the change is `domain`, `application`, `infra`, or `api`.
3. Implement it there.
4. Wire it from `apps/backend` only after the owning crate change exists.

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

## Build, Test, and Development Commands

- `pnpm install`: install JavaScript workspace dependencies.
- `pnpm dev:frontend`: run the frontend on port `8082`.
- `pnpm build:frontend`: build the Next.js frontend.
- `pnpm lint:frontend`: run ESLint for the frontend.
- `just check`: run `cargo check` for the Rust workspace.
- `just build`: build all Rust crates.
- `just test`: run Rust tests with the repository's 60-second timeout wrapper.

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
