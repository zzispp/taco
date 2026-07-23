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

test:
    just _test-partition cargo test -p backend
    just _test-partition cargo test -p file
    just _test-partition cargo test --workspace --exclude backend --exclude file --exclude scheduler
    just _test-partition cargo test -p scheduler

quality-precommit:
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
