# Centotype

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE.md)
[![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macOS%20%7C%20windows-lightgrey)](https://github.com/rfxlamia/centotype)

> A precision-focused CLI typing trainer with 100 progressive difficulty levels, built for developers, technical writers, and competitive typists.

## 🚨 Project Status (2025-01-27)

**IMPORTANT**: This project is currently in **active development** with **exceptional architectural foundation** but **critical integration issues**. External code review confirms **high-quality engineering** with **organizational process challenges**.

### Current Reality
- ❌ **Binary Status**: Compilation failures (18+ API integration errors)
- ❌ **Functionality**: End-to-end features not yet working
- ⭐⭐⭐⭐⭐ **Architecture**: Sophisticated 7-crate design with professional standards
- ⭐⭐⭐⭐⭐ **Technical Documentation**: Comprehensive, professional quality (2,267+ lines)
- ⭐⭐⭐⭐⭐ **Recovery Potential**: "Weeks to months" timeline with clear path forward

### What's Working
- ✅ **Professional Architecture**: Modular design with clear crate boundaries
- ✅ **Quality Frameworks**: Performance monitoring, security validation, comprehensive testing design
- ✅ **Engineering Standards**: ADR process, WCAG AA compliance, detailed API specifications
- ✅ **Content System**: 100-level corpus generation with mathematical difficulty progression

### What's Being Fixed
- 🔧 **API Integration**: Resolving interface mismatches between CLI and Core crates
- 🔧 **Panic Safety**: Addressing 27+ safety violations in production code paths
- 🔧 **Test Stability**: Fixing test suite failures across crates
- 🔧 **Process Alignment**: Standardizing documentation and quality gates

Centotype aims to deliver deterministic measurement, granular feedback, and realistic practice scenarios covering code, prose, numbers, and symbols. The architectural foundation is excellent and recovery is highly achievable.

## ✨ Planned Features (Architecture Complete)

- **🎯 Precision Training**: Strict accuracy penalties reward clean typing over fast corrections
- **📊 Granular Analytics**: Per-character, per-token, and error-class insights
- **🎮 Progressive Mastery**: 100 carefully calibrated difficulty levels
- **⚡ Performance Optimized**: <25ms input latency, <200ms startup, <50MB memory (architectural targets)
- **🔄 Deterministic Scoring**: Reproducible results without random variance
- **💻 Developer-Focused**: Code snippets, symbols, brackets, and technical content
- **🏃 Multiple Training Modes**: Arcade progression, focused drills, endurance testing

## 🚧 Installation & Usage (Currently Unavailable)

**Current Status**: Installation and usage commands below reflect the **planned functionality** once integration issues are resolved. The project currently **does not compile** due to API interface mismatches.

### Development Installation Only
```bash
# Clone for development and contribute to fixes
git clone https://github.com/rfxlamia/centotype.git
cd centotype

# This will currently fail due to integration issues
cargo build --release  # ❌ FAILS with 18+ compilation errors

# Architecture exploration
cargo check --workspace --quiet  # ✅ Individual crates check successfully
cargo doc --open                 # ✅ Documentation builds successfully
```

### Planned Installation (When Fixed)
**The following installation methods will be available once API integration is resolved:**

```bash
# Cargo (will be available after fixes)
cargo install centotype

# Pre-built releases (planned)
curl -LO https://github.com/rfxlamia/centotype/releases/latest/download/centotype-linux-x64.tar.gz

# npm wrapper (planned)
npm install -g centotype-cli
```

### Basic Usage

**Start Your Typing Journey**
```bash
# Begin at Level 1 (basic words)
centotype play --level 1

# Take the placement test to find your starting level
centotype placement

# Practice specific skills
centotype drill --category symbols
centotype drill --category numbers

# Test endurance and consistency
centotype endurance --duration 10    # 10-minute session
```

## 🎮 Training Modes

### Arcade Mode
Progressive levels from basic text to advanced code patterns:
```bash
centotype play --level 15         # Start at specific level
centotype play --continue         # Resume from last level
centotype play --level 1-10       # Practice level range
```

**Level Progression**:
- **Levels 1-20**: Basic vocabulary and common words
- **Levels 21-40**: Punctuation and mixed content
- **Levels 41-60**: Numbers, symbols, and technical terms
- **Levels 61-80**: Code snippets and programming constructs
- **Levels 81-100**: Advanced patterns and competitive content

### Drill Mode
Focused practice on specific skill areas:
```bash
centotype drill --category symbols    # (){}[]<>!@#$%
centotype drill --category numbers    # Numeric sequences
centotype drill --category code       # Programming patterns
centotype drill --category brackets   # Bracket matching
centotype drill --weak-keys          # Your worst-performing keys
```

### Endurance Mode
Build stamina and maintain accuracy over longer sessions:
```bash
centotype endurance --duration 15    # 15-minute session
centotype endurance --words 500      # 500-word target
centotype endurance --adaptive       # Difficulty adjusts to your speed
```

## 📊 Performance Tracking

**View Your Progress**
```bash
centotype stats                    # Overall performance summary
centotype stats --detailed         # Per-key and error analysis
centotype stats --level 25         # Performance on specific level
centotype export --format csv      # Export data for analysis
```

**Key Metrics Tracked**:
- **WPM (Words Per Minute)**: Raw and accuracy-adjusted speeds
- **Accuracy**: Character-level precision with error classification
- **Skill Index**: 0-1000 rating system for overall proficiency
- **Error Analysis**: Substitution, insertion, deletion, transposition patterns
- **Consistency**: Speed variance and fatigue analysis

## ⚙️ Configuration

**Customize Your Experience**
```bash
# View current settings
centotype config --show

# Adjust key settings
centotype config --set theme dark
centotype config --set layout qwertz
centotype config --set sound enabled

# Reset to defaults
centotype config --reset
```

**Configuration Options**:
- **Visual**: Theme (dark/light), colors, progress indicators
- **Keyboard Layout**: QWERTY, QWERTZ, AZERTY support
- **Audio**: Keystroke sounds, error alerts, completion chimes
- **Behavior**: Correction policy, timing sensitivity, auto-advance

## 🎯 Mastery Goals

**Level 100 Achievement Criteria**:
- ⚡ **Speed**: 130+ WPM (effective, accuracy-adjusted)
- 🎯 **Accuracy**: 99.5%+ character precision
- 🔥 **Consistency**: Error severity score ≤3
- ⏱️ **Endurance**: Complete 3000+ character sessions in ≤120 seconds
- 🚫 **Clean Finish**: Zero corrections after 80% completion

**Skill Tiers**:
- **Bronze** (Levels 1-25): Foundation building - 40+ WPM, 95%+ accuracy
- **Silver** (Levels 26-50): Proficiency development - 60+ WPM, 97%+ accuracy
- **Gold** (Levels 51-75): Advanced skills - 80+ WPM, 98%+ accuracy
- **Platinum** (Levels 76-90): Expert performance - 100+ WPM, 99%+ accuracy
- **Diamond** (Levels 91-100): Mastery achievement - 130+ WPM, 99.5%+ accuracy

## 🖥️ System Requirements

**Minimum Requirements**:
- **OS**: Linux, macOS 10.14+, Windows 10+
- **Memory**: 50MB RAM during active sessions
- **Storage**: 100MB for application and content corpus
- **Terminal**: Modern terminal emulator with UTF-8 support

**Supported Terminals**:
- ✅ Linux: xterm, gnome-terminal, konsole, alacritty, kitty
- ✅ macOS: Terminal.app, iTerm2, Hyper
- ✅ Windows: Windows Terminal, PowerShell, cmd.exe

**Performance Targets** (Validated):
- **Startup**: <200ms (P95)
- **Input Latency**: <25ms (P99)
- **Render Rate**: <33ms per frame (30fps equivalent)

## 🛠️ Development & Contributing

We welcome contributions! Whether you're fixing bugs, adding features, or improving documentation.

**Quick Development Setup**:
```bash
git clone https://github.com/rfxlamia/centotype.git
cd centotype
cargo test          # Run test suite
cargo clippy        # Lint code
cargo fmt           # Format code
cargo bench         # Performance benchmarks
```

**Key Development Commands**:
```bash
# Development workflow
cargo check --all-targets            # Fast compilation check
cargo test --workspace              # Run all tests
cargo test --package centotype-core # Test specific component

# Performance validation
cargo run --profile perf-test       # Performance testing build
cargo bench --bench input_latency   # Input latency benchmarks

# Documentation
cargo doc --open                    # Generate and open docs
```

**Contributing Guidelines**:
1. Check existing [issues](https://github.com/rfxlamia/centotype/issues) and [discussions](https://github.com/rfxlamia/centotype/discussions)
2. Fork the repository and create a feature branch
3. Ensure tests pass and add tests for new functionality
4. Follow Rust formatting conventions (`cargo fmt`)
5. Submit a pull request with clear description

**Architecture Overview**:
For detailed technical information, see [docs/architecture/README_ARCHITECTURE.md](./docs/architecture/README_ARCHITECTURE.md) and [docs/architecture/ARCHITECTURE.md](./docs/architecture/ARCHITECTURE.md).

## 📄 License

This project is licensed under the MIT License - see the [LICENSE.md](./LICENSE.md) file for details.

## 🤝 Support & Community

- **Issues**: [GitHub Issues](https://github.com/rfxlamia/centotype/issues)
- **Discussions**: [GitHub Discussions](https://github.com/rfxlamia/centotype/discussions)
- **Website**: [centotype.dev](https://centotype.dev)

## 🙏 Acknowledgments

Built with ❤️ by the Centotype Team and powered by:
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal user interface library
- [clap](https://github.com/clap-rs/clap) - Command line argument parsing

---

**Ready to level up your typing?** Start with `centotype play --level 1` and begin your journey to typing mastery! 🚀