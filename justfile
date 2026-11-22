set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

default:
    @just --list

check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo test --workspace

smoke:
    cargo run -p graphcraft-app -- --smoke

run-app:
    cargo run -p graphcraft-app

run-mcp:
    cargo run -p graphcraft-mcp -- mcp
