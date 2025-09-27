# Centotype User Guide

> **Complete user manual for mastering the Centotype CLI typing trainer**

Welcome to Centotype, the precision-focused CLI typing trainer designed for developers, technical writers, and anyone serious about improving their typing skills. This comprehensive guide will take you from installation to mastery of all 100 difficulty levels.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Training Modes](#training-modes)
3. [Level Progression System](#level-progression-system)
4. [Performance Tracking](#performance-tracking)
5. [Configuration and Customization](#configuration-and-customization)
6. [Advanced Features](#advanced-features)
7. [Tips for Success](#tips-for-success)
8. [Keyboard Shortcuts](#keyboard-shortcuts)
9. [Understanding Your Results](#understanding-your-results)
10. [Troubleshooting](#troubleshooting)

---

## Getting Started

### What is Centotype?

Centotype is a CLI-based typing trainer that focuses on:

- **Precision over Speed**: Clean typing beats fast corrections
- **Progressive Difficulty**: 100 carefully calibrated levels from basic words to advanced code
- **Deterministic Scoring**: Reproducible results without random variance
- **Developer Focus**: Code snippets, symbols, technical content
- **Performance Optimized**: <25ms input latency for responsive experience

### Quick Installation

Choose your preferred installation method:

#### Option 1: Cargo (Recommended)
```bash
cargo install centotype
centotype --version
```

#### Option 2: Pre-built Binary
```bash
# Linux/macOS
curl -LO https://github.com/rfxlamia/centotype/releases/latest/download/centotype-linux-x64.tar.gz
tar -xzf centotype-linux-x64.tar.gz
sudo mv centotype /usr/local/bin/

# Windows
# Download from GitHub releases and add to PATH
```

#### Option 3: npm Wrapper
```bash
npm install -g centotype-cli
```

### Your First Session

Start typing immediately:

```bash
# Begin at Level 1
centotype play --level 1

# Or take the placement test
centotype placement
```

**What you'll see**:
```
┌─ Level 1: Basic Words ─────────────────────────────┐
│ Target: the quick brown fox jumps over the lazy dog │
│ Typed:  the quick                                   │
│         ^^^^^^^^^^                                  │
│                                                     │
│ WPM: 45.2 │ Accuracy: 100% │ Time: 0:23           │
└─────────────────────────────────────────────────────┘
```

**Controls**:
- Type normally to match the target text
- `Backspace` to correct mistakes
- `Ctrl+C` to exit
- `Tab` to pause/resume

---

## Training Modes

### Arcade Mode

Progressive training through 100 carefully designed levels.

```bash
# Start at specific level
centotype play --level 15

# Continue from where you left off
centotype play --continue

# Practice a range of levels
centotype play --level 10-15
```

#### Level Categories

**Bronze Tier (Levels 1-20): Foundation**
- Basic vocabulary and common words
- Simple punctuation (periods, commas)
- Target: 40+ WPM, 95%+ accuracy

**Silver Tier (Levels 21-40): Proficiency**
- Mixed content with punctuation
- Numbers and basic symbols
- Target: 60+ WPM, 97%+ accuracy

**Gold Tier (Levels 41-60): Advanced**
- Complex symbols and technical terms
- Programming concepts
- Target: 80+ WPM, 98%+ accuracy

**Platinum Tier (Levels 61-80): Expert**
- Code snippets and patterns
- Advanced symbol combinations
- Target: 100+ WPM, 99%+ accuracy

**Diamond Tier (Levels 81-100): Mastery**
- Complex programming constructs
- Advanced patterns and edge cases
- Target: 130+ WPM, 99.5%+ accuracy

#### Example Level Progression

```bash
# Level 1: Basic words
"the quick brown fox jumps over the lazy dog and runs fast"

# Level 25: Mixed content
"The user's account (ID: 12345) has $67.89 remaining. Contact support@company.com for help."

# Level 50: Technical content
"function calculateTotal(items) { return items.reduce((sum, item) => sum + item.price, 0); }"

# Level 75: Advanced patterns
"const config: Config = { api: { timeout: 5000, retries: 3 }, db: { host: 'localhost' } };"

# Level 100: Complex constructs
"impl<T: Clone + Send + Sync> AsyncProcessor<T> for DataPipeline where T: Serialize + DeserializeOwned"
```

### Drill Mode

Focused practice on specific skill areas.

```bash
# Practice symbols
centotype drill --category symbols

# Practice numbers
centotype drill --category numbers

# Practice code patterns
centotype drill --category code

# Practice bracket matching
centotype drill --category brackets

# Work on your weakest keys
centotype drill --weak-keys
```

#### Drill Categories

**Symbols Drill**
- Focus: `!@#$%^&*(){}[]<>?/\|~`
- Content: Symbol-heavy sequences and combinations
- Goal: Develop muscle memory for special characters

**Numbers Drill**
- Focus: `0123456789`
- Content: Number sequences, dates, IDs, calculations
- Goal: Improve number typing accuracy and speed

**Code Drill**
- Focus: Programming patterns and syntax
- Content: Function calls, variable declarations, operators
- Goal: Master common coding constructs

**Brackets Drill**
- Focus: `(){}[]<>`
- Content: Nested structures and matching pairs
- Goal: Accurate bracket typing and pairing

**Weak Keys Drill**
- Focus: Your personally difficult keys
- Content: Adaptive based on your error patterns
- Goal: Eliminate personal typing weaknesses

#### Drill Examples

```bash
# Symbols drill content
"!@#$%^&*() {}[]<>?/\|~ += -= *= /= %= <<= >>="

# Numbers drill content
"2024-09-27 14:30:15 UTC | ID: 789012 | Amount: $1,234.56"

# Code drill content
"if (condition && array.length > 0) { return obj.method(param1, param2); }"
```

### Endurance Mode

Build stamina and maintain accuracy over longer sessions.

```bash
# 15-minute endurance session
centotype endurance --duration 15

# 500-word target
centotype endurance --words 500

# Adaptive difficulty based on your performance
centotype endurance --adaptive
```

#### Endurance Features

**Progressive Fatigue Testing**
- Tracks performance degradation over time
- Identifies optimal practice session length
- Helps build typing stamina

**Consistency Metrics**
- Measures speed variance throughout session
- Tracks accuracy stability over time
- Provides fatigue analysis

**Adaptive Difficulty**
- Adjusts content difficulty based on real-time performance
- Maintains optimal challenge level
- Prevents overwhelming or boring content

### Placement Mode

Determines your optimal starting level.

```bash
centotype placement
```

The placement test:
1. Starts with Level 25 (mid-range assessment)
2. Adapts based on your performance
3. Tests accuracy, speed, and consistency
4. Recommends your ideal starting level

**Placement Algorithm**:
- **High Performance**: Moves to harder levels
- **Struggling**: Drops to easier levels
- **Consistent**: Fine-tunes around current level
- **Final Result**: Recommends level where you can achieve 95%+ accuracy

---

## Level Progression System

### Understanding the 100-Level System

Centotype's progression is mathematically designed for consistent difficulty increases:

#### Difficulty Scaling Formulas

**Symbol Density**: 5% → 30% (Level 1 → 100)
```
symbol_ratio = 5% + (tier - 1) × 2.5% + (tier_progress - 1) × 0.3%
```

**Number Density**: 3% → 20% (Level 1 → 100)
```
number_ratio = 3% + (tier - 1) × 1.7% + (tier_progress - 1) × 0.2%
```

**Content Length**: 300 → 3000 characters (Level 1 → 100)
```
content_length = 300 + (tier - 1) × 270 + (tier_progress - 1) × 30
```

### Tier System

#### Bronze Tier (Levels 1-20)
**Goal**: Foundation Building
- **Requirements**: 40+ WPM, 95%+ accuracy
- **Content**: Basic vocabulary, simple punctuation
- **Focus**: Establish proper typing habits
- **Typical Practice Time**: 2-4 weeks

**Level Milestones**:
- Level 5: Comfortable with common words
- Level 10: Basic punctuation mastery
- Level 15: Consistent 35+ WPM
- Level 20: Ready for mixed content

#### Silver Tier (Levels 21-40)
**Goal**: Proficiency Development
- **Requirements**: 60+ WPM, 97%+ accuracy
- **Content**: Mixed content, numbers, basic symbols
- **Focus**: Speed development with accuracy
- **Typical Practice Time**: 3-6 weeks

**Level Milestones**:
- Level 25: Comfortable with numbers
- Level 30: Basic symbols integration
- Level 35: Consistent 50+ WPM
- Level 40: Ready for technical content

#### Gold Tier (Levels 41-60)
**Goal**: Advanced Skills
- **Requirements**: 80+ WPM, 98%+ accuracy
- **Content**: Technical terms, complex symbols
- **Focus**: Professional typing competency
- **Typical Practice Time**: 4-8 weeks

**Level Milestones**:
- Level 45: Technical vocabulary comfort
- Level 50: Complex symbol proficiency
- Level 55: Code-like content adaptation
- Level 60: Ready for expert challenges

#### Platinum Tier (Levels 61-80)
**Goal**: Expert Performance
- **Requirements**: 100+ WPM, 99%+ accuracy
- **Content**: Code snippets, advanced patterns
- **Focus**: Developer-level typing skills
- **Typical Practice Time**: 6-12 weeks

**Level Milestones**:
- Level 65: Comfortable with code syntax
- Level 70: Advanced pattern recognition
- Level 75: Consistent 90+ WPM
- Level 80: Ready for mastery challenges

#### Diamond Tier (Levels 81-100)
**Goal**: Mastery Achievement
- **Requirements**: 130+ WPM, 99.5%+ accuracy
- **Content**: Complex programming constructs
- **Focus**: Elite typing performance
- **Typical Practice Time**: 3-6 months

**Level Milestones**:
- Level 85: Advanced language constructs
- Level 90: Complex nested patterns
- Level 95: Near-perfect accuracy
- Level 100: Typing mastery achieved

### Advancement Criteria

To advance to the next level, you must achieve:

**Minimum Requirements**:
- Complete the level within time limit
- Meet accuracy threshold for current tier
- Demonstrate consistency (low speed variance)

**Recommended Advancement**:
- Exceed tier accuracy requirements by 1%
- Complete level with 10% speed buffer
- Show improvement from previous attempts

**Example Advancement Decision**:
```
Level 35 Requirements:
- Minimum: 50 WPM, 97% accuracy
- Recommended: 55 WPM, 98% accuracy
- Your Performance: 53 WPM, 97.8% accuracy
- Decision: ✅ Advanced to Level 36
```

---

## Performance Tracking

### Core Metrics

#### Words Per Minute (WPM)

**Raw WPM**: Pure typing speed without accuracy adjustment
```
Raw WPM = (Characters Typed ÷ 5) ÷ (Time in Minutes)
```

**Effective WPM**: Accuracy-adjusted speed (most important metric)
```
Effective WPM = Raw WPM × (Accuracy ÷ 100)
```

**Example**:
- Raw WPM: 60
- Accuracy: 95%
- Effective WPM: 60 × 0.95 = 57 WPM

#### Accuracy

**Character Accuracy**: Percentage of correctly typed characters
```
Accuracy = (Correct Characters ÷ Total Characters) × 100
```

**Error Types Tracked**:
- **Substitutions**: Wrong character typed
- **Insertions**: Extra character added
- **Deletions**: Character skipped
- **Transpositions**: Characters swapped

#### Consistency

**Speed Consistency**: Measures typing rhythm stability
```
Consistency = 100 - (Speed Variance ÷ Average Speed) × 100
```

**High Consistency (90%+)**: Steady, even typing rhythm
**Low Consistency (<70%)**: Erratic, bursts and pauses

#### Skill Index

**Overall Proficiency Rating**: 0-1000 scale combining all metrics
```
Skill Index = (Effective WPM × 5) + (Accuracy × 2) + (Consistency × 3)
```

**Skill Index Ranges**:
- **0-200**: Beginner
- **201-400**: Novice
- **401-600**: Intermediate
- **601-800**: Advanced
- **801-1000**: Expert

### Viewing Your Statistics

#### Basic Statistics
```bash
centotype stats
```

**Output Example**:
```
╭─ Centotype Statistics ──────────────────────────────╮
│ Sessions Completed: 156                             │
│ Total Practice Time: 23h 45m                        │
│ Current Level: 42 (Gold Tier)                       │
│                                                     │
│ Best Performance:                                   │
│   WPM (Effective): 78.4                            │
│   Accuracy: 98.7%                                  │
│   Skill Index: 745                                 │
│                                                     │
│ Recent Trend (7 days):                             │
│   Average WPM: 72.1 (+3.2)                        │
│   Average Accuracy: 97.9% (+0.8%)                 │
│   Sessions: 12                                      │
╰─────────────────────────────────────────────────────╯
```

#### Detailed Statistics
```bash
centotype stats --detailed
```

**Additional Information**:
- Per-key accuracy breakdown
- Error pattern analysis
- Time-of-day performance trends
- Level progression history
- Weak character identification

#### Level-Specific Statistics
```bash
centotype stats --level 25
```

**Level Analysis**:
- Attempts on this level
- Best performance achieved
- Improvement trend
- Time to master
- Comparison to tier average

### Performance Graphs

#### Progress Tracking
```bash
centotype stats --graph wpm
```

**Available Graphs**:
- WPM progression over time
- Accuracy trend analysis
- Consistency improvement
- Error rate reduction
- Session duration patterns

#### Export Data
```bash
# Export to CSV for analysis
centotype export --format csv --days 30

# Export detailed session data
centotype export --format json --sessions 100
```

**Use Cases**:
- Track long-term improvement
- Analyze practice patterns
- Identify optimal practice times
- Monitor fatigue effects

---

## Configuration and Customization

### Viewing Current Configuration

```bash
centotype config --show
```

**Output Example**:
```
╭─ Centotype Configuration ───────────────────────────╮
│ Theme: dark                                         │
│ Keyboard Layout: qwerty                             │
│ Sound Effects: enabled                              │
│ Animations: enabled                                 │
│ Auto-advance: true                                  │
│ Strict Mode: false                                  │
│                                                     │
│ Performance:                                        │
│   High Performance Mode: false                     │
│   Input Latency Optimization: true                 │
│   Memory Limit: 50MB                               │
╰─────────────────────────────────────────────────────╯
```

### Visual Customization

#### Theme Selection
```bash
# Dark theme (recommended for extended practice)
centotype config --set theme dark

# Light theme
centotype config --set theme light

# Auto theme (follows system)
centotype config --set theme auto
```

#### Color Customization
```bash
# Text colors
centotype config --set colors.correct green
centotype config --set colors.incorrect red
centotype config --set colors.pending white

# Background colors
centotype config --set colors.background black
centotype config --set colors.highlight blue
```

#### Display Options
```bash
# Show/hide elements
centotype config --set show.wpm true
centotype config --set show.accuracy true
centotype config --set show.timer true
centotype config --set show.progress true

# Progress indicator style
centotype config --set progress.style bar        # Progress bar
centotype config --set progress.style percentage # Percentage only
centotype config --set progress.style both       # Both bar and percentage
```

### Keyboard and Input

#### Keyboard Layout
```bash
# Set your keyboard layout
centotype config --set layout qwerty    # Standard US
centotype config --set layout qwertz    # German/Austrian
centotype config --set layout azerty    # French/Belgian
centotype config --set layout dvorak    # Dvorak layout
```

#### Input Behavior
```bash
# Correction policy
centotype config --set correction.policy strict     # No backspace allowed
centotype config --set correction.policy lenient    # Backspace allowed
centotype config --set correction.policy adaptive   # Adaptive based on level

# Timing sensitivity
centotype config --set timing.sensitivity high      # Precise timing
centotype config --set timing.sensitivity normal    # Standard timing
centotype config --set timing.sensitivity relaxed   # Generous timing
```

### Audio Settings

#### Sound Effects
```bash
# Enable/disable sounds
centotype config --set sound.enabled true

# Individual sound controls
centotype config --set sound.keystroke true     # Key press sounds
centotype config --set sound.error true         # Error sounds
centotype config --set sound.completion true    # Level completion
centotype config --set sound.milestone true     # Achievement sounds

# Volume control
centotype config --set sound.volume 0.7         # 70% volume
```

#### Audio Feedback Types
```bash
# Keystroke sounds
centotype config --set sound.keystroke.type click    # Click sound
centotype config --set sound.keystroke.type typewriter # Typewriter sound
centotype config --set sound.keystroke.type modern    # Modern sound

# Error sounds
centotype config --set sound.error.type beep      # Simple beep
centotype config --set sound.error.type buzz      # Buzz sound
centotype config --set sound.error.type none      # Visual only
```

### Performance Settings

#### High Performance Mode
```bash
# Enable for slower systems
centotype config --set performance.high_performance true

# Reduce visual effects
centotype config --set performance.effects minimal

# Optimize memory usage
centotype config --set performance.memory_mode efficient
```

#### Input Latency Optimization
```bash
# Platform-specific optimizations
centotype config --set performance.input_optimization auto

# Manual optimization
centotype config --set performance.input_optimization aggressive
centotype config --set performance.render_rate 60fps
```

### Training Preferences

#### Auto-Advancement
```bash
# Automatically advance levels
centotype config --set training.auto_advance true
centotype config --set training.advance_threshold 98  # 98% accuracy required

# Retry failed levels
centotype config --set training.auto_retry true
centotype config --set training.retry_limit 3
```

#### Session Settings
```bash
# Default session length
centotype config --set session.default_duration 300  # 5 minutes

# Break reminders
centotype config --set session.break_reminder true
centotype config --set session.break_interval 1800   # 30 minutes

# Session goals
centotype config --set session.daily_goal 30         # 30 minutes daily
```

### Resetting Configuration

```bash
# Reset all settings to defaults
centotype config --reset

# Reset specific category
centotype config --reset ui
centotype config --reset audio
centotype config --reset performance

# Reset specific setting
centotype config --reset theme
```

---

## Advanced Features

### Custom Content

#### Create Custom Lessons
```bash
# Practice with your own text file
centotype custom --file my-code.rs

# Practice specific text
centotype custom --text "Your custom content here"

# Practice from clipboard
centotype custom --clipboard
```

#### Content Filters
```bash
# Filter content by type
centotype play --level 25 --filter code       # Only code content
centotype play --level 25 --filter prose      # Only prose content
centotype play --level 25 --filter mixed      # Mixed content

# Filter by difficulty
centotype play --level 25 --min-difficulty 0.6
centotype play --level 25 --max-difficulty 0.8
```

### Competition Features

#### Online Leaderboards
```bash
# Submit scores to leaderboard (optional)
centotype config --set online.leaderboard true

# View leaderboards
centotype leaderboard --level 50
centotype leaderboard --tier gold
centotype leaderboard --global
```

#### Race Mode
```bash
# Race against your best time
centotype race --level 25 --target personal

# Race against global average
centotype race --level 25 --target global

# Race against specific WPM
centotype race --level 25 --target 75
```

### Analytics and Insights

#### Detailed Error Analysis
```bash
# View error patterns
centotype analyze --errors

# Character-specific analysis
centotype analyze --character q  # Analyze 'q' key performance

# Time-based analysis
centotype analyze --time-of-day
centotype analyze --day-of-week
```

#### Improvement Suggestions
```bash
# Get personalized recommendations
centotype recommend

# Specific improvement areas
centotype recommend --focus speed
centotype recommend --focus accuracy
centotype recommend --focus consistency
```

**Example Recommendations**:
```
╭─ Improvement Recommendations ───────────────────────╮
│ Primary Focus: Accuracy                             │
│                                                     │
│ 1. Practice 'q' key (73% accuracy)                 │
│    Recommended: centotype drill --character q      │
│                                                     │
│ 2. Work on symbol combinations                      │
│    Recommended: centotype drill --category symbols │
│                                                     │
│ 3. Slow down for better accuracy                   │
│    Target: 5 WPM slower, 3% better accuracy       │
╰─────────────────────────────────────────────────────╯
```

### Accessibility Features

#### Visual Accessibility
```bash
# High contrast mode
centotype config --set accessibility.high_contrast true

# Large text mode
centotype config --set accessibility.large_text true

# Color blind support
centotype config --set accessibility.colorblind deuteranopia
```

#### Motor Accessibility
```bash
# Slower typing accommodation
centotype config --set accessibility.timing_adjustment 1.5  # 50% more time

# Single-hand mode
centotype config --set accessibility.single_hand true

# Custom key mapping
centotype config --set accessibility.key_mapping custom.json
```

### Data Management

#### Backup and Restore
```bash
# Backup your data
centotype backup --export backup.json

# Restore from backup
centotype restore --import backup.json

# Sync across devices (cloud storage)
centotype sync --service dropbox
```

#### Privacy Controls
```bash
# Disable telemetry
centotype config --set privacy.telemetry false

# Clear personal data
centotype privacy --clear-data

# Export personal data
centotype privacy --export-data
```

---

## Tips for Success

### Effective Practice Strategies

#### 1. Consistency Over Intensity
- **Best**: 15-30 minutes daily
- **Good**: 45 minutes every other day
- **Avoid**: 2-hour weekend sessions only

#### 2. Accuracy First Philosophy
- Always prioritize accuracy over speed
- Centotype heavily penalizes corrections
- Clean typing builds proper muscle memory
- Speed naturally follows accuracy

#### 3. Progressive Challenge
- Master each level before advancing
- Don't skip levels to reach higher numbers
- Each level builds on previous skills
- Steady progression is faster than rushing

#### 4. Regular Breaks
- Take 5-minute breaks every 20-30 minutes
- Stand, stretch, and rest your hands
- Fatigue leads to bad habits and injuries
- Fresh practice is more effective

### Specific Training Tips

#### For Speed Improvement
```bash
# Practice with metronome-like timing
centotype play --level 20 --pace steady

# Focus on common word patterns
centotype drill --category common-words

# Use burst training
centotype drill --short-bursts
```

#### For Accuracy Improvement
```bash
# Slow down deliberately
centotype play --level 15 --max-wpm 40

# Practice problem characters
centotype drill --weak-keys

# Use strict mode (no corrections)
centotype config --set correction.policy strict
```

#### For Consistency
```bash
# Endurance training
centotype endurance --duration 10

# Focus on rhythm
centotype drill --category rhythm

# Monitor fatigue patterns
centotype analyze --fatigue
```

### Common Mistakes to Avoid

#### 1. Rushing Through Levels
- **Problem**: Skipping levels or advancing too quickly
- **Solution**: Master each level with 98%+ accuracy
- **Why**: Each level builds essential muscle memory

#### 2. Ignoring Error Patterns
- **Problem**: Not analyzing which keys cause problems
- **Solution**: Use `centotype analyze --errors` regularly
- **Why**: Targeted practice is more efficient

#### 3. Practicing When Tired
- **Problem**: Practicing when mentally or physically fatigued
- **Solution**: Practice when alert and focused
- **Why**: Tired practice reinforces bad habits

#### 4. Neglecting Posture
- **Problem**: Poor typing posture and hand position
- **Solution**: Maintain proper ergonomic setup
- **Why**: Good posture enables faster, more accurate typing

### Ergonomic Recommendations

#### Desk Setup
- Monitor at eye level
- Keyboard at elbow height
- Feet flat on floor
- Back supported

#### Hand Position
- Wrists straight, not bent
- Fingers curved over keys
- Light touch, don't pound keys
- Hands floating, not resting

#### Finger Placement
- Home row: ASDF (left), JKL; (right)
- Each finger responsible for specific keys
- Use proper fingering even if slower initially
- Thumb only for spacebar

---

## Keyboard Shortcuts

### Global Shortcuts

| Key | Action | Context |
|-----|---------|---------|
| `Ctrl+C` | Exit/Quit | Any screen |
| `Tab` | Pause/Resume | During typing |
| `Esc` | Cancel/Back | Menus and prompts |
| `Enter` | Confirm/Start | Menus and prompts |

### Typing Session Shortcuts

| Key | Action | Notes |
|-----|---------|-------|
| `Backspace` | Correct mistake | If correction policy allows |
| `Ctrl+R` | Restart level | Start over from beginning |
| `Ctrl+P` | Pause session | Excludes time from metrics |
| `Ctrl+Q` | Quick quit | Exit without saving |
| `Space` | Resume from pause | Continue paused session |

### Navigation Shortcuts

| Key | Action | Context |
|-----|---------|---------|
| `Arrow Keys` | Navigate menus | Menu selection |
| `Page Up/Down` | Scroll content | Long text or results |
| `Home/End` | Jump to start/end | Menu or text navigation |
| `Ctrl+L` | Jump to level | Quick level selection |

### Analysis and Stats

| Key | Action | Notes |
|-----|---------|-------|
| `Ctrl+S` | Show statistics | Quick stats overlay |
| `Ctrl+A` | Analyze session | Detailed analysis |
| `Ctrl+H` | Help screen | Context-sensitive help |
| `Ctrl+D` | Debug info | Performance metrics |

### Configuration Shortcuts

| Key | Action | Effect |
|-----|---------|--------|
| `Ctrl+T` | Toggle theme | Switch dark/light |
| `Ctrl+M` | Toggle sound | Enable/disable audio |
| `Ctrl+F` | Toggle fullscreen | Immersive mode |
| `Ctrl+O` | Options menu | Quick settings |

---

## Understanding Your Results

### Session Results Screen

After completing a level, you'll see detailed results:

```
╭─ Session Complete: Level 25 ────────────────────────╮
│                                                     │
│ Performance Summary                                 │
│ ─────────────────────                              │
│ WPM (Raw):        72.4                             │
│ WPM (Effective):  68.1                             │
│ Accuracy:         94.2%                            │
│ Consistency:      87.6%                            │
│ Session Time:     4:23                             │
│                                                     │
│ Skill Assessment                                    │
│ ─────────────────                                  │
│ Skill Index:      682                              │
│ Grade:            B+                               │
│ Stars:            ★★★☆☆                           │
│ Tier Progress:    Silver (Level 25/40)             │
│                                                     │
│ Areas for Improvement                              │
│ ──────────────────────                             │
│ • Work on 'q' and 'x' keys (lower accuracy)       │
│ • Practice symbol combinations                      │
│ • Focus on maintaining speed consistency            │
│                                                     │
│ Next Recommendation                                │
│ ────────────────────                               │
│ ✓ Ready for Level 26                               │
│   Alternative: Practice symbols drill              │
╰─────────────────────────────────────────────────────╯
```

### Interpreting Your Metrics

#### WPM Analysis
- **Raw WPM**: Your actual typing speed
- **Effective WPM**: Speed adjusted for accuracy (most important)
- **Target WPM**: Varies by tier level
- **Improvement**: Compare to your personal best

#### Accuracy Breakdown
- **Overall Accuracy**: Percentage of correct characters
- **Character Accuracy**: Per-key performance
- **Error Types**: Substitution, insertion, deletion patterns
- **Improvement Areas**: Specific keys or combinations to practice

#### Consistency Rating
- **90%+ Excellent**: Steady, even typing rhythm
- **80-89% Good**: Minor speed variations
- **70-79% Fair**: Noticeable speed inconsistency
- **<70% Needs Work**: Erratic typing pattern

#### Grade System
- **A (90-100%)**: Exceptional performance
- **B (80-89%)**: Good performance
- **C (70-79%)**: Average performance
- **D (60-69%)**: Below average
- **F (<60%)**: Needs significant improvement

### Star Rating System

Stars are awarded based on comprehensive performance:

- ⭐ **1 Star**: Completed level
- ⭐⭐ **2 Stars**: Met accuracy target
- ⭐⭐⭐ **3 Stars**: Met speed and accuracy targets
- ⭐⭐⭐⭐ **4 Stars**: Exceeded targets with good consistency
- ⭐⭐⭐⭐⭐ **5 Stars**: Outstanding performance across all metrics

### Skill Index Calculation

The Skill Index (0-1000) combines multiple factors:

```
Skill Index = (Effective WPM × 5) + (Accuracy × 2) + (Consistency × 3) + Bonus Points
```

**Bonus Points**:
- +50: Perfect accuracy (100%)
- +25: High consistency (95%+)
- +15: Speed improvement from last attempt
- +10: First attempt success
- +5: Long session completion

**Skill Index Ranges**:
- **0-200**: Beginner - Learning basics
- **201-400**: Novice - Building skills
- **401-600**: Intermediate - Developing proficiency
- **601-800**: Advanced - Strong typing skills
- **801-1000**: Expert - Elite performance

### Improvement Tracking

#### Progress Indicators
- **Level Advancement**: Ready for next level
- **Tier Progress**: Position within current tier
- **Skill Trend**: Improving, stable, or declining
- **Personal Records**: Best WPM, accuracy, consistency

#### Comparative Analysis
- **Personal Best**: Your best performance on this level
- **Tier Average**: Average performance for your tier
- **Global Average**: Average across all users
- **Recommended Target**: Next improvement goal

---

## Troubleshooting

### Common Issues

#### Installation Problems

**Issue**: `command not found: centotype`
```bash
# Solution 1: Check PATH
echo $PATH
which centotype

# Solution 2: Reinstall
cargo install centotype --force

# Solution 3: Manual installation
# Download binary and add to PATH
```

**Issue**: Rust/Cargo not installed
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

#### Performance Issues

**Issue**: High input latency or sluggish response
```bash
# Check performance
centotype benchmark

# Enable performance mode
centotype config --set performance.high_performance true

# Reduce visual effects
centotype config --set performance.effects minimal
```

**Issue**: Memory usage warnings
```bash
# Check memory usage
centotype sysinfo

# Reduce cache size
centotype config --set performance.cache_size 25

# Enable memory optimization
centotype config --set performance.memory_mode efficient
```

#### Terminal Compatibility

**Issue**: Display rendering problems
```bash
# Check terminal capabilities
centotype check

# Use compatibility mode
centotype play --level 1 --compat-mode

# Update terminal settings
export TERM=xterm-256color
```

**Issue**: Colors not displaying correctly
```bash
# Test color support
centotype test-colors

# Force color mode
centotype config --set display.force_colors true

# Use monochrome mode
centotype config --set display.monochrome true
```

#### Audio Problems

**Issue**: Sound effects not working
```bash
# Check audio configuration
centotype config --show | grep sound

# Test audio
centotype test-audio

# Verify system audio
# Linux: Check PulseAudio/ALSA
# macOS: Check system volume
# Windows: Check audio drivers
```

### Performance Optimization

#### For Slower Systems
```bash
# Enable high-performance mode
centotype config --set performance.high_performance true

# Reduce visual effects
centotype config --set display.animations false
centotype config --set display.effects minimal

# Optimize memory usage
centotype config --set performance.memory_mode efficient
centotype config --set performance.cache_size 15
```

#### For Better Responsiveness
```bash
# Optimize input processing
centotype config --set performance.input_optimization aggressive

# Increase priority (Linux/macOS)
sudo nice -n -10 centotype play --level 1

# Use dedicated mode
centotype config --set performance.dedicated_mode true
```

### Getting Help

#### Built-in Help
```bash
# General help
centotype help

# Command-specific help
centotype play --help
centotype config --help
centotype stats --help

# Version information
centotype --version
```

#### System Information
```bash
# System compatibility check
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
centotype play --level 1

# Performance profiling
centotype profile --duration 60

# Export debug data
centotype debug --export debug-info.json
```

### Support Resources

- **GitHub Issues**: [Report bugs and request features](https://github.com/rfxlamia/centotype/issues)
- **Documentation**: Complete guides in `/docs`
- **Community**: [GitHub Discussions](https://github.com/rfxlamia/centotype/discussions)
- **Performance**: Reference the Performance Guide for optimization

---

## Conclusion

Centotype is designed to be your comprehensive typing training companion. Whether you're a beginner looking to improve basic typing skills or an advanced user aiming for competitive performance, the 100-level progression system provides a clear path to mastery.

### Remember the Key Principles:

1. **Accuracy First**: Clean typing beats fast corrections
2. **Consistent Practice**: Regular short sessions are more effective than long irregular ones
3. **Progressive Challenge**: Master each level before advancing
4. **Ergonomic Awareness**: Maintain proper posture and hand position
5. **Patient Persistence**: Typing mastery takes time and consistent effort

### Your Journey Ahead:

- **Bronze Tier**: Build foundational skills
- **Silver Tier**: Develop speed with accuracy
- **Gold Tier**: Master technical content
- **Platinum Tier**: Achieve expert performance
- **Diamond Tier**: Reach typing mastery

Start your journey today with `centotype play --level 1` and begin building the typing skills that will serve you throughout your career!

**Happy typing!** 🚀