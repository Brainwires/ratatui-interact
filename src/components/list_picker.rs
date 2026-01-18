//! List picker widget
//!
//! A scrollable list with selection cursor for picking from a set of items.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{ListPicker, ListPickerState, ListPickerStyle};
//! use ratatui::text::Line;
//! use ratatui::layout::Rect;
//!
//! // Create items
//! let items = vec!["Option 1", "Option 2", "Option 3"];
//!
//! // Create state
//! let mut state = ListPickerState::new(items.len());
//!
//! // Create picker
//! let picker = ListPicker::new(&items, &state)
//!     .title("Select an option")
//!     .render_item(|item, idx, selected| {
//!         let text = if selected {
//!             format!("▶ {}", item)
//!         } else {
//!             format!("  {}", item)
//!         };
//!         vec![Line::from(text)]
//!     });
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

/// State for the list picker widget
#[derive(Debug, Clone, Default)]
pub struct ListPickerState {
    /// Currently selected index
    pub selected_index: usize,
    /// Scroll offset
    pub scroll: u16,
    /// Total number of items
    pub total_items: usize,
}

impl ListPickerState {
    /// Create a new list picker state with the given number of items
    pub fn new(total_items: usize) -> Self {
        Self {
            selected_index: 0,
            scroll: 0,
            total_items,
        }
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_index + 1 < self.total_items {
            self.selected_index += 1;
        }
    }

    /// Select a specific index
    pub fn select(&mut self, index: usize) {
        if index < self.total_items {
            self.selected_index = index;
        }
    }

    /// Move selection to first item
    pub fn select_first(&mut self) {
        self.selected_index = 0;
    }

    /// Move selection to last item
    pub fn select_last(&mut self) {
        if self.total_items > 0 {
            self.selected_index = self.total_items - 1;
        }
    }

    /// Ensure selected item is visible in viewport
    pub fn ensure_visible(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }

        if self.selected_index < self.scroll as usize {
            self.scroll = self.selected_index as u16;
        } else if self.selected_index >= self.scroll as usize + viewport_height {
            self.scroll = (self.selected_index - viewport_height + 1) as u16;
        }
    }

    /// Update total items count
    pub fn set_total(&mut self, total: usize) {
        self.total_items = total;
        if self.selected_index >= total && total > 0 {
            self.selected_index = total - 1;
        }
    }
}

/// Style configuration for list picker
#[derive(Debug, Clone)]
pub struct ListPickerStyle {
    /// Style for selected items
    pub selected_style: Style,
    /// Style for normal items
    pub normal_style: Style,
    /// Style for the selection indicator
    pub indicator_style: Style,
    /// Border style
    pub border_style: Style,
    /// Selection indicator character(s)
    pub indicator: &'static str,
    /// Empty indicator (same width as indicator)
    pub indicator_empty: &'static str,
    /// Whether to show borders
    pub bordered: bool,
}

impl Default for ListPickerStyle {
    fn default() -> Self {
        Self {
            selected_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            normal_style: Style::default().fg(Color::White),
            indicator_style: Style::default().fg(Color::Yellow),
            border_style: Style::default().fg(Color::Cyan),
            indicator: "▶ ",
            indicator_empty: "  ",
            bordered: true,
        }
    }
}

impl ListPickerStyle {
    /// Style with arrow indicator
    pub fn arrow() -> Self {
        Self::default()
    }

    /// Style with bracket indicator
    pub fn bracket() -> Self {
        Self {
            indicator: "> ",
            indicator_empty: "  ",
            ..Default::default()
        }
    }

    /// Style with checkbox indicator
    pub fn checkbox() -> Self {
        Self {
            indicator: "[x] ",
            indicator_empty: "[ ] ",
            selected_style: Style::default().fg(Color::Green),
            ..Default::default()
        }
    }

    /// Set whether to show borders
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }
}

/// Default render function type
type DefaultRenderFn<T> = fn(&T, usize, bool) -> Vec<Line<'static>>;

/// List picker widget
pub struct ListPicker<'a, T, F = DefaultRenderFn<T>>
where
    F: Fn(&T, usize, bool) -> Vec<Line<'static>>,
{
    items: &'a [T],
    state: &'a ListPickerState,
    style: ListPickerStyle,
    title: Option<&'a str>,
    footer: Option<Vec<Line<'static>>>,
    render_fn: F,
}

impl<'a, T: std::fmt::Display> ListPicker<'a, T, DefaultRenderFn<T>> {
    /// Create a new list picker with default rendering
    pub fn new(items: &'a [T], state: &'a ListPickerState) -> Self {
        Self {
            items,
            state,
            style: ListPickerStyle::default(),
            title: None,
            footer: None,
            render_fn: |item, _idx, _selected| vec![Line::from(item.to_string())],
        }
    }
}

