# PRD — Centotype CLI Typing Trainer v2.0

## Executive Summary
Centotype is a CLI-based typing trainer built in Rust, featuring 100 progressive difficulty levels targeting developers, technical writers, and competitive typists. The product focuses on deterministic measurement, granular feedback, and realistic practice scenarios (code, prose, numbers, symbols).

**Status**: Revision v2.0 addressing critical gaps identified in audit (Security, Budget, Realistic Timeline, Measurable Success Criteria)

---

## 1. Goals & Objectives

### 1.1 Primary Goals
1. **Precision-first training**: Scoring system rewards accuracy with strict penalties for careless corrections
2. **Granular skill measurement**: Per-character, per-token, per-error-class insights
3. **Progressive mastery**: 100-level curve with consistent, measurable difficulty progression
4. **Deterministic & fair**: Reproducible scoring logic without uncontrolled RNG
5. **CLI-optimized**: Fast, lightweight, offline-capable, scriptable

### 1.2 Success Definition
- **Level 100 Mastery Criteria** (replacing subjective "nyaris mustahil"):
  - WPM ≥ 130 (effective, accuracy-adjusted)
  - Accuracy ≥ 99.5%
  - Error severity score ≤ 3 (weighted: transposition=3, substitution=2, insertion=1, deletion=1)
  - Completion time ≤ 120 seconds for 3000+ character mixed content
  - Zero backspace corrections after 80% completion point

### 1.3 Non-Goals (MVP v1.0)
- GUI interface, cloud sync/leaderboards, biometric tracking, auto-detect keyboard layouts
- Real-time multiplayer, voice integration, mobile apps

---

## 2. Key Performance Indicators (KPIs)

### 2.1 Technical Performance (Validated Targets)
**NOTE**: Targets revised based on prototype validation requirements
- **Startup time**: P95 < 200ms (revised from unrealistic 80ms)
- **Input latency**: P99 < 25ms (revised from unrealistic 10ms, pending cross-platform validation)
- **Render performance**: P95 < 33ms (30fps equivalent)
- **Memory usage**: < 50MB RSS during active session
- **Crash rate**: < 0.01% per session (revised from per 1000 sessions)

### 2.2 User Engagement
- **D7 retention**: ≥ 30% for early adopters (baseline TBD via user research)
- **Skill progression**: Median users advance ≥ 1 tier (10 levels) within 2 weeks
- **Session completion**: ≥ 80% of started sessions completed (not abandoned)
- **Level 100 achievement**: ≥ 5% of users reach Level 90+ within 3 months

### 2.3 Quality Metrics
- **Accuracy improvement**: +5% median accuracy within 14 days of consistent use
- **Speed improvement**: +10 WPM median improvement within 30 days
- **Error reduction**: 50% reduction in error rate for practiced character classes

---

## 3. Scope & Features

### 3.1 In-Scope (MVP v1.0)
- **Core Modes**: Arcade (100 levels), Drill (focused practice), Endurance (fatigue testing)
- **Scoring System**: WPM, accuracy, error taxonomy, combo multipliers, skill index (0-1000)
- **Content**: Curated text corpus (ID/EN), dynamic generation with deterministic seeds
- **Persistence**: Local JSON profile storage, configuration management
- **Layouts**: QWERTY (primary), QWERTZ/AZERTY (secondary)
- **Platforms**: Linux, macOS, Windows (x86_64, ARM64 where applicable)

### 3.2 Phase 2 (v1.1-1.3)
- Advanced drill recommendations, visual heatmaps, CSV export
- Adaptive difficulty adjustment, custom layout support
- User content import, basic plugin system

### 3.3 Explicit Out-of-Scope
- Cloud storage, multiplayer features, commercial licensing
- Non-Latin scripts, IME integration, accessibility beyond WCAG AA
- Anti-cheat beyond basic input validation

---

## 4. User Personas

### 4.1 Primary: Dev Specialist
- **Profile**: Backend/systems developers, 3-5 years experience
- **Needs**: Code-specific practice (Rust/TypeScript/Python), symbol proficiency, bracket balancing
- **Pain Points**: Slow symbol input, context switching between prose and code
- **Success Metrics**: 90+ WPM on mixed code content, <2% error rate on symbols

