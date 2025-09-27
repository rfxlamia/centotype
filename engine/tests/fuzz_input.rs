// Input fuzzing tests for terminal security
// Tests input sanitization and terminal escape sequence handling

use centotype_engine::*;
use std::time::Duration;

#[test]
#[ignore]
fn test_malicious_escape_sequences() {
    let mut engine = TypingEngine::new_test_mode();
    engine.start().expect("Failed to start engine");

    let malicious_inputs = vec![
        // ANSI escape sequences
        "\x1b[2J",         // Clear screen
        "\x1b[H",          // Cursor home
        "\x1b[31m",        // Red color
        "\x1b[1;1H",       // Cursor position
        "\x1b[?1049h",     // Alternative screen buffer
        "\x1b]0;Evil\x07", // Set window title
        // Control characters
        "\x00", // NULL
        "\x01", // SOH
        "\x02", // STX
        "\x03", // ETX (Ctrl+C)
        "\x04", // EOT (Ctrl+D)
        "\x08", // Backspace
        "\x0A", // Line feed
        "\x0D", // Carriage return
        "\x1B", // Escape
        "\x7F", // DEL
        // Terminal-specific escape sequences
        "\x1b[6n",        // Device status report
        "\x1b[c",         // Device attributes
        "\x1b[>c",        // Secondary device attributes
        "\x1b]4;1;?\x07", // Query color
        "\x1b]10;?\x07",  // Query foreground color
        "\x1b]11;?\x07",  // Query background color
        // Dangerous sequences that could execute commands
        "\x1b]52;c;$(whoami)\x07", // Clipboard manipulation
        "\x1b]1337;",              // iTerm2 proprietary
        "\x1b_Gf=32;",             // Kitty graphics protocol
        // Unicode shenanigans
        "\u{200B}", // Zero-width space
        "\u{FEFF}", // Byte order mark
        "\u{202E}", // Right-to-left override
        "\u{2066}", // Left-to-right isolate
    ];

    println!(
        "Testing {} malicious input sequences...",
        malicious_inputs.len()
    );

    for (i, input) in malicious_inputs.iter().enumerate() {
        println!("Testing input {}: {:?}", i + 1, input);

        // Test character by character input
        for ch in input.chars() {
            let result = engine.handle_input(TestInput::Char(ch));

            // Input should either be safely handled or explicitly rejected
            // but should never cause crashes or unexpected behavior
            match result {
                Ok(_) => {
                    // Verify the input was properly sanitized
                    let state = engine.get_test_state();
                    assert!(
                        !state.contains_raw_escape_sequences(),
                        "Engine state contains unsanitized escape sequences after input: {:?}",
                        ch
                    );
                }
                Err(e) => {
                    // Explicit rejection is acceptable
                    println!("Input rejected (expected): {}", e);
                }
            }
        }

        // Test bulk input processing
        let result = engine.handle_bulk_input(input);
        match result {
            Ok(_) => {
                let state = engine.get_test_state();
                assert!(
                    !state.contains_raw_escape_sequences(),
                    "Engine state contains unsanitized escape sequences after bulk input: {:?}",
                    input
                );
            }
            Err(_) => {
                // Bulk rejection is also acceptable
            }
        }
    }

    println!("All malicious input sequences handled safely");
}

#[test]
#[ignore]
fn test_input_buffer_overflow() {
    let mut engine = TypingEngine::new_test_mode();
    engine.start().expect("Failed to start engine");

    // Test various sizes of input to check for buffer overflows
    let sizes = vec![100, 1_000, 10_000, 100_000, 1_000_000];

    for size in sizes {
        println!("Testing input buffer with {} characters...", size);

        // Create large input string
        let large_input = "A".repeat(size);

        let start_time = std::time::Instant::now();
        let result = engine.handle_bulk_input(&large_input);
        let elapsed = start_time.elapsed();

        match result {
            Ok(_) => {
                // Should handle gracefully without excessive time
                assert!(
                    elapsed < Duration::from_secs(5),
                    "Input processing took too long ({:?}) for size {}",
                    elapsed,
                    size
                );

                // Verify engine is still responsive
                let test_result = engine.handle_input(TestInput::Char('x'));
                assert!(
                    test_result.is_ok(),
                    "Engine became unresponsive after large input"
                );
            }
            Err(e) => {
                println!("Large input rejected (size {}): {}", size, e);
                // Rejection is acceptable for very large inputs
            }
        }
    }
}

