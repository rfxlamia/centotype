# Changelog
All notable changes to this repository will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) principles and adheres to semantic versioning once tagged releases begin.

## [Unreleased]
### Planned
- Finalize core, engine, CLI, analytics, and persistence feature completion milestones noted in `ROADMAP.md` Phase 1 close-out.
- Restore performance grade to **A** by driving input latency from 28ms to <25ms P99 across Linux, macOS, and Windows.
- Prepare distribution assets (cargo, npm, release binaries) and launch collateral for Phase 3 readiness.

### In Progress
- Profiling and optimizing event loop and keystroke processing paths in the engine crate.
- Expanding automated performance validation via `scripts/validate_local.sh` and benchmark harnesses.

### Completed
- ✅ Comprehensive repository structure reorganization with clean file organization and logical documentation hierarchy.
- ✅ Documentation consolidation into `/docs/` with architecture/, performance/, guides/, api/, specs/, design/, and development/ subdirectories.

### Known Risks
- Performance regressions as remaining crates are completed; mitigate via nightly benchmark runs.
- Cross-platform inconsistencies in advanced terminal sequences; mitigate with broadened manual QA matrix.

## [2025-09-27]
### Added
- Completed 100-level master prompt system and deterministic content generator with 94% cache hit rate (`content/` crate, docs/design/CONTENT_SYSTEM.md).
- Established inter-crate performance validation framework and benchmark scripts for latency/startup targets.
- Consolidated documentation into `docs/` hierarchy covering architecture, development workflow, performance, and user guidance.
- ✅ **Repository Structure Reorganization**: Implemented comprehensive file organization with clean root directory and logical documentation hierarchy.

### Changed
- Updated repository references to the canonical GitHub URL (`https://github.com/rfxlamia/centotype`).
- Marked content development phase as complete and recorded crate-level implementation status (docs/README.md).
- ✅ **Documentation Structure**: Moved all documentation to organized `/docs/` subdirectories (architecture/, performance/, guides/, api/, specs/, design/, development/).
- ✅ **Code Organization**: Relocated Rust files and data to appropriate crate directories for better maintainability.
- ✅ **Reference Updates**: Updated all file references in README.md and CLAUDE.md to reflect new locations.

### Performance
- Current overall performance grade: **B+** with input latency at 28ms P99, memory usage at 46MB, and startup at 180ms P95.

### Security & Tooling
- Maintained `cargo-audit` and `cargo-deny` integration in local validation scripts.
- Documented security posture and risk buffers per PRD v2.0.