### 4.2 Secondary: Technical Writer  
- **Profile**: Documentation specialists, content creators
- **Needs**: High-volume text input, sustained accuracy, minimal backtracking
- **Pain Points**: Fatigue during long sessions, inconsistent accuracy
- **Success Metrics**: Maintain 95%+ accuracy during 20+ minute sessions

### 4.3 Tertiary: Speed Competitor
- **Profile**: Competitive typing enthusiasts, leaderboard focused
- **Needs**: Precise measurement, challenging content, achievement tracking
- **Pain Points**: Inconsistent difficulty curves, unclear scoring
- **Success Metrics**: Level 100 completion, consistent S-grade performance

---

## 5. Functional Requirements

### 5.1 Core Training Modes

#### 5.1.1 Arcade Mode
- **Requirement**: 100 sequential levels with linear progression unlock
- **Level Structure**: 10 tiers × 10 levels, increasing complexity per tier
- **Session Length**: 60-180 seconds per level (tier-dependent)  
- **Scoring**: Global skill index, star rating (0-3), grade system (S/A/B/C/D)
- **Progression**: Must achieve C-grade minimum to unlock next level

#### 5.1.2 Drill Mode
- **Requirement**: Focused practice on specific skill categories
- **Categories**: Numbers, punctuation, symbols, camelCase, snake_case, operators
- **Duration**: 5-15 minute sessions with repetition tracking
- **Adaptive**: Automatically increase difficulty based on performance

#### 5.1.3 Endurance Mode
- **Requirement**: Extended sessions (10-30 minutes) measuring fatigue impact
- **Metrics**: WPM/accuracy degradation over time, consistency tracking
- **Recovery**: Track performance restoration after breaks

### 5.2 Content Generation System

#### 5.2.1 Static Corpus Management
- **Requirement**: Curated text collections for Indonesian and English
- **Categories**: Technical terms, common phrases, code snippets per tier
- **Quality Control**: Manual review, profanity filtering, readability scoring

#### 5.2.2 Dynamic Generator
- **Requirement**: Parameterized content creation with consistency guarantees
- **Parameters**: Symbol ratio, number density, language mixing, length targets
- **Determinism**: Seed-based reproducible output for level consistency
- **Validation**: Automated difficulty scoring to ensure tier compliance

### 5.3 Scoring & Assessment

#### 5.3.1 Core Metrics
- **WPM Calculation**: (correct_characters / 5) / (time_minutes), adjusted for accuracy
- **Accuracy**: (total_characters - errors) / total_characters × 100%
- **Error Classification**: Substitution, insertion, deletion, transposition (Damerau-Levenshtein)

#### 5.3.2 Advanced Scoring
- **Combo System**: Consecutive correct characters, exponential multiplier
- **Latency Penalty**: Idle time > 500ms increments penalty counter
- **Backspace Cost**: Each correction reduces combo and adds penalty
- **Skill Index Formula**: SI = clamp((EffWPM × ComboMult × TierWeight) - PenaltyTotal, 0, 1000)

### 5.4 User Interface Requirements

#### 5.4.1 CLI Experience
- **Command Structure**: `centotype [play|drill|endurance|stats] [options]`
- **Interactive Mode**: Arrow/vim-style navigation, ESC/q quit, p pause
- **Visual Feedback**: Progress bars, real-time WPM/accuracy, error highlighting
- **Accessibility**: WCAG AA contrast ratios, colorblind-safe themes

#### 5.4.2 Configuration Management
- **Location**: `~/.config/centotype/config.toml`
- **Settings**: Layout, language, visual theme, sound alerts, telemetry opt-in
- **Validation**: Schema validation with helpful error messages for invalid configs

---

## 6. Non-Functional Requirements

### 6.1 Performance Requirements

#### 6.1.1 Response Time
- **Input Processing**: P99 keystroke processing < 25ms (validated via prototype)
- **Screen Refresh**: P95 render time < 33ms (30fps target)
- **Startup Time**: P95 cold start < 200ms (measured on reference hardware)

#### 6.1.2 Resource Utilization
- **Memory**: Maximum 50MB RSS during active typing session
- **CPU**: P95 CPU usage < 10% on 2-core systems during normal operation
- **Storage**: Application binary < 15MB, profile data < 1MB per user

### 6.2 Reliability Requirements

#### 6.2.1 Error Handling
- **Crash Rate**: < 0.01% sessions ending in unhandled panic
- **Data Integrity**: 100% profile data consistency with atomic writes
- **Recovery**: Automatic session state recovery after unexpected termination

