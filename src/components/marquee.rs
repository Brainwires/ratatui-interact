//! MarqueeText widget
//!
//! A scrolling text widget for displaying long text in constrained spaces.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{MarqueeText, MarqueeState, MarqueeStyle, MarqueeMode};
//! use ratatui::layout::Rect;
//! use ratatui::buffer::Buffer;
//! use ratatui::widgets::Widget;
//!
//! let mut state = MarqueeState::new();
//! let style = MarqueeStyle::default();
//!
//! // Continuous scrolling
//! let marquee = MarqueeText::new("/path/to/very/long/file/name.rs", &mut state)
//!     .style(style);
//!
//! // Bounce mode
//! let style = MarqueeStyle::default().mode(MarqueeMode::Bounce);
//! let mut state = MarqueeState::new();
//! let marquee = MarqueeText::new("Long status message here", &mut state)
//!     .style(style);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_width::UnicodeWidthStr;

/// Scroll direction for bounce mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollDir {
    /// Scrolling left (text moves left, revealing more on right)
    #[default]
    Left,
    /// Scrolling right (text moves right, revealing more on left)
    Right,
}

/// Scrolling mode for the marquee
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarqueeMode {
    /// Text loops around continuously: "Hello World   Hello Wor..."
    #[default]
    Continuous,
    /// Text bounces back and forth at edges
    Bounce,
    /// No scrolling, just truncate with ellipsis
    Static,
}

/// State for tracking marquee animation
#[derive(Debug, Clone, Default)]
pub struct MarqueeState {
    /// Current scroll offset (in display columns)
    pub offset: usize,
    /// Current direction (for bounce mode)
    pub direction: ScrollDir,
    /// Counter for edge pause
    pub paused_ticks: usize,
}

impl MarqueeState {
    /// Create a new marquee state
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the state to initial position
    pub fn reset(&mut self) {
        self.offset = 0;
        self.direction = ScrollDir::Left;
        self.paused_ticks = 0;
    }

    /// Advance the animation by one tick
    ///
    /// # Arguments
    /// * `text_width` - Display width of the text in columns
    /// * `viewport_width` - Width of the visible area in columns
    /// * `style` - The marquee style configuration
    pub fn tick(&mut self, text_width: usize, viewport_width: usize, style: &MarqueeStyle) {
        // Only scroll if text is wider than viewport
        if text_width <= viewport_width {
            self.offset = 0;
            return;
        }

        // Handle edge pause
        if self.paused_ticks > 0 {
            self.paused_ticks -= 1;
            return;
        }

        match style.mode {
            MarqueeMode::Continuous => {
                // For continuous mode, we create a virtual string of:
                // "text + separator + text"
                // and scroll through it, wrapping around
                let total_width = text_width + style.separator.width();
                self.offset = (self.offset + style.scroll_speed) % total_width;
            }
            MarqueeMode::Bounce => {
                // Calculate the maximum offset (how far we can scroll)
                let max_offset = text_width.saturating_sub(viewport_width);

                match self.direction {
                    ScrollDir::Left => {
                        // Scrolling left (offset increases)
                        self.offset = self.offset.saturating_add(style.scroll_speed);
                        if self.offset >= max_offset {
                            self.offset = max_offset;
                            self.direction = ScrollDir::Right;
                            self.paused_ticks = style.pause_at_edge;
                        }
                    }
                    ScrollDir::Right => {
                        // Scrolling right (offset decreases)
                        if self.offset <= style.scroll_speed {
                            self.offset = 0;
                            self.direction = ScrollDir::Left;
                            self.paused_ticks = style.pause_at_edge;
                        } else {
                            self.offset = self.offset.saturating_sub(style.scroll_speed);
                        }
                    }
                }
            }
            MarqueeMode::Static => {
                // No animation
            }
        }
    }
}

