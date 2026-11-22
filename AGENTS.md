# GraphCraft - AGENTS.md

## This Repository

`graphcraft` is a standalone Pure-Rust ArtCraft desktop application and MCP server following the 3-tier workspace seam:
- `crates/graphcraft-core` (L1: engine, `petgraph` algorithms, layouts, exports, `Command::dispatch`)
- `crates/graphcraft-mcp` (L2: `clap` CLI + JSON-RPC `MCP 2024-11-05` stdio server)
- `crates/graphcraft-app` (L3: `eframe`/`egui` native desktop studio)

## Hard Rules

1. Zero `unsafe` (`#![forbid(unsafe_code)]`) and zero panics (`unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented` denied).
2. Zero relative `../sibling` dependencies: builds standalone on any fresh clone.
3. No `println!` in L1 or L2 `mcp` mode (`stdout` is strictly JSON-RPC; logs go to `stderr`).
4. Dual-licensed `MIT OR Apache-2.0`, SPDX allowlist only (no GPL/LGPL/AGPL).
5. High contrast in both Light and Dark themes; Ctrl + Mouse Wheel zoom enabled.
6. No em-dash characters anywhere.
