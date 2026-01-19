//! Display utilities for TUI rendering
//!
//! Functions for cleaning, truncating, and formatting strings for terminal display.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::utils::display::{truncate_to_width, format_size, clean_for_display};
//!
//! // Truncate long text
//! let truncated = truncate_to_width("Hello, this is a very long string", 15);
//! assert_eq!(truncated, "Hello, this ...");
//!
//! // Format file sizes
//! assert_eq!(format_size(1024), "1.0 KB");
//! assert_eq!(format_size(1048576), "1.0 MB");
//!
//! // Clean text for display
//! let clean = clean_for_display("Line with \rcarriage return");
//! assert_eq!(clean, "carriage return");
//! ```

use std::sync::LazyLock;

use regex::Regex;
use unicode_width::UnicodeWidthStr;

/// Regex to match ANSI escape sequences (colors, cursor movement, etc.)
static ANSI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Matches:
    // - CSI sequences: \x1b[ ... (params) ... (final byte)
    // - OSC sequences: \x1b] ... \x07 or \x1b]...\x1b\\
    // - Simple escapes: \x1b followed by single char
    Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]|\x1b\][^\x07]*\x07|\x1b\][^\x1b]*\x1b\\|\x1b.").unwrap()
});

/// Clean a string for TUI display.
///
/// This function:
/// 1. Handles carriage returns (keeps only text after last `\r`, like a terminal)
/// 2. Strips ANSI escape sequences
/// 3. Removes other control characters (except space)
///
/// # Example
///
/// ```rust
/// use ratatui_interact::utils::display::clean_for_display;
///
/// let clean = clean_for_display("Progress: 50%\rProgress: 100%");
/// assert_eq!(clean, "Progress: 100%");
///
/// let no_ansi = clean_for_display("\x1b[31mRed\x1b[0m text");
/// assert_eq!(no_ansi, "Red text");
/// ```
pub fn clean_for_display(s: &str) -> String {
    // Handle carriage returns - terminals overwrite from beginning of line
    // So we only want the text after the last \r (if any)
    let after_cr = if let Some(last_cr_pos) = s.rfind('\r') {
        &s[last_cr_pos + 1..]
    } else {
        s
    };

    // Strip ANSI escape sequences
    let no_ansi = ANSI_REGEX.replace_all(after_cr, "");

    // Remove other control characters (except space) that could mess up display
    no_ansi
        .chars()
        .filter(|c| !c.is_control() || *c == ' ')
        .collect()
}

/// Strip only ANSI escape sequences from a string, preserving other content.
///
/// Unlike `clean_for_display`, this does not handle carriage returns
/// or strip other control characters.
///
/// # Example
///
/// ```rust
/// use ratatui_interact::utils::display::strip_ansi;
///
/// let plain = strip_ansi("\x1b[1;32mBold green\x1b[0m");
/// assert_eq!(plain, "Bold green");
/// ```
pub fn strip_ansi(s: &str) -> String {
    ANSI_REGEX.replace_all(s, "").to_string()
}

/// Truncate a string to fit within a maximum display width.
///
/// Returns the truncated string with "..." suffix if truncation occurred.
/// Uses Unicode display width for proper handling of wide characters.
/// Control characters and ANSI sequences are stripped before width calculation.
///
/// # Example
///
/// ```rust
/// use ratatui_interact::utils::display::truncate_to_width;
///
/// assert_eq!(truncate_to_width("Hello", 10), "Hello");
/// assert_eq!(truncate_to_width("Hello World!", 8), "Hello...");
/// ```
pub fn truncate_to_width(s: &str, max_width: usize) -> String {
    // Clean the string first - handles \r, ANSI codes, and control chars
    let clean = clean_for_display(s);
    let width = clean.width();

    if width <= max_width {
        return clean;
    }

    // Need to truncate - find where to cut
    let target_width = max_width.saturating_sub(3); // Reserve space for "..."
    let mut current_width = 0;
    let mut end_idx = 0;

    for (idx, ch) in clean.char_indices() {
        let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_width + ch_width > target_width {
            break;
        }
        current_width += ch_width;
        end_idx = idx + ch.len_utf8();
    }

    format!("{}...", &clean[..end_idx])
}

/// Pad a string to a specific display width with spaces.
///
/// If the string is already wider than target, it's returned as-is.
/// Uses Unicode display width for proper handling of wide characters.
///
/// Note: Assumes input has already been stripped of ANSI codes if needed.
///
/// # Example
///
/// ```rust
/// use ratatui_interact::utils::display::pad_to_width;
///
/// assert_eq!(pad_to_width("Hi", 5), "Hi   ");
/// assert_eq!(pad_to_width("Hello", 3), "Hello"); // Already wider
/// ```
pub fn pad_to_width(s: &str, target_width: usize) -> String {
    let width = s.width();
    if width >= target_width {
        return s.to_string();
    }

    let padding = target_width - width;
    format!("{}{}", s, " ".repeat(padding))
}

/// Format a byte count as a human-readable file size.
///
/// # Example
///
/// ```rust
/// use ratatui_interact::utils::display::format_size;
///
/// assert_eq!(format_size(512), "512 B");
/// assert_eq!(format_size(1024), "1.0 KB");
/// assert_eq!(format_size(1536), "1.5 KB");
/// assert_eq!(format_size(1048576), "1.0 MB");
/// assert_eq!(format_size(1073741824), "1.0 GB");
/// ```
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Calculate the display width of a string.
///
/// This is a convenience wrapper around `unicode_width::UnicodeWidthStr::width()`.
///
/// # Example
///
/// ```rust
/// use ratatui_interact::utils::display::display_width;
///
/// assert_eq!(display_width("Hello"), 5);
/// assert_eq!(display_width("你好"), 4); // CJK characters are 2 cells wide
/// ```
pub fn display_width(s: &str) -> usize {
    s.width()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_for_display_carriage_return() {
        assert_eq!(clean_for_display("abc\rdef"), "def");
        assert_eq!(clean_for_display("no cr here"), "no cr here");
    }

    #[test]
    fn test_clean_for_display_ansi() {
        assert_eq!(clean_for_display("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(
            clean_for_display("\x1b[1;32mbold green\x1b[0m"),
            "bold green"
        );
    }

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m text"), "red text");
    }

    #[test]
    fn test_truncate_to_width() {
        assert_eq!(truncate_to_width("short", 10), "short");
        assert_eq!(truncate_to_width("this is a long string", 10), "this is...");
    }

    #[test]
    fn test_pad_to_width() {
        assert_eq!(pad_to_width("hi", 5), "hi   ");
        assert_eq!(pad_to_width("hello", 3), "hello");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }
}
