# Centotype Troubleshooting Guide

> **Comprehensive issue resolution guide for common problems and performance optimization**

This guide provides step-by-step solutions for common issues, performance problems, and configuration challenges you might encounter while using Centotype.

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [Performance Problems](#performance-problems)
3. [Terminal and Display Issues](#terminal-and-display-issues)
4. [Audio and Sound Problems](#audio-and-sound-problems)
5. [Configuration Issues](#configuration-issues)
6. [Content and Level Problems](#content-and-level-problems)
7. [Data and Profile Issues](#data-and-profile-issues)
8. [Platform-Specific Issues](#platform-specific-issues)
9. [Performance Optimization](#performance-optimization)
10. [Getting Additional Help](#getting-additional-help)

---

## Installation Issues

### Problem: Command Not Found

**Symptoms**:
```bash
$ centotype --version
bash: centotype: command not found
```

**Solutions**:

#### Solution 1: Check PATH Configuration
```bash
# Check if cargo bin is in PATH
echo $PATH | grep -q ".cargo/bin" && echo "✓ Cargo bin in PATH" || echo "✗ Cargo bin missing from PATH"

# Add to PATH (add to ~/.bashrc or ~/.zshrc)
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Verify
which centotype
```

#### Solution 2: Reinstall with Cargo
```bash
# Force reinstall
cargo install centotype --force

# Verify installation
centotype --version
```

#### Solution 3: Manual Binary Installation
```bash
# Download latest release
curl -LO https://github.com/rfxlamia/centotype/releases/latest/download/centotype-linux-x64.tar.gz

# Extract and install
tar -xzf centotype-linux-x64.tar.gz
sudo mv centotype /usr/local/bin/

# Verify
centotype --version
```

### Problem: Rust/Cargo Not Installed

**Symptoms**:
```bash
$ cargo --version
bash: cargo: command not found
```

**Solution**:
```bash
# Install Rust and Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow installation prompts, then:
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version

# Now install Centotype
cargo install centotype
```

### Problem: Permission Denied During Installation

**Symptoms**:
```bash
error: failed to create directory `/usr/local/bin`
Permission denied (os error 13)
```

**Solutions**:

#### Solution 1: Use Sudo for System Installation
```bash
# For system-wide installation
sudo mv centotype /usr/local/bin/
```

#### Solution 2: Local User Installation
```bash
# Install to user directory
mkdir -p ~/.local/bin
mv centotype ~/.local/bin/

# Add to PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Problem: Compilation Errors

**Symptoms**:
```bash
error: failed to compile `centotype v1.0.0`
```

**Solutions**:

#### Solution 1: Update Rust Toolchain
```bash
# Update Rust to latest version
rustup update

# Verify minimum version (1.75.0+)
rustc --version
```

#### Solution 2: Clean and Retry
```bash
# Clean cargo cache
cargo clean

# Try installation again
cargo install centotype --force
```

#### Solution 3: Check System Dependencies
```bash
# Linux: Install required packages
sudo apt update
sudo apt install build-essential pkg-config

# macOS: Install Xcode command line tools
xcode-select --install

# Windows: Install Visual Studio Build Tools
# Download from https://visualstudio.microsoft.com/downloads/
```

---

## Performance Problems

### Problem: High Input Latency

**Symptoms**:
- Noticeable delay between keypress and display update
- Sluggish typing experience
- `centotype benchmark` shows P99 > 30ms

**Diagnostic Steps**:
```bash
# Check current performance
centotype benchmark

# System resource check
centotype sysinfo

# Performance profile
centotype profile --duration 30
```

**Solutions**:

#### Solution 1: Enable Performance Mode
```bash
# Enable high-performance mode
centotype config --set performance.high_performance true

# Reduce visual effects
centotype config --set performance.effects minimal

# Optimize input processing
centotype config --set performance.input_optimization aggressive
```

#### Solution 2: Platform-Specific Optimization
```bash
# Linux: Increase process priority
sudo nice -n -10 centotype play --level 1

# macOS: Use high-performance settings
centotype config --set performance.macos_optimization true

# Windows: Enable low-latency mode
centotype config --set performance.windows_low_latency true
```

#### Solution 3: Terminal Optimization
```bash
# Use faster terminal emulator
# Recommended: Alacritty, Kitty, or Windows Terminal

# Optimize terminal settings
export TERM=xterm-256color

# Disable terminal features that may slow down rendering
centotype config --set display.terminal_optimization true
```

### Problem: High Memory Usage

**Symptoms**:
- Memory usage > 50MB
- System memory warnings
- Performance degradation over time

**Diagnostic Steps**:
```bash
# Check memory usage
centotype sysinfo

# Monitor memory during session
top -p $(pgrep centotype)
```

**Solutions**:

#### Solution 1: Reduce Cache Size
```bash
# Reduce content cache
centotype config --set content.cache_size 15  # Default: 50

# Reduce cache memory limit
centotype config --set content.cache_memory_mb 10  # Default: 20

# Enable aggressive cleanup
centotype config --set content.aggressive_cleanup true
```

#### Solution 2: Memory-Efficient Mode
```bash
# Enable memory-efficient mode
centotype config --set performance.memory_mode efficient

# Disable preloading
centotype config --set content.preloading false

# Clear cache
centotype cache --clear
```

#### Solution 3: Session Management
```bash
# Limit session duration to prevent memory buildup
centotype config --set session.max_duration 1800  # 30 minutes

# Restart application periodically
# For long practice sessions
```

### Problem: Slow Content Loading

**Symptoms**:
- Delays when starting levels
- Content generation taking >50ms
- Frequent "Loading..." messages

**Diagnostic Steps**:
```bash
# Check content performance
centotype debug --content-timing

# Cache statistics
centotype cache --stats
```

**Solutions**:

#### Solution 1: Optimize Content Cache
```bash
# Increase cache size (if memory allows)
centotype config --set content.cache_size 75

# Enable aggressive preloading
centotype config --set content.preload_strategy adaptive

# Preload upcoming levels
centotype preload --levels 5
```

#### Solution 2: Background Generation
```bash
# Enable background content generation
centotype config --set content.background_generation true

# Increase worker threads
centotype config --set content.worker_threads 2
```

---

## Terminal and Display Issues

### Problem: Rendering Problems

**Symptoms**:
- Garbled text or characters
- Missing or incorrect colors
- Layout issues

**Diagnostic Steps**:
```bash
# Check terminal capabilities
centotype check

# Test display features
centotype test-display

# Verify color support
centotype test-colors
```

**Solutions**:

#### Solution 1: Terminal Compatibility
```bash
# Set compatible terminal type
export TERM=xterm-256color

# Use compatibility mode
centotype config --set display.compatibility_mode true

# Force specific rendering mode
centotype play --level 1 --display-mode compatible
```

#### Solution 2: Terminal Configuration
```bash
# Enable UTF-8 support
export LC_ALL=en_US.UTF-8
export LANG=en_US.UTF-8

# Set terminal size appropriately
# Minimum: 80x24, Recommended: 120x30
resize -s 30 120
```

#### Solution 3: Terminal Emulator Issues
```bash
# For GNOME Terminal
gsettings set org.gnome.Terminal.Legacy.Settings default-show-menubar false

# For iTerm2 (macOS)
# Preferences > Profiles > Terminal > Report Terminal Type: xterm-256color

# For Windows Terminal
# Settings > Profiles > Defaults > Command line: Use Windows Terminal defaults
```

### Problem: Colors Not Displaying

**Symptoms**:
- All text appears in single color
- Missing syntax highlighting
- Theme not applying

**Solutions**:

#### Solution 1: Color Support Check
```bash
# Test color capabilities
centotype test-colors

# Force color mode
centotype config --set display.force_colors true

# Check environment variables
echo $COLORTERM
echo $TERM
```

#### Solution 2: Terminal Color Configuration
```bash
# Enable 256-color mode
export TERM=xterm-256color

# For terminals with limited color support
centotype config --set display.color_mode basic

# Use monochrome mode as fallback
centotype config --set display.monochrome true
```

### Problem: Text Size Issues

**Symptoms**:
- Text too small or too large
- Layout doesn't fit terminal
- Scrolling issues

**Solutions**:

#### Solution 1: Terminal Size Adjustment
```bash
# Check current terminal size
tput cols && tput lines

# Resize terminal (if supported)
resize -s 30 120  # 30 rows, 120 columns

# Auto-adjust to terminal size
centotype config --set display.auto_resize true
```

#### Solution 2: Text Scaling
```bash
# Adjust text scaling
centotype config --set display.text_scale 1.2  # 20% larger

# Enable large text mode
centotype config --set accessibility.large_text true

# Use compact mode for small terminals
centotype config --set display.compact_mode true
```

---

## Audio and Sound Problems

### Problem: No Sound Effects

**Symptoms**:
- Audio settings enabled but no sound
- Sound test fails
- Audio feedback missing

**Diagnostic Steps**:
```bash
# Test audio system
centotype test-audio

# Check audio configuration
centotype config --show | grep sound

# System audio test
# Linux: speaker-test
# macOS: say "test"
# Windows: Test-NetConnection with beep
```

**Solutions**:

#### Solution 1: Audio Configuration
```bash
# Enable audio
centotype config --set sound.enabled true

# Set appropriate volume
centotype config --set sound.volume 0.8

# Test individual sound types
centotype config --set sound.keystroke true
centotype config --set sound.error true
```

#### Solution 2: System Audio Check

**Linux**:
```bash
# Check PulseAudio
pulseaudio --check -v

# Check ALSA
aplay /usr/share/sounds/alsa/Front_Left.wav

# Install audio packages if missing
sudo apt install pulseaudio-utils alsa-utils
```

**macOS**:
```bash
# Check system audio
sudo launchctl list | grep audio

# Reset audio system (if needed)
sudo killall coreaudiod
```

**Windows**:
```bash
# Check audio services
Get-Service | Where-Object {$_.Name -like "*audio*"}

# Test system audio
[System.Media.SystemSounds]::Beep.Play()
```

#### Solution 3: Alternative Audio Solutions
```bash
# Disable problematic audio
centotype config --set sound.enabled false

# Use visual feedback only
centotype config --set display.visual_feedback enhanced

# Enable vibration (if supported)
centotype config --set feedback.haptic true
```

### Problem: Audio Latency

**Symptoms**:
- Delayed sound effects
- Audio out of sync with typing
- Performance impact from audio

**Solutions**:

#### Solution 1: Audio Buffer Optimization
```bash
# Reduce audio buffer size
centotype config --set sound.buffer_size 256

# Use low-latency audio mode
centotype config --set sound.low_latency true

# Optimize audio processing
centotype config --set sound.realtime_priority true
```

#### Solution 2: Disable Resource-Heavy Audio
```bash
# Disable keystroke sounds (most frequent)
centotype config --set sound.keystroke false

# Keep only essential audio
centotype config --set sound.error true
centotype config --set sound.completion true
```

---

## Configuration Issues

### Problem: Configuration Not Saving

**Symptoms**:
- Settings reset after restart
- `config --set` commands appear to work but don't persist
- Error messages when saving configuration

**Solutions**:

#### Solution 1: File Permissions
```bash
# Check config directory permissions
ls -la ~/.config/centotype/

# Fix permissions
chmod 755 ~/.config/centotype/
chmod 644 ~/.config/centotype/config.toml

# Create directory if missing
mkdir -p ~/.config/centotype/
```

#### Solution 2: Configuration File Issues
```bash
# Backup and reset configuration
cp ~/.config/centotype/config.toml ~/.config/centotype/config.toml.backup
centotype config --reset

# Manually edit if needed
$EDITOR ~/.config/centotype/config.toml

# Validate configuration
centotype config --validate
```

#### Solution 3: Write Permission Problems
```bash
# Check filesystem space
df -h ~/.config/

# Check file system permissions
touch ~/.config/centotype/test.tmp && rm ~/.config/centotype/test.tmp

# Alternative config location (if needed)
export CENTOTYPE_CONFIG_DIR="$HOME/.centotype"
```

### Problem: Invalid Configuration Values

**Symptoms**:
- Warning messages about invalid settings
- Configuration validation errors
- Unexpected behavior after configuration changes

**Solutions**:

#### Solution 1: Validate Configuration
```bash
# Check configuration validity
centotype config --validate

# Show configuration with validation
centotype config --show --validate

# Reset invalid sections
centotype config --reset ui
centotype config --reset performance
```

#### Solution 2: Manual Configuration Fix
```bash
# Edit configuration file directly
$EDITOR ~/.config/centotype/config.toml

# Example of valid configuration structure:
cat > ~/.config/centotype/config.toml << EOF
[ui]
theme = "dark"
layout = "qwerty"

[performance]
high_performance = false
input_optimization = "auto"

[sound]
enabled = true
volume = 0.7
EOF
```

---

## Content and Level Problems

### Problem: Level Content Not Loading

**Symptoms**:
- "Failed to load content" errors
- Empty or placeholder content
- Content generation timeouts

**Solutions**:

#### Solution 1: Cache Management
```bash
# Clear content cache
centotype cache --clear

# Rebuild cache
centotype cache --rebuild

# Check cache status
centotype cache --status
```

#### Solution 2: Content Generation Issues
```bash
# Test content generation
centotype test-content --level 1

# Enable content debugging
export RUST_LOG=centotype_content=debug
centotype play --level 1

# Force content regeneration
centotype generate --level 1 --force
```

#### Solution 3: Fallback Content
```bash
# Enable fallback content mode
centotype config --set content.enable_fallback true

# Use offline content mode
centotype config --set content.offline_mode true
```

### Problem: Incorrect Difficulty Progression

**Symptoms**:
- Level seems too easy or too hard
- Inconsistent difficulty between similar levels
- Unexpected content types

**Solutions**:

#### Solution 1: Content Validation
```bash
# Validate level progression
centotype validate --levels 1-10

# Check specific level difficulty
centotype analyze --level 25 --difficulty

# Report progression issues
centotype debug --progression-report
```

#### Solution 2: Reset Content
```bash
# Reset content for specific level
centotype reset --level 25

# Reset tier content
centotype reset --tier silver

# Force content regeneration
centotype generate --all --force
```

### Problem: Content Caching Issues

**Symptoms**:
- Slow level loading
- High memory usage
- Cache hit rate < 90%

**Solutions**:

#### Solution 1: Cache Optimization
```bash
# Optimize cache settings
centotype config --set content.cache_size 50
centotype config --set content.preload_strategy adaptive

# Monitor cache performance
centotype monitor --cache

# Cache statistics
centotype cache --stats --detailed
```

#### Solution 2: Preloading Strategy
```bash
# Enable smart preloading
centotype config --set content.preload_strategy adaptive

# Manual preloading
centotype preload --around-level 25 --count 5

# Background preloading
centotype config --set content.background_preload true
```

---

## Data and Profile Issues

### Problem: Profile Data Corruption

**Symptoms**:
- Statistics reset unexpectedly
- Profile loading errors
- Inconsistent progress tracking

**Solutions**:

#### Solution 1: Profile Recovery
```bash
# Check profile integrity
centotype profile --check

# Backup current profile
centotype backup --profile

# Restore from backup
centotype restore --profile --file backup.json
```

#### Solution 2: Profile Rebuild
```bash
# Export current data
centotype export --all --format json

# Reset profile
centotype profile --reset

# Import critical data
centotype import --file exported-data.json
```

### Problem: Statistics Not Updating

**Symptoms**:
- Statistics remain unchanged after sessions
- Missing session data
- Incorrect progress tracking

**Solutions**:

#### Solution 1: Database Maintenance
```bash
# Check database integrity
centotype db --check

# Rebuild statistics
centotype stats --rebuild

# Repair database
centotype db --repair
```

#### Solution 2: Session Data Issues
```bash
# Check session logging
export RUST_LOG=centotype_persistence=debug
centotype play --level 1

# Manual session save
centotype session --save-current

# Verify data location
centotype info --data-directory
```

---

## Platform-Specific Issues

### Linux Issues

#### Problem: Permission Denied Errors
```bash
# Fix binary permissions
chmod +x /usr/local/bin/centotype

# Fix config directory permissions
chmod 755 ~/.config/centotype/

# Check SELinux (if applicable)
getenforce
```

#### Problem: Missing Dependencies
```bash
# Install required packages
sudo apt update
sudo apt install libc6-dev build-essential

# For audio support
sudo apt install libasound2-dev pulseaudio-utils

# For terminal support
sudo apt install libncurses5-dev
```

### macOS Issues

#### Problem: Security Warnings
```bash
# Allow unsigned binary (if using pre-built)
sudo spctl --add centotype
sudo xattr -dr com.apple.quarantine /usr/local/bin/centotype

# Or build from source
cargo install centotype
```

#### Problem: Terminal Compatibility
```bash
# Set proper terminal type
export TERM=xterm-256color

# For iTerm2 users
# Preferences > Profiles > Terminal > Report Terminal Type: xterm-256color

# Enable Unicode support
export LC_ALL=en_US.UTF-8
```

### Windows Issues

#### Problem: Command Prompt Limitations
```bash
# Use Windows Terminal instead of cmd.exe
# Download from Microsoft Store

# Enable UTF-8 support
chcp 65001

# Set environment variables
setx TERM xterm-256color
```

#### Problem: PATH Issues
```powershell
# Add to PATH in PowerShell
$env:PATH += ";C:\path\to\centotype"

# Permanent PATH update
[Environment]::SetEnvironmentVariable("PATH", $env:PATH + ";C:\path\to\centotype", "User")
```

---

## Performance Optimization

### System-Level Optimization

#### CPU Priority Optimization
```bash
# Linux: Increase process priority
sudo nice -n -10 centotype play --level 1

# Set CPU affinity (multi-core systems)
taskset -c 0 centotype play --level 1

# Real-time scheduling (advanced)
sudo chrt -f 50 centotype play --level 1
```

#### Memory Optimization
```bash
# Monitor memory usage
top -p $(pgrep centotype)

# Enable memory-efficient mode
centotype config --set performance.memory_mode efficient

# Reduce buffer sizes
centotype config --set performance.buffer_size 1024
```

#### I/O Optimization
```bash
# Use faster storage for config/data
# Move config to SSD if on HDD
ln -s /path/to/ssd/centotype-config ~/.config/centotype

# Optimize file system
# Enable noatime for better performance
```

### Application-Level Optimization

#### Input Processing Optimization
```bash
# Enable optimized input handling
centotype config --set performance.input_optimization aggressive

# Reduce input buffer size for lower latency
centotype config --set input.buffer_size 1

# Use dedicated input thread
centotype config --set input.dedicated_thread true
```

#### Rendering Optimization
```bash
# Optimize display rendering
centotype config --set display.vsync false
centotype config --set display.double_buffer true

# Reduce render complexity
centotype config --set display.effects minimal
centotype config --set display.animations false
```

#### Content System Optimization
```bash
# Optimize content caching
centotype config --set content.cache_strategy lru_optimized
centotype config --set content.compression true

# Background processing
centotype config --set content.background_workers 1
```

### Network and Connectivity (if applicable)

#### Disable Network Features
```bash
# Disable telemetry for better performance
centotype config --set privacy.telemetry false

# Disable update checks
centotype config --set updates.auto_check false

# Offline mode
centotype config --set general.offline_mode true
```

---

## Getting Additional Help

### Built-in Diagnostics

#### System Information
```bash
# Comprehensive system check
centotype sysinfo

# Performance diagnostics
centotype benchmark

# Configuration validation
centotype config --validate
```

#### Debug Information
```bash
# Enable debug logging
export RUST_LOG=debug
centotype play --level 1 2>&1 | tee debug.log

# Performance profiling
centotype profile --duration 60 --output profile.json

# Export diagnostic data
centotype debug --export diagnostics.zip
```

#### Health Checks
```bash
# Component health check
centotype check --all

# Performance health
centotype check --performance

# Content system health
centotype check --content
```

### Documentation Resources

#### Online Documentation
- **Complete Guide**: [GitHub Repository Documentation](https://github.com/rfxlamia/centotype/tree/main/docs)
- **API Reference**: [API Documentation](./API_REFERENCE.md)
- **Performance Guide**: [Performance Optimization](./PERFORMANCE_GUIDE.md)
- **Developer Guide**: [Development Documentation](./DEVELOPER_GUIDE.md)

#### Local Help
```bash
# Built-in help system
centotype help

# Command-specific help
centotype play --help
centotype config --help

# Manual pages (if installed)
man centotype
```

### Community Support

#### GitHub Resources
- **Issues**: [Report bugs and request features](https://github.com/rfxlamia/centotype/issues)
- **Discussions**: [Community Q&A](https://github.com/rfxlamia/centotype/discussions)
- **Wiki**: [Community guides and tips](https://github.com/rfxlamia/centotype/wiki)

#### Bug Reports

When reporting issues, include:

```bash
# Generate diagnostic report
centotype debug --generate-report

# Include system information
centotype sysinfo > system-info.txt

# Performance data (if relevant)
centotype benchmark > benchmark-results.txt

# Configuration dump
centotype config --show > current-config.txt
```

#### Feature Requests

For feature requests, provide:
- Clear description of desired functionality
- Use case explanation
- Expected behavior
- Current workaround (if any)

### Emergency Recovery

#### Complete Reset
```bash
# Backup important data
centotype backup --all

# Reset to factory defaults
centotype reset --all --confirm

# Reinstall application
cargo install centotype --force
```

#### Data Recovery
```bash
# Attempt data recovery
centotype recover --profile
centotype recover --statistics

# Manual backup restoration
centotype restore --backup backup-file.json
```

---

## Quick Reference

### Most Common Fixes

| Problem | Quick Fix |
|---------|-----------|
| Command not found | `export PATH="$HOME/.cargo/bin:$PATH"` |
| High latency | `centotype config --set performance.high_performance true` |
| Display issues | `centotype config --set display.compatibility_mode true` |
| No sound | `centotype test-audio && centotype config --set sound.enabled true` |
| Config not saving | `chmod 755 ~/.config/centotype/` |
| Slow loading | `centotype cache --clear && centotype cache --rebuild` |

### Emergency Commands

```bash
# Quick performance fix
centotype config --set performance.high_performance true

# Display compatibility fix
centotype config --set display.compatibility_mode true

# Reset to working state
centotype config --reset

# Complete reinstall
cargo install centotype --force
```

### Useful Aliases

Add to your shell configuration (`~/.bashrc` or `~/.zshrc`):

```bash
# Quick shortcuts
alias ct='centotype'
alias ctp='centotype play'
alias cts='centotype stats'
alias ctc='centotype config'
alias ctfix='centotype config --reset && centotype cache --clear'
```

This troubleshooting guide covers the most common issues and their solutions. For issues not covered here, please check the GitHub repository or create an issue with detailed diagnostic information.