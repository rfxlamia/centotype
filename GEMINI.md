# Centotype Gemini Context

This document provides context for the Centotype project, a CLI-based typing trainer written in Rust.

## Project Overview

Centotype is a precision-focused CLI typing trainer with 100 progressive difficulty levels. It is designed for developers, technical writers, and competitive typists.

The project is a Rust workspace with the following crates:

*   `centotype-bin`: The main binary crate.
*   `cli`: Command-line interface with interactive navigation and menu systems.
*   `core`: Core business logic, including session management, scoring, and level progression.
*   `engine`: Core event loop, input handling, and render system.
*   `content`: Text generation and corpus management.
*   `analytics`: Performance analysis and error classification.
*   `persistence`: Profile storage and configuration.
*   `platform`: OS abstraction and terminal detection.

## Building and Running

### Build
```bash
cargo build --release
```

### Run
```bash
# Begin at Level 1 (basic words)
./target/release/centotype play --level 1

# Take the placement test to find your starting level
./target/release/centotype placement

# Practice specific skills
./target/release/centotype drill --category symbols
./target/release/centotype drill --category numbers

# Test endurance and consistency
./target/release/centotype endurance --duration 10
```

### Test
```bash
cargo test
```

### Benchmarks
```bash
cargo bench
```

## Development Conventions

*   **Formatting:** `cargo fmt`
*   **Linting:** `cargo clippy`
*   **Architecture:** The project follows a modular architecture with a clear separation of concerns between the different crates. The `ARCHITECTURE.md` file provides a detailed overview of the data flow and component interactions.
*   **Performance:** The project has strict performance targets, including <25ms input latency and <200ms startup time. The `engine` crate is designed for high performance, using `tokio` for asynchronous event processing.
*   **Error Handling:** The project uses the `thiserror` and `anyhow` crates for error handling.
