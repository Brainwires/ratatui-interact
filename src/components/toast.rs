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
#[derive(Debug, Clone, Default)]
pub struct ToastState {
    /// Current message to display (if any)
    message: Option<String>,
    /// Expiration time (epoch milliseconds)
    expires_at: Option<i64>,
}

impl ToastState {
    /// Create a new toast state
    pub fn new() -> Self {
        Self::default()
    }

    /// Show a toast message for a specified duration (in milliseconds)
    pub fn show(&mut self, message: impl Into<String>, duration_ms: i64) {
        let now = Self::current_time_ms();
        self.message = Some(message.into());
        self.expires_at = Some(now + duration_ms);
    }

    /// Get the current message if the toast hasn't expired
    pub fn get_message(&self) -> Option<&str> {
        if let (Some(msg), Some(expires)) = (&self.message, self.expires_at) {
            let now = Self::current_time_ms();
            if now < expires {
                return Some(msg.as_str());
            }
        }
        None
    }

    /// Check if a toast is currently visible
    pub fn is_visible(&self) -> bool {
        self.get_message().is_some()
    }

    /// Clear the toast if it has expired
    pub fn clear_if_expired(&mut self) {
        if let Some(expires) = self.expires_at {
            let now = Self::current_time_ms();
            if now >= expires {
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

    /// Get current time in milliseconds since epoch
    fn current_time_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }
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
        // Calculate toast dimensions
        let max_content_width = (area.width as usize)
            .saturating_sub(8)
            .min(self.max_width as usize);
        let content_width = self.message.len() + 4; // padding
        let toast_width = content_width.min(max_content_width).max(20) as u16;

        // Calculate height based on text wrapping
        let inner_width = toast_width.saturating_sub(2) as usize; // account for borders
        let lines_needed = (self.message.len() + inner_width - 1) / inner_width.max(1);
        let toast_height = (lines_needed as u16 + 2).min(self.max_height); // +2 for borders

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
        let toast_area = self.calculate_area(area);

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
    fn test_toast_style_colors() {
        assert_eq!(ToastStyle::Info.border_color(), Color::Cyan);
        assert_eq!(ToastStyle::Success.border_color(), Color::Green);
        assert_eq!(ToastStyle::Warning.border_color(), Color::Yellow);
        assert_eq!(ToastStyle::Error.border_color(), Color::Red);
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
    fn test_toast_render() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 20));
        let toast = Toast::new("Test toast message");

        toast.render_with_clear(Rect::new(0, 0, 60, 20), &mut buf);

        // Check that something was rendered (borders at least)
        // The toast should contain the message text
        let content: String = buf.content.iter().map(|c| c.symbol()).collect();
        assert!(content.contains("Test"));
    }
}
