#!/bin/bash

# watch.sh
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch..."
    cargo install --force cargo-watch
fi


# Watch for changes in src directory and restart server (RUST_LOG=debug for console debug logs)
export RUST_LOG="${RUST_LOG:-info}"
cargo watch -x 'run --bin lensisku -- --jobs 1' -w src -w Cargo.toml