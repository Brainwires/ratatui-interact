//! Scrollable content component
//!
//! A scrollable text pane with focus support, keyboard navigation, and mouse scrolling.
//! Ideal for displaying log output, help text, or any scrollable content.
//!
//! # Example
//!
//! ```rust,ignore
//! use ratatui_interact::components::{
//!     ScrollableContent, ScrollableContentState, ScrollableContentStyle,
//!     handle_scrollable_content_key, handle_scrollable_content_mouse,
//! };
//! use ratatui::prelude::*;
//!
//! // Create state with content
//! let mut state = ScrollableContentState::new(vec![
//!     "Line 1".to_string(),
//!     "Line 2".to_string(),
//!     "Line 3".to_string(),
//! ]);
//! state.set_focused(true);
//!
//! // In render:
//! let content = ScrollableContent::new(&state)
//!     .title("My Content")
//!     .style(ScrollableContentStyle::default());
//! content.render(area, buf);
//!
//! // Handle events:
//! handle_scrollable_content_key(&mut state, &key_event, visible_height);
//! handle_scrollable_content_mouse(&mut state, &mouse_event, content_area);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};

/// Actions that can result from scrollable content interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollableContentAction {
    /// Content was scrolled up
    ScrollUp,
    /// Content was scrolled down
    ScrollDown,
    /// Scrolled to top
    ScrollToTop,
    /// Scrolled to bottom
    ScrollToBottom,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Toggle fullscreen
    ToggleFullscreen,
}

/// State for the ScrollableContent component
#[derive(Debug, Clone)]
pub struct ScrollableContentState {
    /// Content lines to display
    lines: Vec<String>,
    /// Current scroll position (line offset from top)
    scroll_offset: usize,
    /// Whether this pane is focused
    focused: bool,
    /// Whether this pane is in fullscreen mode
    fullscreen: bool,
    /// Title for the content pane
    title: Option<String>,
}

impl ScrollableContentState {
    /// Create a new state with the given content lines
    pub fn new(lines: Vec<String>) -> Self {
        Self {
            lines,
            scroll_offset: 0,
            focused: false,
            fullscreen: false,
            title: None,
        }
    }

    /// Create an empty state
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// Set the content lines
    pub fn set_lines(&mut self, lines: Vec<String>) {
        self.lines = lines;
        // Clamp scroll offset to valid range
        if !self.lines.is_empty() {
            self.scroll_offset = self.scroll_offset.min(self.lines.len() - 1);
        } else {
            self.scroll_offset = 0;
        }
    }

    /// Get the content lines
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Push a line to the content
    pub fn push_line(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }

    /// Clear all content
    pub fn clear(&mut self) {
        self.lines.clear();
        self.scroll_offset = 0;
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get the current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Set the scroll offset
    pub fn set_scroll_offset(&mut self, offset: usize) {
        if !self.lines.is_empty() {
            self.scroll_offset = offset.min(self.lines.len() - 1);
        } else {
            self.scroll_offset = 0;
        }
    }

    /// Check if focused
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Check if in fullscreen mode
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    /// Set fullscreen mode
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
    }

    /// Toggle fullscreen mode
    pub fn toggle_fullscreen(&mut self) -> bool {
        self.fullscreen = !self.fullscreen;
        self.fullscreen
    }

    /// Set the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Get the title
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Scroll up by the given number of lines
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// Scroll down by the given number of lines
    pub fn scroll_down(&mut self, lines: usize, visible_height: usize) {
        if self.lines.is_empty() {
            return;
        }
        let max_offset = self.lines.len().saturating_sub(visible_height);
        self.scroll_offset = (self.scroll_offset + lines).min(max_offset);
    }

    /// Scroll to the top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to the bottom
    pub fn scroll_to_bottom(&mut self, visible_height: usize) {
        if self.lines.is_empty() {
            return;
        }
        self.scroll_offset = self.lines.len().saturating_sub(visible_height);
    }

    /// Page up
    pub fn page_up(&mut self, visible_height: usize) {
        self.scroll_up(visible_height.saturating_sub(1));
    }

    /// Page down
    pub fn page_down(&mut self, visible_height: usize) {
        self.scroll_down(visible_height.saturating_sub(1), visible_height);
    }

    /// Get a slice of visible lines based on current scroll position and height
    pub fn visible_lines(&self, height: usize) -> &[String] {
        if self.lines.is_empty() {
            return &[];
        }
        let start = self.scroll_offset.min(self.lines.len() - 1);
        let end = (start + height).min(self.lines.len());
        &self.lines[start..end]
    }

