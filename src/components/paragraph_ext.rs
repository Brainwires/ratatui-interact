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
    text::{Line, StyledGrapheme},
    widgets::Widget,
};
use unicode_width::UnicodeWidthStr;

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
    /// Each wrapped line is a vector of styled graphemes together with their
    /// display width. Control characters and zero-width graphemes are
    /// dropped, and wrapping is done by display width (so wide characters
    /// such as CJK take two columns).
    fn wrap_lines(&self, width: u16) -> Vec<Vec<(StyledGrapheme<'_>, usize)>> {
        let width = width as usize;
        if width == 0 {
            return vec![];
        }

        let mut wrapped = Vec::new();

        for line in &self.lines {
            // Flatten spans to graphemes with styles. `styled_graphemes`
            // filters out control characters, so raw escape sequences can
            // never reach the terminal.
            let graphemes: Vec<(StyledGrapheme<'_>, usize)> = line
                .spans
                .iter()
                .flat_map(|span| span.styled_graphemes(Style::default()))
                .map(|g| {
                    let w = g.symbol.width();
                    (g, w)
                })
                .filter(|(_, w)| *w > 0)
                .collect();

            if graphemes.is_empty() {
                wrapped.push(vec![]);
                continue;
            }

            // Word wrap by display width
            let mut start = 0;
            while start < graphemes.len() {
                // Find how many graphemes fit in `width` columns
                let mut end = start;
                let mut used = 0;
                while end < graphemes.len() && used + graphemes[end].1 <= width {
                    used += graphemes[end].1;
                    end += 1;
                }

                if end == graphemes.len() {
                    wrapped.push(graphemes[start..].to_vec());
                    break;
                }

                // A single grapheme wider than the whole line: emit it alone
                // (it will be clipped) so we always make progress.
                if end == start {
                    end = start + 1;
                }

                let mut break_at = end;

                // Find last space for word break
                for i in (start..end).rev() {
                    if graphemes[i].0.symbol == " " {
                        break_at = i + 1;
                        break;
                    }
                }

                wrapped.push(graphemes[start..break_at].to_vec());
                start = break_at;

                // Skip leading spaces on continuation
                while start < graphemes.len() && graphemes[start].0.symbol == " " {
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

        // Never touch cells outside the buffer
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }

        // Clear area first
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].reset();
            }
        }

        let visible = wrapped.iter().skip(scroll).take(area.height as usize);

        for (row, line_graphemes) in visible.enumerate() {
            let y = area.y + row as u16;
            if y >= area.bottom() {
                break;
            }

            // Only write actual content graphemes (no trailing spaces)
            let mut x = area.x;
            for (grapheme, w) in line_graphemes {
                let remaining = area.right().saturating_sub(x) as usize;
                if *w > remaining {
                    break;
                }
                // `set_stringn` handles wide graphemes (resetting the
                // cells they cover) and filters control characters.
                let (next_x, _) = buf.set_stringn(x, y, grapheme.symbol, remaining, grapheme.style);
                x = next_x;
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
    fn test_area_larger_than_buffer_does_not_panic() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 2));
        ParagraphExt::new(vec![Line::from("hi")]).render(Rect::new(0, 0, 10, 5), &mut buf);
        assert_eq!(buf[(0, 0)].symbol(), "h");
    }

    #[test]
    fn test_wide_chars_advance_by_display_width() {
        let area = Rect::new(0, 0, 10, 1);
        let mut buf = Buffer::empty(area);
        ParagraphExt::new(vec![Line::from("你好ab")]).render(area, &mut buf);
        // 你 is 2 columns wide, so 好 must start at x=2
        assert_eq!(buf[(0, 0)].symbol(), "你");
        assert_eq!(buf[(2, 0)].symbol(), "好");
        assert_eq!(buf[(4, 0)].symbol(), "a");
        assert_eq!(buf[(5, 0)].symbol(), "b");
    }

    #[test]
    fn test_wide_chars_wrap_by_display_width() {
        let widget = ParagraphExt::new(vec![Line::from("你好世界")]);
        // 8 columns of content into width 4 -> 2 rows
        assert_eq!(widget.line_count(4), 2);
        // Width 3 fits only one wide char per row
        assert_eq!(widget.line_count(3), 4);
    }

    #[test]
    fn test_control_characters_are_not_written() {
        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        ParagraphExt::new(vec![Line::from("a\x1b]0;pwned\x07\x1b[2Jb")]).render(area, &mut buf);
        for x in 0..area.width {
            let sym = buf[(x, 0)].symbol();
            assert!(
                !sym.chars().any(char::is_control),
                "control character written at x={x}: {sym:?}"
            );
        }
        assert_eq!(buf[(0, 0)].symbol(), "a");
    }

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
        let lines = vec![Line::from("Line 1"), Line::from(""), Line::from("Line 3")];
        let widget = ParagraphExt::new(lines);
        let count = widget.line_count(20);
        assert_eq!(count, 3);
    }
}
