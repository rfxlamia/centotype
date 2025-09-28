# ADR-001: Inter-Crate Data Flow and Performance Boundaries

## Status

**Accepted** - Frozen for Week 3-4 coordinated development

## Context

The Centotype engine integration requires stable data flow patterns and performance boundaries to enable parallel development across multiple teams (rust-pro, ui-ux-designer, performance-engineer). Without frozen interfaces, breaking changes would cascade across all crates during development.

## Decision

We establish the following inter-crate data flow architecture and performance boundaries:

### Core Data Flow Pattern

```
content/ → core/ → engine/ → cli/ (main execution path)
analytics/ ← engine/ (side effects, non-blocking)
persistence/ ← engine/ (async, non-blocking)
platform/ ↔ engine/ (bidirectional, low-level)
```

### Performance Boundaries

| Interface | Performance Requirement | Measurement |
|-----------|------------------------|-------------|
| content/ → core/ | <5ms content lookup (cache hit) | P99 latency |
| core/ → engine/ | <5ms scoring calculation | P99 latency |
| engine/ → cli/ | <15ms render update | P99 latency |
| **Total input-to-visual** | **<25ms** | **P99 end-to-end** |

### Interface Contracts

#### 1. Core ↔ Engine: `ScoringEngine` trait
- **Hot Path**: `process_keystroke()` - P99 < 5ms, no heap allocations
- **State Access**: `current_state()` - P99 < 1ms, read-only
- **Session Control**: `complete_session()` - can be slower, called once per session
- **Arc Pattern**: Store `Arc<dyn ScoringEngine>`, clone Arc not data

#### 2. Engine ↔ Content: `ContentLoader` trait
- **Cache Hit**: `get_cached_content()` - P99 < 5ms
- **Cache Miss**: `load_level_content()` - P99 < 25ms (includes generation)
- **Cache Hit Rate**: >90% required for performance targets
- **Background Preloading**: `preload_next_levels()` - non-blocking

#### 3. Engine ↔ Analytics: `AnalyticsCollector` trait
- **Event Recording**: `record_keystroke()` - P99 < 1ms, async queue
- **Live Metrics**: `calculate_wpm()`, `calculate_accuracy()` - fast calculations
- **No Critical Path Impact**: All operations non-blocking to main game loop

#### 4. Engine ↔ Persistence: `SessionPersistence` trait
- **Session Save**: `save_session()` - 1-2 seconds acceptable, async
- **Profile Load**: `load_profile()` - P95 < 500ms
- **Progress Update**: `update_progress()` - P99 < 100ms, atomic operations

#### 5. Engine ↔ Platform: `TerminalManager` trait
- **Input Polling**: `poll_event()` - P99 < 10ms
- **Mode Changes**: `enter_raw_mode()`, etc. - P95 < 50ms, synchronous
- **Emergency Cleanup**: Must be synchronous and never fail

### Data Architecture Constraints

#### Memory Management
- **Arc Usage**: All shared data uses `Arc<T>` to minimize clone overhead
- **Clone Pattern**: Clone the Arc, not the contained data
- **Hot Path**: No heap allocations in `process_keystroke()` and `poll_event()`
- **Memory Target**: <50MB total application memory usage

#### Async Boundaries
- **Blocking Operations**: Only at persistence/ and content/ loading
- **Critical Path**: Core scoring and input handling are synchronous
- **Error Propagation**: Use `Result<T, CentotypeError>` pattern consistently
- **Background Tasks**: Analytics and preloading use background async tasks

#### Error Handling Strategy
```rust
// Consistent error pattern across all traits
async fn example_operation(&self) -> Result<ReturnType, CentotypeError>;

// Error types align with crate boundaries
CentotypeError::Content(String)     // content/ crate errors
CentotypeError::Platform(String)    // platform/ crate errors
CentotypeError::Persistence(String) // persistence/ crate errors
```

### Event System Architecture

#### GameEvent Flow
```
Input → KeyIn → Hit/Miss → Analytics Recording
     ↓
Scoring → Live Metrics → Render Events → UI Update
     ↓
Session State → Persistence (async)
```

#### Event Processing Constraints
- **Event Queue Depth**: <100 events under normal load
- **Event Processing**: P99 < 5ms per event
- **Batch Processing**: Events can be batched for performance
- **Memory Overhead**: <1KB per event batch

### Testing and Validation

#### Performance Validation
```bash
# Validate each interface meets performance targets
cargo bench --bench input_latency_benchmark     # Platform interface
cargo bench --bench content_performance_benchmark # Content interface
cargo bench --bench scoring_performance_benchmark # Core interface
cargo bench --bench end_to_end_latency_benchmark # Total system
```

#### Interface Compliance
- All traits must compile without warnings
- Mock implementations provided for testing
- Performance benchmarks validate each boundary
- Integration tests verify data flow patterns

## Consequences

### Positive
- **Parallel Development**: Teams can work independently on different crates
- **Performance Guarantees**: Clear targets enable optimization efforts
- **Maintainability**: Well-defined boundaries reduce coupling
- **Testing**: Mockable interfaces enable comprehensive testing

### Negative
- **Interface Rigidity**: Changes require cross-team coordination
- **Performance Pressure**: Strict targets may limit implementation flexibility
- **Arc Overhead**: Some memory overhead from Arc usage
- **Complexity**: Multiple async boundaries add complexity

### Risk Mitigation
- **Frozen Contract Period**: Interfaces locked for Week 3-4 development
- **Performance Monitoring**: Continuous benchmarking prevents regressions
- **Mock Implementations**: Available for development without full implementations
- **Rollback Plan**: ADR can be revised if fundamental issues discovered

## Implementation Notes

### Priority Order
1. **Critical Path**: ScoringEngine and TerminalManager (blocking engine development)
2. **Content System**: ContentLoader (needed for level content)
3. **Supporting Systems**: Analytics and Persistence (can use mocks initially)

### Coordination Points
- **Weekly Reviews**: Validate interface compliance and performance metrics
- **Breaking Changes**: Require ADR revision and team coordination
- **Performance Regressions**: Trigger immediate investigation and fixes

### Mock Implementation Strategy
```rust
// Example mock for parallel development
pub struct MockScoringEngine {
    // Minimal implementation for testing
}

#[async_trait]
impl ScoringEngine for MockScoringEngine {
    async fn process_keystroke(&mut self, key: KeyCode, timestamp: Instant) -> Result<ScoringResult> {
        // Fast mock implementation with realistic timing
        Ok(ScoringResult::mock_correct_keystroke())
    }
    // ... other methods
}
```

## References

- [Performance Validation Report](../performance/PERFORMANCE_VALIDATION_REPORT.md)
- [Master Prompt System](../design/MASTER_PROMPT.md)
- [API Reference Documentation](../api/)
- [Benchmarking Guide](../performance/benchmarking.md)

---

**Document Control**
- **Version**: 1.0
- **Created**: 2025-09-28
- **Authors**: Backend System Architect
- **Review**: Required for any modifications
- **Next Review**: Week 4 development checkpoint