#### 6.2.2 Input Validation
- **Terminal Compatibility**: Graceful degradation on limited terminals
- **Key Sequence Handling**: Proper escape sequence parsing and validation
- **Buffer Management**: Protection against buffer overflow attacks

### 6.3 Security Requirements

#### 6.3.1 Input Security
- **Injection Protection**: Sanitization of all user input, terminal escape sequence filtering
- **File System Access**: Restricted to documented configuration and profile directories
- **Process Isolation**: No shell execution, network access only for telemetry (opt-in)

#### 6.3.2 Data Protection
- **Local Storage**: Profile encryption for sensitive practice content (optional)
- **Telemetry**: Anonymization of all collected performance data
- **Privacy**: No keystroke logging beyond immediate processing window

### 6.4 Platform Compatibility

#### 6.4.1 Operating Systems
- **Primary**: Linux (Ubuntu 20.04+), macOS (10.15+), Windows 10+
- **Terminal Support**: xterm, gnome-terminal, iTerm2, Windows Terminal, cmd.exe
- **Architecture**: x86_64 (primary), ARM64 (macOS/Linux)

### 6.5 Accessibility Requirements

#### 6.5.1 Visual Accessibility
- **Contrast**: WCAG AA compliance (4.5:1 normal text, 3:1 large text)
- **Color Independence**: No information conveyed through color alone
- **Font Support**: Monospace font requirement with clear character distinction

### 6.6 Scalability Requirements

#### 6.6.1 Content Scaling
- **Level Generation**: Support for 1000+ generated levels without performance degradation
- **Corpus Size**: Handle text databases up to 100MB without memory issues
- **User Data**: Profile storage scalable to 10,000+ completed levels per user

---

## 7. Architecture

### 7.1 System Architecture

#### 7.1.1 Core Modules (Rust Crates)
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

#### 7.1.2 Data Flow
1. **Initialization**: Load config → validate terminal → initialize corpus
2. **Session**: Generate content → enter raw mode → process input loop
3. **Scoring**: Real-time diff → error classification → metric calculation  
4. **Persistence**: Atomic profile updates → graceful TTY restoration

### 7.2 Input Processing Pipeline

#### 7.2.1 Keystroke Handling
- **Raw Mode**: Direct terminal input bypass canonical processing
- **Event Translation**: Map platform-specific keycodes to common events
- **Timestamp Recording**: Nanosecond precision for latency measurement
- **Buffer Management**: Circular buffer for input history and correction tracking

#### 7.2.2 Error Detection Algorithm
- **Real-time Diff**: Character-by-character comparison with target text
- **Transposition Detection**: Damerau-Levenshtein with sliding window (N=5)
- **Correction Tracking**: Link backspaces to original errors for penalty calculation

### 7.3 Content Generation Architecture

#### 7.3.1 Static Content System  
- **Corpus Storage**: JSON files with metadata (difficulty, category, language)
- **Indexing**: Pre-computed difficulty scores, character class histograms
- **Caching**: LRU cache for frequently accessed content

#### 7.3.2 Dynamic Generator
- **Template Engine**: Parameterized text generation with constraint solving
- **Seeding**: Deterministic pseudo-random generation for level consistency
- **Validation Pipeline**: Automated difficulty verification, readability checking

### 7.4 Scoring Engine

#### 7.4.1 Real-time Calculation
- **Incremental Updates**: Streaming WPM/accuracy calculation
- **Error Weighting**: Configurable penalty matrix for different error types
- **Combo Tracking**: Exponential decay function for sustained accuracy rewards

### 7.5 Persistence Layer

#### 7.5.1 Profile Management
- **Format**: JSON with versioned schema for forward compatibility
- **Atomicity**: Write-rename pattern to prevent corruption
- **Compression**: Optional gzip compression for large profiles
- **Migration**: Automatic schema updates with fallback

---

## 8. Acceptance Tests

### 8.1 Core Functionality Tests

#### 8.1.1 Basic Operation
```
GIVEN: Fresh installation on target platform
WHEN: User runs `centotype play --level 1`
THEN: 
  - Level 1 content loads within 200ms
  - All keystrokes register correctly
  - Session completes with valid score
  - Profile updates atomically
```

