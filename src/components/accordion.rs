//! Accordion widget
//!
//! A flexible, reusable accordion widget with collapsible/expandable sections.
//! Supports both single-expand (true accordion) and multiple-expand modes.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{Accordion, AccordionState, AccordionStyle, AccordionMode};
//! use ratatui::layout::Rect;
//! use ratatui::buffer::Buffer;
//! use ratatui::text::Line;
//!
//! // Define your accordion items
//! #[derive(Debug)]
//! struct FaqItem {
//!     id: String,
//!     question: String,
//!     answer: String,
//! }
//!
//! let items = vec![
//!     FaqItem {
//!         id: "1".into(),
//!         question: "What is ratatui?".into(),
//!         answer: "A Rust library for building terminal UIs.".into(),
//!     },
//!     FaqItem {
//!         id: "2".into(),
//!         question: "How do I install it?".into(),
//!         answer: "Add ratatui to your Cargo.toml.".into(),
//!     },
//! ];
//!
//! // Create state (single mode = only one expanded at a time)
//! let mut state = AccordionState::new(items.len()).with_mode(AccordionMode::Single);
//!
//! // Create the accordion widget
//! let accordion = Accordion::new(&items, &state)
//!     .id_fn(|item, _| item.id.clone())
//!     .render_header(|item, _idx, is_focused| {
//!         Line::raw(item.question.clone())
//!     })
//!     .render_content(|item, _idx, area, buf| {
//!         // Render answer content here
//!     });
//! ```

use std::collections::HashSet;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

/// Expansion mode for the accordion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccordionMode {
    /// Only one section can be expanded at a time (traditional accordion)
    Single,
    /// Any number of sections can be expanded simultaneously
    #[default]
    Multiple,
}

/// State for the accordion widget
#[derive(Debug, Clone)]
pub struct AccordionState {
    /// Set of expanded item IDs
    pub expanded: HashSet<String>,
    /// Currently focused item index
    pub focused_index: usize,
    /// Total number of items
    pub total_items: usize,
    /// Expansion mode
    pub mode: AccordionMode,
    /// Scroll offset for when content exceeds viewport
    pub scroll: u16,
}

impl AccordionState {
    /// Create a new accordion state
    pub fn new(total_items: usize) -> Self {
        Self {
            expanded: HashSet::new(),
            focused_index: 0,
            total_items,
            mode: AccordionMode::Multiple,
            scroll: 0,
        }
    }