#[test]
#[ignore]
fn test_unicode_edge_cases() {
    let mut engine = TypingEngine::new_test_mode();
    engine.start().expect("Failed to start engine");

    let unicode_tests = vec![
        // Combining characters
        "e\u{0301}",         // e with acute accent
        "n\u{0303}",         // n with tilde
        "a\u{0300}\u{0301}", // a with multiple combining marks
        // Emoji and extended characters
        "🦀", // Rust crab emoji
        "👨‍💻", // Man technologist (multi-codepoint)
        "🏳️‍🌈", // Rainbow flag (flag + ZWJ + rainbow)
        // Directional and formatting characters
        "\u{061C}", // Arabic letter mark
        "\u{200D}", // Zero-width joiner
        "\u{200C}", // Zero-width non-joiner
        // Surrogate pairs and edge cases
        "\u{10000}", // Linear B syllable
        "\u{1F4A9}", // Pile of poo emoji
        "\u{E000}",  // Private use area
        // Normalization edge cases
        "Ä",         // A with diaeresis (single codepoint)
        "A\u{0308}", // A + combining diaeresis (two codepoints)
    ];

    println!("Testing {} Unicode edge cases...", unicode_tests.len());

    for (i, input) in unicode_tests.iter().enumerate() {
        println!("Testing Unicode input {}: {:?}", i + 1, input);

        for ch in input.chars() {
            let result = engine.handle_input(TestInput::Char(ch));

            // Should handle Unicode gracefully
            match result {
                Ok(_) => {
                    // Verify proper Unicode handling
                    let state = engine.get_test_state();
                    assert!(
                        state.is_unicode_safe(),
                        "Unicode safety violation after input: {:?}",
                        ch
                    );
                }
                Err(e) => {
                    // Some edge cases might be rejected, which is fine
                    println!("Unicode input rejected: {} ({})", ch, e);
                }
            }
        }
    }
}

#[test]
#[ignore]
fn test_concurrent_input_safety() {
    use std::sync::Arc;
    use std::thread;

    const NUM_THREADS: usize = 8;
    const INPUTS_PER_THREAD: usize = 100;

    let engine = Arc::new(TypingEngine::new_test_mode());
    engine.start().expect("Failed to start engine");

    let mut handles = Vec::new();

    println!(
        "Testing concurrent input safety with {} threads...",
        NUM_THREADS
    );

    for thread_id in 0..NUM_THREADS {
        let engine_clone = Arc::clone(&engine);

        let handle = thread::spawn(move || {
            let malicious_chars = vec![
                '\x1b', '\x00', '\x03', '\x04', '\x08', '\x7F', '🦀', '\u{202E}', '\u{200B}',
            ];

            for i in 0..INPUTS_PER_THREAD {
                let ch = malicious_chars[i % malicious_chars.len()];

                // This should never crash or cause data races
                let _result = engine_clone.handle_input(TestInput::Char(ch));

                if i % 20 == 0 {
                    println!("Thread {} progress: {}/{}", thread_id, i, INPUTS_PER_THREAD);
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle
            .join()
            .expect("Thread panicked during concurrent input test");
    }

    // Verify engine is still in a consistent state
    let final_state = engine.get_test_state();
    assert!(
        final_state.is_consistent(),
        "Engine state inconsistent after concurrent input test"
    );

    println!("Concurrent input safety test completed successfully");
}

#[test]
#[ignore]
fn test_input_timing_attacks() {
    let mut engine = TypingEngine::new_test_mode();
    engine.start().expect("Failed to start engine");

    // Test that input processing time doesn't reveal sensitive information
    let inputs = vec![
        "password123", // Common password
        "admin",       // Common username
        "\x1b[2J",     // Escape sequence
        "normal text", // Normal input
        "🦀🦀🦀",      // Unicode
    ];

    let mut timing_results = Vec::new();

    println!("Testing input timing consistency...");

    for (i, input) in inputs.iter().enumerate() {
        let mut times = Vec::new();

        // Measure processing time for each input multiple times
        for _ in 0..50 {
            let start = std::time::Instant::now();
            let _ = engine.handle_bulk_input(input);
            let elapsed = start.elapsed();
            times.push(elapsed);
        }

        times.sort();
        let median_time = times[times.len() / 2];
        timing_results.push((input, median_time));

        println!("Input {}: median time {:?}", i, median_time);
    }

    // Check that timing differences aren't too extreme
    let min_time = timing_results.iter().map(|(_, t)| *t).min().unwrap();
    let max_time = timing_results.iter().map(|(_, t)| *t).max().unwrap();

    let timing_ratio = max_time.as_nanos() as f64 / min_time.as_nanos() as f64;

    assert!(
        timing_ratio < 10.0,
        "Timing difference too large ({}x), potential timing attack vector",
        timing_ratio
    );

    println!(
        "Input timing consistency verified (max ratio: {:.2}x)",
        timing_ratio
    );
}
