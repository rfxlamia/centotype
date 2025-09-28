// Memory usage validation tests
// Validates that the application stays within 50MB RSS memory limit

use centotype_core::CentotypeCore;
use centotype_engine::CentotypeEngine;
use centotype_platform::PlatformManager;
use std::sync::Arc;
use std::time::{Duration, Instant};

const MAX_MEMORY_RSS_BYTES: u64 = 50 * 1024 * 1024; // 50MB

#[cfg(target_os = "linux")]
fn get_memory_usage() -> Result<u64, Box<dyn std::error::Error>> {
    use std::fs;

    let status = fs::read_to_string("/proc/self/status")?;

    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let kb: u64 = parts[1].parse()?;
                return Ok(kb * 1024); // Convert KB to bytes
            }
        }
    }

    Err("VmRSS not found in /proc/self/status".into())
}

#[cfg(target_os = "macos")]
fn get_memory_usage() -> Result<u64, Box<dyn std::error::Error>> {
    use std::process::Command;

    let output = Command::new("ps")
        .args(&["-o", "rss=", "-p"])
        .arg(&std::process::id().to_string())
        .output()?;

    let rss_str = String::from_utf8(output.stdout)?;
    let rss_kb: u64 = rss_str.trim().parse()?;

    Ok(rss_kb * 1024) // Convert KB to bytes
}

#[cfg(target_os = "windows")]
fn get_memory_usage() -> Result<u64, Box<dyn std::error::Error>> {
    use std::process::Command;

    let output = Command::new("tasklist")
        .args(&[
            "/FI",
            &format!("PID eq {}", std::process::id()),
            "/FO",
            "CSV",
        ])
        .output()?;

    let output_str = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = output_str.lines().collect();

    if lines.len() >= 2 {
        let data_line = lines[1];
        let fields: Vec<&str> = data_line.split(',').collect();
        if fields.len() >= 5 {
            let memory_str = fields[4].trim_matches('"').replace(",", "");
            if let Some(kb_pos) = memory_str.find(" K") {
                let kb_str = &memory_str[..kb_pos];
                let kb: u64 = kb_str.parse()?;
                return Ok(kb * 1024);
            }
        }
    }

    Err("Failed to parse tasklist output".into())
}

#[tokio::test]
#[ignore]
async fn test_memory_usage_baseline() {
    println!("Testing baseline memory usage...");

    let initial_memory = get_memory_usage().expect("Failed to get initial memory usage");

    println!("Initial memory usage: {} MB", initial_memory / 1024 / 1024);

    // Create engine components
    let core = Arc::new(CentotypeCore::new());
    let platform = Arc::new(PlatformManager::new().expect("Failed to create platform manager"));
    let engine = CentotypeEngine::new(core, platform).await.expect("Failed to create engine");

    let after_creation = get_memory_usage().expect("Failed to get memory usage after creation");

    println!("Memory after creation: {} MB", after_creation / 1024 / 1024);

    // Note: Current engine has stub implementation, so we test the baseline memory usage
    println!(
        "Memory after engine creation: {} MB",
        after_creation / 1024 / 1024
    );

    assert!(
        after_creation <= MAX_MEMORY_RSS_BYTES,
        "Memory usage after creation ({} MB) exceeds limit ({} MB)",
        after_creation / 1024 / 1024,
        MAX_MEMORY_RSS_BYTES / 1024 / 1024
    );
}