    /// Check if scrolled to top
    pub fn is_at_top(&self) -> bool {
        self.scroll_offset == 0
    }

    /// Check if scrolled to bottom (given visible height)
    pub fn is_at_bottom(&self, visible_height: usize) -> bool {
        if self.lines.is_empty() {
            return true;
        }
        self.scroll_offset >= self.lines.len().saturating_sub(visible_height)
    }

    /// Get the content as a single string (for clipboard copy)
    pub fn content_as_string(&self) -> String {
        self.lines.join("\n")
    }
}

impl Default for ScrollableContentState {
    fn default() -> Self {
        Self::empty()
    }
}

/// Style configuration for ScrollableContent
#[derive(Debug, Clone)]
pub struct ScrollableContentStyle {
    /// Border style when not focused
    pub border_style: Style,
    /// Border style when focused
    pub focused_border_style: Style,
    /// Text style
    pub text_style: Style,
    /// Whether to show borders
    pub show_borders: bool,
    /// Whether to show scroll indicators
    pub show_scroll_indicators: bool,
}

impl Default for ScrollableContentStyle {
    fn default() -> Self {
        Self {
            border_style: Style::default().fg(Color::DarkGray),
            focused_border_style: Style::default().fg(Color::Cyan),
            text_style: Style::default().fg(Color::White),
            show_borders: true,
            show_scroll_indicators: true,
        }
    }
}

impl ScrollableContentStyle {
    /// Create a minimal style without borders
    pub fn borderless() -> Self {
        Self {
            show_borders: false,
            ..Default::default()
        }
    }

    /// Create a style with custom focus color
    pub fn with_focus_color(mut self, color: Color) -> Self {
        self.focused_border_style = Style::default().fg(color);
        self
    }

    /// Set the text style
    pub fn text_style(mut self, style: Style) -> Self {
        self.text_style = style;
        self
    }
}

/// Scrollable content widget
///
/// A scrollable text pane that displays content with optional borders
/// and scroll indicators. Highlights when focused.
pub struct ScrollableContent<'a> {
    state: &'a ScrollableContentState,
    style: ScrollableContentStyle,
    title: Option<&'a str>,
}

impl<'a> ScrollableContent<'a> {
    /// Create a new scrollable content widget
    pub fn new(state: &'a ScrollableContentState) -> Self {
        Self {
            state,
            style: ScrollableContentStyle::default(),
            title: state.title.as_deref(),
        }
    }

    /// Set the style
    pub fn style(mut self, style: ScrollableContentStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the title (overrides state title)
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Calculate the inner area (content area without borders)
    pub fn inner_area(&self, area: Rect) -> Rect {
        if self.style.show_borders {
            Rect {
                x: area.x + 1,
                y: area.y + 1,
                width: area.width.saturating_sub(2),
                height: area.height.saturating_sub(2),
            }
        } else {
            area
        }
    }
}

impl Widget for ScrollableContent<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let border_style = if self.state.focused {
            self.style.focused_border_style
        } else {
            self.style.border_style
        };

        // Create block with optional title
        let mut block = Block::default().border_style(border_style);
        if self.style.show_borders {
            block = block.borders(Borders::ALL);
        }
        if let Some(title) = self.title {
            let title_style = if self.state.focused {
                border_style.add_modifier(Modifier::BOLD)
            } else {
                border_style
            };
            block = block.title(format!(" {} ", title)).title_style(title_style);
        }

        let inner = block.inner(area);
        block.render(area, buf);

        // Render content
        let visible_height = inner.height as usize;
        let visible_lines = self.state.visible_lines(visible_height);

        let lines: Vec<Line> = visible_lines
            .iter()
            .map(|s| Line::from(s.as_str()).style(self.style.text_style))
            .collect();

        let paragraph = Paragraph::new(lines);
        paragraph.render(inner, buf);

        // Render scroll indicators if enabled
        if self.style.show_scroll_indicators && self.style.show_borders {
            let has_content_above = !self.state.is_at_top();
            let has_content_below = !self.state.is_at_bottom(visible_height);

            if has_content_above && area.height > 2 {
                buf.set_string(
                    area.x + area.width - 2,
                    area.y,
                    "▲",
                    Style::default().fg(Color::DarkGray),
                );
            }
            if has_content_below && area.height > 2 {
                buf.set_string(
                    area.x + area.width - 2,
                    area.y + area.height - 1,
                    "▼",
                    Style::default().fg(Color::DarkGray),
                );
            }
        }
    }
}

