//! Log viewer widget
//!
//! A scrollable text viewer with line numbers, search highlighting, and log-level coloring.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{LogViewer, LogViewerState, LogViewerStyle};
//! use ratatui::layout::Rect;
//!
//! // Create content
//! let content: Vec<String> = vec![
//!     "[INFO] Application started".into(),
//!     "[DEBUG] Loading config...".into(),
//!     "[WARN] Config file not found, using defaults".into(),
//!     "[ERROR] Connection failed".into(),
//! ];
//!
//! // Create state
//! let mut state = LogViewerState::new(content);
//!
//! // Create viewer
//! let viewer = LogViewer::new(&state)
//!     .title("Application Log")
//!     .show_line_numbers(true);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Widget,
    },
};

/// State for the log viewer widget
#[derive(Debug, Clone)]
pub struct LogViewerState {
    /// Content lines
    pub content: Vec<String>,
    /// Vertical scroll position
    pub scroll_y: usize,
    /// Horizontal scroll position
    pub scroll_x: usize,
    /// Visible viewport height (set during render)
    pub visible_height: usize,
    /// Visible viewport width (set during render)
    pub visible_width: usize,
    /// Search state
    pub search: SearchState,
}

/// Search state for log viewer
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    /// Whether search is active
    pub active: bool,
    /// Current search query
    pub query: String,
    /// Line indices that match the query
    pub matches: Vec<usize>,
    /// Current match index
    pub current_match: usize,
}

impl LogViewerState {
    /// Create a new log viewer state with content
    pub fn new(content: Vec<String>) -> Self {
        Self {
            content,
            scroll_y: 0,
            scroll_x: 0,
            visible_height: 0,
            visible_width: 0,
            search: SearchState::default(),
        }
    }

    /// Create an empty log viewer state
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// Set content
    pub fn set_content(&mut self, content: Vec<String>) {
        self.content = content;
        self.scroll_y = 0;
        self.scroll_x = 0;
        self.search.matches.clear();
    }

    /// Append a line to content
    pub fn append(&mut self, line: String) {
        self.content.push(line);
    }

    /// Scroll up by one line
    pub fn scroll_up(&mut self) {
        self.scroll_y = self.scroll_y.saturating_sub(1);
    }

    /// Scroll down by one line
    pub fn scroll_down(&mut self) {
        if self.scroll_y + 1 < self.content.len() {
            self.scroll_y += 1;
        }
    }

    /// Scroll up by one page
    pub fn page_up(&mut self) {
        self.scroll_y = self.scroll_y.saturating_sub(self.visible_height);
    }

    /// Scroll down by one page
    pub fn page_down(&mut self) {
        let max_scroll = self.content.len().saturating_sub(self.visible_height);
        self.scroll_y = (self.scroll_y + self.visible_height).min(max_scroll);
    }

    /// Scroll left
    pub fn scroll_left(&mut self) {
        self.scroll_x = self.scroll_x.saturating_sub(4);
    }

    /// Scroll right
    pub fn scroll_right(&mut self) {
        self.scroll_x += 4;
    }

    /// Go to top
    pub fn go_to_top(&mut self) {
        self.scroll_y = 0;
    }

    /// Go to bottom
    pub fn go_to_bottom(&mut self) {
        self.scroll_y = self.content.len().saturating_sub(self.visible_height);
    }

    /// Go to a specific line (0-indexed)
    pub fn go_to_line(&mut self, line: usize) {
        self.scroll_y = line.min(self.content.len().saturating_sub(1));
    }

    /// Start search mode
    pub fn start_search(&mut self) {
        self.search.active = true;
        self.search.query.clear();
        self.search.matches.clear();
        self.search.current_match = 0;
    }

    /// Cancel search mode
    pub fn cancel_search(&mut self) {
        self.search.active = false;
    }

    /// Update search with new query
    pub fn update_search(&mut self) {
        self.search.matches.clear();
        self.search.current_match = 0;

        if self.search.query.is_empty() {
            return;
        }

        let query = self.search.query.to_lowercase();
        for (idx, line) in self.content.iter().enumerate() {
            if line.to_lowercase().contains(&query) {
                self.search.matches.push(idx);
            }
        }

        // Jump to first match if any
        if !self.search.matches.is_empty() {
            self.scroll_y = self.search.matches[0];
        }
    }

