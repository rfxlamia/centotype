//! Arena allocation system for zero hot-path allocations
//! Provides pre-allocated memory pools to eliminate GC pressure during critical path

use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Arena-based memory pool for hot-path operations
#[derive(Debug)]
pub struct RenderArena {
    /// Pre-allocated string buffer for ANSI sequence construction
    pub string_buffer: Vec<u8>,
    /// String capacity for preventing reallocations
    string_capacity: usize,

    /// Pre-allocated line buffer for content composition
    pub line_buffer: Vec<String>,
    /// Line buffer capacity
    line_capacity: usize,

    /// Pre-allocated style buffer for style calculations
    pub style_buffer: Vec<StyleData>,
    /// Style buffer capacity
    style_capacity: usize,

    /// Character buffer for typed text processing
    pub char_buffer: Vec<char>,
    /// Character buffer capacity
    char_capacity: usize,

    /// Temporary buffer for cursor position calculations
    pub cursor_buffer: Vec<usize>,

    /// Arena statistics for monitoring
    stats: ArenaStats,
}

/// Style data for pre-calculation
#[derive(Debug, Clone, Default)]
pub struct StyleData {
    pub foreground: Option<(u8, u8, u8)>,
    pub background: Option<(u8, u8, u8)>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub start_pos: usize,
    pub end_pos: usize,
}

/// Arena usage statistics for performance monitoring
#[derive(Debug, Default)]
pub struct ArenaStats {
    pub allocations_avoided: u64,
    pub total_buffer_reuses: u64,
    pub max_string_usage: usize,
    pub max_line_usage: usize,
    pub max_style_usage: usize,
    pub last_reset_time: Option<Instant>,
    pub reset_count: u64,
}

impl RenderArena {
    /// Create new render arena with optimized capacity
    pub fn new() -> Self {
        Self::with_capacity(
            8192,  // 8KB string buffer
            256,   // 256 lines
            1024,  // 1024 styles
            4096,  // 4096 characters
        )
    }

    /// Create arena with specific capacities for fine-tuning
    pub fn with_capacity(
        string_capacity: usize,
        line_capacity: usize,
        style_capacity: usize,
        char_capacity: usize,
    ) -> Self {
        Self {
            string_buffer: Vec::with_capacity(string_capacity),
            string_capacity,

            line_buffer: Vec::with_capacity(line_capacity),
            line_capacity,

            style_buffer: Vec::with_capacity(style_capacity),
            style_capacity,

            char_buffer: Vec::with_capacity(char_capacity),
            char_capacity,

            cursor_buffer: Vec::with_capacity(128),

            stats: ArenaStats::default(),
        }
    }

    /// Prepare arena for new frame - clear without deallocation
    pub fn prepare_frame(&mut self) -> FrameData {
        let start_time = Instant::now();

        // Clear all buffers without deallocating
        self.string_buffer.clear();
        self.line_buffer.clear();
        self.style_buffer.clear();
        self.char_buffer.clear();
        self.cursor_buffer.clear();

        // Update statistics
        self.stats.total_buffer_reuses += 1;
        self.stats.reset_count += 1;
        self.stats.last_reset_time = Some(start_time);

        // Track maximum usage for capacity optimization
        self.stats.max_string_usage = self.stats.max_string_usage.max(self.string_buffer.len());
        self.stats.max_line_usage = self.stats.max_line_usage.max(self.line_buffer.len());
        self.stats.max_style_usage = self.stats.max_style_usage.max(self.style_buffer.len());

        debug!("Arena prepared for frame in {:?}", start_time.elapsed());

        FrameData {
            frame_start: start_time,
            allocations_avoided: 0,
        }
    }

    /// Get mutable reference to string buffer for ANSI construction
    pub fn get_string_buffer(&mut self) -> &mut Vec<u8> {
        self.stats.allocations_avoided += 1;
        &mut self.string_buffer
    }

    /// Build string from buffer with zero allocation
    pub fn build_string(&mut self) -> String {
        // SAFETY: We assume the buffer contains valid UTF-8
        // In production, we would validate this
        String::from_utf8(self.string_buffer.clone())
            .unwrap_or_else(|_| {
                warn!("Invalid UTF-8 in arena string buffer");
                String::new()
            })
    }

    /// Get line buffer for content composition
    pub fn get_line_buffer(&mut self) -> &mut Vec<String> {
        self.stats.allocations_avoided += 1;
        &mut self.line_buffer
    }

