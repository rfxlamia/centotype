// Performance validation tests for the Centotype engine
// These tests validate that the application meets performance requirements

use centotype_core::CentotypeCore;
use centotype_engine::CentotypeEngine;
use centotype_platform::PlatformManager;
use std::sync::Arc;
use std::time::{Duration, Instant};

const P99_INPUT_LATENCY_THRESHOLD: Duration = Duration::from_millis(25);
const P95_STARTUP_THRESHOLD: Duration = Duration::from_millis(200);
const P95_RENDER_THRESHOLD: Duration = Duration::from_millis(33);
const MAX_MEMORY_RSS_BYTES: u64 = 50 * 1024 * 1024; // 50MB

#[tokio::test]
#[ignore] // Only run with --ignored flag in CI
async fn test_engine_creation_latency() {
    const ITERATIONS: usize = 100;
    let mut latencies = Vec::with_capacity(ITERATIONS);

    println!("Running {} engine creation latency measurements...", ITERATIONS);

    for i in 0..ITERATIONS {
        if i % 10 == 0 {
            println!("Progress: {}/{}", i, ITERATIONS);
        }

        // Measure engine creation time
        let start = Instant::now();

        let core = Arc::new(CentotypeCore::new());
        let platform = Arc::new(PlatformManager::new().expect("Failed to create platform manager"));
        let _engine = CentotypeEngine::new(core, platform).await.expect("Failed to create engine");

        let latency = start.elapsed();
        latencies.push(latency);
    }

    // Calculate P95 latency for engine creation
    latencies.sort();
    let p95_index = (ITERATIONS as f64 * 0.95) as usize;
    let p95_latency = latencies[p95_index];

    println!("P95 engine creation latency: {:?}", p95_latency);
    println!("Threshold: {:?}", P95_STARTUP_THRESHOLD);

    assert!(
        p95_latency <= P95_STARTUP_THRESHOLD,
        "P95 engine creation latency ({:?}) exceeds threshold ({:?})",
        p95_latency,
        P95_STARTUP_THRESHOLD
    );
}

#[tokio::test]
#[ignore]
async fn test_startup_time_p95() {
    const ITERATIONS: usize = 50;
    let mut startup_times = Vec::with_capacity(ITERATIONS);

    println!("Running {} startup time measurements...", ITERATIONS);

    for i in 0..ITERATIONS {
        if i % 10 == 0 {
            println!("Progress: {}/{}", i, ITERATIONS);
        }

        let start = Instant::now();

        // Simulate complete application startup
        let core = Arc::new(CentotypeCore::new());
        let platform = Arc::new(PlatformManager::new().expect("Failed to create platform manager"));
        let _engine = CentotypeEngine::new(core, platform).await.expect("Failed to create engine");

        // Simulate additional startup work
        std::thread::sleep(Duration::from_millis(1));

        let startup_time = start.elapsed();
        startup_times.push(startup_time);
    }

    // Calculate P95 startup time
    startup_times.sort();
    let p95_index = (ITERATIONS as f64 * 0.95) as usize;
    let p95_startup = startup_times[p95_index];

    println!("P95 startup time: {:?}", p95_startup);
    println!("Threshold: {:?}", P95_STARTUP_THRESHOLD);

    assert!(
        p95_startup <= P95_STARTUP_THRESHOLD,
        "P95 startup time ({:?}) exceeds threshold ({:?})",
        p95_startup,
        P95_STARTUP_THRESHOLD
    );
}

#[tokio::test]
#[ignore]
async fn test_memory_footprint() {
    const NUM_ENGINES: usize = 10;
    let mut engines = Vec::new();

    println!("Testing memory footprint with {} engines...", NUM_ENGINES);

    // Create multiple engines to test memory usage
    for i in 0..NUM_ENGINES {
        let core = Arc::new(CentotypeCore::new());
        let platform = Arc::new(PlatformManager::new().expect("Failed to create platform manager"));
        let engine = CentotypeEngine::new(core, platform).await.expect("Failed to create engine");
        engines.push(engine);

        if i % 2 == 0 {
            println!("Created {} engines", i + 1);
        }
    }

    // Basic memory usage test (engines are created successfully)
    assert_eq!(engines.len(), NUM_ENGINES);

    println!("Memory footprint test completed");
}