    /// Go to next search match
    pub fn next_match(&mut self) {
        if self.search.matches.is_empty() {
            return;
        }
        self.search.current_match = (self.search.current_match + 1) % self.search.matches.len();
        self.scroll_y = self.search.matches[self.search.current_match];
    }

    /// Go to previous search match
    pub fn prev_match(&mut self) {
        if self.search.matches.is_empty() {
            return;
        }
        if self.search.current_match == 0 {
            self.search.current_match = self.search.matches.len() - 1;
        } else {
            self.search.current_match -= 1;
        }
        self.scroll_y = self.search.matches[self.search.current_match];
    }
}

/// Style configuration for log viewer
#[derive(Debug, Clone)]
pub struct LogViewerStyle {
    /// Border style
    pub border_style: Style,
    /// Line number style
    pub line_number_style: Style,
    /// Default content style
    pub content_style: Style,
    /// Current search match highlight
    pub current_match_style: Style,
    /// Other search match highlight
    pub match_style: Style,
    /// Log level colors
    pub level_colors: LogLevelColors,
    /// Whether to show line numbers
    pub show_line_numbers: bool,
    /// Line number width
    pub line_number_width: usize,
}

/// Colors for different log levels
#[derive(Debug, Clone)]
pub struct LogLevelColors {
    pub error: Color,
    pub warn: Color,
    pub info: Color,
    pub debug: Color,
    pub trace: Color,
    pub success: Color,
}

impl Default for LogLevelColors {
    fn default() -> Self {
        Self {
            error: Color::Red,
            warn: Color::Yellow,
            info: Color::White,
            debug: Color::DarkGray,
            trace: Color::DarkGray,
            success: Color::Green,
        }
    }
}

impl Default for LogViewerStyle {
    fn default() -> Self {
        Self {
            border_style: Style::default().fg(Color::Cyan),
            line_number_style: Style::default().fg(Color::DarkGray),
            content_style: Style::default().fg(Color::White),
            current_match_style: Style::default().bg(Color::Yellow).fg(Color::Black),
            match_style: Style::default()
                .bg(Color::Rgb(60, 60, 30))
                .fg(Color::Yellow),
            level_colors: LogLevelColors::default(),
            show_line_numbers: true,
            line_number_width: 6,
        }
    }
}

impl LogViewerStyle {
    /// Get style for a line based on its content
    pub fn style_for_line(&self, line: &str) -> Style {
        // Check for log level indicators
        let lower = line.to_lowercase();

        if lower.contains("[error]") || lower.contains("error:") || lower.contains("failed") {
            Style::default().fg(self.level_colors.error)
        } else if lower.contains("[warn]") || lower.contains("warning:") {
            Style::default().fg(self.level_colors.warn)
        } else if lower.contains("[debug]") {
            Style::default().fg(self.level_colors.debug)
        } else if lower.contains("[trace]") {
            Style::default().fg(self.level_colors.trace)
        } else if lower.contains("✓")
            || lower.contains("success")
            || lower.contains("completed")
            || lower.contains("[ok]")
        {
            Style::default().fg(self.level_colors.success)
        } else if lower.contains("✗") {
            Style::default().fg(self.level_colors.error)
        } else if lower.contains("▶") || lower.contains("starting") {
            Style::default().fg(Color::Blue)
        } else {
            self.content_style
        }
    }
}

/// Log viewer widget
pub struct LogViewer<'a> {
    state: &'a LogViewerState,
    style: LogViewerStyle,
    title: Option<&'a str>,
}

impl<'a> LogViewer<'a> {
    /// Create a new log viewer
    pub fn new(state: &'a LogViewerState) -> Self {
        Self {
            state,
            style: LogViewerStyle::default(),
            title: None,
        }
    }

    /// Set the title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Set the style
    pub fn style(mut self, style: LogViewerStyle) -> Self {
        self.style = style;
        self
    }