/// Handle keyboard input for scrollable content
///
/// Supports:
/// - Up/Down or j/k: Scroll by one line
/// - PageUp/PageDown: Scroll by page
/// - Home/End: Scroll to top/bottom
/// - F10/Enter: Toggle fullscreen
///
/// Returns the action taken, if any.
pub fn handle_scrollable_content_key(
    state: &mut ScrollableContentState,
    key: &crossterm::event::KeyEvent,
    visible_height: usize,
) -> Option<ScrollableContentAction> {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.scroll_up(1);
            Some(ScrollableContentAction::ScrollUp)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.scroll_down(1, visible_height);
            Some(ScrollableContentAction::ScrollDown)
        }
        KeyCode::PageUp => {
            state.page_up(visible_height);
            Some(ScrollableContentAction::PageUp)
        }
        KeyCode::PageDown => {
            state.page_down(visible_height);
            Some(ScrollableContentAction::PageDown)
        }
        KeyCode::Home => {
            state.scroll_to_top();
            Some(ScrollableContentAction::ScrollToTop)
        }
        KeyCode::End => {
            state.scroll_to_bottom(visible_height);
            Some(ScrollableContentAction::ScrollToBottom)
        }
        KeyCode::F(10) | KeyCode::Enter => {
            state.toggle_fullscreen();
            Some(ScrollableContentAction::ToggleFullscreen)
        }
        _ => None,
    }
}

