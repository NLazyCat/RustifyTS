//! Source code location tracking
//!
//! This module provides types for tracking source code locations using zero-based
//! byte offsets internally and converting to 1-based line/column for display.

use std::fmt;
use std::ops::Range;

/// A span representing a range in source code
///
/// Uses zero-based byte offsets internally. When converting to line/column
/// for display purposes, both are converted to 1-based indexing for human readability.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span {
    /// Start byte offset (zero-based)
    start: usize,
    /// End byte offset (zero-based, exclusive)
    end: usize,
}

impl Span {
    /// Create a new span from start and end byte offsets
    ///
    /// # Arguments
    ///
    /// * `start` - Start byte offset (zero-based)
    /// * `end` - End byte offset (zero-based, exclusive)
    ///
    /// # Panics
    ///
    /// Panics if `end < start`
    ///
    /// # Example
    ///
    /// ```
    /// # use crate::parser::ast::Span;
    /// let span = Span::new(10, 20);
    /// assert_eq!(span.start(), 10);
    /// assert_eq!(span.end(), 20);
    /// ```
    #[inline]
    pub const fn new(start: usize, end: usize) -> Self {
        if end < start {
            panic!("Span end must be >= start");
        }
        Span { start, end }
    }

    /// Create a new span at a single point (zero-length span)
    ///
    /// # Arguments
    ///
    /// * `position` - Byte offset where the span is located
    ///
    /// # Example
    ///
    /// ```
    /// # use crate::parser::ast::Span;
    /// let span = Span::point(10);
    /// assert_eq!(span.len(), 0);
    /// assert_eq!(span.start(), 10);
    /// ```
    #[inline]
    pub const fn point(position: usize) -> Self {
        Span {
            start: position,
            end: position,
        }
    }

    /// Create an empty span at position 0
    #[inline]
    pub const fn empty() -> Self {
        Span { start: 0, end: 0 }
    }

    /// Get the start byte offset (zero-based)
    #[inline]
    pub const fn start(self) -> usize {
        self.start
    }

    /// Get the end byte offset (zero-based, exclusive)
    #[inline]
    pub const fn end(self) -> usize {
        self.end
    }

    /// Get the length of the span in bytes
    #[inline]
    pub const fn len(self) -> usize {
        self.end - self.start
    }

    /// Check if the span is empty (zero-length)
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Check if the span contains the given byte offset
    ///
    /// # Arguments
    ///
    /// * `offset` - Byte offset to check
    ///
    /// # Example
    ///
    /// ```
    /// # use crate::parser::ast::Span;
    /// let span = Span::new(10, 20);
    /// assert!(span.contains(15));
    /// assert!(!span.contains(9));
    /// assert!(!span.contains(20));
    /// ```
    #[inline]
    pub fn contains(self, offset: usize) -> bool {
        offset >= self.start && offset < self.end
    }

    /// Extend this span to include another span
    ///
    /// # Arguments
    ///
    /// * `other` - Span to include
    ///
    /// # Example
    ///
    /// ```
    /// # use crate::parser::ast::Span;
    /// let span1 = Span::new(10, 20);
    /// let span2 = Span::new(30, 40);
    /// let extended = span1.extend(span2);
    /// assert_eq!(extended.start(), 10);
    /// assert_eq!(extended.end(), 40);
    /// ```
    #[inline]
    pub fn extend(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }

    /// Get the span as a Range<usize>
    #[inline]
    pub fn as_range(self) -> Range<usize> {
        self.start..self.end
    }

    /// Merge two spans into a new span that covers both
    #[inline]
    pub fn merge(span1: Span, span2: Span) -> Span {
        span1.extend(span2)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A line map for converting byte offsets to line/column positions
///
/// Stores the byte offset of each line start, enabling O(log n) lookup
/// of line and column positions from byte offsets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineMap {
    /// Byte offset of each line start (zero-based)
    line_starts: Vec<usize>,
    /// Total length of the source code
    source_len: usize,
}

impl LineMap {
    /// Create a line map from source code
    ///
    /// # Arguments
    ///
    /// * `source` - The source code to analyze
    ///
    /// # Example
    ///
    /// ```
    /// # use crate::parser::ast::LineMap;
    /// let source = "line 1\nline 2\nline 3";
    /// let line_map = LineMap::from_source(source);
    /// ```
    pub fn from_source(source: &str) -> Self {
        let mut line_starts = vec![0]; // Line 1 starts at byte 0

        for (idx, ch) in source.char_indices() {
            if ch == '\n' {
                // Next line starts after the newline character
                line_starts.push(idx + ch.len_utf8());
            }
        }

        // Store the total source length
        let source_len = source.len();

        LineMap {
            line_starts,
            source_len,
        }
    }

    /// Get the line and column (1-based) for a byte offset
    ///
    /// # Arguments
    ///
    /// * `byte_offset` - Byte offset in the source code
    ///
    /// # Returns
    ///
    /// A tuple of (line, column) where both are 1-based for display
    ///
    /// # Example
    ///
    /// ```
    /// # use crate::parser::ast::LineMap;
    /// let source = "line 1\nline 2\nline 3";
    /// let line_map = LineMap::from_source(source);
    /// let (line, col) = line_map.line_col(10);
    /// // line and col are 1-based
    /// ```
    pub fn line_col(&self, byte_offset: usize) -> (usize, usize) {
        // Clamp to source length
        let byte_offset = byte_offset.min(self.source_len);

        // Find the line using binary search
        let line_idx = match self.line_starts.binary_search(&byte_offset) {
            Ok(i) => i, // Exact match - byte offset is at line start
            Err(i) => i.saturating_sub(1), // Find previous line start
        };

        // Get the line start offset
        let line_start = self.line_starts[line_idx];

        // Calculate column (1-based)
        let column = (byte_offset - line_start) + 1;

        // Return 1-based line and column
        (line_idx + 1, column)
    }