    /// Set the expansion mode
    pub fn with_mode(mut self, mode: AccordionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set initially expanded items
    pub fn with_expanded(mut self, ids: impl IntoIterator<Item = String>) -> Self {
        match self.mode {
            AccordionMode::Multiple => {
                self.expanded = ids.into_iter().collect();
            }
            AccordionMode::Single => {
                // In single mode, only keep the last item
                if let Some(id) = ids.into_iter().last() {
                    self.expanded.clear();
                    self.expanded.insert(id);
                }
            }
        }
        self
    }

    /// Toggle the expansion state of an item
    pub fn toggle(&mut self, id: &str) {
        if self.expanded.contains(id) {
            self.expanded.remove(id);
        } else {
            match self.mode {
                AccordionMode::Single => {
                    // Collapse all others first
                    self.expanded.clear();
                    self.expanded.insert(id.to_string());
                }
                AccordionMode::Multiple => {
                    self.expanded.insert(id.to_string());
                }
            }
        }
    }

    /// Expand an item
    pub fn expand(&mut self, id: &str) {
        match self.mode {
            AccordionMode::Single => {
                self.expanded.clear();
                self.expanded.insert(id.to_string());
            }
            AccordionMode::Multiple => {
                self.expanded.insert(id.to_string());
            }
        }
    }

    /// Collapse an item
    pub fn collapse(&mut self, id: &str) {
        self.expanded.remove(id);
    }

    /// Check if an item is expanded
    pub fn is_expanded(&self, id: &str) -> bool {
        self.expanded.contains(id)
    }

    /// Expand all items (only effective in Multiple mode)
    pub fn expand_all(&mut self, ids: impl Iterator<Item = String>) {
        match self.mode {
            AccordionMode::Single => {
                // In single mode, expand only the last
                if let Some(id) = ids.last() {
                    self.expanded.clear();
                    self.expanded.insert(id);
                }
            }
            AccordionMode::Multiple => {
                for id in ids {
                    self.expanded.insert(id);
                }
            }
        }
    }

    /// Collapse all items
    pub fn collapse_all(&mut self) {
        self.expanded.clear();
    }

    /// Move focus to the next item
    pub fn focus_next(&mut self) {
        if self.focused_index + 1 < self.total_items {
            self.focused_index += 1;
        }
    }

    /// Move focus to the previous item
    pub fn focus_prev(&mut self) {
        self.focused_index = self.focused_index.saturating_sub(1);
    }

    /// Set focus to a specific index
    pub fn focus(&mut self, index: usize) {
        if index < self.total_items {
            self.focused_index = index;
        }
    }

    /// Get the currently focused index
    pub fn focused_index(&self) -> usize {
        self.focused_index
    }

    /// Update the total items count
    pub fn set_total_items(&mut self, total: usize) {
        self.total_items = total;
        if self.focused_index >= total && total > 0 {
            self.focused_index = total - 1;
        }
    }

    /// Ensure focused item is visible in viewport
    pub fn ensure_visible(&mut self, viewport_height: u16, item_heights: &[u16]) {
        // Calculate the Y position of the focused item
        let mut y_pos: u16 = 0;
        let mut focused_start: u16 = 0;
        let mut focused_height: u16 = 1;

        for (idx, &height) in item_heights.iter().enumerate() {
            if idx == self.focused_index {
                focused_start = y_pos;
                focused_height = height;
                break;
            }
            y_pos += height;
        }

        // Scroll up if focused item is above viewport
        if focused_start < self.scroll {
            self.scroll = focused_start;
        }
        // Scroll down if focused item is below viewport
        else if focused_start + focused_height > self.scroll + viewport_height {
            self.scroll = (focused_start + focused_height).saturating_sub(viewport_height);
        }
    }
}

impl Default for AccordionState {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Style configuration for accordion
#[derive(Debug, Clone)]
pub struct AccordionStyle {
    /// Style for headers
    pub header_style: Style,
    /// Style for focused headers
    pub header_focused_style: Style,
    /// Style for content
    pub content_style: Style,
    /// Icon for expanded items
    pub expanded_icon: &'static str,
    /// Icon for collapsed items
    pub collapsed_icon: &'static str,
    /// Style for borders
    pub border_style: Style,
    /// Whether to show borders between items
    pub show_borders: bool,
    /// Indentation for content (in characters)
    pub content_indent: u16,
    /// Style for icons
    pub icon_style: Style,
}

impl Default for AccordionStyle {
    fn default() -> Self {
        Self {
            header_style: Style::default().fg(Color::White),
            header_focused_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            content_style: Style::default().fg(Color::Gray),
            expanded_icon: "▼ ",
            collapsed_icon: "▶ ",
            border_style: Style::default().fg(Color::DarkGray),
            show_borders: false,
            content_indent: 2,
            icon_style: Style::default().fg(Color::Cyan),
        }
    }
}

impl AccordionStyle {
    /// Create a minimal style without icons
    pub fn minimal() -> Self {
        Self {
            expanded_icon: "- ",
            collapsed_icon: "+ ",
            ..Default::default()
        }
    }

    /// Create a bordered style
    pub fn bordered() -> Self {
        Self {
            show_borders: true,
            ..Default::default()
        }
    }

    /// Set the header style
    pub fn header_style(mut self, style: Style) -> Self {
        self.header_style = style;
        self
    }

    /// Set the focused header style
    pub fn header_focused_style(mut self, style: Style) -> Self {
        self.header_focused_style = style;
        self
    }