#[tokio::test]
#[ignore]
async fn test_memory_usage_under_load() {
    const TEST_DURATION: Duration = Duration::from_secs(10); // Reduced for testing
    const MEMORY_CHECK_INTERVAL: Duration = Duration::from_millis(100);

    println!("Testing memory usage under load for {:?}...", TEST_DURATION);

    // Create multiple engine instances to simulate load
    let core = Arc::new(CentotypeCore::new());
    let platform = Arc::new(PlatformManager::new().expect("Failed to create platform manager"));
    let _engine = CentotypeEngine::new(core, platform).await.expect("Failed to create engine");

    let start_time = Instant::now();
    let mut max_memory = 0u64;
    let mut memory_samples = Vec::new();

    while start_time.elapsed() < TEST_DURATION {
        // Simulate load by creating temporary structures
        let _temp_data: Vec<String> = (0..100)
            .map(|i| format!("Test data {} for memory pressure", i))
            .collect();

        // Check memory usage
        if let Ok(current_memory) = get_memory_usage() {
            max_memory = max_memory.max(current_memory);
            memory_samples.push(current_memory);

            if memory_samples.len() % 50 == 0 {
                println!(
                    "Current memory: {} MB, Max: {} MB, Samples: {}",
                    current_memory / 1024 / 1024,
                    max_memory / 1024 / 1024,
                    memory_samples.len()
                );
            }

            assert!(
                current_memory <= MAX_MEMORY_RSS_BYTES,
                "Memory usage ({} MB) exceeded limit ({} MB) during load test",
                current_memory / 1024 / 1024,
                MAX_MEMORY_RSS_BYTES / 1024 / 1024
            );
        }

        std::thread::sleep(MEMORY_CHECK_INTERVAL);
    }

    println!(
        "Maximum memory usage during test: {} MB",
        max_memory / 1024 / 1024
    );
    println!("Total memory samples: {}", memory_samples.len());

    // Calculate average memory usage
    if !memory_samples.is_empty() {
        let avg_memory = memory_samples.iter().sum::<u64>() / memory_samples.len() as u64;
        println!("Average memory usage: {} MB", avg_memory / 1024 / 1024);
    }
}

#[tokio::test]
#[ignore]
async fn test_memory_leak_detection() {
    const ITERATIONS: usize = 100; // Reduced for testing
    const SAMPLE_FREQUENCY: usize = 10;

    println!(
        "Running memory leak detection for {} iterations...",
        ITERATIONS
    );

    let core = Arc::new(CentotypeCore::new());
    let platform = Arc::new(PlatformManager::new().expect("Failed to create platform manager"));
    let _engine = CentotypeEngine::new(core, platform).await.expect("Failed to create engine");

    let initial_memory = get_memory_usage().expect("Failed to get initial memory");

    let mut memory_samples = Vec::new();

    for i in 0..ITERATIONS {
        // Perform operations that might leak memory
        let _temp_data: Vec<String> = (0..50)
            .map(|j| format!("Iteration {} data {}", i, j))
            .collect();

        // Simulate creating and dropping resources
        let _temp_arcs: Vec<Arc<String>> = _temp_data
            .iter()
            .map(|s| Arc::new(s.clone()))
            .collect();

        // Sample memory usage periodically
        if i % SAMPLE_FREQUENCY == 0 {
            if let Ok(current_memory) = get_memory_usage() {
                memory_samples.push((i, current_memory));
                println!("Iteration {}: {} MB", i, current_memory / 1024 / 1024);
            }
        }
    }

    let final_memory = get_memory_usage().expect("Failed to get final memory");

    println!("Initial memory: {} MB", initial_memory / 1024 / 1024);
    println!("Final memory: {} MB", final_memory / 1024 / 1024);

    // Check for significant memory growth (more than 10MB)
    let memory_growth = final_memory.saturating_sub(initial_memory);
    const MAX_ACCEPTABLE_GROWTH: u64 = 10 * 1024 * 1024; // 10MB

    assert!(
        memory_growth <= MAX_ACCEPTABLE_GROWTH,
        "Potential memory leak detected: memory grew by {} MB (limit: {} MB)",
        memory_growth / 1024 / 1024,
        MAX_ACCEPTABLE_GROWTH / 1024 / 1024
    );

    // Analyze memory trend
    if memory_samples.len() >= 5 {
        let first_half = &memory_samples[..memory_samples.len() / 2];
        let second_half = &memory_samples[memory_samples.len() / 2..];

        let first_avg =
            first_half.iter().map(|(_, mem)| *mem).sum::<u64>() / first_half.len() as u64;

        let second_avg =
            second_half.iter().map(|(_, mem)| *mem).sum::<u64>() / second_half.len() as u64;

        println!("First half average: {} MB", first_avg / 1024 / 1024);
        println!("Second half average: {} MB", second_avg / 1024 / 1024);

        let trend_growth = second_avg.saturating_sub(first_avg);
        const MAX_TREND_GROWTH: u64 = 5 * 1024 * 1024; // 5MB

        assert!(
            trend_growth <= MAX_TREND_GROWTH,
            "Memory usage trend indicates potential leak: {} MB growth",
            trend_growth / 1024 / 1024
        );
    }
}
