# GraphCraft

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](./LICENSE-MIT)
[![Rust: Safe](https://img.shields.io/badge/rust-forbid%20unsafe__code-orange?logo=rust)](./Cargo.toml)
[![GUI: egui 0.37](https://img.shields.io/badge/GUI-egui%200.37-00a8e8)](./crates/graphcraft-app/Cargo.toml)
[![Graph: petgraph 0.6](https://img.shields.io/badge/graph-petgraph%200.6-2ea44f)](./crates/graphcraft-core/Cargo.toml)
[![Storage: SQLite](https://img.shields.io/badge/storage-SQLite%20bundled-003b57?logo=sqlite)](./crates/graphcraft-core/src/db.rs)
[![MCP: 2026-07-28 stdio](https://img.shields.io/badge/MCP-2026--07--28%20stdio-7c3aed)](./crates/graphcraft-mcp/src/main.rs)
[![ArtCraft: 3-Tier](https://img.shields.io/badge/ArtCraft-L1%20%7C%20L2%20%7C%20L3-16a34a)](./Cargo.toml)

Pure-Rust Knowledge Graph, Brainstorming & Network Analysis Studio (**Labeled Property Graph**) with a unified CLI + MCP `2026-07-28` command bus and local SQLite persistence.

Clean-room desktop alternative to bloated Electron note canvases and legacy Java graph tools. Zero GC pauses, zero runtime dependencies, zero network sockets required for desktop operation.

> **Scope note**: GraphCraft is a **brainstorming and system-mapping studio** -- **not** a backoffice/ERP craft app (no invoices, payroll, or inventory).

---

## Why GraphCraft?

- **Local-First & Native**: Compiles to a single fast native Rust binary (`#![forbid(unsafe_code)]`, clippy `unwrap`/`expect`/`panic`/`todo` denied) with GPU-accelerated rendering via `eframe`/`egui` 0.37.
- **Visual Thinking + Graph Theory**: Sketch ideas, people, repos, hardware, documents, clusters, and multi-line `Drawer` cards on an infinite pan/zoom canvas, then run **PageRank centrality**, **Dijkstra weighted shortest path**, **cycle detection**, and **Fruchterman-Reingold force physics** directly in-process.
- **One Engine, Three Interfaces**: The desktop workbench (`graphcraft-app`), scriptable CLI (`graphcraft-mcp --json`), and AI agent server (`graphcraft-mcp mcp`) all execute through the exact same `Command::dispatch` core and share the same SQLite database.
- **Instant First-Paint Workspace**: First launch seeds the database with the **Vienna Open Robotics & Craft** demo graph so you can pan, zoom, run physics, and trace shortest paths immediately.

---

## Feature Matrix

| Capability | Status | Details |
|---|---|---|
| Infinite GPU Canvas (`egui`/`eframe`) | **Implemented** | Pan, drag nodes, PageRank-scaled circles & multi-line `Drawer` cards, `Ctrl + Wheel` zoom, Light/Dark WCAG AAA themes |
| SQLite Workspace Persistence (`rusqlite`) | **Implemented** | Auto-saves nodes, edges, notes, and positions to `%LOCALAPPDATA%\graphcraft\graphcraft.db` (or `--db <PATH>`) |
| Topological Algorithms (`petgraph`) | **Implemented** | Iterative PageRank centrality, Dijkstra weighted shortest path, cycle detection, density, connected components |
| Physics & Deterministic Layouts | **Implemented** | Live Fruchterman-Reingold force-directed layout, circular layout, grid layout |
| Multi-Format Export | **Implemented** | JSON Graph, Graphviz DOT, Mermaid Flowchart (`flowchart LR`), Gephi GEXF 1.3 |
| Universal Command Bus (`L1/L2/L3`) | **Implemented** | Single `Command::dispatch` shared by CLI (`--json`), MCP stdio (`2026-07-28`), and egui desktop app |
| Undo / Redo Snapshot Stack | **Implemented** | 64-step state snapshot stack across node/edge edits, layouts, and resets |
| Interactive Node Inspector | **Implemented** | Real-time label, semantic kind (`Concept`, `Project`, `Person`, `Hardware`, `Document`, `Cluster`, `Drawer`), markdown notes, tags, metadata, and incident edge management |
| In-App Help & Craft Floor | **Implemented** | 6-tab Help (`Get Started`, `Canvas & Controls`, `Algorithms & Layouts`, `CLI & MCP`, `Shortcuts & Tips`, `Technical Info`), [HELP.md](./HELP.md), `English`/`Deutsch` locale, `scripts/craft-floor.ps1` |
| Interactive Format Import (DOT/GEXF) | **Planned** | Currently imports GraphCraft JSON documents; DOT/GEXF parser planned for v0.2 |
| NSIS Current-User Installer | **Partial** | `installer/craft-installer.nsi` + `scripts/release.ps1` scaffold vendored; release build pending |

---

## Graph Model & Exporters

GraphCraft models a **Labeled Property Graph (LPG)** -- a directed, weighted, heterogeneous multigraph backed by `petgraph::graph::DiGraph<String, f64>`:

| Node Kind | Visual Representation | Typical Use |
|---|---|---|
| `Concept` | PageRank-scaled circle | Ideas, principles, architecture topics |
| `Project` | PageRank-scaled circle | Repositories, initiatives, products |
| `Person` | PageRank-scaled circle | Collaborators, authors, stakeholders |
| `Hardware` | PageRank-scaled circle | Robots, servers, boards, instruments |
| `Document` | PageRank-scaled circle | Specs, papers, notes, guides |
| `Cluster` | PageRank-scaled circle | Thematic groupings and domains |
| `Drawer` | Multi-line card node | Grouped sub-lists (e.g. `Backoffice apps`) |

Every node stores `id`, `label`, `kind`, `notes`, `tags`, `metadata`, `weight`, and canvas `pos (x, y)`. Every edge stores `id`, `source`, `target`, `relation`, and `weight`.

**Export Formats**: `json` (full GraphCraft document), `dot` (Graphviz), `mermaid` (`flowchart LR`), and `gexf` (Gephi 1.3).

---

## Architecture (3-Tier Workspace Seam)

```text
graphcraft-app (L3: eframe/egui desktop studio)  --+
                                                   +--> graphcraft-core (L1: Command::dispatch, petgraph, layouts, SQLite)
graphcraft-mcp (L2: clap CLI + MCP stdio server) --+
```

- **L1 (`graphcraft-core`)**: Pure-Rust domain model, `petgraph` algorithms, layout engines, format exporters, bundled `rusqlite` persistence, and `Command::dispatch` (zero UI dependencies).
- **L2 (`graphcraft-mcp`)**: CLI-first `clap` binary (`status`, `list`, `stats`, `pagerank`, `path`, `export`) with `--json`, `--db <PATH>`, and `--file <PATH>` flags, plus a pure JSON-RPC `MCP 2026-07-28` stdio server.
- **L3 (`graphcraft-app`)**: Native `eframe`/`egui` GPU desktop workbench calling `graphcraft-core` directly in-process, with `Ctrl+K` Command Palette, `F1` Help, `Ctrl+MouseWheel` zoom, and an optional local `Chat` tab (makes no network calls unless a model name is explicitly entered).

---

## CLI & MCP Reference

### CLI Subcommands (`graphcraft-mcp`)

| Command | Description |
|---|---|
| `status` | Workspace title, node count, and edge count |
| `list [--query Q] [--kind K]` | List or filter nodes by search text or semantic kind |
| `stats` | Node/edge counts, density, cycle detection, components, and top PageRank nodes |
| `pagerank [--damping 0.85]` | Compute iterative PageRank centrality scores |
| `path <SOURCE> <TARGET> [--directed]` | Find shortest weighted path between two node IDs |
| `export [json\|dot\|mermaid\|gexf]` | Export the graph (defaults to `mermaid`) |
| `mcp` | Run the JSON-RPC `MCP 2026-07-28` stdio server |

### MCP Tools (`7` tools)

| Tool | Purpose |
|---|---|
| `graph_status` | Return workspace title, node count, and edge count |
| `graph_list_nodes` | Search or filter nodes by `query` and/or `kind` |
| `graph_add_node` | Add a node with `label`, optional `id`, `kind`, `notes`, `tags`, `x`, `y` |
| `graph_add_edge` | Connect `source` -> `target` with a labeled `relation` and optional `weight` |
| `graph_stats` | Compute topology stats, cycles, density, and top PageRank hubs |
| `graph_shortest_path` | Run Dijkstra weighted shortest path between `source` and `target` |
| `graph_export` | Export workspace to `json`, `dot`, `mermaid`, or `gexf` |

---

## Quick Start

```powershell
# Build & launch the desktop studio (or double-click start.bat)
.\start.ps1

# Run craft floor + headless app & CLI smoke checks
.\start.ps1 -Smoke

# Run automated verification gates
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace

# Run CLI commands (supports --json, --db <PATH>, --file <PATH>)
.\start.ps1 -Cli status
.\start.ps1 -Cli stats
.\start.ps1 -Cli pagerank
.\start.ps1 -Cli path boomy storytold
.\start.ps1 -Cli export mermaid

# Run MCP 2026-07-28 stdio server
.\start.ps1 -Mcp
```

### Keyboard & Canvas Controls

- **Pan**: Drag canvas background
- **Move node**: Drag any node
- **Zoom**: `Ctrl + Mouse Wheel`
- **Command Palette**: `Ctrl + K`
- **In-App Help**: `F1`
- **Themes & Locale**: `View` -> `Dark` / `Light`, `View` -> `English` / `Deutsch`

---

## Documentation

- [HELP.md](./HELP.md) -- Full user guide, database paths, workflows, and tips
- [CHANGELOG.md](./CHANGELOG.md) -- Version history
- [ATTRIBUTION.md](./ATTRIBUTION.md) & [NOTICE](./NOTICE) -- Third-party credits and notices

## Lineage & Licensing

GraphCraft follows the Storytold ArtCraft 3-tier craft-app pattern ([storytold/artcraft](https://github.com/storytold/artcraft)): headless-first native Rust binaries sharing one L1 `Command::dispatch` bus across GUI, CLI, and MCP stdio.

Dual-licensed under `MIT OR Apache-2.0`. See [LICENSE-MIT](./LICENSE-MIT), [LICENSE-APACHE](./LICENSE-APACHE), [NOTICE](./NOTICE), and [ATTRIBUTION.md](./ATTRIBUTION.md).
