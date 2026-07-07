#!/usr/bin/env bash
set -euo pipefail

mode="${1:-complete}"

install_cargo_tool() {
    local command_name="$1"
    local crate_name="$2"

    if cargo --list | awk '{print $1}' | grep -qx "$command_name"; then
        return
    fi

    printf 'Required cargo subcommand `%s` is missing. Installing with: cargo install --locked %s\n' "$command_name" "$crate_name" >&2
    cargo install --locked "$crate_name"
}

install_rustup_component() {
    local component_name="$1"

    if rustup component list --installed | grep -Eq "^${component_name}(-|$)"; then
        return
    fi

    printf 'Required rustup component `%s` is missing. Installing with: rustup component add %s\n' "$component_name" "$component_name" >&2
    rustup component add "$component_name"
}

install_precommit_tools() {
    install_rustup_component rustfmt
    install_rustup_component clippy
}

install_complete_tools() {
    install_precommit_tools
    install_cargo_tool audit cargo-audit
    install_cargo_tool deny cargo-deny
    install_cargo_tool geiger cargo-geiger
    install_cargo_tool outdated cargo-outdated
    install_cargo_tool udeps cargo-udeps
    install_cargo_tool expand cargo-expand
    install_rustup_component miri
}

case "$mode" in
    precommit)
        install_precommit_tools
        ;;
    complete)
        install_complete_tools
        ;;
    expand)
        install_cargo_tool expand cargo-expand
        ;;
    *)
        printf 'Unknown tool installation mode `%s`. Expected: precommit, complete, or expand.\n' "$mode" >&2
        exit 2
        ;;
esac
