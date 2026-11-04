//! Toast notification widget
//!
//! A transient notification popup that displays a message for a configurable duration.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{Toast, ToastState, ToastStyle};
//! use ratatui::layout::Rect;
//!
//! // Create state
//! let mut state = ToastState::new();
//!
//! // Show a toast for 3 seconds
//! state.show("File saved successfully!", 3000);
//!
//! // In your render function
//! if let Some(message) = state.get_message() {
//!     let toast = Toast::new(message).style(ToastStyle::Info);
//!     // render toast...
//! }
//!
//! // In your event loop, periodically clear expired toasts
//! state.clear_if_expired();
//! ```

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};
use std::time::{Duration, Instant};
use unicode_width::UnicodeWidthStr;

/// Style variants for toast notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastStyle {
    /// Default informational style (cyan border)
    #[default]
    Info,
    /// Success style (green border)
    Success,
    /// Warning style (yellow border)
    Warning,
    /// Error style (red border)
    Error,
}

impl ToastStyle {
    /// Get the border color for this style
    pub fn border_color(&self) -> Color {
        match self {
            ToastStyle::Info => Color::Cyan,
            ToastStyle::Success => Color::Green,
            ToastStyle::Warning => Color::Yellow,
            ToastStyle::Error => Color::Red,
        }
    }

    /// Get the border color for this style, derived from a theme palette.
    pub fn themed_border_color(&self, theme: &crate::theme::Theme) -> Color {
        let p = &theme.palette;
        match self {
            ToastStyle::Info => p.info,
            ToastStyle::Success => p.success,
            ToastStyle::Warning => p.warning,
            ToastStyle::Error => p.error,
        }
    }

    /// Auto-detect style from message content
    pub fn from_message(message: &str) -> Self {
        let lower = message.to_lowercase();
        if lower.contains("error") || lower.contains("fail") {
            ToastStyle::Error
        } else if lower.contains("warning") || lower.contains("warn") {
            ToastStyle::Warning
        } else if lower.contains("success") || lower.contains("saved") || lower.contains("done") {
            ToastStyle::Success
        } else {
            ToastStyle::Info
        }
    }
}

/// State for managing toast visibility and expiration
///
/// Expiration uses a monotonic clock ([`Instant`]), so it is not affected by
/// changes to the system wall clock.
#[derive(Debug, Clone, Default)]
pub struct ToastState {
    /// Current message to display (if any)
    message: Option<String>,
    /// When the toast was shown and for how long it stays visible
    expires_at: Option<(Instant, Duration)>,
}

impl ToastState {
    /// Create a new toast state
    pub fn new() -> Self {
        Self::default()
    }

    /// Show a toast message for a specified duration (in milliseconds)
    ///
    /// A zero or negative duration shows a toast that is already expired.
    /// Very large durations (up to `i64::MAX`) are fine and effectively never expire.
    pub fn show(&mut self, message: impl Into<String>, duration_ms: i64) {
        let duration = Duration::from_millis(u64::try_from(duration_ms).unwrap_or(0));
        self.message = Some(message.into());
        self.expires_at = Some((Instant::now(), duration));
    }

    /// Get the current message if the toast hasn't expired
    pub fn get_message(&self) -> Option<&str> {
        match (&self.message, self.expires_at) {
            (Some(msg), Some(expires)) if !Self::is_expired(expires) => Some(msg.as_str()),
            _ => None,
        }
    }

    /// Check if a toast is currently visible
    pub fn is_visible(&self) -> bool {
        self.get_message().is_some()
    }

    /// Clear the toast if it has expired
    pub fn clear_if_expired(&mut self) {
        if let Some(expires) = self.expires_at {
            if Self::is_expired(expires) {
                self.message = None;
                self.expires_at = None;
            }
        }
    }

    /// Force clear the toast immediately
    pub fn clear(&mut self) {
        self.message = None;
        self.expires_at = None;
    }