/// Style configuration for marquee text
#[derive(Debug, Clone)]
pub struct MarqueeStyle {
    /// Style for the text (color, modifiers)
    pub text_style: Style,
    /// Columns to scroll per tick (default: 1)
    pub scroll_speed: usize,
    /// Ticks to pause at each edge (default: 3)
    pub pause_at_edge: usize,
    /// Scrolling mode
    pub mode: MarqueeMode,
    /// Gap between repeated text for continuous mode (default: "   ")
    pub separator: &'static str,
    /// Ellipsis string for static mode truncation (default: "...")
    pub ellipsis: &'static str,
}

impl Default for MarqueeStyle {
    fn default() -> Self {
        Self {
            text_style: Style::default(),
            scroll_speed: 1,
            pause_at_edge: 3,
            mode: MarqueeMode::default(),
            separator: "   ",
            ellipsis: "...",
        }
    }
}

impl MarqueeStyle {
    /// Create a new style with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the text style
    pub fn text_style(mut self, style: Style) -> Self {
        self.text_style = style;
        self
    }

    /// Set the scroll speed (columns per tick)
    pub fn scroll_speed(mut self, speed: usize) -> Self {
        self.scroll_speed = speed.max(1);
        self
    }

    /// Set the pause duration at edges (in ticks)
    pub fn pause_at_edge(mut self, ticks: usize) -> Self {
        self.pause_at_edge = ticks;
        self
    }

    /// Set the scrolling mode
    pub fn mode(mut self, mode: MarqueeMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the separator for continuous mode
    pub fn separator(mut self, sep: &'static str) -> Self {
        self.separator = sep;
        self
    }

    /// Set the ellipsis for static mode
    pub fn ellipsis(mut self, ellipsis: &'static str) -> Self {
        self.ellipsis = ellipsis;
        self
    }

    /// Create a style for file paths (cyan text, bounce mode)
    pub fn file_path() -> Self {
        Self {
            text_style: Style::default().fg(Color::Cyan),
            mode: MarqueeMode::Bounce,
            pause_at_edge: 5,
            ..Default::default()
        }
    }

    /// Create a style for status messages (yellow text, continuous)
    pub fn status() -> Self {
        Self {
            text_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            mode: MarqueeMode::Continuous,
            scroll_speed: 1,
            ..Default::default()
        }
    }

    /// Create a style for titles (bold text, bounce mode)
    pub fn title() -> Self {
        Self {
            text_style: Style::default().add_modifier(Modifier::BOLD),
            mode: MarqueeMode::Bounce,
            pause_at_edge: 10,
            ..Default::default()
        }
    }
}

/// A scrolling text widget for displaying long text in limited space.
///
/// The marquee can operate in three modes:
/// - `Continuous`: Text loops around with a separator
/// - `Bounce`: Text scrolls back and forth
/// - `Static`: Text is truncated with ellipsis
pub struct MarqueeText<'a> {
    /// The text to display
    text: &'a str,
    /// Style configuration
    style: MarqueeStyle,
    /// Mutable state for animation
    state: &'a mut MarqueeState,
}

impl<'a> MarqueeText<'a> {
    /// Create a new marquee text widget
    pub fn new(text: &'a str, state: &'a mut MarqueeState) -> Self {
        Self {
            text,
            style: MarqueeStyle::default(),
            state,
        }
    }

    /// Set the style
    pub fn style(mut self, style: MarqueeStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the text style directly (shorthand)
    pub fn text_style(mut self, style: Style) -> Self {
        self.style.text_style = style;
        self
    }

    /// Set the mode directly (shorthand)
    pub fn mode(mut self, mode: MarqueeMode) -> Self {
        self.style.mode = mode;
        self
    }

    /// Extract a visible slice from the text based on offset and width
    ///
    /// Returns a string that fits within `width` display columns,
    /// starting from `offset` display columns into the text.
    fn extract_visible_slice(text: &str, offset: usize, width: usize) -> String {
        let mut result = String::new();
        let mut current_col = 0;
        let mut skip_cols = offset;

        for ch in text.chars() {
            let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);

            // Skip characters until we reach the offset
            if skip_cols > 0 {
                if ch_width <= skip_cols {
                    skip_cols -= ch_width;
                    continue;
                } else {
                    // Character spans the boundary, add padding
                    result.push(' ');
                    current_col += 1;
                    skip_cols = 0;
                    continue;
                }
            }

            // Check if this character fits
            if current_col + ch_width > width {
                // If it's a wide character that doesn't fit, add padding
                if ch_width > 1 && current_col + 1 <= width {
                    result.push(' ');
                    current_col += 1;
                }
                break;
            }

            result.push(ch);
            current_col += ch_width;
        }

        // Pad to full width if needed
        while current_col < width {
            result.push(' ');
            current_col += 1;
        }

        result
    }

