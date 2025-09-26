# Centotype Project Session Continuation Prompt

## Quick Start (2 Minutes)

**Project:** Centotype CLI Typing Trainer
**Owner:** rafi-rfxlamia
**Current Phase:** Foundation Complete → Content Development
**Memory Namespace:** `centotype`
**Status:** All components compile, architecture in place, ready for content development

### Immediate Context Restoration

```bash
# Get current project status
search_nodes query="project-status centotype"

# Review architecture and current work
search_nodes query="architecture foundation-complete"

# Check for any blockers or recent decisions
search_nodes query="blockers decisions current-work"
```

### Current Priority Actions
1. **Content Development:** Text corpus integration and level progression
2. **Performance Validation:** Test content loading against 25ms latency target
3. **UI Implementation:** Begin terminal interface development

---

## Memory Restoration Commands

### Essential Context (Copy-Paste Ready)

```bash
# Project overview and current status
search_nodes query="centotype project status foundation"
open_nodes names=["project-overview", "current-status", "foundation-complete"]

# Architecture and technical requirements
search_nodes query="architecture crates performance-targets"
open_nodes names=["7-crate-architecture", "performance-requirements"]

# Team and resource allocation
search_nodes query="team resources roles responsibilities"

# Recent work and decisions
search_nodes query="recent-work decisions blockers progress"

# Next phase planning
search_nodes query="content-development next-actions phase-2"
```

### Deep Context Restoration

```bash
# Technical specifications
search_nodes query="performance latency memory security"
open_nodes names=["performance-targets", "security-requirements", "platform-support"]

# Development infrastructure
search_nodes query="development build test infrastructure"

# Content strategy and requirements
search_nodes query="content text-corpus level-progression difficulty"

# UI and terminal requirements
search_nodes query="terminal interface input-handling rendering"
```

---

## Project Overview

### Architecture: 7-Crate Rust Workspace

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

### Performance Targets (Critical)
- **P99 Input Latency:** < 25ms
- **P95 Startup Time:** < 200ms
- **P95 Render Time:** < 33ms
- **Memory Usage:** < 50MB RSS during active sessions

### Platform Support
- **OS:** Linux, macOS, Windows (x86_64, ARM64)
- **Terminals:** xterm, gnome-terminal, iTerm2, Windows Terminal, cmd.exe

### Team Structure
- **1.0 FTE Senior Rust Developer** (architecture, performance)
- **1.0 FTE CLI Specialist** (terminal interface, UX)
- **0.5 FTE Technical Writer** (content, documentation)
- **0.5 FTE QA Engineer** (testing, validation)

---

## Current Status: Foundation → Content Development

### ✅ Foundation Phase Complete (Weeks 1-6)
- [x] 7-crate architecture implemented
- [x] All components compile successfully
- [x] Core traits and interfaces defined
- [x] Development infrastructure setup
- [x] Performance monitoring framework
- [x] Security input handling foundation

### 🚧 Content Development Phase (Current - Weeks 7-10)
- [ ] Text corpus integration and validation
- [ ] 100-level difficulty progression system
- [ ] Content caching and performance optimization
- [ ] Level mastery criteria implementation
- [ ] Content generation algorithms

### 📋 Upcoming: UI Implementation (Weeks 8-10)
- [ ] Terminal interface development
- [ ] Real-time input handling
- [ ] Performance display and feedback
- [ ] User experience optimization

---

## Work Patterns by Phase

### Content Development Workflow

```bash
# Start content development session
cd /home/v/project/centotype

# Restore memory context
search_nodes query="content-development text-corpus level-progression"

# Check current content work
cargo test --package centotype-content

# Content development commands
cargo run --bin content-validator
cargo run --bin level-generator
cargo bench --package centotype-content

# Performance validation
cargo run --bin perf-test -- --content-loading
```

### UI Development Workflow

```bash
# Start UI development session
search_nodes query="terminal interface input-handling rendering"

# UI development commands
cargo run --bin centotype -- --debug-mode
cargo test --package centotype-engine -- --nocapture
cargo run --example terminal-test

# Performance profiling
cargo run --release --bin centotype -- --profile-input
```

### Performance Testing Workflow

```bash
# Performance validation session
search_nodes query="performance-targets latency memory"

# Performance testing commands
cargo bench
cargo run --release --bin perf-test -- --full-suite
cargo run --bin memory-profiler

# Target validation
./scripts/validate-performance-targets.sh
```

---

## Development Commands