    /// Set the content style
    pub fn content_style(mut self, style: Style) -> Self {
        self.content_style = style;
        self
    }

    /// Set the expanded icon
    pub fn expanded_icon(mut self, icon: &'static str) -> Self {
        self.expanded_icon = icon;
        self
    }

    /// Set the collapsed icon
    pub fn collapsed_icon(mut self, icon: &'static str) -> Self {
        self.collapsed_icon = icon;
        self
    }

    /// Set the icon style
    pub fn icon_style(mut self, style: Style) -> Self {
        self.icon_style = style;
        self
    }

    /// Set content indentation
    pub fn content_indent(mut self, indent: u16) -> Self {
        self.content_indent = indent;
        self
    }

    /// Set whether to show borders
    pub fn show_borders(mut self, show: bool) -> Self {
        self.show_borders = show;
        self
    }
}

/// Accordion widget with collapsible sections
pub struct Accordion<'a, T, H, C, I>
where
    H: Fn(&T, usize, bool) -> Line<'static>,
    C: Fn(&T, usize, Rect, &mut Buffer),
    I: Fn(&T, usize) -> String,
{
    items: &'a [T],
    state: &'a AccordionState,
    style: AccordionStyle,
    render_header: H,
    render_content: C,
    id_fn: I,
    content_heights: Option<&'a [u16]>,
}

impl<'a, T>
    Accordion<
        'a,
        T,
        fn(&T, usize, bool) -> Line<'static>,
        fn(&T, usize, Rect, &mut Buffer),
        fn(&T, usize) -> String,
    >
{
    /// Create a new accordion with default rendering
    #[allow(clippy::type_complexity)]
    pub fn new(
        items: &'a [T],
        state: &'a AccordionState,
    ) -> Accordion<
        'a,
        T,
        fn(&T, usize, bool) -> Line<'static>,
        fn(&T, usize, Rect, &mut Buffer),
        fn(&T, usize) -> String,
    >
    where
        T: std::fmt::Debug,
    {
        Accordion {
            items,
            state,
            style: AccordionStyle::default(),
            render_header: |_item, idx, _focused| Line::raw(format!("Item {}", idx)),
            render_content: |_item, _idx, _area, _buf| {},
            id_fn: |_item, idx| idx.to_string(),
            content_heights: None,
        }
    }
}

impl<'a, T, H, C, I> Accordion<'a, T, H, C, I>
where
    H: Fn(&T, usize, bool) -> Line<'static>,
    C: Fn(&T, usize, Rect, &mut Buffer),
    I: Fn(&T, usize) -> String,
{
    /// Set the ID extraction function
    pub fn id_fn<I2>(self, id_fn: I2) -> Accordion<'a, T, H, C, I2>
    where
        I2: Fn(&T, usize) -> String,
    {
        Accordion {
            items: self.items,
            state: self.state,
            style: self.style,
            render_header: self.render_header,
            render_content: self.render_content,
            id_fn,
            content_heights: self.content_heights,
        }
    }

    /// Set the header render function
    pub fn render_header<H2>(self, render_header: H2) -> Accordion<'a, T, H2, C, I>
    where
        H2: Fn(&T, usize, bool) -> Line<'static>,
    {
        Accordion {
            items: self.items,
            state: self.state,
            style: self.style,
            render_header,
            render_content: self.render_content,
            id_fn: self.id_fn,
            content_heights: self.content_heights,
        }
    }

    /// Set the content render function
    pub fn render_content<C2>(self, render_content: C2) -> Accordion<'a, T, H, C2, I>
    where
        C2: Fn(&T, usize, Rect, &mut Buffer),
    {
        Accordion {
            items: self.items,
            state: self.state,
            style: self.style,
            render_header: self.render_header,
            render_content,
            id_fn: self.id_fn,
            content_heights: self.content_heights,
        }
    }

    /// Set the style
    pub fn style(mut self, style: AccordionStyle) -> Self {
        self.style = style;
        self
    }

    /// Set content heights for proper scrolling
    pub fn content_heights(mut self, heights: &'a [u16]) -> Self {
        self.content_heights = Some(heights);
        self
    }

    /// Get the ID for an item at the given index
    fn get_id(&self, item: &T, idx: usize) -> String {
        (self.id_fn)(item, idx)
    }

    /// Calculate heights for all items (useful for scrolling calculations)
    pub fn calculate_item_heights(&self) -> Vec<u16> {
        self.items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let id = self.get_id(item, idx);
                let header_height = 1u16;
                let content_height = if self.state.is_expanded(&id) {
                    self.content_heights
                        .and_then(|h| h.get(idx).copied())
                        .unwrap_or(3) // Default content height
                } else {
                    0
                };
                let border_height = if self.style.show_borders { 1 } else { 0 };
                header_height + content_height + border_height
            })
            .collect()
    }
}

