# GraphCraft

Pure-Rust Knowledge Graph, Network Analysis & Interactive Node Canvas with a dual CLI + MCP 2024-11-05 command bus.

Clean-room desktop alternative to bloated Electron note canvases and legacy Java graph tools. Zero GC pauses, zero runtime dependencies, zero network sockets required for desktop operation.

## Feature Matrix

| Capability | Status | Details |
|---|---|---|
| Infinite GPU Canvas (`egui`/`eframe`) | **Implemented** | Pan, drag nodes, PageRank-scaled node sizes, Ctrl+Wheel zoom, Light/Dark WCAG AAA themes |
| Topological Algorithms (`petgraph`) | **Implemented** | Iterative PageRank centrality, Dijkstra weighted shortest path, cycle detection, density, components |
| Physics & Deterministic Layouts | **Implemented** | Live Fruchterman-Reingold force-directed layout, circular layout, grid layout |
| Multi-Format Export | **Implemented** | JSON Graph, Graphviz DOT, Mermaid Flowchart (`flowchart LR`), Gephi GEXF 1.3 |
| Universal Command Bus (`L1/L2/L3`) | **Implemented** | Single `Command::dispatch` shared by CLI (`--json`), MCP stdio (`2024-11-05`), and egui desktop app |
| Undo / Redo Snapshot Stack | **Implemented** | 64-step state snapshot stack across node/edge edits, layouts, and resets |
| Interactive Node Inspector | **Implemented** | Real-time label, semantic kind, markdown notes, and incident edge management |
| Interactive Format Import (DOT/GEXF) | **Planned** | Currently imports GraphCraft JSON documents; DOT/GEXF parser planned for v0.2 |
| NSIS Current-User Installer | **Planned** | Release binary packaging via `installer.nsi` |

## Architecture (3-Tier Workspace Seam)

- **L1 (`graphcraft-core`)**: Pure-Rust domain model, `petgraph` algorithms, layout engines, format exporters, and `Command::dispatch` (zero UI dependencies).
- **L2 (`graphcraft-mcp`)**: CLI-first `clap` binary (`status`, `list`, `stats`, `pagerank`, `path`, `export`) and `mcp` JSON-RPC 2024-11-05 stdio server.
- **L3 (`graphcraft-app`)**: Native `eframe`/`egui` GPU desktop workbench calling `graphcraft-core` directly in-process.

## Quick Start

```powershell
# Run automated verification gates
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace

# Launch the native desktop studio
cargo run -p graphcraft-app

# Run headless smoke test
cargo run -p graphcraft-app -- --smoke

# Run MCP stdio server
cargo run -p graphcraft-mcp -- mcp
```

## Licensing

Dual-licensed under `MIT OR Apache-2.0`. See [LICENSE-MIT](./LICENSE-MIT), [LICENSE-APACHE](./LICENSE-APACHE), [NOTICE](./NOTICE), and [ATTRIBUTION.md](./ATTRIBUTION.md).
