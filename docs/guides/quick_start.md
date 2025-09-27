# Centotype Quick Start Guide

> **Goal**: Get from zero to your first typing session in under 2 minutes

This guide gets you up and running with Centotype, the precision CLI typing trainer, as quickly as possible. Follow these steps to start improving your typing speed and accuracy immediately.

## Prerequisites

- **Operating System**: Linux, macOS 10.14+, or Windows 10+
- **Terminal**: Modern terminal emulator with UTF-8 support
- **Rust** (Optional): Version 1.75+ if building from source

## Installation Options

### Option 1: Cargo Install (Recommended - 30 seconds)

If you have Rust installed:

```bash
cargo install centotype
```

**Verification**:
```bash
centotype --version
# Should output: centotype 1.0.0
```

### Option 2: Pre-built Binary (45 seconds)

**Linux/macOS**:
```bash
# Download and install
curl -LO https://github.com/rfxlamia/centotype/releases/latest/download/centotype-linux-x64.tar.gz
tar -xzf centotype-linux-x64.tar.gz
sudo mv centotype /usr/local/bin/

# Verify installation
centotype --version
```

**Windows**:
```powershell
# Download from GitHub releases
# Extract to a directory in your PATH
# Verify with: centotype --version
```

### Option 3: npm Wrapper (Quick alternative)

```bash
npm install -g centotype-cli
centotype --version
```

## Your First Typing Session (30 seconds)

### 1. Basic Level Practice

Start with Level 1 (basic words):

```bash
centotype play --level 1
```

**What you'll see**:
- Target text at the top
- Your typed text highlighted in real-time
- Live WPM and accuracy metrics
- Progress indicator

**Controls**:
- `Ctrl+C`: Exit session
- `Backspace`: Correct mistakes
- `Tab`: Pause/resume

### 2. Find Your Starting Level

Take the placement test to determine your ideal starting level:

```bash
centotype placement
```

This 2-minute assessment will recommend your optimal starting level based on your current typing ability.

### 3. Practice Specific Skills

Focus on areas that need improvement:

```bash
# Practice symbols and punctuation
centotype drill --category symbols

# Practice numbers
centotype drill --category numbers

# Practice your weakest keys
centotype drill --weak-keys
```

## Understanding Your Results

After completing a session, you'll see:

```
Session Complete!
╭─────────────────────────────────────╮
│ Level 5 - Mixed Content            │
│ ─────────────────────────────────── │
│ WPM (Raw):        72.4             │
│ WPM (Effective):  68.1             │
│ Accuracy:         94.2%            │
│ Consistency:      87.6%            │
│ Skill Index:      682              │
│ Grade:            B+               │
│ Stars:            ★★★☆☆           │
╰─────────────────────────────────────╯
```

**Key Metrics**:
- **Effective WPM**: Speed adjusted for accuracy (most important)
- **Skill Index**: 0-1000 rating of overall proficiency
- **Grade**: A-F letter grade for the session
- **Stars**: 1-5 star rating for level performance

## Quick Configuration

### Essential Settings

```bash
# View current configuration
centotype config --show

# Set dark theme (recommended for extended practice)
centotype config --set theme dark

# Enable audio feedback
centotype config --set sound enabled

# Set your keyboard layout
centotype config --set layout qwerty  # or qwertz, azerty
```

### Performance Optimization

For best experience on slower systems:

```bash
# Reduce visual effects
centotype config --set effects minimal

# Optimize for performance
centotype config --set render-rate 30fps
```

## Quick Tips for Success

### Accuracy First
- **Always prioritize accuracy over speed**
- Centotype penalizes corrections heavily
- Clean typing beats fast corrections

### Practice Consistency
```bash
# Short, focused sessions work best
centotype play --level 10 --duration 5min  # 5-minute sessions
```

### Track Progress
```bash
# View your improvement over time
centotype stats --detailed

# Export data for analysis
centotype export --format csv --days 7
```

### Progressive Training
- Start at your placement level
- Achieve 95%+ accuracy before advancing
- Use drill mode to target weak areas

## Troubleshooting Common Issues

### Terminal Compatibility
If you see rendering issues:

```bash
# Check terminal capabilities
centotype check

# Use compatibility mode if needed
centotype play --level 1 --compat-mode
```

### Performance Issues
If input feels laggy:

```bash
# Run performance test
centotype benchmark

# Check system requirements
centotype sysinfo
```

### Audio Not Working
```bash
# Verify audio settings
centotype config --show | grep sound

# Test audio
centotype test-audio
```

## Next Steps

### Level Progression Path

1. **Start**: Complete placement test
2. **Foundation**: Master Levels 1-25 (Bronze tier)
3. **Proficiency**: Progress through Levels 26-50 (Silver tier)
4. **Advanced**: Tackle Levels 51-75 (Gold tier)
5. **Expert**: Challenge Levels 76-90 (Platinum tier)
6. **Master**: Conquer Levels 91-100 (Diamond tier)

### Advanced Features

Once comfortable with basics:

```bash
# Endurance training
centotype endurance --duration 15

# Custom content practice
centotype custom --file my-code.rs

# Competition mode
centotype race --online
```

### Performance Targets

**Bronze Tier Goals** (Levels 1-25):
- 40+ WPM effective speed
- 95%+ accuracy
- Consistent performance

**Silver Tier Goals** (Levels 26-50):
- 60+ WPM effective speed
- 97%+ accuracy
- Reduced error severity

**Gold+ Tier Goals** (Levels 51+):
- 80+ WPM effective speed
- 98%+ accuracy
- Expert-level consistency

## Getting Help

### Built-in Help
```bash
# General help
centotype help

# Command-specific help
centotype play --help
centotype config --help
```

### Documentation
- **Complete Guide**: `/docs/USER_GUIDE.md`
- **Technical Details**: `/docs/ARCHITECTURE.md`
- **Performance Tuning**: `/docs/PERFORMANCE_GUIDE.md`

### Community Support
- **GitHub Issues**: [Report bugs and request features](https://github.com/rfxlamia/centotype/issues)
- **Discussions**: [Get help and share tips](https://github.com/rfxlamia/centotype/discussions)

---

## Summary: Your 2-Minute Start

1. **Install** (30s): `cargo install centotype` or download binary
2. **Test** (30s): `centotype placement` to find your level
3. **Practice** (60s): `centotype play --level X` where X is your recommended level
4. **Success!** You're now on the path to typing mastery

**Pro Tip**: Practice in short, focused 5-10 minute sessions rather than long marathon sessions. Consistency beats intensity for skill development.

Happy typing! 🚀