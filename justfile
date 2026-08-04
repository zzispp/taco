test_timeout_seconds := "60"

list:
    just --list

build:
    cargo build

build-release:
    pnpm --filter frontend build:embedded
    cargo build --locked --release -p backend --bin taco --features embedded-frontend

check:
    cargo check

format:
    cargo fmt -q --all

lint:
    cargo clippy --workspace --all-targets -- -D warnings

_test-partition *ARGS:
    perl -e 'my $timeout = shift; my $pid = fork(); die "fork failed: $!" unless defined $pid; if ($pid == 0) { exec @ARGV or die "exec failed: $!"; } $SIG{ALRM} = sub { kill "TERM", $pid; exit 124; }; alarm $timeout; waitpid($pid, 0); exit($? >> 8);' {{test_timeout_seconds}} {{ARGS}}

_prepare-scheduler-trybuild FLAGS:
    #!/usr/bin/env bash
    set -euo pipefail
    task_host="$(rustc -vV | sed -n 's/^host: //p')"
    task_target_dir="${CARGO_TARGET_DIR:-target}/tests/trybuild"
    CARGO_TARGET_DIR="$task_target_dir" CARGO_INCREMENTAL=0 RUSTFLAGS="{{FLAGS}}" cargo test -p scheduler --no-run --target "$task_host"

test-build:
    cargo test -p backend --no-run
    cargo test -p file --no-run
    cargo test --workspace --exclude backend --exclude file --exclude scheduler --no-run
    cargo test -p scheduler --no-run
    just _prepare-scheduler-trybuild '--cfg trybuild --verbose -A dead_code'
    just _prepare-scheduler-trybuild '--cfg trybuild --verbose -A dead_code --diagnostic-width=140'

test: test-build
    just _test-partition cargo test -p backend
    just _test-partition cargo test -p file
    just _test-partition cargo test --workspace --exclude backend --exclude file --exclude scheduler
    just _test-partition cargo test -p scheduler --lib --test scheduler_core
    just _test-partition cargo test -p scheduler --test scheduler_macros_ui scheduled_task_macro_pass_contracts_compile_as_declared -- --exact --test-threads=1
    just _test-partition cargo test -p scheduler --test scheduler_macros_ui scheduled_task_macro_fail_contracts_reject_invalid_declarations -- --exact --test-threads=1

quality-precommit:
    cargo run -p compliance
    node scripts/quality/check-frontend-compliance.mjs
    scripts/quality/ensure-rust-quality-tools.sh precommit
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo check --workspace --all-targets
    just test

quality-complete: quality-precommit
    scripts/quality/ensure-rust-quality-tools.sh complete
    cargo audit
    cargo deny check

install-git-hooks:
    mkdir -p .git/hooks
    cp scripts/git-hooks/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit

run-backend:
    cargo run -p backend --bin taco -- --config config/config.yaml

backend-migration ARGS:
    cargo run -p backend --bin taco -- --config config/config.yaml migration {{ARGS}}

services-up:
    COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose up -d

services-down:
    COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose down