    /// Enable or disable line numbers
    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.style.show_line_numbers = show;
        self
    }

    /// Build content lines
    fn build_lines(&self, inner: Rect) -> Vec<Line<'static>> {
        let visible_height = inner.height as usize;
        let visible_width = if self.style.show_line_numbers {
            inner
                .width
                .saturating_sub(self.style.line_number_width as u16 + 1) as usize
        } else {
            inner.width as usize
        };

        let start_line = self.state.scroll_y;
        let end_line = (start_line + visible_height).min(self.state.content.len());

        let mut lines = Vec::new();

        for line_idx in start_line..end_line {
            let line = &self.state.content[line_idx];

            // Check if this line is a search match
            let is_match = self.state.search.matches.contains(&line_idx);
            let is_current_match = self
                .state
                .search
                .matches
                .get(self.state.search.current_match)
                == Some(&line_idx);

            // Apply horizontal scroll
            let chars: Vec<char> = line.chars().collect();
            let display_line: String = chars
                .iter()
                .skip(self.state.scroll_x)
                .take(visible_width)
                .collect();

            // Determine content style
            let content_style = if is_current_match {
                self.style.current_match_style
            } else if is_match {
                self.style.match_style
            } else {
                self.style.style_for_line(line)
            };

            let mut spans = Vec::new();

            // Line number
            if self.style.show_line_numbers {
                let line_num = format!(
                    "{:>width$} ",
                    line_idx + 1,
                    width = self.style.line_number_width
                );
                spans.push(Span::styled(line_num, self.style.line_number_style));
            }

            // Content
            spans.push(Span::styled(display_line, content_style));

            lines.push(Line::from(spans));
        }

        lines
    }
}

impl Widget for LogViewer<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Layout: content + status bar + optional search bar
        let constraints = if self.state.search.active {
            vec![
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ]
        } else {
            vec![Constraint::Min(1), Constraint::Length(1)]
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // Content area
        let title = self.title.map(|t| format!(" {} ", t)).unwrap_or_default();
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(self.style.border_style);

        let inner = block.inner(chunks[0]);
        block.render(chunks[0], buf);

        // Content
        let lines = self.build_lines(inner);
        let para = Paragraph::new(lines);
        para.render(inner, buf);

        // Scrollbar
        if self.state.content.len() > inner.height as usize {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state =
                ScrollbarState::new(self.state.content.len()).position(self.state.scroll_y);
            scrollbar.render(inner, buf, &mut scrollbar_state);
        }

        // Status bar
        render_status_bar(self.state, chunks[1], buf);

        // Search bar
        if self.state.search.active && chunks.len() > 2 {
            render_search_bar(self.state, chunks[2], buf);
        }
    }
}

