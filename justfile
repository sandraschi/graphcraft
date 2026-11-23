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

# Build and stage dist/ (plain exes, installer, launcher zip, SHA256SUMS). Nothing is published.
release-dry:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/release.ps1

# Same, then tag, push the tag and upload to GitHub (clean, pushed trunk required).
release:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/release.ps1 -Publish