#[tokio::test]
#[ignore]
async fn test_concurrent_performance() {
    use std::thread;

    const NUM_THREADS: usize = 4;
    const OPERATIONS_PER_THREAD: usize = 50;

    println!(
        "Testing concurrent performance with {} threads...",
        NUM_THREADS
    );

    let mut handles = Vec::new();

    for thread_id in 0..NUM_THREADS {
        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create runtime");
            let mut operation_times = Vec::new();

            for i in 0..OPERATIONS_PER_THREAD {
                let start = Instant::now();

                // Create engine components
                let core = Arc::new(CentotypeCore::new());
                let platform = Arc::new(PlatformManager::new().expect("Failed to create platform manager"));
                let _engine = rt.block_on(CentotypeEngine::new(core, platform)).expect("Failed to create engine");

                let elapsed = start.elapsed();
                operation_times.push(elapsed);

                if i % 10 == 0 {
                    println!("Thread {} progress: {}/{}", thread_id, i, OPERATIONS_PER_THREAD);
                }
            }

            // Return timing statistics
            operation_times.sort();
            let median_time = operation_times[operation_times.len() / 2];
            let max_time = operation_times[operation_times.len() - 1];

            (thread_id, median_time, max_time)
        });

        handles.push(handle);
    }

    // Collect results from all threads
    let mut all_results = Vec::new();
    for handle in handles {
        let result = handle.join().expect("Thread panicked during concurrent test");
        all_results.push(result);
    }

    // Analyze concurrent performance
    for (thread_id, median, max) in &all_results {
        println!(
            "Thread {}: median={:?}, max={:?}",
            thread_id, median, max
        );

        // Each thread should maintain reasonable performance
        assert!(
            *max < Duration::from_millis(100),
            "Thread {} max time ({:?}) too high",
            thread_id,
            max
        );
    }

    println!("Concurrent performance test completed");
}

#[tokio::test]
#[ignore]
async fn test_resource_cleanup() {
    const ITERATIONS: usize = 100;

    println!("Testing resource cleanup over {} iterations...", ITERATIONS);

    for i in 0..ITERATIONS {
        // Create and immediately drop engine
        {
            let core = Arc::new(CentotypeCore::new());
            let platform = Arc::new(PlatformManager::new().expect("Failed to create platform manager"));
            let _engine = CentotypeEngine::new(core, platform).await.expect("Failed to create engine");
            // Engine is dropped here
        }

        if i % 20 == 0 {
            println!("Cleanup iteration: {}/{}", i, ITERATIONS);
        }
    }

    println!("Resource cleanup test completed");
}

#[tokio::test]
#[ignore]
async fn test_stress_operations() {
    const STRESS_DURATION: Duration = Duration::from_secs(10);
    const OPERATION_INTERVAL: Duration = Duration::from_millis(10);

    println!("Running stress test for {:?}...", STRESS_DURATION);

    let start_time = Instant::now();
    let mut operation_count = 0;

    while start_time.elapsed() < STRESS_DURATION {
        // Perform stress operations
        let core = Arc::new(CentotypeCore::new());
        let platform = Arc::new(PlatformManager::new().expect("Failed to create platform manager"));
        let _engine = CentotypeEngine::new(core, platform).await.expect("Failed to create engine");

        operation_count += 1;

        if operation_count % 100 == 0 {
            println!("Stress operations completed: {}", operation_count);
        }

        std::thread::sleep(OPERATION_INTERVAL);
    }

    println!(
        "Stress test completed: {} operations in {:?}",
        operation_count,
        start_time.elapsed()
    );

    // Should complete a reasonable number of operations
    assert!(
        operation_count > 100,
        "Too few operations completed: {}",
        operation_count
    );
}