test_timeout_seconds := "60"

list:
    just --list

build:
    cargo build

check:
    cargo check

format:
    cargo fmt -q --all

lint:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    perl -e 'my $timeout = shift; my $pid = fork(); die "fork failed: $!" unless defined $pid; if ($pid == 0) { exec @ARGV or die "exec failed: $!"; } $SIG{ALRM} = sub { kill "TERM", $pid; exit 124; }; alarm $timeout; waitpid($pid, 0); exit($? >> 8);' {{test_timeout_seconds}} cargo test

run-backend:
    cargo run -p backend

run-backend-config CONFIG:
    cargo run -p backend -- --config {{CONFIG}}

backend-migration ARGS:
    cargo run -p backend -- migration {{ARGS}}

backend-migration-config CONFIG ARGS:
    cargo run -p backend -- --config {{CONFIG}} migration {{ARGS}}