#### 8.1.2 Cross-Platform Compatibility
```
GIVEN: Installation on Linux/macOS/Windows
WHEN: User completes identical session (same seed)
THEN:
  - Scores match within 1% variance
  - Input latency meets P99 targets
  - Terminal restoration works correctly
```

### 8.2 Performance Validation Tests

#### 8.2.1 Latency Requirements
```
GIVEN: Reference hardware configuration
WHEN: Sustaining 10+ keystrokes per second
THEN:
  - P99 input processing < 25ms
  - No dropped keystrokes
  - Smooth visual feedback
```

#### 8.2.2 Resource Constraints
```
GIVEN: 30-minute endurance session
WHEN: Monitoring system resources
THEN:
  - Memory usage remains < 50MB
  - CPU usage P95 < 10%
  - No memory leaks detected
```

### 8.3 Security Validation Tests

#### 8.3.1 Input Security
```
GIVEN: Malicious terminal escape sequences
WHEN: Input is processed through typing engine
THEN:
  - No arbitrary command execution
  - Terminal state remains stable
  - User data integrity preserved
```

#### 8.3.2 File System Security
```
GIVEN: Restricted file system permissions
WHEN: Application attempts file operations
THEN:
  - Access limited to designated directories
  - Graceful handling of permission errors
  - No unauthorized data exposure
```

### 8.4 Level Progression Tests

#### 8.4.1 Difficulty Curve Validation
```
GIVEN: Automated testing bot with fixed performance profile
WHEN: Bot completes levels 1-100
THEN:
  - Clear difficulty progression measurable
  - No sudden difficulty spikes > 20%
  - Level 100 meets mastery criteria definition
```

#### 8.4.2 Score Consistency
```
GIVEN: Identical text content and input sequence
WHEN: Repeated across multiple sessions/platforms
THEN:
  - Scores vary by < 0.5%
  - Error classification identical
  - Skill index calculation reproducible
```

### 8.5 User Experience Tests

#### 8.5.1 Accessibility Compliance
```
GIVEN: WCAG AA accessibility requirements
WHEN: Testing with accessibility tools
THEN:
  - Contrast ratios meet minimum standards
  - Color-independent information conveyance
  - Keyboard navigation fully functional
```

#### 8.5.2 Error Recovery
```
GIVEN: Unexpected interruption (Ctrl+C, terminal closure)
WHEN: Application restarts
THEN:
  - Profile data consistent
  - No corrupted configuration
  - Session can resume normally
```

---

## 9. Observability & Monitoring

### 9.1 Logging Strategy

#### 9.1.1 Log Levels
- **ERROR**: Crashes, data corruption, security violations
- **WARN**: Performance degradation, compatibility issues, recoverable errors
- **INFO**: Session start/end, level progression, configuration changes
- **DEBUG**: Detailed performance metrics, input processing details
- **TRACE**: Keystroke-level debugging (disabled by default)

#### 9.1.2 Log Management
- **Location**: `~/.local/share/centotype/logs/`
- **Rotation**: Daily rotation, 7-day retention by default
- **Privacy**: No keystroke content logged, only metadata and timings
- **Configuration**: User-controllable log levels via config file

### 9.2 Performance Metrics Collection

#### 9.2.1 Real-time Metrics
- **Input Latency**: P50/P95/P99 response times per session
- **Render Performance**: Frame timing, drop rate
- **Memory Usage**: RSS tracking, allocation patterns
- **Error Rates**: Crash frequency, input processing errors

#### 9.2.2 User Progress Analytics
- **Skill Development**: WPM/accuracy trends over time
- **Level Completion**: Success rates, retry patterns, abandonment points
- **Error Analysis**: Common mistake patterns, improvement areas

### 9.3 Telemetry (Opt-in Only)

#### 9.3.1 Collected Data (Anonymous)
- Performance metrics (latency, resource usage)
- Platform information (OS, terminal type, architecture)  
- Usage patterns (session length, level distribution)
- Error aggregates (frequency by category, no content)

#### 9.3.2 Privacy Safeguards
- **Explicit Opt-in**: Default disabled, clear consent required
- **No Personal Data**: No usernames, file paths, or typed content
- **Local Control**: Complete data export/deletion capabilities
- **Transparency**: Open source telemetry collection code

---

## 10. Risk Management & Mitigation

### 10.1 Technical Risks