    /// Render the marquee into the buffer
    fn render_internal(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let viewport_width = area.width as usize;
        let text_width = self.text.width();

        // If text fits, just render it (left-aligned)
        if text_width <= viewport_width {
            let padded = format!("{:<width$}", self.text, width = viewport_width);
            buf.set_string(area.x, area.y, &padded, self.style.text_style);
            return;
        }

        // Handle scrolling modes
        match self.style.mode {
            MarqueeMode::Static => {
                // Truncate with ellipsis
                let ellipsis_width = self.style.ellipsis.width();
                if viewport_width <= ellipsis_width {
                    // Not enough room for ellipsis, just truncate
                    let visible = Self::extract_visible_slice(self.text, 0, viewport_width);
                    buf.set_string(area.x, area.y, &visible, self.style.text_style);
                } else {
                    // Show truncated text with ellipsis
                    let text_space = viewport_width - ellipsis_width;
                    let visible = Self::extract_visible_slice(self.text, 0, text_space);
                    let display = format!("{}{}", visible.trim_end(), self.style.ellipsis);
                    // Pad to full width
                    let padded = format!("{:<width$}", display, width = viewport_width);
                    buf.set_string(area.x, area.y, &padded, self.style.text_style);
                }
            }
            MarqueeMode::Bounce => {
                // Simple offset-based slicing
                let visible =
                    Self::extract_visible_slice(self.text, self.state.offset, viewport_width);
                buf.set_string(area.x, area.y, &visible, self.style.text_style);
            }
            MarqueeMode::Continuous => {
                // Create virtual looped string and extract visible portion
                // Virtual string: "text + separator + text + separator + ..."
                // We need at least viewport_width characters from offset
                let separator = self.style.separator;
                let sep_width = separator.width();
                let cycle_width = text_width + sep_width;

                // Calculate where we are in the cycle
                let effective_offset = self.state.offset % cycle_width;

                // Build enough of the virtual string to fill the viewport
                let mut virtual_text = String::new();
                let mut built_width = 0;
                let mut pos = 0;

                // Skip to effective_offset
                let mut skip = effective_offset;

                // We'll iterate through cycles until we have enough
                while built_width < viewport_width {
                    // Determine if we're in text or separator portion
                    let cycle_pos = pos % cycle_width;
                    let in_text = cycle_pos < text_width;

                    if in_text {
                        // Extract from text
                        let text_offset = cycle_pos;
                        for ch in self.text.chars().skip_while(|_| {
                            let w = 0; // placeholder
                            w < text_offset
                        }) {
                            let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                            if skip > 0 {
                                if ch_width <= skip {
                                    skip -= ch_width;
                                    pos += ch_width;
                                    continue;
                                } else {
                                    skip = 0;
                                    pos += ch_width;
                                    virtual_text.push(' ');
                                    built_width += 1;
                                    continue;
                                }
                            }
                            if built_width + ch_width > viewport_width {
                                break;
                            }
                            virtual_text.push(ch);
                            built_width += ch_width;
                            pos += ch_width;
                        }
                        // Move to separator
                        pos = (pos / cycle_width) * cycle_width + text_width;
                    } else {
                        // Extract from separator
                        let sep_offset = cycle_pos - text_width;
                        for (i, ch) in separator.chars().enumerate() {
                            if i < sep_offset {
                                continue;
                            }
                            let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                            if skip > 0 {
                                if ch_width <= skip {
                                    skip -= ch_width;
                                    pos += ch_width;
                                    continue;
                                } else {
                                    skip = 0;
                                    pos += ch_width;
                                    virtual_text.push(' ');
                                    built_width += 1;
                                    continue;
                                }
                            }
                            if built_width + ch_width > viewport_width {
                                break;
                            }
                            virtual_text.push(ch);
                            built_width += ch_width;
                            pos += ch_width;
                        }
                        // Move to next text
                        pos = ((pos / cycle_width) + 1) * cycle_width;
                    }
                }

                // Pad if needed
                while built_width < viewport_width {
                    virtual_text.push(' ');
                    built_width += 1;
                }

                buf.set_string(area.x, area.y, &virtual_text, self.style.text_style);
            }
        }
    }
}

