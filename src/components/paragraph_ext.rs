//! Extended Paragraph Widget
//!
//! A styled text widget with word-wrapping and scrolling support.
//! Similar to ratatui's `Paragraph` but with cleaner rendering (no trailing spaces)
//! and more control over wrapping behavior.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::ParagraphExt;
//! use ratatui::text::Line;
//! use ratatui::layout::Rect;
//! use ratatui::buffer::Buffer;
//! use ratatui::widgets::Widget;
//!
//! let lines = vec![
//!     Line::from("Hello, world!"),
//!     Line::from("This is a long line that will be word-wrapped automatically."),
//! ];
//!
//! let widget = ParagraphExt::new(lines)
//!     .scroll(0)
//!     .width(40);
//!
//! let area = Rect::new(0, 0, 40, 10);
//! let mut buf = Buffer::empty(area);
//! widget.render(area, &mut buf);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::Widget,
};

/// Extended paragraph widget with word-wrapping and scrolling.
///
/// Unlike ratatui's `Paragraph`, this widget:
/// - Does not pad lines with trailing spaces
/// - Provides fine-grained control over word wrapping
/// - Preserves per-character styling through wrapping
/// - Supports vertical scrolling
pub struct ParagraphExt<'a> {
    lines: Vec<Line<'a>>,
    scroll: u16,
    width: Option<u16>,
}

impl<'a> ParagraphExt<'a> {
    /// Create a new ParagraphExt with the given lines.
    pub fn new(lines: Vec<Line<'a>>) -> Self {
        Self {
            lines,
            scroll: 0,
            width: None,
        }
    }

    /// Set the vertical scroll offset (number of wrapped lines to skip).
    pub fn scroll(mut self, scroll: u16) -> Self {
        self.scroll = scroll;
        self
    }

    /// Set the width for word wrapping.
    ///
    /// If not set, the render area width will be used.
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Word-wrap lines and return wrapped line content.
    ///
    /// Each wrapped line is a vector of (char, Style) tuples.
    fn wrap_lines(&self, width: u16) -> Vec<Vec<(char, Style)>> {
        let width = width as usize;
        if width == 0 {
            return vec![];
        }

        let mut wrapped = Vec::new();

        for line in &self.lines {
            // Flatten spans to chars with styles
            let mut chars: Vec<(char, Style)> = Vec::new();
            for span in &line.spans {
                for ch in span.content.chars() {
                    chars.push((ch, span.style));
                }
            }

            if chars.is_empty() {
                wrapped.push(vec![]);
                continue;
            }

            // Word wrap
            let mut start = 0;
            while start < chars.len() {
                let remaining = chars.len() - start;
                if remaining <= width {
                    wrapped.push(chars[start..].to_vec());
                    break;
                }

                let end = start + width;
                let mut break_at = end;

                // Find last space for word break
                for i in (start..end).rev() {
                    if chars[i].0 == ' ' {
                        break_at = i + 1;
                        break;
                    }
                }

                wrapped.push(chars[start..break_at].to_vec());
                start = break_at;

                // Skip leading spaces on continuation
                while start < chars.len() && chars[start].0 == ' ' {
                    start += 1;
                }
            }
        }

        wrapped
    }

    /// Calculate the total number of wrapped lines.
    ///
    /// This is useful for calculating scroll bounds.
    pub fn line_count(&self, width: u16) -> usize {
        self.wrap_lines(width).len()
    }
}

impl Widget for ParagraphExt<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = self.width.unwrap_or(area.width);
        let wrapped = self.wrap_lines(width);
        let scroll = self.scroll as usize;

        // Clear area first
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf[(x, y)].reset();
            }
        }

        let visible = wrapped.iter().skip(scroll).take(area.height as usize);

        for (row, line_chars) in visible.enumerate() {
            let y = area.y + row as u16;
            if y >= area.y + area.height {
                break;
            }

            // Only write actual content characters (no trailing spaces)
            for (col, (ch, style)) in line_chars.iter().enumerate() {
                let x = area.x + col as u16;
                if x >= area.x + area.width {
                    break;
                }
                buf[(x, y)].set_char(*ch).set_style(*style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;
    use ratatui::text::Span;

    #[test]
    fn test_empty_lines() {
        let widget = ParagraphExt::new(vec![]);
        let area = Rect::new(0, 0, 20, 5);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn test_simple_render() {
        let lines = vec![Line::from("Hello")];
        let widget = ParagraphExt::new(lines);
        let area = Rect::new(0, 0, 20, 5);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Check first 5 characters
        assert_eq!(buf[(0, 0)].symbol(), "H");
        assert_eq!(buf[(1, 0)].symbol(), "e");
        assert_eq!(buf[(2, 0)].symbol(), "l");
        assert_eq!(buf[(3, 0)].symbol(), "l");
        assert_eq!(buf[(4, 0)].symbol(), "o");
    }

    #[test]
    fn test_word_wrap() {
        let lines = vec![Line::from("Hello world this is a test")];
        let widget = ParagraphExt::new(lines).width(10);
        let area = Rect::new(0, 0, 10, 5);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // First line should be "Hello "
        assert_eq!(buf[(0, 0)].symbol(), "H");
        // Second line should start with "world "
        assert_eq!(buf[(0, 1)].symbol(), "w");
    }

    #[test]
    fn test_scroll() {
        let lines = vec![
            Line::from("Line 1"),
            Line::from("Line 2"),
            Line::from("Line 3"),
        ];
        let widget = ParagraphExt::new(lines).scroll(1);
        let area = Rect::new(0, 0, 20, 2);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // First visible line should be "Line 2"
        assert_eq!(buf[(0, 0)].symbol(), "L");
        assert_eq!(buf[(5, 0)].symbol(), "2");
    }

    #[test]
    fn test_styled_text() {
        let lines = vec![Line::from(vec![
            Span::styled("Red", Style::default().fg(Color::Red)),
            Span::raw(" "),
            Span::styled("Blue", Style::default().fg(Color::Blue)),
        ])];
        let widget = ParagraphExt::new(lines);
        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Check that styles are preserved
        assert_eq!(buf[(0, 0)].fg, Color::Red);
        assert_eq!(buf[(4, 0)].fg, Color::Blue);
    }

    #[test]
    fn test_line_count() {
        let lines = vec![Line::from("Hello world this is a long line")];
        let widget = ParagraphExt::new(lines);

        // With width 10, should wrap into multiple lines
        let count = widget.line_count(10);
        assert!(count > 1);
    }

    #[test]
    fn test_empty_line_preserved() {
        let lines = vec![
            Line::from("Line 1"),
            Line::from(""),
            Line::from("Line 3"),
        ];
        let widget = ParagraphExt::new(lines);
        let count = widget.line_count(20);
        assert_eq!(count, 3);
    }
}
