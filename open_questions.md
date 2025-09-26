# Critical Open Questions - Centotype PRD v2.0

*Sorted by Impact × Urgency (High to Low)*

---

## 1. **Performance Target Validation** *(Impact: CRITICAL, Urgency: IMMEDIATE)*

**Question**: Are the revised performance targets (P99 input latency < 25ms, P95 startup < 200ms) technically achievable across all target platforms (Linux/macOS/Windows, various terminal emulators)?

**Why Critical**: Core user experience depends on meeting these targets. Failure would require major architecture changes or compromised user satisfaction.

**Dependencies**: Blocks architectural decisions, affects technology choices
**Required**: Prototype testing on representative hardware/terminal combinations
**Owner**: Tech Lead
**Timeline**: Must resolve before Week 2 (architecture freeze)

---

## 2. **Team Resource Availability** *(Impact: CRITICAL, Urgency: IMMEDIATE)*

**Question**: Do we have confirmed availability of the required team members (1.0 FTE Senior Rust Dev, 1.0 FTE CLI Dev, 0.5 FTE Content Writer, 0.5 FTE QA, 0.25 FTE PM) for the full 16-week timeline?

**Why Critical**: Budget assumes specific personnel costs and expertise levels. Resource shortage would delay or compromise delivery.

**Dependencies**: Blocks project start, affects all milestone planning
**Required**: Signed resource commitments or alternative staffing plan
**Owner**: Project Manager  
**Timeline**: Must resolve immediately (project kickoff dependent)

---

## 3. **Market Size & User Validation** *(Impact: HIGH, Urgency: HIGH)*

**Question**: What is the actual addressable market for CLI-based typing trainers among our target personas (developers, technical writers, speed competitors)?

**Why Critical**: Validates product-market fit assumptions. Small market could impact sustainability and adoption goals.

**Dependencies**: Affects success metrics, marketing strategy, feature prioritization
**Required**: User surveys, competitive analysis, CLI comfort level data
**Owner**: Product Manager
**Timeline**: Must resolve before Week 4 (affects feature scope decisions)

---

## 4. **Security Audit Scope & Budget** *(Impact: HIGH, Urgency: HIGH)*

**Question**: What specific security requirements should the external security audit cover, and is the $15,000 budget sufficient for comprehensive input validation, file system access, and terminal security review?

**Why Critical**: Security vulnerabilities could cause complete product failure. Inadequate audit budget risks missing critical issues.

**Dependencies**: Blocks security implementation planning, affects total budget
**Required**: Security audit RFP, vendor quotes, detailed scope definition
**Owner**: Security Engineer + Project Manager
**Timeline**: Must resolve by Week 3 (to schedule audit for Week 8-9)

---

## 5. **Level 100 Difficulty Calibration** *(Impact: HIGH, Urgency: MEDIUM)*

**Question**: How do we validate that the Level 100 mastery criteria (130 WPM effective, 99.5% accuracy) represent achievable-but-challenging targets rather than arbitrary impossibility?

**Why Critical**: Level 100 is a key product differentiator. Too easy undermines "nearly impossible" claim; too hard causes user frustration and churn.

**Dependencies**: Affects content generation, scoring algorithms, user retention
**Required**: Baseline typing speed research, expert typist validation testing
**Owner**: UX Designer + Content Writer
**Timeline**: Must resolve by Week 6 (before final level tuning)

---

## 6. **Content Creation Resource Requirements** *(Impact: HIGH, Urgency: MEDIUM)*

**Question**: Is 0.5 FTE Technical Writer sufficient to create and validate 100 levels of curated content across 10 tiers with appropriate difficulty progression, plus bilingual (ID/EN) corpus development?

**Why Critical**: Content quality directly impacts user experience. Inadequate content creation time could result in poor difficulty curves or insufficient content variety.

**Dependencies**: Affects content quality, timeline, may require additional resources
**Required**: Detailed content creation timeline, sample level development
**Owner**: Technical Writer + Project Manager
**Timeline**: Must resolve by Week 3 (affects resource allocation)

---

## 7. **Distribution & Installation Strategy** *(Impact: MEDIUM, Urgency: HIGH)*

**Question**: What is the most effective distribution strategy for cross-platform CLI tools, and how do we handle platform-specific installation challenges (npm global packages, cargo install, system PATH management)?

**Why Critical**: Poor installation experience creates adoption barriers. Complex distribution increases support burden.

**Dependencies**: Affects user onboarding, support requirements, technical architecture
**Required**: Distribution platform research, installation testing across platforms
**Owner**: DevOps Engineer + Tech Lead  
**Timeline**: Must resolve by Week 5 (before packaging implementation)

---

## 8. **Accessibility Compliance Scope** *(Impact: MEDIUM, Urgency: MEDIUM)*

**Question**: Beyond WCAG AA compliance, what specific accessibility features are feasible for a CLI application (screen reader compatibility, motor disability support, color blindness accommodations)?

**Why Critical**: Accessibility compliance affects legal requirements and user inclusivity. Over-commitment could impact timeline; under-commitment risks compliance issues.

**Dependencies**: Affects UI implementation, testing requirements, budget for accessibility audit
**Required**: Accessibility expert consultation, CLI accessibility standards research
**Owner**: UX Designer + QA Engineer
**Timeline**: Must resolve by Week 4 (before UI implementation)

---

## 9. **Error Detection Algorithm Complexity** *(Impact: MEDIUM, Urgency: MEDIUM)*

**Question**: Is the Damerau-Levenshtein algorithm with sliding window (N=5) for real-time transposition detection computationally feasible within our performance targets (P99 < 25ms input processing)?

**Why Critical**: Core scoring functionality depends on accurate error classification. Algorithm complexity could violate performance requirements.

**Dependencies**: Affects scoring accuracy, performance targets, algorithm selection
**Required**: Algorithm prototyping and benchmarking on target hardware
**Owner**: Senior Rust Developer
**Timeline**: Must resolve by Week 4 (before scoring engine implementation)

---

## 10. **Telemetry Privacy & Legal Review** *(Impact: LOW, Urgency: MEDIUM)*

**Question**: What specific legal review is required for the opt-in telemetry system, especially regarding GDPR compliance and data anonymization procedures?

**Why Critical**: Privacy violations could create legal liability. Inadequate review risks compliance issues or user trust problems.

**Dependencies**: Affects telemetry implementation, legal budget, user communication
**Required**: Legal consultation on data collection practices, privacy policy review
**Owner**: Legal Counsel + Project Manager
**Timeline**: Must resolve by Week 6 (before telemetry implementation)

---

## Resolution Priority Summary

**IMMEDIATE (Week 1-2)**:
- Questions 1, 2: Technical feasibility and resource confirmation

**HIGH PRIORITY (Week 3-4)**:  
- Questions 3, 4, 7, 8, 9: Market validation, security planning, distribution strategy

**MEDIUM PRIORITY (Week 5-6)**:
- Questions 5, 6, 10: Content validation, resource adequacy, legal review

---

## Next Steps

1. **Assign ownership** for each question to designated team members
2. **Schedule resolution meetings** with specific deadlines
3. **Track progress** weekly in project standup meetings
4. **Escalate blockers** immediately if resolution timelines are at risk
5. **Update PRD** based on question resolutions before development start

**Note**: Failure to resolve Questions 1-4 within specified timelines should trigger project timeline reassessment or scope reduction.