# Changelog

## [0.1.0] - 2026-11-22

### Added
- Initial 3-tier Pure-Rust workspace (`graphcraft-core`, `graphcraft-mcp`, `graphcraft-app`).
- L1 `petgraph` engine with PageRank, Dijkstra shortest path, cycle detection, component count, density, and Fruchterman-Reingold / circular / grid layouts.
- Multi-format export to JSON, Graphviz DOT, Mermaid flowchart, and Gephi GEXF 1.3.
- L2 CLI (`--json`) and MCP `2024-11-05` stdio server with 7 tools.
- L3 `eframe`/`egui` desktop studio with infinite pan/zoom canvas, shortest path highlighting, live physics toggle, WCAG AAA Light/Dark themes, Ctrl+Wheel zoom, and Command Palette (`Ctrl+K`).
- Vienna Open Robotics & Craft Ecosystem seed dataset for immediate first-paint exploration.
