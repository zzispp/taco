# Backend Agent Instructions

## Backend Scope

- `apps/backend` is the Axum binary and composition root.
- Keep runtime wiring here: load `configuration::Settings`, connect storage, build repositories, services, token settings, and routes.
- Keep business rules in the shared crates under `crates/*`; do not move domain behavior into `main.rs`.
- When backend work touches shared crates, follow the root `AGENTS.md` plus any crate-local instructions.

## Architecture Rules

- Inject concrete infrastructure at the composition root; application and domain logic should depend on traits or explicit parameters.
- Do not hardcode secrets, credentials, ports, or database URLs in source. Use configuration or environment-backed settings.
- Let startup failures fail loudly with clear messages. Do not add silent fallbacks for missing config, database connection errors, or schema setup failures.
- In API, application, and storage layers, propagate typed errors instead of using `unwrap`, `expect`, or stringly-typed error checks.
- Never silently discard fallible results. Use `?`, explicit `match`, or visible logging when an operation is intentionally best-effort.
- Async failures should propagate to the API boundary so callers receive meaningful error responses.
- Prefer private modules with explicit public crate exports.
- Newly added traits must have doc comments that explain their role and implementation expectations.
- Keep modules focused and small. Extract helpers when validation, mapping, or routing code starts mixing responsibilities.

## Logging Rules

- Backend code must use the internal `taco_tracing` crate for logging. Do not call `println!`, `eprintln!`, `dbg!`, `log::*`, `tracing::*`, or `tracing_subscriber::*` directly from backend modules.
- Initialize tracing exactly once in the backend composition root after schema readiness and loading `sys.observability.tracingConfig` from PostgreSQL.
- Tracing runtime behavior must be driven solely by the persisted observability parameter. Invalid logging configuration must fail startup or parameter updates explicitly; do not fall back to environment variables or a hardcoded default.
- Add operationally mutable logging controls to the observability runtime parameter schema and seed migration, not the immutable encrypted installation profile.

## Rust Style Rules

- Prioritize correctness and clarity over micro-optimizations unless performance is part of the requested behavior.
- Inline `format!` arguments when possible, such as `format!("{name}")` instead of `format!("{}", name)`.
- Collapse nested `if` statements when the conditions belong to the same decision.
- Use method references instead of closures when it stays readable.
- Prefer exhaustive `match` arms over wildcard arms when the enum variants are known.
- Avoid unnecessary `.clone()` calls; prefer borrowing when lifetimes and ownership stay clear.
- Be careful with indexing and other panic-prone operations in production code.
- Use full words for variable names instead of unclear abbreviations.
- Write comments only to explain non-obvious intent or constraints, not to restate what the code does.
- Do not add dependencies unless the task genuinely needs them.
- Avoid ambiguous boolean or `Option` positional arguments in public APIs. Prefer enums, newtypes, named methods, or an options struct when that makes call sites self-documenting.
- Do not create tiny helper methods that are referenced only once unless they clarify a real boundary or are needed to satisfy size/complexity limits.
- When changing backend API or configuration behavior, update the relevant docs, examples, or config files in the same change.

## Tokio Rules

- Do not create nested Tokio runtimes or call `Runtime::block_on()` from async code.
- Do not hide runtime creation inside helpers, libraries, repositories, or services.
- Do not hold mutex or database guards across `.await`.
- Use `tokio::time::sleep` instead of `std::thread::sleep` in async code.
- Use `tokio::task::spawn_blocking` or a dedicated thread for blocking CPU or IO work.
- Prefer `.await`, `tokio::spawn`, and channels for async coordination, while keeping task errors observable.

## Testing Policy

- Use Test Driven Development (TDD) for backend changes.
- Write or update a failing unit test before changing production behavior.
- Keep tests close to the module they verify.
- Prefer deterministic unit tests over tests that require external services.
- Assert stable invariants in integration-style tests; avoid tolerance-based assertions against live network, database, or recomputed external values.
- For exact numeric or transformation behavior, cover the pure calculation with deterministic unit tests.
- Prefer asserting equality of complete values over checking fields one by one when the whole value is meaningful.
- Prefer one cohesive test function with related assertions over many single-assertion tests for the same behavior.
- Put reusable test doubles in crate-local test support, such as `test_support.rs` or a `testkit` module. Inline one-off fixtures are fine.
- Avoid mutating process environment in tests; pass environment-derived flags or dependencies from the caller instead.
- Do not add fallback paths or fake success behavior to production code to make tests pass.

## Task Completion

- Review the change for simplification: remove dead code, reduce duplication, and keep the diff focused on the requested behavior.
- Avoid unrelated refactors, drive-by cleanups, and formatting-only churn.
- Run `just test` before finishing backend work so the repository 60-second timeout wrapper is used.
- Run `cargo clippy -p backend --all-targets -- -D warnings` for changes in this app; use the relevant crate name when backend behavior changes in `crates/*`.
- If Rust dependencies change, include the generated `Cargo.lock` update and run `cargo check`.
- Use `cargo check` for fast compile validation while iterating.
- Run `just format` after Rust edits.
