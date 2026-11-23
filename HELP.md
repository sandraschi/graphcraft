# GraphCraft Help

## Get Started

### What is this?

GraphCraft is a **knowledge-graph, brainstorming, and system-mapping studio** -- **not** a backoffice/ERP craft app (no invoices, payroll, or inventory). Use it to map ideas (`Concept`), collaborators (`Person`), repos (`Project`), devices (`Hardware`), notes (`Document`), and groups (`Cluster`) on an infinite canvas.

### How does it tie in?

- **Brainstorming to Structure**: Sketch nodes and typed relations visually, then untangle clusters with force-directed, circular, or grid layouts.
- **Network Analysis & Export**: Rank hubs with PageRank, trace connections with Dijkstra shortest-path queries, and export live diagrams to Mermaid, Graphviz DOT, Gephi GEXF 1.3, or JSON.
- **Agent & CLI Integration**: The desktop app, CLI, and MCP stdio server (`graphcraft-mcp mcp`) share the same `Command::dispatch` core engine and SQLite database.

### Database Persistence

Every node (`Person`, `Concept`, `Project`, `Hardware`, `Document`, `Cluster`), edge, note, and dragged position is persisted automatically in SQLite (`%LOCALAPPDATA%\graphcraft\graphcraft.db` on Windows, or `--db <PATH>`). On first launch, the database is seeded with the Vienna Open Robotics & Craft demo graph.

Launch from the repository root with `.\start.ps1` (or double-click `start.bat`), or from `mcp-central-docs\starts\graphcraft-start.bat`. Use `View` -> `Dark` / `Light` for themes and `View` -> `English` / `Deutsch` for UI language.

## Functions

- **L1 (`graphcraft-core`)**: In-memory `petgraph` model, iterative PageRank centrality, Dijkstra weighted shortest path, cycle detection, component count, density, Fruchterman-Reingold force-directed / circular / grid layouts, 64-step undo/redo stack, and JSON / Graphviz DOT / Mermaid / GEXF 1.3 exporters. All mutations go through `Command::dispatch`.
- **L2 (`graphcraft-mcp`)**: CLI-first binary (`--json`) and pure JSON-RPC `MCP 2024-11-05` stdio server sharing `Command::dispatch`.
- **L3 (`graphcraft-app`)**: Native `eframe`/`egui` GPU desktop workbench calling `graphcraft-core` directly in-process.
- **Chat**: Local assistant panel in the `Chat` tab. Enter a model name before sending; an empty model displays `no model selected` and makes no network calls.

## Usage

- **Desktop App**: `.\start.ps1` (or `cargo run -p graphcraft-app`)
- **Smoke Check**: `.\start.ps1 -Smoke` (or `cargo run -p graphcraft-app -- --smoke`)
- **App CLI Help**: `cargo run -p graphcraft-app -- --help`
- **CLI Commands**: `.\start.ps1 -Cli status`, `.\start.ps1 -Cli stats`, `.\start.ps1 -Cli pagerank`, `.\start.ps1 -Cli path boomy storytold`, `.\start.ps1 -Cli export mermaid`
- **MCP Stdio Server**: `.\start.ps1 -Mcp` (or `cargo run -p graphcraft-mcp -- mcp`)
- **Canvas Controls**: Drag background to pan, drag a node to move it, `Ctrl + Mouse Wheel` to zoom, `Ctrl + K` for the Command Palette, `F1` for Help.

## Tips

- Run `Commands` -> `Force Physics (30 steps)` or toggle `Live Physics` in the left sidebar to untangle dense clusters automatically.
- Node radii scale with PageRank centrality so hub nodes stand out immediately.
- Use `Export` -> `Mermaid` or `DOT` to paste live topology diagrams directly into markdown documentation or Graphviz pipelines.

## Other Crafts

GraphCraft is part of the Pure-Rust ArtCraft desktop suite alongside `gamecraft`, `formcraft`, `gaincraft`, `djcraft`, `sdrcraft`, and other standalone tools. Each craft owns its own 3-tier workspace (`*-core`, `*-mcp`, `*-app`) with zero sibling crate dependencies.

## Technical Info

GraphCraft models a **Labeled Property Graph (LPG)** -- specifically a **directed, weighted, heterogeneous multigraph** backed by `petgraph::graph::DiGraph<String, f64>`:

- **Node Kinds**: `Concept`, `Project`, `Person`, `Hardware`, `Document`, `Cluster` (circles scaled by PageRank), and `Drawer` (multi-line card node, such as `Backoffice apps`).
- **Properties**: Every node carries `id`, `label`, `kind`, `notes`, `tags`, `metadata`, `weight`, and `pos (x, y)`; every edge carries `id`, `source`, `target`, `relation`, and `weight`.
- **Persistence**: Stored in SQLite (`workspace_meta`, `nodes`, `edges`) via bundled `rusqlite`.