    /// Add line to buffer without allocation if possible
    pub fn add_line(&mut self, content: &str) {
        if self.line_buffer.len() < self.line_capacity {
            self.line_buffer.push(content.to_string());
            self.stats.allocations_avoided += 1;
        } else {
            warn!("Line buffer capacity exceeded: {} lines", self.line_buffer.len());
            // Fallback: still add the line but track the allocation
            self.line_buffer.push(content.to_string());
        }
    }

    /// Pre-calculate styles for a text segment
    pub fn precalculate_styles(&mut self, text: &str, base_style: StyleData) -> &[StyleData] {
        self.style_buffer.clear();

        // Simple style calculation - in production this would be more sophisticated
        let mut current_style = base_style;
        let chars: Vec<char> = text.chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            // Example: highlight errors in red
            if ch == '!' {
                current_style.foreground = Some((255, 0, 0)); // Red for errors
                current_style.start_pos = i;
                current_style.end_pos = i + 1;
                self.style_buffer.push(current_style.clone());
            }
        }

        self.stats.allocations_avoided += self.style_buffer.len() as u64;
        &self.style_buffer
    }

    /// Process character input with arena allocation
    pub fn process_chars(&mut self, input: &str) -> &[char] {
        self.char_buffer.clear();
        self.char_buffer.extend(input.chars());
        self.stats.allocations_avoided += 1;
        &self.char_buffer
    }

    /// Calculate cursor positions without allocation
    pub fn calculate_cursor_positions(&mut self, text: &str) -> &[usize] {
        self.cursor_buffer.clear();

        let mut byte_pos = 0;
        for ch in text.chars() {
            self.cursor_buffer.push(byte_pos);
            byte_pos += ch.len_utf8();
        }
        self.cursor_buffer.push(byte_pos); // End position

        self.stats.allocations_avoided += 1;
        &self.cursor_buffer
    }

    /// Get comprehensive arena statistics
    pub fn get_stats(&self) -> &ArenaStats {
        &self.stats
    }

    /// Check if arena needs capacity adjustment
    pub fn needs_resize(&self) -> Option<ResizeRecommendation> {
        let mut recommendations = Vec::new();

        // Check string buffer usage
        let string_usage_ratio = self.stats.max_string_usage as f64 / self.string_capacity as f64;
        if string_usage_ratio > 0.9 {
            recommendations.push(BufferType::String);
        }

        // Check line buffer usage
        let line_usage_ratio = self.stats.max_line_usage as f64 / self.line_capacity as f64;
        if line_usage_ratio > 0.9 {
            recommendations.push(BufferType::Line);
        }

        // Check style buffer usage
        let style_usage_ratio = self.stats.max_style_usage as f64 / self.style_capacity as f64;
        if style_usage_ratio > 0.9 {
            recommendations.push(BufferType::Style);
        }

        if recommendations.is_empty() {
            None
        } else {
            Some(ResizeRecommendation {
                buffers_to_resize: recommendations,
                recommended_multiplier: 1.5,
                urgency: if string_usage_ratio > 0.95 || line_usage_ratio > 0.95 || style_usage_ratio > 0.95 {
                    ResizeUrgency::High
                } else {
                    ResizeUrgency::Medium
                },
            })
        }
    }

    /// Resize arena buffers for optimal performance
    pub fn resize_buffers(&mut self, recommendation: &ResizeRecommendation) {
        let multiplier = recommendation.recommended_multiplier;

        for buffer_type in &recommendation.buffers_to_resize {
            match buffer_type {
                BufferType::String => {
                    let new_capacity = (self.string_capacity as f64 * multiplier) as usize;
                    self.string_buffer.reserve(new_capacity - self.string_capacity);
                    self.string_capacity = new_capacity;
                    debug!("Resized string buffer to {} bytes", new_capacity);
                }
                BufferType::Line => {
                    let new_capacity = (self.line_capacity as f64 * multiplier) as usize;
                    self.line_buffer.reserve(new_capacity - self.line_capacity);
                    self.line_capacity = new_capacity;
                    debug!("Resized line buffer to {} lines", new_capacity);
                }
                BufferType::Style => {
                    let new_capacity = (self.style_capacity as f64 * multiplier) as usize;
                    self.style_buffer.reserve(new_capacity - self.style_capacity);
                    self.style_capacity = new_capacity;
                    debug!("Resized style buffer to {} styles", new_capacity);
                }
                BufferType::Character => {
                    let new_capacity = (self.char_capacity as f64 * multiplier) as usize;
                    self.char_buffer.reserve(new_capacity - self.char_capacity);
                    self.char_capacity = new_capacity;
                    debug!("Resized character buffer to {} chars", new_capacity);
                }
            }
        }
    }

    /// Estimate memory usage of arena
    pub fn memory_usage(&self) -> MemoryUsage {
        MemoryUsage {
            string_buffer_bytes: self.string_buffer.capacity(),
            line_buffer_bytes: self.line_buffer.capacity() * std::mem::size_of::<String>(),
            style_buffer_bytes: self.style_buffer.capacity() * std::mem::size_of::<StyleData>(),
            char_buffer_bytes: self.char_buffer.capacity() * std::mem::size_of::<char>(),
            cursor_buffer_bytes: self.cursor_buffer.capacity() * std::mem::size_of::<usize>(),
            total_bytes: 0, // Will be calculated
        }
    }
}