impl Widget for MarqueeText<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_internal(area, buf);
    }
}

/// Helper function to create a simple continuous marquee
pub fn continuous_marquee<'a>(text: &'a str, state: &'a mut MarqueeState) -> MarqueeText<'a> {
    MarqueeText::new(text, state).mode(MarqueeMode::Continuous)
}

/// Helper function to create a simple bounce marquee
pub fn bounce_marquee<'a>(text: &'a str, state: &'a mut MarqueeState) -> MarqueeText<'a> {
    MarqueeText::new(text, state).mode(MarqueeMode::Bounce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marquee_state_new() {
        let state = MarqueeState::new();
        assert_eq!(state.offset, 0);
        assert_eq!(state.direction, ScrollDir::Left);
        assert_eq!(state.paused_ticks, 0);
    }

    #[test]
    fn test_marquee_state_reset() {
        let mut state = MarqueeState::new();
        state.offset = 10;
        state.direction = ScrollDir::Right;
        state.paused_ticks = 5;

        state.reset();

        assert_eq!(state.offset, 0);
        assert_eq!(state.direction, ScrollDir::Left);
        assert_eq!(state.paused_ticks, 0);
    }

    #[test]
    fn test_marquee_state_tick_short_text() {
        let mut state = MarqueeState::new();
        let style = MarqueeStyle::default();

        // Text width (5) <= viewport width (10), should not scroll
        state.tick(5, 10, &style);
        assert_eq!(state.offset, 0);
    }

    #[test]
    fn test_marquee_state_tick_continuous() {
        let mut state = MarqueeState::new();
        let style = MarqueeStyle::default()
            .mode(MarqueeMode::Continuous)
            .scroll_speed(1);

        // Text width 20, viewport 10
        state.tick(20, 10, &style);
        assert_eq!(state.offset, 1);

        state.tick(20, 10, &style);
        assert_eq!(state.offset, 2);
    }

    #[test]
    fn test_marquee_state_tick_bounce() {
        let mut state = MarqueeState::new();
        let style = MarqueeStyle::default()
            .mode(MarqueeMode::Bounce)
            .scroll_speed(5)
            .pause_at_edge(0);

        // Text width 20, viewport 10, max_offset = 10
        // Should bounce at offset 10

        // First few ticks going left
        state.tick(20, 10, &style);
        assert_eq!(state.offset, 5);
        assert_eq!(state.direction, ScrollDir::Left);

        state.tick(20, 10, &style);
        assert_eq!(state.offset, 10);
        assert_eq!(state.direction, ScrollDir::Right); // Should have reversed
    }

    #[test]
    fn test_marquee_state_pause() {
        let mut state = MarqueeState::new();
        let style = MarqueeStyle::default()
            .mode(MarqueeMode::Bounce)
            .scroll_speed(10)
            .pause_at_edge(2);

        // Text width 15, viewport 10, max_offset = 5
        // First tick should reach the edge
        state.tick(15, 10, &style);
        assert_eq!(state.offset, 5);
        assert_eq!(state.paused_ticks, 2);
        assert_eq!(state.direction, ScrollDir::Right);

        // Next ticks should decrement pause
        state.tick(15, 10, &style);
        assert_eq!(state.offset, 5); // No movement
        assert_eq!(state.paused_ticks, 1);

        state.tick(15, 10, &style);
        assert_eq!(state.offset, 5); // No movement
        assert_eq!(state.paused_ticks, 0);

        // Now should move again
        state.tick(15, 10, &style);
        assert_eq!(state.offset, 0); // Moved back (saturating)
    }

    #[test]
    fn test_marquee_style_default() {
        let style = MarqueeStyle::default();
        assert_eq!(style.scroll_speed, 1);
        assert_eq!(style.pause_at_edge, 3);
        assert_eq!(style.mode, MarqueeMode::Continuous);
        assert_eq!(style.separator, "   ");
        assert_eq!(style.ellipsis, "...");
    }

    #[test]
    fn test_marquee_style_builder() {
        let style = MarqueeStyle::new()
            .scroll_speed(2)
            .pause_at_edge(5)
            .mode(MarqueeMode::Bounce)
            .separator(" | ")
            .ellipsis("…");

        assert_eq!(style.scroll_speed, 2);
        assert_eq!(style.pause_at_edge, 5);
        assert_eq!(style.mode, MarqueeMode::Bounce);
        assert_eq!(style.separator, " | ");
        assert_eq!(style.ellipsis, "…");
    }

    #[test]
    fn test_marquee_render_fits() {
        let mut state = MarqueeState::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
        let marquee = MarqueeText::new("Hello", &mut state);

        marquee.render(Rect::new(0, 0, 20, 1), &mut buf);

        // Text should be left-aligned and padded
        let content: String = buf
            .content
            .iter()
            .map(|c| c.symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(content.starts_with("Hello"));
    }

    #[test]
    fn test_marquee_render_scroll() {
        let mut state = MarqueeState::new();
        state.offset = 5;

        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));
        let style = MarqueeStyle::default().mode(MarqueeMode::Bounce);
        let marquee = MarqueeText::new("Hello World This Is Long", &mut state).style(style);

        marquee.render(Rect::new(0, 0, 10, 1), &mut buf);

        // Should show text from offset 5
        let content: String = buf
            .content
            .iter()
            .map(|c| c.symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(content.starts_with(" World T") || content.starts_with("World Th"));
    }

    #[test]
    fn test_marquee_render_static() {
        let mut state = MarqueeState::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));
        let style = MarqueeStyle::default().mode(MarqueeMode::Static);
        let marquee = MarqueeText::new("This is a very long text", &mut state).style(style);

        marquee.render(Rect::new(0, 0, 10, 1), &mut buf);

        // Should be truncated with ellipsis
        let content: String = buf
            .content
            .iter()
            .map(|c| c.symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(content.contains("..."));
    }

    #[test]
    fn test_marquee_render_unicode() {
        let mut state = MarqueeState::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));
        let marquee = MarqueeText::new("日本語テスト", &mut state);

        // Just verify it doesn't panic with wide characters
        marquee.render(Rect::new(0, 0, 10, 1), &mut buf);
    }

    #[test]
    fn test_extract_visible_slice() {
        // Basic ASCII
        let slice = MarqueeText::extract_visible_slice("Hello World", 0, 5);
        assert_eq!(slice, "Hello");

        // With offset
        let slice = MarqueeText::extract_visible_slice("Hello World", 6, 5);
        assert_eq!(slice, "World");

        // Padding when shorter
        let slice = MarqueeText::extract_visible_slice("Hi", 0, 5);
        assert_eq!(slice, "Hi   ");
    }

    #[test]
    fn test_helper_functions() {
        let mut state1 = MarqueeState::new();
        let m1 = continuous_marquee("test", &mut state1);
        assert_eq!(m1.style.mode, MarqueeMode::Continuous);

        let mut state2 = MarqueeState::new();
        let m2 = bounce_marquee("test", &mut state2);
        assert_eq!(m2.style.mode, MarqueeMode::Bounce);
    }
}