impl<'a, T, H, C, I> Widget for Accordion<'a, T, H, C, I>
where
    H: Fn(&T, usize, bool) -> Line<'static>,
    C: Fn(&T, usize, Rect, &mut Buffer),
    I: Fn(&T, usize) -> String,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let mut y = area.y;
        let scroll = self.state.scroll;
        let mut current_y: u16 = 0;

        for (idx, item) in self.items.iter().enumerate() {
            let id = self.get_id(item, idx);
            let is_expanded = self.state.is_expanded(&id);
            let is_focused = idx == self.state.focused_index;

            // Calculate item height
            let content_height = if is_expanded {
                self.content_heights
                    .and_then(|h| h.get(idx).copied())
                    .unwrap_or(3)
            } else {
                0
            };
            let header_height = 1u16;
            let item_height = header_height + content_height;

            // Skip items above scroll position
            if current_y + item_height <= scroll {
                current_y += item_height;
                continue;
            }

            // Stop if we've filled the area
            if y >= area.y + area.height {
                break;
            }

            // Calculate visible portion
            let skip_lines = scroll.saturating_sub(current_y);
            let available_height = (area.y + area.height).saturating_sub(y);

            // Render header (if visible)
            if skip_lines == 0 && available_height > 0 {
                let header_area = Rect::new(area.x, y, area.width, 1);

                // Build header line with icon
                let icon = if is_expanded {
                    self.style.expanded_icon
                } else {
                    self.style.collapsed_icon
                };

                let header_line = (self.render_header)(item, idx, is_focused);
                let style = if is_focused {
                    self.style.header_focused_style
                } else {
                    self.style.header_style
                };

                // Render icon
                let icon_span = Span::styled(icon.to_string(), self.style.icon_style);
                let mut spans = vec![icon_span];
                spans.extend(
                    header_line
                        .spans
                        .into_iter()
                        .map(|s| Span::styled(s.content, style)),
                );

                let line = Line::from(spans);
                let paragraph = Paragraph::new(line);
                paragraph.render(header_area, buf);

                y += 1;
            } else if skip_lines > 0 {
                // Header is above scroll, skip it
            }

            // Render content (if expanded and visible)
            if is_expanded && y < area.y + area.height {
                let content_start_in_item = header_height;
                let content_skip = skip_lines.saturating_sub(content_start_in_item);
                let content_available = (area.y + area.height)
                    .saturating_sub(y)
                    .min(content_height.saturating_sub(content_skip));

                if content_available > 0 {
                    let indent = self.style.content_indent;
                    let content_area = Rect::new(
                        area.x + indent,
                        y,
                        area.width.saturating_sub(indent),
                        content_available,
                    );
                    (self.render_content)(item, idx, content_area, buf);
                    y += content_available;
                }
            }

            // Render border (if enabled)
            if self.style.show_borders && y < area.y + area.height {
                let border_char = "─";
                for x in area.x..area.x + area.width {
                    buf.set_string(x, y, border_char, self.style.border_style);
                }
                y += 1;
            }

            current_y += item_height;
        }
    }
}