impl<'a, T, F> ListPicker<'a, T, F>
where
    F: Fn(&T, usize, bool) -> Vec<Line<'static>>,
{
    /// Set the render function for items
    ///
    /// The function receives: item reference, index, is_selected
    /// Returns a Vec of Lines (to support multi-line items)
    pub fn render_item<G>(self, render_fn: G) -> ListPicker<'a, T, G>
    where
        G: Fn(&T, usize, bool) -> Vec<Line<'static>>,
    {
        ListPicker {
            items: self.items,
            state: self.state,
            style: self.style,
            title: self.title,
            footer: self.footer,
            render_fn,
        }
    }

    /// Set the title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Set the footer lines
    pub fn footer(mut self, footer: Vec<Line<'static>>) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Set the style
    pub fn style(mut self, style: ListPickerStyle) -> Self {
        self.style = style;
        self
    }

    /// Build the lines for rendering
    fn build_lines(&self, _area: Rect, inner_height: u16) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Header
        if let Some(title) = self.title {
            lines.push(Line::from(vec![Span::styled(
                title.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from("")); // Empty line after title
        }

        // Calculate available height for items
        let header_lines = if self.title.is_some() { 2 } else { 0 };
        let footer_lines = self.footer.as_ref().map(|f| f.len()).unwrap_or(0);
        let available_height = inner_height as usize - header_lines - footer_lines;

        // Items
        if self.items.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "No items",
                Style::default().fg(Color::Gray),
            )]));
        } else {
            let scroll = self.state.scroll as usize;
            for (idx, item) in self.items.iter().enumerate().skip(scroll).take(available_height) {
                let is_selected = idx == self.state.selected_index;
                let indicator = if is_selected {
                    self.style.indicator
                } else {
                    self.style.indicator_empty
                };

                let item_style = if is_selected {
                    self.style.selected_style
                } else {
                    self.style.normal_style
                };

                let item_lines = (self.render_fn)(item, idx, is_selected);
                for (line_idx, line) in item_lines.into_iter().enumerate() {
                    let mut spans = Vec::new();

                    // Only show indicator on first line of item
                    if line_idx == 0 {
                        spans.push(Span::styled(indicator.to_string(), self.style.indicator_style));
                    } else {
                        // Indent continuation lines
                        spans.push(Span::raw(" ".repeat(self.style.indicator.len())));
                    }

                    // Add the line content with appropriate style
                    for span in line.spans {
                        spans.push(Span::styled(span.content.to_string(), item_style));
                    }

                    lines.push(Line::from(spans));
                }
            }
        }

        // Footer
        if let Some(footer) = &self.footer {
            for line in footer {
                lines.push(line.clone());
            }
        }

        lines
    }
}

impl<'a, T, F> Widget for ListPicker<'a, T, F>
where
    F: Fn(&T, usize, bool) -> Vec<Line<'static>>,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = if self.style.bordered {
            Some(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.style.border_style),
            )
        } else {
            None
        };

        let inner = if let Some(ref block) = block {
            block.inner(area)
        } else {
            area
        };

        if let Some(block) = block {
            block.render(area, buf);
        }

        let lines = self.build_lines(area, inner.height);
        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        paragraph.render(inner, buf);
    }
}

/// Helper function to create a simple footer with key hints
pub fn key_hints_footer(hints: &[(&str, &str)]) -> Vec<Line<'static>> {
    let mut spans = Vec::new();
    for (idx, (key, desc)) in hints.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::raw(" | "));
        }
        spans.push(Span::styled(
            key.to_string(),
            Style::default().fg(Color::Green),
        ));
        spans.push(Span::raw(format!(": {}", desc)));
    }
    vec![Line::from(""), Line::from(spans)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_navigation() {
        let mut state = ListPickerState::new(5);
        assert_eq!(state.selected_index, 0);

        state.select_next();
        assert_eq!(state.selected_index, 1);

        state.select_prev();
        assert_eq!(state.selected_index, 0);

        state.select_prev(); // Should not go below 0
        assert_eq!(state.selected_index, 0);

        state.select_last();
        assert_eq!(state.selected_index, 4);

        state.select_next(); // Should not go above total
        assert_eq!(state.selected_index, 4);
    }

    #[test]
    fn test_ensure_visible() {
        let mut state = ListPickerState::new(20);
        state.selected_index = 15;
        state.ensure_visible(10);
        assert!(state.scroll >= 6); // 15 - 10 + 1 = 6
    }

    #[test]
    fn test_list_picker_render() {
        let items = vec!["Item 1", "Item 2", "Item 3"];
        let state = ListPickerState::new(items.len());
        let picker = ListPicker::new(&items, &state).title("Test");

        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 10));
        picker.render(Rect::new(0, 0, 40, 10), &mut buf);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_key_hints_footer() {
        let footer = key_hints_footer(&[("↑↓", "Navigate"), ("Enter", "Select")]);
        assert_eq!(footer.len(), 2);
    }
}