/// Handle mouse input for scrollable content
///
/// Supports scroll wheel for scrolling.
///
/// Returns the action taken, if any.
pub fn handle_scrollable_content_mouse(
    state: &mut ScrollableContentState,
    mouse: &crossterm::event::MouseEvent,
    content_area: Rect,
    visible_height: usize,
) -> Option<ScrollableContentAction> {
    use crossterm::event::MouseEventKind;

    // Check if mouse is within content area
    if mouse.column < content_area.x
        || mouse.column >= content_area.x + content_area.width
        || mouse.row < content_area.y
        || mouse.row >= content_area.y + content_area.height
    {
        return None;
    }

    match mouse.kind {
        MouseEventKind::ScrollUp => {
            state.scroll_up(3);
            Some(ScrollableContentAction::ScrollUp)
        }
        MouseEventKind::ScrollDown => {
            state.scroll_down(3, visible_height);
            Some(ScrollableContentAction::ScrollDown)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lines() -> Vec<String> {
        (1..=100).map(|i| format!("Line {}", i)).collect()
    }

    #[test]
    fn test_state_new() {
        let lines = vec!["a".to_string(), "b".to_string()];
        let state = ScrollableContentState::new(lines.clone());
        assert_eq!(state.lines(), &lines);
        assert_eq!(state.scroll_offset(), 0);
        assert!(!state.is_focused());
        assert!(!state.is_fullscreen());
    }

    #[test]
    fn test_state_empty() {
        let state = ScrollableContentState::empty();
        assert!(state.lines().is_empty());
        assert_eq!(state.line_count(), 0);
    }

    #[test]
    fn test_scroll_up() {
        let mut state = ScrollableContentState::new(sample_lines());
        state.set_scroll_offset(50);
        assert_eq!(state.scroll_offset(), 50);

        state.scroll_up(10);
        assert_eq!(state.scroll_offset(), 40);

        state.scroll_up(100); // Should clamp to 0
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_scroll_down() {
        let mut state = ScrollableContentState::new(sample_lines());
        let visible_height = 20;

        state.scroll_down(10, visible_height);
        assert_eq!(state.scroll_offset(), 10);

        state.scroll_down(1000, visible_height); // Should clamp to max
        assert_eq!(state.scroll_offset(), 100 - visible_height);
    }

    #[test]
    fn test_scroll_to_top_bottom() {
        let mut state = ScrollableContentState::new(sample_lines());
        let visible_height = 20;

        state.scroll_to_bottom(visible_height);
        assert_eq!(state.scroll_offset(), 80);
        assert!(state.is_at_bottom(visible_height));

        state.scroll_to_top();
        assert_eq!(state.scroll_offset(), 0);
        assert!(state.is_at_top());
    }

    #[test]
    fn test_page_up_down() {
        let mut state = ScrollableContentState::new(sample_lines());
        let visible_height = 20;

        state.page_down(visible_height);
        assert_eq!(state.scroll_offset(), 19); // visible_height - 1

        state.page_up(visible_height);
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_visible_lines() {
        let state = ScrollableContentState::new(sample_lines());
        let visible = state.visible_lines(5);
        assert_eq!(visible.len(), 5);
        assert_eq!(visible[0], "Line 1");
        assert_eq!(visible[4], "Line 5");
    }

    #[test]
    fn test_focus_and_fullscreen() {
        let mut state = ScrollableContentState::empty();

        assert!(!state.is_focused());
        state.set_focused(true);
        assert!(state.is_focused());

        assert!(!state.is_fullscreen());
        assert!(state.toggle_fullscreen());
        assert!(state.is_fullscreen());
        assert!(!state.toggle_fullscreen());
        assert!(!state.is_fullscreen());
    }

    #[test]
    fn test_content_as_string() {
        let lines = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let state = ScrollableContentState::new(lines);
        assert_eq!(state.content_as_string(), "a\nb\nc");
    }

    #[test]
    fn test_set_lines_clamps_scroll() {
        let mut state = ScrollableContentState::new(sample_lines());
        state.set_scroll_offset(50);

        // Set shorter content
        state.set_lines(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(state.scroll_offset(), 1); // Clamped to max valid offset
    }

    #[test]
    fn test_style_default() {
        let style = ScrollableContentStyle::default();
        assert!(style.show_borders);
        assert!(style.show_scroll_indicators);
    }

    #[test]
    fn test_style_borderless() {
        let style = ScrollableContentStyle::borderless();
        assert!(!style.show_borders);
    }

    #[test]
    fn test_handle_key_scroll() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut state = ScrollableContentState::new(sample_lines());
        let visible_height = 20;

        // Down arrow
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let action = handle_scrollable_content_key(&mut state, &key, visible_height);
        assert_eq!(action, Some(ScrollableContentAction::ScrollDown));
        assert_eq!(state.scroll_offset(), 1);

        // j key
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        handle_scrollable_content_key(&mut state, &key, visible_height);
        assert_eq!(state.scroll_offset(), 2);

        // Up arrow
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let action = handle_scrollable_content_key(&mut state, &key, visible_height);
        assert_eq!(action, Some(ScrollableContentAction::ScrollUp));
        assert_eq!(state.scroll_offset(), 1);

        // k key
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        handle_scrollable_content_key(&mut state, &key, visible_height);
        assert_eq!(state.scroll_offset(), 0);

        // Home
        state.set_scroll_offset(50);
        let key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        let action = handle_scrollable_content_key(&mut state, &key, visible_height);
        assert_eq!(action, Some(ScrollableContentAction::ScrollToTop));
        assert_eq!(state.scroll_offset(), 0);

        // End
        let key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
        let action = handle_scrollable_content_key(&mut state, &key, visible_height);
        assert_eq!(action, Some(ScrollableContentAction::ScrollToBottom));
        assert_eq!(state.scroll_offset(), 80);
    }

    #[test]
    fn test_handle_key_fullscreen() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut state = ScrollableContentState::new(sample_lines());
        let visible_height = 20;

        // F10
        let key = KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE);
        let action = handle_scrollable_content_key(&mut state, &key, visible_height);
        assert_eq!(action, Some(ScrollableContentAction::ToggleFullscreen));
        assert!(state.is_fullscreen());

        // Enter
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        handle_scrollable_content_key(&mut state, &key, visible_height);
        assert!(!state.is_fullscreen());
    }

    #[test]
    fn test_widget_render() {
        let state = ScrollableContentState::new(vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ]);
        let widget = ScrollableContent::new(&state).title("Test");
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 10));

        widget.render(Rect::new(0, 0, 20, 10), &mut buf);

        // Check that content was rendered
        let content: String = buf.content.iter().map(|c| c.symbol()).collect();
        assert!(content.contains("Line 1"));
    }

    #[test]
    fn test_inner_area() {
        let state = ScrollableContentState::empty();
        let content = ScrollableContent::new(&state);
        let area = Rect::new(0, 0, 20, 10);

        let inner = content.inner_area(area);
        assert_eq!(inner.x, 1);
        assert_eq!(inner.y, 1);
        assert_eq!(inner.width, 18);
        assert_eq!(inner.height, 8);
    }

    #[test]
    fn test_title() {
        let mut state = ScrollableContentState::empty();
        state.set_title("My Title");
        assert_eq!(state.title(), Some("My Title"));

        let widget = ScrollableContent::new(&state);
        assert_eq!(widget.title, Some("My Title"));

        // Override with widget title
        let widget = ScrollableContent::new(&state).title("Override");
        assert_eq!(widget.title, Some("Override"));
    }
}