### Build and Test
```bash
# Full build and test
cargo build --release
cargo test --all-features
cargo clippy -- -D warnings
cargo fmt --check

# Performance build
cargo build --release --features="perf-optimized"

# Platform-specific builds
cargo build --target x86_64-pc-windows-gnu
cargo build --target aarch64-apple-darwin
```

### Performance Monitoring
```bash
# Input latency testing
cargo run --bin latency-test -- --target-p99 25

# Memory profiling
cargo run --bin memory-test -- --target-max 50MB

# Startup performance
cargo run --bin startup-test -- --target-p95 200ms

# Render performance
cargo run --bin render-test -- --target-p95 33ms
```

### Security Testing
```bash
# Input sanitization tests
cargo test security::input_sanitization

# Terminal escape sequence filtering
cargo test security::escape_filtering

# File system access restrictions
cargo test security::fs_restrictions
```

---

## Troubleshooting Guide

### Common Issues

**Performance Target Misses**
```bash
# Diagnose latency issues
cargo run --bin perf-debug -- --component input-handling
search_nodes query="performance-optimization latency-issues"
```

**Memory Usage Exceeded**
```bash
# Memory leak detection
cargo run --bin memory-debug -- --detect-leaks
search_nodes query="memory-optimization memory-leaks"
```

**Cross-Platform Compatibility**
```bash
# Platform-specific testing
cargo test --target x86_64-pc-windows-gnu
search_nodes query="platform-compatibility windows macos linux"
```

**Terminal Compatibility Issues**
```bash
# Terminal detection and fallback
cargo run --bin terminal-test -- --detect-capabilities
search_nodes query="terminal-compatibility crossterm"
```

### Performance Debugging
```bash
# Profile specific components
cargo run --bin profiler -- --component content-loading
cargo run --bin profiler -- --component input-handling
cargo run --bin profiler -- --component rendering

# Benchmark comparisons
cargo bench -- --save-baseline current
cargo bench -- --baseline current
```

---

## Memory Management During Sessions

### Update Session Progress
```bash
# Record current work and progress
create_entities entities='[
  {
    "name": "session-2024-01-XX",
    "entityType": "work-session",
    "observations": ["Working on content development", "Progress on level progression"]
  }
]'

# Note blockers or decisions
add_observations observations='[
  {
    "entityName": "current-blockers",
    "contents": ["Blocker description", "Decision needed on X"]
  }
]'
```

### End Session Documentation
```bash
# Update project status
add_observations observations='[
  {
    "entityName": "project-status",
    "contents": ["Content development 40% complete", "UI implementation ready to start"]
  }
]'

# Record next actions
create_entities entities='[
  {
    "name": "next-session-actions",
    "entityType": "action-items",
    "observations": ["Complete level progression algorithm", "Start terminal interface implementation"]
  }
]'
```

---

## Success Criteria & Milestones

### Content Development Complete When:
- [ ] 100 levels with validated difficulty progression
- [ ] Text corpus loads within performance targets
- [ ] Content validation system operational
- [ ] Level mastery criteria implemented (130 WPM, 99.5% accuracy)

### UI Implementation Complete When:
- [ ] Real-time input handling meets latency targets
- [ ] Cross-platform terminal compatibility verified
- [ ] User interface responsive and intuitive
- [ ] Performance feedback system operational

### Phase 2 Success Metrics:
- [ ] All performance targets validated
- [ ] Content system fully operational
- [ ] UI foundation ready for testing phase
- [ ] Security audit preparation complete

---

## Emergency Scenarios

### Critical Performance Regression
```bash
# Immediate diagnosis
cargo run --bin perf-emergency -- --full-report
search_nodes query="performance-regression emergency-protocol"

# Rollback if needed
git log --oneline -10
git revert [commit-hash]
```

### Build Failures
```bash
# Comprehensive build check
cargo clean && cargo build --all-features
cargo check --all-targets

# Check dependencies
cargo tree --duplicates
cargo audit
```

### Memory Corruption Issues
```bash
# Safe mode debugging
cargo run --bin safe-debug
search_nodes query="memory-safety debugging-protocol"
```

---

## Contact & Escalation

**Project Owner:** rafi-rfxlamia
**Memory Namespace:** `centotype`
**Repository:** /home/v/project/centotype

For session continuity issues, always start with memory restoration commands and project status verification.

---

*Last Updated: Session restoration prompt v1.0*
*Next Review: After Phase 2 completion*