fn render_status_bar(state: &LogViewerState, area: Rect, buf: &mut Buffer) {
    let total_lines = state.content.len();
    let current_line = state.scroll_y + 1;
    let percent = if total_lines > 0 {
        (current_line as f64 / total_lines as f64 * 100.0) as u16
    } else {
        0
    };

    let h_scroll_info = if state.scroll_x > 0 {
        format!(" | Col: {}", state.scroll_x + 1)
    } else {
        String::new()
    };

    let search_info = if !state.search.matches.is_empty() {
        format!(
            " | Match {}/{}",
            state.search.current_match + 1,
            state.search.matches.len()
        )
    } else if !state.search.query.is_empty() && state.search.matches.is_empty() {
        " | No matches".to_string()
    } else {
        String::new()
    };

    let status = Line::from(vec![
        Span::styled(" ↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(": scroll | "),
        Span::styled("/", Style::default().fg(Color::Yellow)),
        Span::raw(": search | "),
        Span::styled("n/N", Style::default().fg(Color::Yellow)),
        Span::raw(": next/prev | "),
        Span::styled("g/G", Style::default().fg(Color::Yellow)),
        Span::raw(": top/bottom | "),
        Span::raw(format!(
            "Line {}/{} ({}%){}{}",
            current_line, total_lines, percent, h_scroll_info, search_info
        )),
    ]);

    let para = Paragraph::new(status).style(Style::default().bg(Color::DarkGray));
    para.render(area, buf);
}

fn render_search_bar(state: &LogViewerState, area: Rect, buf: &mut Buffer) {
    let search_line = Line::from(vec![
        Span::styled(" Search: ", Style::default().fg(Color::Yellow)),
        Span::raw(state.search.query.clone()),
        Span::styled("▌", Style::default().fg(Color::White)),
    ]);

    let para = Paragraph::new(search_line).style(Style::default().bg(Color::Rgb(40, 40, 60)));
    para.render(area, buf);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_viewer_state_new() {
        let content = vec!["Line 1".into(), "Line 2".into()];
        let state = LogViewerState::new(content);
        assert_eq!(state.content.len(), 2);
        assert_eq!(state.scroll_y, 0);
        assert_eq!(state.scroll_x, 0);
    }

    #[test]
    fn test_log_viewer_state_empty() {
        let state = LogViewerState::empty();
        assert!(state.content.is_empty());
    }

    #[test]
    fn test_log_viewer_state() {
        let content = vec!["Line 1".into(), "Line 2".into(), "Line 3".into()];
        let mut state = LogViewerState::new(content);

        assert_eq!(state.scroll_y, 0);
        state.scroll_down();
        assert_eq!(state.scroll_y, 1);
        state.scroll_up();
        assert_eq!(state.scroll_y, 0);
    }

    #[test]
    fn test_horizontal_scroll() {
        let content = vec!["Long line of text".into()];
        let mut state = LogViewerState::new(content);

        state.scroll_right();
        assert_eq!(state.scroll_x, 4);
        state.scroll_right();
        assert_eq!(state.scroll_x, 8);
        state.scroll_left();
        assert_eq!(state.scroll_x, 4);
        state.scroll_left();
        assert_eq!(state.scroll_x, 0);
        state.scroll_left(); // Should not go negative
        assert_eq!(state.scroll_x, 0);
    }

    #[test]
    fn test_page_navigation() {
        let content: Vec<String> = (0..100).map(|i| format!("Line {}", i)).collect();
        let mut state = LogViewerState::new(content);
        state.visible_height = 10;

        state.page_down();
        assert_eq!(state.scroll_y, 10);
        state.page_down();
        assert_eq!(state.scroll_y, 20);

        state.page_up();
        assert_eq!(state.scroll_y, 10);
        state.page_up();
        assert_eq!(state.scroll_y, 0);
    }

    #[test]
    fn test_go_to_top_bottom() {
        let content: Vec<String> = (0..50).map(|i| format!("Line {}", i)).collect();
        let mut state = LogViewerState::new(content);
        state.visible_height = 10;

        state.go_to_bottom();
        assert_eq!(state.scroll_y, 40); // 50 - 10

        state.go_to_top();
        assert_eq!(state.scroll_y, 0);
    }

    #[test]
    fn test_go_to_line() {
        let content: Vec<String> = (0..50).map(|i| format!("Line {}", i)).collect();
        let mut state = LogViewerState::new(content);

        state.go_to_line(25);
        assert_eq!(state.scroll_y, 25);

        state.go_to_line(100); // Clamped to max
        assert_eq!(state.scroll_y, 49);
    }

    #[test]
    fn test_set_content() {
        let mut state = LogViewerState::new(vec!["Old content".into()]);
        state.scroll_y = 10;
        state.scroll_x = 5;
        state.search.query = "test".into();

        state.set_content(vec!["New content".into()]);
        assert_eq!(state.content.len(), 1);
        assert_eq!(state.content[0], "New content");
        assert_eq!(state.scroll_y, 0);
        assert_eq!(state.scroll_x, 0);
    }

    #[test]
    fn test_append() {
        let mut state = LogViewerState::new(vec!["Line 1".into()]);
        state.append("Line 2".into());
        assert_eq!(state.content.len(), 2);
        assert_eq!(state.content[1], "Line 2");
    }

    #[test]
    fn test_search() {
        let content = vec![
            "First line".into(),
            "Second line with error".into(),
            "Third line".into(),
            "Another error here".into(),
        ];
        let mut state = LogViewerState::new(content);

        state.start_search();
        state.search.query = "error".into();
        state.update_search();

        assert_eq!(state.search.matches.len(), 2);
        assert_eq!(state.search.matches[0], 1);
        assert_eq!(state.search.matches[1], 3);
    }

    #[test]
    fn test_search_case_insensitive() {
        let content = vec![
            "ERROR message".into(),
            "error again".into(),
            "No match".into(),
        ];
        let mut state = LogViewerState::new(content);

        state.search.query = "error".into();
        state.update_search();

        assert_eq!(state.search.matches.len(), 2);
    }

    #[test]
    fn test_search_empty_query() {
        let content = vec!["Line 1".into(), "Line 2".into()];
        let mut state = LogViewerState::new(content);

        state.search.query = "".into();
        state.update_search();

        assert!(state.search.matches.is_empty());
    }

    #[test]
    fn test_cancel_search() {
        let mut state = LogViewerState::new(vec!["Test".into()]);
        state.start_search();
        assert!(state.search.active);
        state.cancel_search();
        assert!(!state.search.active);
    }

    #[test]
    fn test_next_prev_match() {
        let content = vec![
            "Line 1".into(),
            "Match here".into(),
            "Line 3".into(),
            "Match here too".into(),
        ];
        let mut state = LogViewerState::new(content);

        state.search.query = "match".into();
        state.update_search();

        assert_eq!(state.search.current_match, 0);
        state.next_match();
        assert_eq!(state.search.current_match, 1);
        state.next_match();
        assert_eq!(state.search.current_match, 0); // Wrap around
        state.prev_match();
        assert_eq!(state.search.current_match, 1);
    }

    #[test]
    fn test_next_prev_match_empty() {
        let mut state = LogViewerState::new(vec!["No matches".into()]);
        state.search.query = "xyz".into();
        state.update_search();

        // Should not panic with empty matches
        state.next_match();
        state.prev_match();
        assert_eq!(state.search.current_match, 0);
    }

    #[test]
    fn test_style_for_line() {
        let style = LogViewerStyle::default();

        let error_style = style.style_for_line("[ERROR] Something failed");
        assert_eq!(error_style.fg, Some(Color::Red));

        let warn_style = style.style_for_line("[WARN] Warning message");
        assert_eq!(warn_style.fg, Some(Color::Yellow));

        let success_style = style.style_for_line("✓ Task completed");
        assert_eq!(success_style.fg, Some(Color::Green));
    }

    #[test]
    fn test_style_for_line_debug_trace() {
        let style = LogViewerStyle::default();

        let debug_style = style.style_for_line("[DEBUG] Debug message");
        assert_eq!(debug_style.fg, Some(Color::DarkGray));

        let trace_style = style.style_for_line("[TRACE] Trace message");
        assert_eq!(trace_style.fg, Some(Color::DarkGray));
    }

    #[test]
    fn test_style_default_values() {
        let style = LogViewerStyle::default();
        assert!(style.show_line_numbers);
        assert_eq!(style.line_number_width, 6);
    }

    #[test]
    fn test_log_level_colors_default() {
        let colors = LogLevelColors::default();
        assert_eq!(colors.error, Color::Red);
        assert_eq!(colors.warn, Color::Yellow);
        assert_eq!(colors.info, Color::White);
        assert_eq!(colors.debug, Color::DarkGray);
        assert_eq!(colors.success, Color::Green);
    }

    #[test]
    fn test_log_viewer_render() {
        let content = vec!["[INFO] Test".into(), "[ERROR] Error".into()];
        let state = LogViewerState::new(content);
        let viewer = LogViewer::new(&state).title("Test Log");

        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 20));
        viewer.render(Rect::new(0, 0, 80, 20), &mut buf);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_log_viewer_show_line_numbers() {
        let content = vec!["Line 1".into()];
        let state = LogViewerState::new(content);
        let viewer = LogViewer::new(&state).show_line_numbers(false);

        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 10));
        viewer.render(Rect::new(0, 0, 40, 10), &mut buf);
    }
}