    /// Get the number of lines in the source
    #[inline]
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Get the byte offset at the start of a line (1-based)
    ///
    /// # Arguments
    ///
    /// * `line` - Line number (1-based)
    ///
    /// # Returns
    ///
    /// Byte offset of the line start, or None if line is out of bounds
    pub fn line_start(&self, line: usize) -> Option<usize> {
        let line_idx = line.saturating_sub(1);
        self.line_starts.get(line_idx).copied()
    }

    /// Get total source length in bytes
    #[inline]
    pub fn source_len(&self) -> usize {
        self.source_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Span tests

    #[test]
    fn test_span_new() {
        let span = Span::new(10, 20);
        assert_eq!(span.start(), 10);
        assert_eq!(span.end(), 20);
        assert_eq!(span.len(), 10);
    }

    #[test]
    #[should_panic]
    fn test_span_invalid() {
        Span::new(20, 10);
    }

    #[test]
    fn test_span_point() {
        let span = Span::point(10);
        assert_eq!(span.start(), 10);
        assert_eq!(span.end(), 10);
        assert_eq!(span.len(), 0);
        assert!(span.is_empty());
    }

    #[test]
    fn test_span_empty() {
        let span = Span::empty();
        assert_eq!(span.start(), 0);
        assert_eq!(span.end(), 0);
        assert!(span.is_empty());
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(10, 20);

        assert!(!span.contains(9));  // Before start
        assert!(span.contains(10)); // At start
        assert!(span.contains(15)); // In middle
        assert!(span.contains(19)); // At end - 1
        assert!(!span.contains(20)); // At end (exclusive)
        assert!(!span.contains(21)); // After end
    }

    #[test]
    fn test_span_extend() {
        let span1 = Span::new(10, 20);
        let span2 = Span::new(30, 40);
        let extended = span1.extend(span2);

        assert_eq!(extended.start(), 10);
        assert_eq!(extended.end(), 40);
    }

    #[test]
    fn test_span_extend_reverse() {
        let span1 = Span::new(30, 40);
        let span2 = Span::new(10, 20);
        let extended = span1.extend(span2);

        assert_eq!(extended.start(), 10);
        assert_eq!(extended.end(), 40);
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(10, 20);
        let span2 = Span::new(30, 40);
        let merged = Span::merge(span1, span2);

        assert_eq!(merged.start(), 10);
        assert_eq!(merged.end(), 40);
    }

    #[test]
    fn test_span_as_range() {
        let span = Span::new(10, 20);
        let range = span.as_range();

        assert_eq!(range.start, 10);
        assert_eq!(range.end, 20);
    }

    // LineMap tests

    #[test]
    fn test_line_map_creation() {
        let source = "line 1\nline 2\nline 3";
        let line_map = LineMap::from_source(source);

        assert_eq!(line_map.line_count(), 3);
        assert_eq!(line_map.line_start(1), Some(0));
        assert_eq!(line_map.line_start(2), Some(7));
        assert_eq!(line_map.line_start(3), Some(14));
        assert_eq!(line_map.line_start(4), None);
    }

    #[test]
    fn test_line_map_empty() {
        let source = "";
        let line_map = LineMap::from_source(source);

        assert_eq!(line_map.line_count(), 1);
        assert_eq!(line_map.line_start(1), Some(0));
    }

    #[test]
    fn test_line_map_single_line() {
        let source = "single line";
        let line_map = LineMap::from_source(source);

        assert_eq!(line_map.line_count(), 1);
        assert_eq!(line_map.line_start(1), Some(0));
    }

    #[test]
    fn test_line_col_lookup() {
        let source = "line 1\nline 2\nline 3";
        let line_map = LineMap::from_source(source);

        // Test various positions (1-based display)
        let (line, col) = line_map.line_col(0);
        assert_eq!(line, 1);
        assert_eq!(col, 1);

        let (line, col) = line_map.line_col(5);
        assert_eq!(line, 1);
        assert_eq!(col, 6);

        let (line, col) = line_map.line_col(7); // Start of line 2
        assert_eq!(line, 2);
        assert_eq!(col, 1);

        let (line, col) = line_map.line_col(10);
        assert_eq!(line, 2);
        assert_eq!(col, 4);

        let (line, col) = line_map.line_col(14); // Start of line 3
        assert_eq!(line, 3);
        assert_eq!(col, 1);
    }

    #[test]
    fn test_line_col_out_of_bounds() {
        let source = "line 1\nline 2";
        let line_map = LineMap::from_source(source);

        // Test out of bounds - should clamp to source length
        let len = source.len();
        let (line, col) = line_map.line_col(len + 100);

        assert_eq!(line, 2);
        assert!(col > 0);
    }

    #[test]
    fn test_line_map_windows_line_endings() {
        let source = "line 1\r\nline 2\r\nline 3";
        let line_map = LineMap::from_source(source);

        assert_eq!(line_map.line_count(), 3);
        assert_eq!(line_map.line_start(1), Some(0));
    }

    #[test]
    fn test_line_map_no_trailing_newline() {
        let source = "line 1\nline 2";
        let line_map = LineMap::from_source(source);

        assert_eq!(line_map.line_count(), 2);
        assert_eq!(line_map.line_start(1), Some(0));
        assert_eq!(line_map.line_start(2), Some(7));
    }

    #[test]
    fn test_line_map_trailing_newline() {
        let source = "line 1\nline 2\n";
        let line_map = LineMap::from_source(source);

        assert_eq!(line_map.line_count(), 3);
        assert_eq!(line_map.line_start(1), Some(0));
        assert_eq!(line_map.line_start(2), Some(7));
    }
}