    /// Whether a toast shown at `shown_at` for `duration` has expired
    fn is_expired((shown_at, duration): (Instant, Duration)) -> bool {
        shown_at.elapsed() >= duration
    }
}

/// Compute the `(width, height)` of a toast showing `message` within `area`.
///
/// The width fits the widest line (by display width) plus padding, is at least
/// 20 columns where possible, and never exceeds `area`. The height counts the
/// wrapped display-width lines of every message line plus the borders, capped
/// by `max_height` and `area`.
pub(crate) fn toast_size(message: &str, area: Rect, max_width: u16, max_height: u16) -> (u16, u16) {
    let max_content_width = (area.width as usize)
        .saturating_sub(8)
        .min(max_width as usize);
    let text_width = message.lines().map(|l| l.width()).max().unwrap_or(0);
    let content_width = text_width + 4; // padding
    let toast_width = content_width
        .min(max_content_width)
        .max(20)
        .min(area.width as usize) as u16;

    // Calculate height based on text wrapping
    let inner_width = (toast_width.saturating_sub(2) as usize).max(1); // account for borders
    let lines_needed: usize = message
        .lines()
        .map(|l| l.width().div_ceil(inner_width).max(1))
        .sum::<usize>()
        .max(1);
    let toast_height = u16::try_from(lines_needed)
        .unwrap_or(u16::MAX)
        .saturating_add(2) // +2 for borders
        .min(max_height)
        .min(area.height);

    (toast_width, toast_height)
}

/// Toast notification widget
///
/// Renders a centered popup with the given message.
#[derive(Debug, Clone)]
pub struct Toast<'a> {
    message: &'a str,
    style: ToastStyle,
    auto_style: bool,
    max_width: u16,
    max_height: u16,
    top_offset: u16,
}

impl<'a> Toast<'a> {
    /// Create a new toast with the given message
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            style: ToastStyle::Info,
            auto_style: true,
            max_width: 80,
            max_height: 8,
            top_offset: 3,
        }
    }

    /// Set the toast style
    ///
    /// This disables auto-style detection.
    pub fn style(mut self, style: ToastStyle) -> Self {
        self.style = style;
        self.auto_style = false;
        self
    }

    /// Enable auto-style detection from message content
    pub fn auto_style(mut self) -> Self {
        self.auto_style = true;
        self
    }

    /// Set the maximum width of the toast
    pub fn max_width(mut self, width: u16) -> Self {
        self.max_width = width;
        self
    }

    /// Set the maximum height of the toast
    pub fn max_height(mut self, height: u16) -> Self {
        self.max_height = height;
        self
    }

    /// Set the offset from the top of the area
    pub fn top_offset(mut self, offset: u16) -> Self {
        self.top_offset = offset;
        self
    }

    /// Calculate the toast area centered within the given area
    pub fn calculate_area(&self, area: Rect) -> Rect {
        // Calculate toast dimensions (always within `area`)
        let (toast_width, toast_height) =
            toast_size(self.message, area, self.max_width, self.max_height);

        // Center horizontally and position from top
        let x = area.x + (area.width.saturating_sub(toast_width)) / 2;
        let y = area.y
            + self
                .top_offset
                .min(area.height.saturating_sub(toast_height));

        Rect::new(x, y, toast_width, toast_height)
    }

    /// Render the toast, clearing the area behind it
    ///
    /// This is the preferred method as it ensures the toast appears on top.
    pub fn render_with_clear(self, area: Rect, buf: &mut Buffer) {
        let toast_area = self.calculate_area(area).intersection(*buf.area());

        // Clear the area behind the toast
        Clear.render(toast_area, buf);

        // Render the toast
        self.render_in_area(toast_area, buf);
    }

    /// Render the toast in a specific pre-calculated area
    fn render_in_area(self, area: Rect, buf: &mut Buffer) {
        let border_color = if self.auto_style {
            ToastStyle::from_message(self.message).border_color()
        } else {
            self.style.border_color()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(Color::Black));

        let paragraph = Paragraph::new(self.message)
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::White));

        paragraph.render(area, buf);
    }
}

