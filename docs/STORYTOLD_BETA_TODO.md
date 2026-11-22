# Storytold Beta Readiness Checklist - GraphCraft

**Target Intake:** https://getartcraft.com/beta

- [x] Standalone fresh-clone build (zero `../sibling` path deps)
- [x] Dual license `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`)
- [x] `NOTICE` non-affiliation disclaimer + `ATTRIBUTION.md` SPDX allowlist
- [x] `#![forbid(unsafe_code)]` and `clippy::deny(unwrap_used, expect_used, panic, todo, unimplemented)` across all crates
- [x] Universal Command Bus (`Command::dispatch`) shared by CLI, MCP stdio, and egui desktop app
- [x] High-contrast Light + Dark themes + `Ctrl + Wheel` zoom + standard menu bar
- [x] Honest README Feature Matrix (`Implemented` / `Planned`)
- [ ] Cross-platform GitHub Actions CI (Windows, macOS, Linux) + `SHA256SUMS.txt`
- [ ] Dark + Light desktop screenshots captured (`docs/screenshots/`)
- [ ] Expand test count from Tier 1 (>=20) to Tier 2 (>=50)
- [ ] Submit 4-field application at https://getartcraft.com/beta
