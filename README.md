# GraphCraft

Pure-Rust Knowledge Graph, Brainstorming & Network Analysis Studio (Labeled Property Graph) with a dual CLI + MCP 2024-11-05 command bus and local SQLite persistence.

Clean-room desktop alternative to bloated Electron note canvases and legacy Java graph tools. Zero GC pauses, zero runtime dependencies, zero network sockets required for desktop operation. GraphCraft is a **brainstorming and system-mapping studio** (not a backoffice ERP craft app).

## Feature Matrix

| Capability | Status | Details |
|---|---|---|
| Infinite GPU Canvas (`egui`/`eframe`) | **Implemented** | Pan, drag nodes, PageRank-scaled circles & multi-line `Drawer` cards, Ctrl+Wheel zoom, Light/Dark WCAG AAA themes |
| SQLite Workspace Persistence (`rusqlite`) | **Implemented** | Auto-saves nodes, edges, notes, and positions to `%LOCALAPPDATA%\graphcraft\graphcraft.db` (or `--db <PATH>`) |
| Topological Algorithms (`petgraph`) | **Implemented** | Iterative PageRank centrality, Dijkstra weighted shortest path, cycle detection, density, components |
| Physics & Deterministic Layouts | **Implemented** | Live Fruchterman-Reingold force-directed layout, circular layout, grid layout |
| Multi-Format Export | **Implemented** | JSON Graph, Graphviz DOT, Mermaid Flowchart (`flowchart LR`), Gephi GEXF 1.3 |
| Universal Command Bus (`L1/L2/L3`) | **Implemented** | Single `Command::dispatch` shared by CLI (`--json`), MCP stdio (`2024-11-05`), and egui desktop app |
| Undo / Redo Snapshot Stack | **Implemented** | 64-step state snapshot stack across node/edge edits, layouts, and resets |
| Interactive Node Inspector | **Implemented** | Real-time label, semantic kind (`Concept`, `Project`, `Person`, `Hardware`, `Document`, `Cluster`, `Drawer`), markdown notes, and incident edge management |
| In-App Help & Craft Floor | **Implemented** | 6-tab Help (`Get Started`, `Canvas & Controls`, `Algorithms & Layouts`, `CLI & MCP`, `Shortcuts & Tips`, `Technical Info`), [HELP.md](./HELP.md), `English`/`Deutsch` locale, `scripts/craft-floor.ps1` |
| Interactive Format Import (DOT/GEXF) | **Planned** | Currently imports GraphCraft JSON documents; DOT/GEXF parser planned for v0.2 |
| NSIS Current-User Installer | **Partial** | `installer/craft-installer.nsi` + `scripts/release.ps1` scaffold vendored; release build pending |

## Architecture (3-Tier Workspace Seam)

- **L1 (`graphcraft-core`)**: Pure-Rust domain model, `petgraph` algorithms, layout engines, format exporters, and `Command::dispatch` (zero UI dependencies).
- **L2 (`graphcraft-mcp`)**: CLI-first `clap` binary (`status`, `list`, `stats`, `pagerank`, `path`, `export`) and `mcp` JSON-RPC 2024-11-05 stdio server.
- **L3 (`graphcraft-app`)**: Native `eframe`/`egui` GPU desktop workbench calling `graphcraft-core` directly in-process.

## Quick Start

```powershell
# Build & launch the desktop studio (or double-click start.bat)
.\start.ps1

# Run craft floor + headless app & CLI smoke checks
.\start.ps1 -Smoke

# Run automated verification gates
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace

# Run CLI or MCP stdio server
.\start.ps1 -Cli status
.\start.ps1 -Mcp
```

## Licensing

Dual-licensed under `MIT OR Apache-2.0`. See [LICENSE-MIT](./LICENSE-MIT), [LICENSE-APACHE](./LICENSE-APACHE), [NOTICE](./NOTICE), and [ATTRIBUTION.md](./ATTRIBUTION.md).