impl Widget for Toast<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // When used as a Widget directly, render in the given area
        // For proper centering, use render_with_clear instead
        self.render_in_area(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_state_new() {
        let state = ToastState::new();
        assert!(state.message.is_none());
        assert!(state.expires_at.is_none());
    }

    #[test]
    fn test_show_huge_duration_does_not_overflow() {
        let mut state = ToastState::new();
        state.show("sticky", i64::MAX);
        assert!(state.is_visible());
        state.clear_if_expired();
        assert!(state.is_visible());
    }

    #[test]
    fn test_show_negative_duration_is_expired() {
        let mut state = ToastState::new();
        state.show("gone", -1);
        assert!(!state.is_visible());
        state.show("gone", i64::MIN);
        assert!(!state.is_visible());
    }

    #[test]
    fn test_render_with_clear_narrow_area() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 15, 10));
        Toast::new("hi").render_with_clear(buf.area, &mut buf);
    }

    #[test]
    fn test_render_with_clear_short_area() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 2));
        Toast::new("hi").render_with_clear(buf.area, &mut buf);
    }

    #[test]
    fn test_toast_stays_inside_narrow_area() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 10));
        for x in 0..40 {
            for y in 0..10 {
                buf[(x, y)].set_symbol("X");
            }
        }
        let pane = Rect::new(0, 0, 15, 10);
        Toast::new("hi").render_with_clear(pane, &mut buf);
        let r = Toast::new("hi").calculate_area(pane);
        assert!(r.x + r.width <= 15, "toast rect {r:?}");
        let outside: Vec<&str> = (15..40).map(|x| buf[(x, 3)].symbol()).collect();
        assert!(
            outside.iter().all(|s| *s == "X"),
            "toast drew outside its 15-col pane: {outside:?}"
        );
    }

    #[test]
    fn test_multiline_height() {
        let toast = Toast::new("line one\nline two\nline three");
        let r = toast.calculate_area(Rect::new(0, 0, 80, 24));
        assert_eq!(r.height, 5, "3 lines + 2 borders");
    }

    #[test]
    fn test_wide_chars_size() {
        // 10 wide chars = 20 columns (but only 30 bytes)
        let r = Toast::new("日本語日本語日本語日").calculate_area(Rect::new(0, 0, 80, 24));
        assert_eq!(r.width, 24);
        assert_eq!(r.height, 3);
    }

    #[test]
    fn test_toast_state_lifecycle() {
        let mut state = ToastState::new();

        // Initially no message
        assert!(state.get_message().is_none());
        assert!(!state.is_visible());

        // Show a toast with very long duration
        state.show("Test message", 100_000);
        assert!(state.is_visible());
        assert_eq!(state.get_message(), Some("Test message"));

        // Force clear
        state.clear();
        assert!(!state.is_visible());
    }

    #[test]
    fn test_toast_show_replaces_existing() {
        let mut state = ToastState::new();

        state.show("First message", 100_000);
        assert_eq!(state.get_message(), Some("First message"));

        state.show("Second message", 100_000);
        assert_eq!(state.get_message(), Some("Second message"));
    }

    #[test]
    fn test_toast_clear_if_expired() {
        let mut state = ToastState::new();

        // Show toast that expires immediately (duration 0)
        state.show("Quick message", 0);
        std::thread::sleep(std::time::Duration::from_millis(10));

        state.clear_if_expired();
        assert!(!state.is_visible());
    }

    #[test]
    fn test_toast_style_detection() {
        assert_eq!(
            ToastStyle::from_message("error occurred"),
            ToastStyle::Error
        );
        assert_eq!(ToastStyle::from_message("File saved"), ToastStyle::Success);
        assert_eq!(
            ToastStyle::from_message("Warning: low disk"),
            ToastStyle::Warning
        );
        assert_eq!(ToastStyle::from_message("Hello world"), ToastStyle::Info);
    }

    #[test]
    fn test_toast_style_detection_case_insensitive() {
        assert_eq!(
            ToastStyle::from_message("ERROR OCCURRED"),
            ToastStyle::Error
        );
        assert_eq!(ToastStyle::from_message("FILE SAVED"), ToastStyle::Success);
        assert_eq!(ToastStyle::from_message("WARNING"), ToastStyle::Warning);
    }

    #[test]
    fn test_toast_style_detection_variants() {
        assert_eq!(
            ToastStyle::from_message("failed to load"),
            ToastStyle::Error
        );
        assert_eq!(
            ToastStyle::from_message("done processing"),
            ToastStyle::Success
        );
        assert_eq!(
            ToastStyle::from_message("warn: deprecated"),
            ToastStyle::Warning
        );
    }

    #[test]
    fn test_toast_style_colors() {
        assert_eq!(ToastStyle::Info.border_color(), Color::Cyan);
        assert_eq!(ToastStyle::Success.border_color(), Color::Green);
        assert_eq!(ToastStyle::Warning.border_color(), Color::Yellow);
        assert_eq!(ToastStyle::Error.border_color(), Color::Red);
    }

    #[test]
    fn test_toast_style_default() {
        let style = ToastStyle::default();
        assert_eq!(style, ToastStyle::Info);
    }

    #[test]
    fn test_toast_area_calculation() {
        let toast = Toast::new("Hello");
        let area = Rect::new(0, 0, 100, 50);
        let toast_area = toast.calculate_area(area);

        // Should be centered horizontally
        assert!(toast_area.x > 0);
        assert!(toast_area.x + toast_area.width <= area.width);

        // Should be near top
        assert_eq!(toast_area.y, 3); // default top_offset
    }

    #[test]
    fn test_toast_area_calculation_long_message() {
        let long_msg = "This is a very long message that should wrap to multiple lines";
        let toast = Toast::new(long_msg);
        let area = Rect::new(0, 0, 40, 20);
        let toast_area = toast.calculate_area(area);

        // Width should be constrained
        assert!(toast_area.width <= area.width);
    }

    #[test]
    fn test_toast_area_calculation_custom_offset() {
        let toast = Toast::new("Hello").top_offset(10);
        let area = Rect::new(0, 0, 100, 50);
        let toast_area = toast.calculate_area(area);

        assert_eq!(toast_area.y, 10);
    }

    #[test]
    fn test_toast_builder_methods() {
        let toast = Toast::new("Test")
            .style(ToastStyle::Error)
            .max_width(60)
            .max_height(5)
            .top_offset(5);

        assert_eq!(toast.style, ToastStyle::Error);
        assert_eq!(toast.max_width, 60);
        assert_eq!(toast.max_height, 5);
        assert_eq!(toast.top_offset, 5);
        assert!(!toast.auto_style); // auto_style disabled when style is set
    }

    #[test]
    fn test_toast_auto_style() {
        let toast = Toast::new("Test").auto_style();
        assert!(toast.auto_style);
    }

    #[test]
    fn test_toast_render() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 20));
        let toast = Toast::new("Test toast message");

        toast.render_with_clear(Rect::new(0, 0, 60, 20), &mut buf);

        // Check that something was rendered (borders at least)
        // The toast should contain the message text
        let content: String = buf.content.iter().map(|c| c.symbol()).collect();
        assert!(content.contains("Test"));
    }

    #[test]
    fn test_toast_render_with_style() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 20));
        let toast = Toast::new("Success!").style(ToastStyle::Success);

        toast.render_with_clear(Rect::new(0, 0, 60, 20), &mut buf);

        let content: String = buf.content.iter().map(|c| c.symbol()).collect();
        assert!(content.contains("Success"));
    }

    #[test]
    fn test_toast_widget_render() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 5));
        let toast = Toast::new("Widget test");
        let area = Rect::new(0, 0, 30, 5);

        toast.render(area, &mut buf);
        // Should not panic
    }
}