/// Calculate the total height needed for an accordion
pub fn calculate_height<T, I>(
    items: &[T],
    state: &AccordionState,
    id_fn: I,
    content_heights: &[u16],
    show_borders: bool,
) -> u16
where
    I: Fn(&T, usize) -> String,
{
    items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let id = id_fn(item, idx);
            let header_height = 1u16;
            let content_height = if state.is_expanded(&id) {
                content_heights.get(idx).copied().unwrap_or(3)
            } else {
                0
            };
            let border_height = if show_borders { 1 } else { 0 };
            header_height + content_height + border_height
        })
        .sum()
}

/// Handle keyboard input for accordion navigation
pub fn handle_accordion_key(
    state: &mut AccordionState,
    key: &crossterm::event::KeyEvent,
    get_id: impl Fn(usize) -> String,
) -> bool {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.focus_prev();
            true
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.focus_next();
            true
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            let id = get_id(state.focused_index);
            state.toggle(&id);
            true
        }
        KeyCode::Home => {
            state.focus(0);
            true
        }
        KeyCode::End => {
            if state.total_items > 0 {
                state.focus(state.total_items - 1);
            }
            true
        }
        _ => false,
    }
}

/// Handle mouse click for accordion
pub fn handle_accordion_mouse(
    state: &mut AccordionState,
    mouse: &crossterm::event::MouseEvent,
    item_areas: &[(usize, Rect, String)], // (index, header_area, id)
) -> bool {
    use crossterm::event::MouseEventKind;

    if let MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse.kind {
        for (idx, area, id) in item_areas {
            if mouse.column >= area.x
                && mouse.column < area.x + area.width
                && mouse.row >= area.y
                && mouse.row < area.y + area.height
            {
                state.focus(*idx);
                state.toggle(id);
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accordion_state_new() {
        let state = AccordionState::new(5);
        assert_eq!(state.total_items, 5);
        assert_eq!(state.focused_index, 0);
        assert!(state.expanded.is_empty());
        assert_eq!(state.mode, AccordionMode::Multiple);
    }

    #[test]
    fn test_accordion_state_toggle() {
        let mut state = AccordionState::new(3);

        state.toggle("item1");
        assert!(state.is_expanded("item1"));

        state.toggle("item1");
        assert!(!state.is_expanded("item1"));
    }

    #[test]
    fn test_accordion_state_single_mode() {
        let mut state = AccordionState::new(3).with_mode(AccordionMode::Single);

        state.expand("item1");
        assert!(state.is_expanded("item1"));

        state.expand("item2");
        assert!(!state.is_expanded("item1"));
        assert!(state.is_expanded("item2"));
    }

    #[test]
    fn test_accordion_state_multiple_mode() {
        let mut state = AccordionState::new(3).with_mode(AccordionMode::Multiple);

        state.expand("item1");
        state.expand("item2");

        assert!(state.is_expanded("item1"));
        assert!(state.is_expanded("item2"));
    }

    #[test]
    fn test_accordion_state_expand_collapse() {
        let mut state = AccordionState::new(3);

        state.expand("item1");
        assert!(state.is_expanded("item1"));

        state.collapse("item1");
        assert!(!state.is_expanded("item1"));
    }

    #[test]
    fn test_accordion_state_navigation() {
        let mut state = AccordionState::new(5);

        assert_eq!(state.focused_index(), 0);

        state.focus_next();
        assert_eq!(state.focused_index(), 1);

        state.focus_next();
        assert_eq!(state.focused_index(), 2);

        state.focus_prev();
        assert_eq!(state.focused_index(), 1);

        state.focus(4);
        assert_eq!(state.focused_index(), 4);

        // Should not go past last item
        state.focus_next();
        assert_eq!(state.focused_index(), 4);

        // Should not go below 0
        state.focus(0);
        state.focus_prev();
        assert_eq!(state.focused_index(), 0);
    }

    #[test]
    fn test_accordion_state_collapse_all() {
        let mut state = AccordionState::new(3).with_mode(AccordionMode::Multiple);

        state.expand("item1");
        state.expand("item2");
        state.expand("item3");

        assert_eq!(state.expanded.len(), 3);

        state.collapse_all();
        assert!(state.expanded.is_empty());
    }

    #[test]
    fn test_accordion_style_default() {
        let style = AccordionStyle::default();
        assert_eq!(style.expanded_icon, "▼ ");
        assert_eq!(style.collapsed_icon, "▶ ");
        assert!(!style.show_borders);
        assert_eq!(style.content_indent, 2);
    }

    #[test]
    fn test_accordion_style_minimal() {
        let style = AccordionStyle::minimal();
        assert_eq!(style.expanded_icon, "- ");
        assert_eq!(style.collapsed_icon, "+ ");
    }

    #[test]
    fn test_accordion_render_collapsed() {
        #[derive(Debug)]
        struct Item {
            id: String,
            title: String,
        }

        let items = vec![
            Item {
                id: "1".into(),
                title: "First".into(),
            },
            Item {
                id: "2".into(),
                title: "Second".into(),
            },
        ];
        let state = AccordionState::new(items.len());

        let accordion = Accordion::new(&items, &state)
            .id_fn(|item, _| item.id.clone())
            .render_header(|item, _, _| Line::raw(item.title.clone()))
            .render_content(|_, _, _, _| {});

        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);
        accordion.render(area, &mut buf);

        // Check that first line contains the collapsed icon and title
        let line0 = buf
            .content
            .iter()
            .take(20)
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(line0.contains("▶"));
        assert!(line0.contains("First"));
    }

    #[test]
    fn test_accordion_render_expanded() {
        #[derive(Debug)]
        struct Item {
            id: String,
            title: String,
        }

        let items = vec![
            Item {
                id: "1".into(),
                title: "First".into(),
            },
            Item {
                id: "2".into(),
                title: "Second".into(),
            },
        ];
        let mut state = AccordionState::new(items.len());
        state.expand("1");

        let accordion = Accordion::new(&items, &state)
            .id_fn(|item, _| item.id.clone())
            .render_header(|item, _, _| Line::raw(item.title.clone()))
            .render_content(|_, _, area, buf| {
                let text = Paragraph::new("Content here");
                text.render(area, buf);
            })
            .content_heights(&[2, 2]);

        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);
        accordion.render(area, &mut buf);

        // Check that first line contains the expanded icon
        let line0 = buf
            .content
            .iter()
            .take(20)
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(line0.contains("▼"));
        assert!(line0.contains("First"));

        // Content should be rendered below the header
        let line1 = buf
            .content
            .iter()
            .skip(20)
            .take(20)
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(line1.contains("Content"));
    }

    #[test]
    fn test_calculate_height() {
        #[derive(Debug)]
        struct Item {
            id: String,
        }

        let items = vec![
            Item { id: "1".into() },
            Item { id: "2".into() },
            Item { id: "3".into() },
        ];
        let mut state = AccordionState::new(items.len());
        let content_heights = vec![3u16, 5, 2];

        // All collapsed: 3 headers = 3
        let height = calculate_height(
            &items,
            &state,
            |item, _| item.id.clone(),
            &content_heights,
            false,
        );
        assert_eq!(height, 3);

        // One expanded: 3 headers + 3 content = 6
        state.expand("1");
        let height = calculate_height(
            &items,
            &state,
            |item, _| item.id.clone(),
            &content_heights,
            false,
        );
        assert_eq!(height, 6);

        // Two expanded: 3 headers + 3 + 5 = 11
        state.expand("2");
        let height = calculate_height(
            &items,
            &state,
            |item, _| item.id.clone(),
            &content_heights,
            false,
        );
        assert_eq!(height, 11);

        // With borders: 11 + 3 borders = 14
        let height = calculate_height(
            &items,
            &state,
            |item, _| item.id.clone(),
            &content_heights,
            true,
        );
        assert_eq!(height, 14);
    }
}