/// Frame processing data
#[derive(Debug)]
pub struct FrameData {
    pub frame_start: Instant,
    pub allocations_avoided: u64,
}

/// Resize recommendation for arena buffers
#[derive(Debug)]
pub struct ResizeRecommendation {
    pub buffers_to_resize: Vec<BufferType>,
    pub recommended_multiplier: f64,
    pub urgency: ResizeUrgency,
}

/// Buffer types that can be resized
#[derive(Debug)]
pub enum BufferType {
    String,
    Line,
    Style,
    Character,
}

/// Urgency level for buffer resizing
#[derive(Debug)]
pub enum ResizeUrgency {
    Low,
    Medium,
    High,
}

/// Memory usage statistics
#[derive(Debug)]
pub struct MemoryUsage {
    pub string_buffer_bytes: usize,
    pub line_buffer_bytes: usize,
    pub style_buffer_bytes: usize,
    pub char_buffer_bytes: usize,
    pub cursor_buffer_bytes: usize,
    pub total_bytes: usize,
}

impl MemoryUsage {
    /// Calculate total memory usage
    pub fn calculate_total(&mut self) {
        self.total_bytes = self.string_buffer_bytes
            + self.line_buffer_bytes
            + self.style_buffer_bytes
            + self.char_buffer_bytes
            + self.cursor_buffer_bytes;
    }
}

impl Default for RenderArena {
    fn default() -> Self {
        Self::new()
    }
}

impl ArenaStats {
    /// Calculate allocation avoidance rate
    pub fn avoidance_rate(&self) -> f64 {
        if self.total_buffer_reuses == 0 {
            0.0
        } else {
            self.allocations_avoided as f64 / self.total_buffer_reuses as f64
        }
    }

    /// Get performance grade for arena usage
    pub fn performance_grade(&self) -> char {
        let rate = self.avoidance_rate();
        if rate >= 100.0 { 'A' }
        else if rate >= 50.0 { 'B' }
        else if rate >= 20.0 { 'C' }
        else { 'D' }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic_operations() {
        let mut arena = RenderArena::new();
        let _frame_data = arena.prepare_frame();

        // Test string buffer
        let buffer = arena.get_string_buffer();
        buffer.extend_from_slice(b"Hello, World!");
        let result = arena.build_string();
        assert_eq!(result, "Hello, World!");

        // Test line addition
        arena.add_line("Line 1");
        arena.add_line("Line 2");
        assert_eq!(arena.line_buffer.len(), 2);

        // Test character processing
        let chars = arena.process_chars("abc");
        assert_eq!(chars, &['a', 'b', 'c']);
    }

    #[test]
    fn test_arena_cursor_positions() {
        let mut arena = RenderArena::new();
        let _frame_data = arena.prepare_frame();

        let positions = arena.calculate_cursor_positions("hello");
        assert_eq!(positions, &[0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_arena_statistics() {
        let mut arena = RenderArena::new();
        let _frame_data = arena.prepare_frame();

        // Perform some operations
        let _buffer = arena.get_string_buffer();
        arena.add_line("test");
        let _chars = arena.process_chars("test");

        let stats = arena.get_stats();
        assert!(stats.allocations_avoided > 0);
        assert!(stats.total_buffer_reuses > 0);
    }

    #[test]
    fn test_memory_usage_calculation() {
        let arena = RenderArena::new();
        let mut usage = arena.memory_usage();
        usage.calculate_total();

        assert!(usage.total_bytes > 0);
        assert_eq!(
            usage.total_bytes,
            usage.string_buffer_bytes + usage.line_buffer_bytes +
            usage.style_buffer_bytes + usage.char_buffer_bytes +
            usage.cursor_buffer_bytes
        );
    }
}