#### 10.1.1 Performance Target Risk (HIGH/HIGH)
- **Risk**: P99 input latency < 25ms unachievable across all platforms
- **Impact**: Core user experience degraded, competitive disadvantage
- **Mitigation**: 
  - Prototype testing on target platforms before committing to metrics
  - Platform-specific optimizations and fallback modes
  - Transparent performance reporting to users
- **Owner**: Tech Lead

#### 10.1.2 Cross-Platform Compatibility Risk (HIGH/MEDIUM)
- **Risk**: Terminal input handling inconsistencies break core functionality
- **Impact**: Application unusable on significant user segments
- **Mitigation**:
  - Comprehensive terminal emulator testing matrix
  - Graceful degradation for limited terminals
  - Platform-specific testing in CI/CD pipeline
- **Owner**: QA Engineer

### 10.2 User Adoption Risks

#### 10.2.1 CLI Barrier Risk (HIGH/CRITICAL)
- **Risk**: Limited CLI comfort among target users prevents adoption
- **Impact**: Market size significantly smaller than projected
- **Mitigation**:
  - User research with non-technical typing enthusiasts
  - Improved installation and onboarding experience
  - Clear documentation and tutorials
- **Owner**: Product Manager

#### 10.2.2 Level 100 Frustration Risk (MEDIUM/HIGH)
- **Risk**: "Nearly impossible" difficulty level causes user churn
- **Impact**: Negative reviews, community backlash, retention issues
- **Mitigation**:
  - Implement adaptive difficulty suggestions
  - Clear skill progression feedback and intermediate goals
  - Community-driven level balancing feedback
- **Owner**: UX Designer

### 10.3 Security Risks

#### 10.3.1 Input Processing Vulnerabilities (LOW/CRITICAL)
- **Risk**: Terminal escape sequence injection or buffer overflow
- **Impact**: System compromise, data loss, security breach
- **Mitigation**:
  - Security audit of input processing code
  - Input sanitization and validation
  - Regular security dependency updates
- **Owner**: Security Engineer

### 10.4 Project Risks

#### 10.4.1 Resource Shortage Risk (HIGH/MEDIUM)
- **Risk**: Development team lacks sufficient Rust/terminal expertise
- **Impact**: Delayed delivery, technical debt, poor quality
- **Mitigation**:
  - Realistic timeline estimation with buffer allocation
  - External Rust expertise consultation
  - Incremental delivery with early feedback
- **Owner**: Project Manager

#### 10.4.2 Scope Creep Risk (MEDIUM/MEDIUM)
- **Risk**: 100 levels + multi-language support exceeds capacity
- **Impact**: Feature cuts, delayed release, incomplete implementation
- **Mitigation**:
  - Strict MVP scope enforcement
  - Phased feature delivery
  - Regular scope review meetings
- **Owner**: Product Manager

---

## 11. Timeline & Milestones

### 11.1 Development Phases

#### Phase 1: Foundation (6 weeks)
**Week 1-2: Architecture & Setup**
- Repository setup, CI/CD pipeline
- Core module structure implementation
- Terminal input/output foundation
- **Deliverables**: Basic CLI skeleton, input capture working

**Week 3-4: Core Engine**
- Scoring algorithm implementation
- Error detection and classification
- Content loading system
- **Deliverables**: Functional typing engine, basic scoring

**Week 5-6: MVP Features**  
- Level system implementation (Tiers 1-3)
- Profile persistence
- Basic UI polish
- **Deliverables**: Playable MVP with first 30 levels

#### Phase 2: Content & Polish (4 weeks)
**Week 7-8: Content Development**
- Complete 100-level content creation
- Dynamic generator implementation
- Multi-language corpus integration
- **Deliverables**: All levels playable, content validated

**Week 9-10: Testing & Optimization**
- Cross-platform testing
- Performance optimization
- Bug fixes and stability improvements
- **Deliverables**: Production-ready build

#### Phase 3: Release Preparation (2 weeks)
**Week 11: Distribution**
- Package creation (npm, cargo, releases)
- Documentation completion
- Installation testing
- **Deliverables**: Public release artifacts

**Week 12: Launch**
- Release announcement
- Community feedback collection
- Initial support and bug fixes
- **Deliverables**: v1.0 public release

### 11.2 Critical Dependencies
- **Rust expertise**: 2+ developers with systems programming experience
- **Content creation**: Technical writer for corpus development
- **Testing resources**: Access to diverse terminal environments
- **Design input**: UX consultation for accessibility and usability

