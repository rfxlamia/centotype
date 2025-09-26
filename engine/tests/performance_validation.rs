// Performance validation tests for the Centotype engine
// These tests validate that the application meets performance requirements

use centotype_engine::*;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

const P99_INPUT_LATENCY_THRESHOLD: Duration = Duration::from_millis(25);
const P95_STARTUP_THRESHOLD: Duration = Duration::from_millis(200);
const P95_RENDER_THRESHOLD: Duration = Duration::from_millis(33);
const MAX_MEMORY_RSS_BYTES: u64 = 50 * 1024 * 1024; // 50MB

#[test]
#[ignore] // Only run with --ignored flag in CI
fn test_input_latency_p99() {
    const ITERATIONS: usize = 1000;
    let mut latencies = Vec::with_capacity(ITERATIONS);

    // Initialize the engine in test mode
    let mut engine = TypingEngine::new_test_mode();
    engine.start().expect("Failed to start engine");

    println!("Running {} input latency measurements...", ITERATIONS);

    for i in 0..ITERATIONS {
        if i % 100 == 0 {
            println!("Progress: {}/{}", i, ITERATIONS);
        }

        // Simulate keystroke input
        let start = Instant::now();

        // Inject a test keystroke
        let result = engine.handle_input(TestInput::Char('a'));

        let latency = start.elapsed();
        latencies.push(latency);

        assert!(result.is_ok(), "Input handling failed at iteration {}", i);
    }

    // Calculate P99 latency
    latencies.sort();
    let p99_index = (ITERATIONS as f64 * 0.99) as usize;
    let p99_latency = latencies[p99_index];

    println!("P99 input latency: {:?}", p99_latency);
    println!("Threshold: {:?}", P99_INPUT_LATENCY_THRESHOLD);

    assert!(
        p99_latency <= P99_INPUT_LATENCY_THRESHOLD,
        "P99 input latency ({:?}) exceeds threshold ({:?})",
        p99_latency,
        P99_INPUT_LATENCY_THRESHOLD
    );
}

#[test]
#[ignore]
fn test_render_performance_p95() {
    const ITERATIONS: usize = 500;
    let mut render_times = Vec::with_capacity(ITERATIONS);

    let mut engine = TypingEngine::new_test_mode();
    engine.start().expect("Failed to start engine");

    // Load test text to render
    let test_text = "The quick brown fox jumps over the lazy dog. ".repeat(20);
    engine.load_text(&test_text).expect("Failed to load test text");

    println!("Running {} render performance measurements...", ITERATIONS);

    for i in 0..ITERATIONS {
        if i % 50 == 0 {
            println!("Progress: {}/{}", i, ITERATIONS);
        }

        let start = Instant::now();

        // Render frame
        let _frame = engine.render_frame().expect("Render failed");

        let render_time = start.elapsed();
        render_times.push(render_time);
    }

    // Calculate P95 render time
    render_times.sort();
    let p95_index = (ITERATIONS as f64 * 0.95) as usize;
    let p95_render_time = render_times[p95_index];

    println!("P95 render time: {:?}", p95_render_time);
    println!("Threshold: {:?}", P95_RENDER_THRESHOLD);

    assert!(
        p95_render_time <= P95_RENDER_THRESHOLD,
        "P95 render time ({:?}) exceeds threshold ({:?})",
        p95_render_time,
        P95_RENDER_THRESHOLD
    );
}

#[test]
#[ignore]
fn test_startup_time_p95() {
    const ITERATIONS: usize = 50; // Fewer iterations for startup tests
    let mut startup_times = Vec::with_capacity(ITERATIONS);

    println!("Running {} startup time measurements...", ITERATIONS);

    for i in 0..ITERATIONS {
        println!("Startup test {}/{}", i + 1, ITERATIONS);

        let start = Instant::now();

        // Full engine initialization
        let engine = TypingEngine::new();
        let _initialized = engine.initialize().expect("Failed to initialize");

        let startup_time = start.elapsed();
        startup_times.push(startup_time);

        // Clean shutdown
        drop(engine);
    }

    // Calculate P95 startup time
    startup_times.sort();
    let p95_index = (ITERATIONS as f64 * 0.95) as usize;
    let p95_startup_time = startup_times[p95_index];

    println!("P95 startup time: {:?}", p95_startup_time);
    println!("Threshold: {:?}", P95_STARTUP_THRESHOLD);

    assert!(
        p95_startup_time <= P95_STARTUP_THRESHOLD,
        "P95 startup time ({:?}) exceeds threshold ({:?})",
        p95_startup_time,
        P95_STARTUP_THRESHOLD
    );
}

#[test]
#[ignore]
fn test_sustained_performance() {
    const TEST_DURATION: Duration = Duration::from_secs(60);
    const SAMPLE_INTERVAL: Duration = Duration::from_millis(100);

    let mut engine = TypingEngine::new_test_mode();
    engine.start().expect("Failed to start engine");

    let start_time = Instant::now();
    let mut samples = Vec::new();

    println!("Running sustained performance test for {:?}...", TEST_DURATION);

    while start_time.elapsed() < TEST_DURATION {
        let sample_start = Instant::now();

        // Simulate typical operation: input + render
        let _input_result = engine.handle_input(TestInput::Char('a'));
        let _frame = engine.render_frame();

        let sample_time = sample_start.elapsed();
        samples.push(sample_time);

        std::thread::sleep(SAMPLE_INTERVAL);
    }

    // Analyze sustained performance
    samples.sort();
    let p95_index = (samples.len() as f64 * 0.95) as usize;
    let p95_sustained = samples[p95_index];

    println!("Sustained P95 operation time: {:?}", p95_sustained);
    println!("Total samples: {}", samples.len());

    // Should maintain performance under sustained load
    assert!(
        p95_sustained <= Duration::from_millis(50),
        "Sustained P95 operation time ({:?}) indicates performance degradation",
        p95_sustained
    );
}

#[test]
#[ignore]
fn test_concurrent_performance() {
    use std::thread;

    const NUM_THREADS: usize = 4;
    const OPERATIONS_PER_THREAD: usize = 250;

    let latency_accumulator = Arc::new(AtomicU64::new(0));
    let mut handles = Vec::new();

    println!("Running concurrent performance test with {} threads...", NUM_THREADS);

    for thread_id in 0..NUM_THREADS {
        let latency_acc = Arc::clone(&latency_accumulator);

        let handle = thread::spawn(move || {
            let mut engine = TypingEngine::new_test_mode();
            engine.start().expect("Failed to start engine");

            let mut max_latency = Duration::ZERO;

            for i in 0..OPERATIONS_PER_THREAD {
                let start = Instant::now();

                let _result = engine.handle_input(TestInput::Char(
                    char::from(b'a' + (i % 26) as u8)
                ));

                let latency = start.elapsed();
                max_latency = max_latency.max(latency);

                if i % 50 == 0 {
                    println!("Thread {} progress: {}/{}", thread_id, i, OPERATIONS_PER_THREAD);
                }
            }

            latency_acc.store(max_latency.as_nanos() as u64, Ordering::Relaxed);
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let max_latency_nanos = latency_accumulator.load(Ordering::Relaxed);
    let max_latency = Duration::from_nanos(max_latency_nanos);

    println!("Maximum latency across all threads: {:?}", max_latency);

    assert!(
        max_latency <= Duration::from_millis(100),
        "Concurrent operation latency ({:?}) too high",
        max_latency
    );
}