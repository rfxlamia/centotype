# Centotype Product Roadmap

## Overview
Centotype aims to deliver a precision-first CLI typing trainer with 100 progressive levels, deterministic scoring, and sub-25ms input latency. The project currently operates between Phase 1 and Phase 2 of the v2.0 PRD: core architectural foundations are in place, the content system is production-ready, and platform abstractions are complete, while the core, engine, CLI, analytics, and persistence crates still require feature hardening. Performance stands at a **B+ overall grade** (docs/README.md) with latency optimization as the primary focus area.

## Phase Snapshot (from PRD §11.1)
| Phase | Target Duration | Status | Notes |
|-------|-----------------|--------|-------|
| **Phase 1 – Foundation** | Weeks 1-6 | ⚠️ In progress (~70%) | Architecture, scoring engine, CLI skeleton established; core/engine/cli crates need completion and validation.
| **Phase 2 – Content & Polish** | Weeks 7-10 | ✅ Content complete / 🔄 Optimization pending | 100-level corpus and generator finished; cross-platform performance tuning and bug sweeps outstanding.
| **Phase 3 – Release Preparation** | Weeks 11-12 | ⏳ Not started | Packaging, launch comms, feedback loops and support readiness to follow after stabilization.

## Milestone Breakdown
### Phase 1 – Foundation
- **Architecture & Tooling**: Rust workspace, CI/CD, performance harnesses (DONE)
- **Core Engine**: Finalize scoring, error classification, and keystroke pipeline (IN PROGRESS)
- **CLI Surface**: Complete command orchestration, interactive UI flows, help UX (IN PROGRESS)
- **Exit Criteria**: All crates compile with full unit coverage; latency baseline under 30ms P99; validated persistence read/write paths.

### Phase 2 – Content & Polish
- **Content Development**: Maintain 100-level corpus, finalize difficulty formulas, add multilingual extensions (DONE, follow-up QA ongoing)
- **Performance Optimization**: Achieve <25ms input latency in adverse scenarios, confirm 45ms content load P95 (IN PROGRESS)
- **Stability & QA**: Cross-platform regression suite, fuzzing, ignored performance tests (PLANNED)
- **Exit Criteria**: Benchmarks green, security audits clear, documentation updated for all flows.

### Phase 3 – Release Preparation
- **Distribution**: Ship cargo, npm, and binary artifacts; verify install guides
- **Go-to-Market**: Finalize launch messaging, update website, prepare community onboarding
- **Support Playbooks**: Incident runbooks, community moderation, monitoring dashboards
- **Exit Criteria**: Public v1.0 tag cut, release notes published, support queue staffed for launch window.

## Cross-Cutting Initiatives
- **Performance & Analytics**: Complete analytics crate, integrate inter-crate metrics collector, automate performance gates in CI.
- **Security & Compliance**: Maintain cargo-audit/deny pipelines, sanitize content ingestion, prepare external audit (PRD §12.1.3).
- **Documentation**: ✅ **COMPLETED** - Comprehensive docs/ suite reorganization with logical hierarchy and updated references. Keep synced with implementation, expand troubleshooting/quick-start guides as features ship.
- **Content Lifecycle**: Establish update cadence for corpus tiers, automate regression comparison using existing benchmark scripts.

## Dependencies & Risks
- **Key Dependencies**: Dedicated Rust engineering bandwidth, availability of diverse terminal environments for testing, content QA resources (PRD §11.2).
- **Primary Risks**:
  - Input latency plateau at ~28ms (current) delaying launch → Mitigation: prioritize engine hot path profiling in Phase 1 close-out.
  - Cross-platform regressions from advanced terminal features → Mitigation: expand platform test matrix before Phase 2 exit.
  - Content balancing drift as new tiers are introduced → Mitigation: enforce validation metrics and nightly generator audits.

---
*Next review checkpoint: align roadmap status with CHANGELOG.md updates after completing Phase 1 close-out validations.*