### 11.3 Risk Buffers
- **Technical risk**: +2 weeks for performance optimization iterations
- **Content risk**: +1 week for difficulty balancing and validation
- **Testing risk**: +1 week for cross-platform compatibility issues

**Total Timeline: 12 weeks + 4 weeks buffer = 16 weeks (4 months)**

---

## 12. Budget & Resource Requirements

### 12.1 Development Costs

#### 12.1.1 Personnel (16 weeks)
- **Senior Rust Developer** (1.0 FTE): $120k annual × 16/52 = $36,923
- **Frontend/CLI Developer** (1.0 FTE): $100k annual × 16/52 = $30,769  
- **Technical Writer/Content** (0.5 FTE): $80k annual × 16/52 × 0.5 = $12,308
- **QA Engineer** (0.5 FTE): $90k annual × 16/52 × 0.5 = $13,846
- **Project Manager** (0.25 FTE): $110k annual × 16/52 × 0.25 = $8,462
- **Total Personnel**: $102,308

#### 12.1.2 Infrastructure & Tools
- **CI/CD Services** (GitHub Actions): $200/month × 4 = $800
- **Development Tools & Licenses**: $2,000
- **Testing Hardware/VMs**: $1,500
- **Domain & Hosting**: $500
- **Total Infrastructure**: $4,800

#### 12.1.3 External Services
- **Security Audit**: $15,000
- **Accessibility Testing**: $5,000
- **Legal Review** (open source licensing): $3,000
- **Total External**: $23,000

**Total Development Budget: $130,108**

### 12.2 Ongoing Operational Costs (Annual)

#### 12.2.1 Maintenance & Support
- **Maintenance Developer** (0.2 FTE): $100k × 0.2 = $20,000
- **Community Support** (0.1 FTE): $60k × 0.1 = $6,000
- **Infrastructure**: $2,400
- **Security Updates**: $5,000
- **Total Annual**: $33,400

### 12.3 Revenue Model Considerations
- **Open Source**: No direct revenue, community-driven development
- **Pro Version**: Advanced analytics, team features ($5-10/month)
- **Corporate Training**: Bulk licensing for companies ($1000-5000/year)
- **Break-even**: ~100 pro users or 5-10 corporate contracts

### 12.4 Cost Optimization
- **Open Source Strategy**: Community contributions for content and testing
- **Cloud Efficiency**: Usage-based infrastructure scaling
- **Automation**: Minimize manual support through good documentation

---

## 13. Rollout & Rollback Strategy

### 13.1 Rollout Plan

#### 13.1.1 Alpha Release (Week 10)
- **Audience**: Internal team, 5-10 friendly developers
- **Scope**: Core functionality, Levels 1-30, basic scoring
- **Success Criteria**: No critical bugs, positive initial feedback
- **Rollback**: Simple - remove release artifacts

#### 13.1.2 Beta Release (Week 11)
- **Audience**: 50-100 early adopters, typing enthusiasts
- **Scope**: Full feature set, all 100 levels, cross-platform
- **Success Criteria**: <1% crash rate, positive retention metrics
- **Rollback**: Coordinated - notify users, provide downgrade path

#### 13.1.3 Public Release (Week 12)
- **Audience**: General public via package managers
- **Scope**: Production-ready v1.0
- **Success Criteria**: Stable downloads, community adoption
- **Rollback**: Version management - previous version remains available

### 13.2 Rollback Procedures

#### 13.2.1 Technical Rollback
- **Package Managers**: Keep previous version available, update docs
- **Configuration**: Backward compatibility for profile formats
- **Data Migration**: Automatic downgrade of profile schema if needed

#### 13.2.2 Communication Plan
- **User Notification**: Clear communication about issues and solutions
- **Documentation**: Updated installation guides with version options
- **Support Channels**: Dedicated support for rollback assistance

### 13.3 Success Metrics for Each Phase
- **Alpha**: Functionality completeness, no data-loss bugs
- **Beta**: Performance targets met, user satisfaction positive
- **Production**: Download metrics, community engagement, error rates

### 13.4 Go/No-Go Criteria
- **Performance**: All acceptance tests passing
- **Security**: Security audit completed with no high-severity issues
- **Quality**: <0.1% crash rate in beta testing
- **Documentation**: Complete user guides and troubleshooting

