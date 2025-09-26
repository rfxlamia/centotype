# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Centotype is a CLI-based typing trainer built in Rust with 100 progressive difficulty levels. This repository currently contains **planning and specification documents only** - the actual implementation has not yet started.

**Current Status**: Pre-development planning phase (documentation-only repository)

## Key Documents

- **`prd_vnext.md`**: Complete PRD v2.0 with comprehensive specifications
- **`open_questions.md`**: Critical unresolved implementation decisions
- **`open_questions_answer.md`**: Detailed answers to implementation questions
- **`diff_patch.txt`**: Version control showing evolution from v1 to v2 PRD

## Architecture (Planned)

When implementation begins, the project will follow this Rust crate structure:

```
centotype/
├── core/           # State management, scoring engine, content generation
├── engine/         # Input handling, TTY management, render loop
├── content/        # Text corpus loading, caching, validation
├── analytics/      # Performance analysis, error classification
├── cli/            # Command parsing, interactive navigation
├── persistence/    # Profile storage, configuration management
└── platform/       # OS-specific integrations, terminal detection
```

## Development Commands (Future)

Once implementation starts, these commands will be available:

```bash
# Build and development
cargo build --release
cargo test
cargo clippy
cargo fmt

# Application usage (post-implementation)
centotype play --level 1
centotype drill --category symbols
centotype endurance --duration 15
```

## Key Technical Requirements

- **Performance Targets**: P99 input latency < 25ms, P95 startup < 200ms, P95 render < 33ms
- **Platforms**: Linux, macOS, Windows (x86_64, ARM64)
- **Terminal Support**: xterm, gnome-terminal, iTerm2, Windows Terminal, cmd.exe
- **Memory**: < 50MB RSS during active sessions
- **Security**: Input sanitization, terminal escape sequence filtering, restricted file system access

## Critical Implementation Decisions

Per `open_questions_answer.md`, key decisions include:

1. **Performance validation** via prototype testing before Week 2
2. **Resource allocation**: 1.0 FTE Senior Rust + 1.0 FTE CLI + 0.5 FTE Writer + 0.5 FTE QA
3. **Level 100 mastery criteria**: 130 WPM effective, 99.5% accuracy, ≤3 error severity
4. **Security audit budget**: $15k for terminal/input security review
5. **Distribution**: npm wrapper + cargo install + GitHub releases

## Timeline (Planned)

- **Phase 1 (Weeks 1-6)**: Foundation - architecture, core engine, MVP features
- **Phase 2 (Weeks 7-10)**: Content development, testing, optimization
- **Phase 3 (Weeks 11-12)**: Distribution, launch preparation
- **Total**: 16 weeks (12 weeks + 4 weeks buffer)

## Current Phase

This repository is in the **specification phase**. Before starting implementation:

1. Resolve all questions in `open_questions.md`
2. Confirm team resource availability
3. Validate performance targets via prototyping
4. Set up development infrastructure and CI/CD

## Notes for Development

- Follow security-first approach for all input handling
- Implement deterministic scoring for reproducible results
- Use crossterm for cross-platform terminal handling
- Profile data stored as local JSON with atomic writes
- Support graceful degradation on limited terminals