---

## 14. Compliance & Legal

### 14.1 Open Source Compliance

#### 14.1.1 Licensing Strategy
- **Primary License**: MIT License for maximum compatibility
- **Dependencies**: Audit all Rust crates for license compatibility
- **Attribution**: Proper credit for all third-party components
- **Documentation**: Clear licensing information in README

#### 14.1.2 Contribution Guidelines
- **CLA**: Contributor License Agreement for code contributions
- **Code of Conduct**: Clear community standards
- **Review Process**: All contributions require maintainer review

### 14.2 Privacy Compliance

#### 14.2.1 Data Protection
- **GDPR Compliance**: Right to data export/deletion for EU users
- **Local Data**: Clear documentation of local storage locations
- **Telemetry**: Explicit opt-in with detailed data usage explanation

#### 14.2.2 User Rights
- **Transparency**: Open source code allows full inspection
- **Control**: Users control all local data and configuration
- **Export**: JSON format allows easy data portability

### 14.3 Accessibility Compliance

#### 14.3.1 WCAG AA Standards
- **Visual**: Minimum contrast ratios, no color-only information
- **Keyboard**: Full keyboard navigation without mouse
- **Documentation**: Accessibility features clearly documented

#### 14.3.2 Platform Accessibility
- **Screen Readers**: Basic compatibility where technically feasible
- **High Contrast**: Support for system high-contrast modes
- **Keyboard Shortcuts**: Customizable key bindings

### 14.4 Security Compliance

#### 14.4.1 Secure Development
- **Code Review**: All changes require security-focused review
- **Dependency Management**: Regular security updates for all dependencies
- **Vulnerability Response**: Clear process for handling security reports

#### 14.4.2 User Security
- **Input Validation**: All user input properly sanitized
- **File System**: Restricted access to only necessary directories  
- **Network**: No unauthorized network access, clear telemetry boundaries

---

## 15. Appendices

### 15.1 Revised Skill Index Formula
```
// Base metrics
RawWPM = (chars_typed / 5) / (duration_minutes)
EffWPM = RawWPM × (accuracy_percentage / 100)

// Penalty calculation
error_penalty = Σ(error_weight[type] × error_count[type])
latency_penalty = idle_events × 0.5
backspace_penalty = backspace_count × tier_multiplier

// Combo multiplier
combo_multiplier = 1 + log2(max(1, longest_streak) / 20)

// Final skill index
SkillIndex = clamp(
  (EffWPM × combo_multiplier × tier_weight) - 
  (error_penalty + latency_penalty + backspace_penalty), 
  0, 1000
)

// Grade thresholds (tier-adjusted)
Grade = {
  SI ≥ 900 × tier_factor: S
  SI ≥ 800 × tier_factor: A  
  SI ≥ 700 × tier_factor: B
  SI ≥ 600 × tier_factor: C
  SI < 600 × tier_factor: D
}
```

### 15.2 Level 100 Detailed Specifications
- **Content Composition**:
  - 35% mixed-case alphanumeric (camelCase/snake_case random)
  - 30% heavy symbols (`<>|&^~%$#@`) with nested brackets
  - 20% numbers (hex, binary prefixes, bit masks)
  - 15% technical terms (ID/EN) with Unicode accents
- **Language Switching**: Every 50±10 characters, no advance warning
- **Time Limit**: Fixed 120 seconds
- **Target Length**: 3000-3500 characters
- **Mastery Criteria**: 
  - WPM ≥ 130 (effective)
  - Accuracy ≥ 99.5%
  - Error severity ≤ 3 points
  - Zero backspaces in final 20% of text

### 15.3 Platform Compatibility Matrix
| Platform | Primary | Secondary | Status |
|----------|---------|-----------|---------|
| Ubuntu 20.04+ | ✓ | - | Full support |
| macOS 11+ | ✓ | - | Full support |
| Windows 10+ | ✓ | - | Full support |
| Debian/CentOS | - | ✓ | Best effort |
| Terminal.app | ✓ | - | Optimized |
| iTerm2 | ✓ | - | Optimized |
| Windows Terminal | ✓ | - | Optimized |
| cmd.exe | - | ✓ | Basic support |

---

**Document Version**: 2.0
**Last Updated**: 2025-09-26
**Status**: Ready for Development
**Next Review**: Pre-implementation checkpoint (Week 2)