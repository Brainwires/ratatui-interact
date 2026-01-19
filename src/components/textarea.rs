//! TextArea component - Multi-line text input with cursor
//!
//! Supports multi-line text editing with cursor movement, line numbers,
//! scrolling, focus styling, and click-to-focus.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{TextArea, TextAreaState, TextAreaStyle};
//!
//! let mut state = TextAreaState::new("Hello\nWorld");
//!
//! // Cursor starts at beginning
//! assert_eq!(state.cursor_line, 0);
//! assert_eq!(state.cursor_col, 0);
//!
//! // Navigate to end
//! state.move_to_end();
//! assert_eq!(state.cursor_line, 1);
//! assert_eq!(state.cursor_col, 5);
//!
//! // Create widget (state passed to render_stateful)
//! let textarea = TextArea::new()
//!     .label("Editor")
//!     .placeholder("Enter text...");
//! ```

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::traits::{ClickRegion, FocusId};

/// Convert character index to byte index in a string.
fn char_to_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

/// Get character at index in a string.
fn char_at(s: &str, index: usize) -> Option<char> {
    s.chars().nth(index)
}

/// Actions a textarea can emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextAreaAction {
    /// Focus the textarea.
    Focus,
}

/// Tab handling configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabConfig {
    /// Insert spaces (default: 4 spaces).
    Spaces(usize),
    /// Insert a literal tab character.
    Literal,
}

impl Default for TabConfig {
    fn default() -> Self {
        TabConfig::Spaces(4)
    }
}

/// Wrap mode for long lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WrapMode {
    /// No wrapping - horizontal scroll instead.
    #[default]
    None,
    /// Soft wrap at word boundaries.
    Soft,
}

/// State for a multi-line text area.
#[derive(Debug, Clone)]
pub struct TextAreaState {
    /// Lines of text.
    pub lines: Vec<String>,
    /// Current line (0-indexed).
    pub cursor_line: usize,
    /// Cursor column (character index within line).
    pub cursor_col: usize,
    /// Vertical scroll offset.
    pub scroll_y: usize,
    /// Horizontal scroll offset (for no-wrap mode).
    pub scroll_x: usize,
    /// Visible viewport height (set during render).
    pub visible_height: usize,
    /// Whether the textarea has focus.
    pub focused: bool,
    /// Whether the textarea is enabled.
    pub enabled: bool,
    /// Tab configuration.
    pub tab_config: TabConfig,
}

impl Default for TextAreaState {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_y: 0,
            scroll_x: 0,
            visible_height: 0,
            focused: false,
            enabled: true,
            tab_config: TabConfig::default(),
        }
    }
}

impl TextAreaState {
    /// Create a new textarea state with initial text.
    ///
    /// Cursor is positioned at the start of the text.
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(|s| s.to_string()).collect()
        };
        // Ensure at least one line
        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };

        Self {
            lines,
            cursor_line: 0,
            cursor_col: 0,
            scroll_y: 0,
            scroll_x: 0,
            visible_height: 0,
            focused: false,
            enabled: true,
            tab_config: TabConfig::default(),
        }
    }

    /// Create an empty textarea state.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Set the tab configuration.
    pub fn with_tab_config(mut self, config: TabConfig) -> Self {
        self.tab_config = config;
        self
    }

    // ========================================================================
    // Character operations
    // ========================================================================

    /// Insert a character at cursor position.
    pub fn insert_char(&mut self, c: char) {
        if !self.enabled {
            return;
        }
        let byte_pos = char_to_byte_index(&self.lines[self.cursor_line], self.cursor_col);
        self.lines[self.cursor_line].insert(byte_pos, c);
        self.cursor_col += 1;
    }

    /// Insert a string at cursor position (handles multi-line input).
    pub fn insert_str(&mut self, s: &str) {
        if !self.enabled {
            return;
        }
        for c in s.chars() {
            if c == '\n' {
                self.insert_newline();
            } else if c != '\r' {
                self.insert_char(c);
            }
        }
    }

    /// Insert a newline at cursor position.
    pub fn insert_newline(&mut self) {
        if !self.enabled {
            return;
        }

        let byte_pos = char_to_byte_index(&self.lines[self.cursor_line], self.cursor_col);

        // Split the current line
        let rest = self.lines[self.cursor_line][byte_pos..].to_string();
        self.lines[self.cursor_line].truncate(byte_pos);

        // Insert new line after current
        self.cursor_line += 1;
        self.lines.insert(self.cursor_line, rest);
        self.cursor_col = 0;

        self.ensure_cursor_visible();
    }

    /// Insert a tab (spaces or literal depending on config).
    pub fn insert_tab(&mut self) {
        if !self.enabled {
            return;
        }
        match self.tab_config {
            TabConfig::Spaces(n) => {
                for _ in 0..n {
                    self.insert_char(' ');
                }
            }
            TabConfig::Literal => {
                self.insert_char('\t');
            }
        }
    }

    // ========================================================================
    // Deletion operations
    // ========================================================================

    /// Delete character before cursor (backspace).
    ///
    /// At the start of a line, merges with previous line.
    /// Returns `true` if any change was made.
    pub fn delete_char_backward(&mut self) -> bool {
        if !self.enabled {
            return false;
        }

        if self.cursor_col > 0 {
            // Delete character within line
            self.cursor_col -= 1;
            let byte_pos = char_to_byte_index(&self.lines[self.cursor_line], self.cursor_col);
            if let Some(c) = self.lines[self.cursor_line][byte_pos..].chars().next() {
                self.lines[self.cursor_line]
                    .replace_range(byte_pos..byte_pos + c.len_utf8(), "");
                return true;
            }
        } else if self.cursor_line > 0 {
            // Merge with previous line
            let current_line = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].chars().count();
            self.lines[self.cursor_line].push_str(&current_line);
            self.ensure_cursor_visible();
            return true;
        }
        false
    }

    /// Delete character at cursor (delete key).
    ///
    /// At the end of a line, merges with next line.
    /// Returns `true` if any change was made.
    pub fn delete_char_forward(&mut self) -> bool {
        if !self.enabled {
            return false;
        }

        let line_len = self.lines[self.cursor_line].chars().count();

        if self.cursor_col < line_len {
            // Delete character within line
            let byte_pos = char_to_byte_index(&self.lines[self.cursor_line], self.cursor_col);
            if let Some(c) = self.lines[self.cursor_line][byte_pos..].chars().next() {
                self.lines[self.cursor_line]
                    .replace_range(byte_pos..byte_pos + c.len_utf8(), "");
                return true;
            }
        } else if self.cursor_line + 1 < self.lines.len() {
            // Merge with next line
            let next_line = self.lines.remove(self.cursor_line + 1);
            self.lines[self.cursor_line].push_str(&next_line);
            return true;
        }
        false
    }

    /// Delete word before cursor.
    ///
    /// Returns `true` if any characters were deleted.
    pub fn delete_word_backward(&mut self) -> bool {
        if !self.enabled || (self.cursor_col == 0 && self.cursor_line == 0) {
            return false;
        }

        // If at start of line, just merge with previous line
        if self.cursor_col == 0 {
            return self.delete_char_backward();
        }

        let start_col = self.cursor_col;
        let line = &self.lines[self.cursor_line];

        // Skip trailing whitespace
        while self.cursor_col > 0 {
            if let Some(c) = char_at(line, self.cursor_col - 1) {
                if c.is_whitespace() {
                    self.cursor_col -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Delete word characters
        while self.cursor_col > 0 {
            if let Some(c) = char_at(&self.lines[self.cursor_line], self.cursor_col - 1)
            {
                if !c.is_whitespace() {
                    self.delete_char_backward();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        start_col != self.cursor_col
    }

    /// Delete entire current line.
    ///
    /// If there's only one line, clears it instead.
    pub fn delete_line(&mut self) {
        if !self.enabled {
            return;
        }

        if self.lines.len() == 1 {
            self.lines[0].clear();
            self.cursor_col = 0;
        } else {
            self.lines.remove(self.cursor_line);
            if self.cursor_line >= self.lines.len() {
                self.cursor_line = self.lines.len().saturating_sub(1);
            }
            // Adjust cursor column to fit new line
            let new_line_len = self.lines[self.cursor_line].chars().count();
            self.cursor_col = self.cursor_col.min(new_line_len);
        }
        self.ensure_cursor_visible();
    }

    /// Delete from cursor to line start (Ctrl+U).
    pub fn delete_to_line_start(&mut self) {
        if !self.enabled || self.cursor_col == 0 {
            return;
        }

        let line = &self.lines[self.cursor_line];
        let byte_pos = char_to_byte_index(line, self.cursor_col);
        self.lines[self.cursor_line] = line[byte_pos..].to_string();
        self.cursor_col = 0;
    }

    /// Delete from cursor to line end (Ctrl+K).
    pub fn delete_to_line_end(&mut self) {
        if !self.enabled {
            return;
        }

        let line = &self.lines[self.cursor_line];
        let byte_pos = char_to_byte_index(line, self.cursor_col);
        self.lines[self.cursor_line] = line[..byte_pos].to_string();
    }

    // ========================================================================
    // Cursor movement - Horizontal
    // ========================================================================

    /// Move cursor left by one character.
    ///
    /// At the start of a line, moves to end of previous line.
    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].chars().count();
            self.ensure_cursor_visible();
        }
    }

    /// Move cursor right by one character.
    ///
    /// At the end of a line, moves to start of next line.
    pub fn move_right(&mut self) {
        let line_len = self.lines[self.cursor_line].chars().count();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
            self.ensure_cursor_visible();
        }
    }

    /// Move cursor to start of line (Home).
    pub fn move_line_start(&mut self) {
        self.cursor_col = 0;
    }

    /// Move cursor to end of line (End).
    pub fn move_line_end(&mut self) {
        self.cursor_col = self.lines[self.cursor_line].chars().count();
    }

    /// Move cursor left by one word.
    pub fn move_word_left(&mut self) {
        if self.cursor_col == 0 {
            if self.cursor_line > 0 {
                self.cursor_line -= 1;
                self.cursor_col = self.lines[self.cursor_line].chars().count();
                self.ensure_cursor_visible();
            }
            return;
        }

        let line = &self.lines[self.cursor_line];

        // Skip whitespace
        while self.cursor_col > 0 {
            if let Some(c) = char_at(line, self.cursor_col - 1) {
                if c.is_whitespace() {
                    self.cursor_col -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Skip word characters
        while self.cursor_col > 0 {
            if let Some(c) = char_at(line, self.cursor_col - 1) {
                if !c.is_whitespace() {
                    self.cursor_col -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Move cursor right by one word.
    pub fn move_word_right(&mut self) {
        let line = &self.lines[self.cursor_line];
        let line_len = line.chars().count();

        if self.cursor_col >= line_len {
            if self.cursor_line + 1 < self.lines.len() {
                self.cursor_line += 1;
                self.cursor_col = 0;
                self.ensure_cursor_visible();
            }
            return;
        }

        // Skip current word
        while self.cursor_col < line_len {
            if let Some(c) = char_at(&self.lines[self.cursor_line], self.cursor_col) {
                if !c.is_whitespace() {
                    self.cursor_col += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Skip whitespace
        let line_len = self.lines[self.cursor_line].chars().count();
        while self.cursor_col < line_len {
            if let Some(c) = char_at(&self.lines[self.cursor_line], self.cursor_col) {
                if c.is_whitespace() {
                    self.cursor_col += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    // ========================================================================
    // Cursor movement - Vertical
    // ========================================================================

    /// Move cursor up by one line.
    pub fn move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            // Clamp column to new line length
            let new_line_len = self.lines[self.cursor_line].chars().count();
            self.cursor_col = self.cursor_col.min(new_line_len);
            self.ensure_cursor_visible();
        }
    }

    /// Move cursor down by one line.
    pub fn move_down(&mut self) {
        if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            // Clamp column to new line length
            let new_line_len = self.lines[self.cursor_line].chars().count();
            self.cursor_col = self.cursor_col.min(new_line_len);
            self.ensure_cursor_visible();
        }
    }

    /// Move cursor up by one page.
    pub fn move_page_up(&mut self) {
        let page_size = self.visible_height.max(1);
        if self.cursor_line >= page_size {
            self.cursor_line -= page_size;
        } else {
            self.cursor_line = 0;
        }
        // Clamp column to new line length
        let new_line_len = self.lines[self.cursor_line].chars().count();
        self.cursor_col = self.cursor_col.min(new_line_len);
        self.ensure_cursor_visible();
    }

    /// Move cursor down by one page.
    pub fn move_page_down(&mut self) {
        let page_size = self.visible_height.max(1);
        let max_line = self.lines.len().saturating_sub(1);
        self.cursor_line = (self.cursor_line + page_size).min(max_line);
        // Clamp column to new line length
        let new_line_len = self.lines[self.cursor_line].chars().count();
        self.cursor_col = self.cursor_col.min(new_line_len);
        self.ensure_cursor_visible();
    }

    /// Move cursor to start of document (Ctrl+Home).
    pub fn move_to_start(&mut self) {
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.ensure_cursor_visible();
    }

    /// Move cursor to end of document (Ctrl+End).
    pub fn move_to_end(&mut self) {
        self.cursor_line = self.lines.len().saturating_sub(1);
        self.cursor_col = self.lines[self.cursor_line].chars().count();
        self.ensure_cursor_visible();
    }

    // ========================================================================
    // Scroll management
    // ========================================================================

    /// Scroll to make cursor visible.
    pub fn scroll_to_cursor(&mut self) {
        // Vertical scroll
        if self.cursor_line < self.scroll_y {
            self.scroll_y = self.cursor_line;
        } else if self.visible_height > 0 && self.cursor_line >= self.scroll_y + self.visible_height
        {
            self.scroll_y = self.cursor_line - self.visible_height + 1;
        }
    }

    /// Ensure cursor is visible (alias for scroll_to_cursor).
    pub fn ensure_cursor_visible(&mut self) {
        self.scroll_to_cursor();
    }

    /// Scroll up by one line.
    pub fn scroll_up(&mut self) {
        self.scroll_y = self.scroll_y.saturating_sub(1);
    }

    /// Scroll down by one line.
    pub fn scroll_down(&mut self) {
        let max_scroll = self.lines.len().saturating_sub(self.visible_height.max(1));
        if self.scroll_y < max_scroll {
            self.scroll_y += 1;
        }
    }

    /// Scroll left (for no-wrap mode).
    pub fn scroll_left(&mut self) {
        self.scroll_x = self.scroll_x.saturating_sub(4);
    }

    /// Scroll right (for no-wrap mode).
    pub fn scroll_right(&mut self) {
        self.scroll_x += 4;
    }

    // ========================================================================
    // Content helpers
    // ========================================================================

    /// Get full text content (all lines joined with newlines).
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Set text content.
    ///
    /// Cursor moves to the end.
    pub fn set_text(&mut self, text: impl Into<String>) {
        let text = text.into();
        self.lines = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(|s| s.to_string()).collect()
        };
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor_line = self.lines.len().saturating_sub(1);
        self.cursor_col = self.lines[self.cursor_line].chars().count();
        self.scroll_y = 0;
        self.scroll_x = 0;
    }

    /// Clear all text.
    pub fn clear(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_y = 0;
        self.scroll_x = 0;
    }

    /// Get number of lines.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get current line content.
    pub fn current_line(&self) -> &str {
        &self.lines[self.cursor_line]
    }

    /// Check if textarea is empty.
    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    /// Get total character count (including newlines).
    pub fn len(&self) -> usize {
        let line_chars: usize = self.lines.iter().map(|l| l.chars().count()).sum();
        let newlines = self.lines.len().saturating_sub(1);
        line_chars + newlines
    }

    /// Get text before cursor on current line.
    pub fn text_before_cursor(&self) -> &str {
        let line = &self.lines[self.cursor_line];
        let byte_pos = char_to_byte_index(line, self.cursor_col);
        &line[..byte_pos]
    }

    /// Get text after cursor on current line.
    pub fn text_after_cursor(&self) -> &str {
        let line = &self.lines[self.cursor_line];
        let byte_pos = char_to_byte_index(line, self.cursor_col);
        &line[byte_pos..]
    }
}

/// Configuration for textarea appearance.
#[derive(Debug, Clone)]
pub struct TextAreaStyle {
    /// Border color when focused.
    pub focused_border: Color,
    /// Border color when unfocused.
    pub unfocused_border: Color,
    /// Border color when disabled.
    pub disabled_border: Color,
    /// Text foreground color.
    pub text_fg: Color,
    /// Cursor color.
    pub cursor_fg: Color,
    /// Placeholder text color.
    pub placeholder_fg: Color,
    /// Line number foreground color.
    pub line_number_fg: Color,
    /// Current line background highlight (optional).
    pub current_line_bg: Option<Color>,
    /// Whether to show line numbers.
    pub show_line_numbers: bool,
}

impl Default for TextAreaStyle {
    fn default() -> Self {
        Self {
            focused_border: Color::Yellow,
            unfocused_border: Color::Gray,
            disabled_border: Color::DarkGray,
            text_fg: Color::White,
            cursor_fg: Color::Yellow,
            placeholder_fg: Color::DarkGray,
            line_number_fg: Color::DarkGray,
            current_line_bg: None,
            show_line_numbers: false,
        }
    }
}

impl TextAreaStyle {
    /// Set the focused border color.
    pub fn focused_border(mut self, color: Color) -> Self {
        self.focused_border = color;
        self
    }

    /// Set the unfocused border color.
    pub fn unfocused_border(mut self, color: Color) -> Self {
        self.unfocused_border = color;
        self
    }

    /// Set the disabled border color.
    pub fn disabled_border(mut self, color: Color) -> Self {
        self.disabled_border = color;
        self
    }

    /// Set the text color.
    pub fn text_fg(mut self, color: Color) -> Self {
        self.text_fg = color;
        self
    }

    /// Set the cursor color.
    pub fn cursor_fg(mut self, color: Color) -> Self {
        self.cursor_fg = color;
        self
    }

    /// Set the placeholder color.
    pub fn placeholder_fg(mut self, color: Color) -> Self {
        self.placeholder_fg = color;
        self
    }

    /// Set the line number color.
    pub fn line_number_fg(mut self, color: Color) -> Self {
        self.line_number_fg = color;
        self
    }

    /// Set the current line background highlight.
    pub fn current_line_bg(mut self, color: Option<Color>) -> Self {
        self.current_line_bg = color;
        self
    }

    /// Enable or disable line numbers.
    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }
}

/// TextArea widget.
///
/// A multi-line text input field with cursor, label, and placeholder support.
pub struct TextArea<'a> {
    label: Option<&'a str>,
    placeholder: Option<&'a str>,
    style: TextAreaStyle,
    focus_id: FocusId,
    with_border: bool,
    wrap_mode: WrapMode,
}

impl TextArea<'_> {
    /// Create a new textarea widget.
    pub fn new() -> Self {
        Self {
            label: None,
            placeholder: None,
            style: TextAreaStyle::default(),
            focus_id: FocusId::default(),
            with_border: true,
            wrap_mode: WrapMode::default(),
        }
    }
}

impl Default for TextArea<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TextArea<'a> {

    /// Set the label (displayed in the border title).
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set the placeholder text (shown when empty).
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    /// Set the textarea style.
    pub fn style(mut self, style: TextAreaStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the focus ID.
    pub fn focus_id(mut self, id: FocusId) -> Self {
        self.focus_id = id;
        self
    }

    /// Enable or disable the border.
    pub fn with_border(mut self, with_border: bool) -> Self {
        self.with_border = with_border;
        self
    }

    /// Set the wrap mode.
    pub fn wrap_mode(mut self, mode: WrapMode) -> Self {
        self.wrap_mode = mode;
        self
    }

    /// Render the textarea and return the click region.
    pub fn render_stateful(
        self,
        frame: &mut Frame,
        area: Rect,
        state: &mut TextAreaState,
    ) -> ClickRegion<TextAreaAction> {
        let border_color = if !state.enabled {
            self.style.disabled_border
        } else if state.focused {
            self.style.focused_border
        } else {
            self.style.unfocused_border
        };

        let block = if self.with_border {
            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color));
            if let Some(label) = self.label {
                block = block.title(format!(" {} ", label));
            }
            Some(block)
        } else {
            None
        };

        let inner_area = if let Some(ref b) = block {
            b.inner(area)
        } else {
            area
        };

        // Update visible height in state
        state.visible_height = inner_area.height as usize;

        // Calculate line number gutter width
        let line_num_width = if self.style.show_line_numbers {
            let max_line = state.lines.len();
            let digits = max_line.to_string().len();
            digits + 2 // digits + space + separator
        } else {
            0
        };

        // Calculate content width
        let content_width = (inner_area.width as usize).saturating_sub(line_num_width);

        // Handle empty state
        if state.is_empty() && !state.focused {
            if let Some(placeholder) = self.placeholder {
                let display_line = Line::from(Span::styled(
                    placeholder,
                    Style::default().fg(self.style.placeholder_fg),
                ));
                let paragraph = Paragraph::new(display_line);

                if let Some(block) = block {
                    frame.render_widget(block, area);
                }
                frame.render_widget(paragraph, inner_area);
                return ClickRegion::new(area, TextAreaAction::Focus);
            }
        }

        // Build visible lines
        let start_line = state.scroll_y;
        let end_line = (start_line + state.visible_height).min(state.lines.len());

        let mut display_lines: Vec<Line> = Vec::new();

        for line_idx in start_line..end_line {
            let line = &state.lines[line_idx];
            let is_cursor_line = line_idx == state.cursor_line;

            // Apply horizontal scroll
            let chars: Vec<char> = line.chars().collect();
            let visible_chars: String = chars
                .iter()
                .skip(state.scroll_x)
                .take(content_width)
                .collect();

            let mut spans = Vec::new();

            // Line number
            if self.style.show_line_numbers {
                let line_num = format!(
                    "{:>width$} ",
                    line_idx + 1,
                    width = line_num_width.saturating_sub(2)
                );
                spans.push(Span::styled(
                    line_num,
                    Style::default().fg(self.style.line_number_fg),
                ));
            }

            // Determine line style
            let line_style = if is_cursor_line {
                if let Some(bg) = self.style.current_line_bg {
                    Style::default().fg(self.style.text_fg).bg(bg)
                } else {
                    Style::default().fg(self.style.text_fg)
                }
            } else {
                Style::default().fg(self.style.text_fg)
            };

            // Build content with cursor
            if is_cursor_line && state.focused {
                // Calculate visible cursor position
                let cursor_visible_col =
                    state.cursor_col.saturating_sub(state.scroll_x);

                let visible_char_count = visible_chars.chars().count();

                if cursor_visible_col <= visible_char_count {
                    let before: String = visible_chars.chars().take(cursor_visible_col).collect();
                    let cursor_char: String = visible_chars
                        .chars()
                        .skip(cursor_visible_col)
                        .take(1)
                        .collect();
                    let after: String = visible_chars.chars().skip(cursor_visible_col + 1).collect();

                    if !before.is_empty() {
                        spans.push(Span::styled(before, line_style));
                    }

                    // Cursor with inverted colors (block cursor)
                    let cursor_style = Style::default()
                        .fg(self.style.cursor_fg)
                        .bg(self.style.text_fg);
                    let cursor_display = if cursor_char.is_empty() { " " } else { &cursor_char };
                    spans.push(Span::styled(cursor_display.to_string(), cursor_style));

                    if !after.is_empty() {
                        spans.push(Span::styled(after, line_style));
                    }
                } else {
                    // Cursor is scrolled off to the right
                    spans.push(Span::styled(visible_chars, line_style));
                }
            } else {
                spans.push(Span::styled(visible_chars, line_style));
            }

            display_lines.push(Line::from(spans));
        }

        // Handle case when there are no lines to display (but cursor is active)
        if display_lines.is_empty() && state.focused {
            let mut spans = Vec::new();
            if self.style.show_line_numbers {
                let line_num = format!("{:>width$} ", 1, width = line_num_width.saturating_sub(2));
                spans.push(Span::styled(
                    line_num,
                    Style::default().fg(self.style.line_number_fg),
                ));
            }
            // Block cursor (inverted colors)
            let cursor_style = Style::default()
                .fg(self.style.cursor_fg)
                .bg(self.style.text_fg);
            spans.push(Span::styled(" ", cursor_style));
            display_lines.push(Line::from(spans));
        }

        let paragraph = Paragraph::new(display_lines);

        if let Some(block) = block {
            frame.render_widget(block, area);
        }
        frame.render_widget(paragraph, inner_area);

        ClickRegion::new(area, TextAreaAction::Focus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // State construction tests
    // ========================================================================

    #[test]
    fn test_state_default() {
        let state = TextAreaState::default();
        assert_eq!(state.lines.len(), 1);
        assert!(state.lines[0].is_empty());
        assert_eq!(state.cursor_line, 0);
        assert_eq!(state.cursor_col, 0);
        assert!(!state.focused);
        assert!(state.enabled);
    }

    #[test]
    fn test_state_new_single_line() {
        let state = TextAreaState::new("Hello");
        assert_eq!(state.lines.len(), 1);
        assert_eq!(state.lines[0], "Hello");
        assert_eq!(state.cursor_line, 0);
        assert_eq!(state.cursor_col, 0);
    }

    #[test]
    fn test_state_new_multi_line() {
        let state = TextAreaState::new("Hello\nWorld");
        assert_eq!(state.lines.len(), 2);
        assert_eq!(state.lines[0], "Hello");
        assert_eq!(state.lines[1], "World");
        assert_eq!(state.cursor_line, 0);
        assert_eq!(state.cursor_col, 0);
    }

    #[test]
    fn test_state_new_empty() {
        let state = TextAreaState::new("");
        assert_eq!(state.lines.len(), 1);
        assert!(state.lines[0].is_empty());
        assert_eq!(state.cursor_line, 0);
        assert_eq!(state.cursor_col, 0);
    }

    #[test]
    fn test_state_empty() {
        let state = TextAreaState::empty();
        assert!(state.is_empty());
    }

    // ========================================================================
    // Character operations tests
    // ========================================================================

    #[test]
    fn test_insert_char() {
        let mut state = TextAreaState::new("Hello");
        state.move_to_end();
        state.insert_char('!');
        assert_eq!(state.lines[0], "Hello!");
        assert_eq!(state.cursor_col, 6);
    }

    #[test]
    fn test_insert_char_middle() {
        let mut state = TextAreaState::new("Hllo");
        state.cursor_col = 1;
        state.insert_char('e');
        assert_eq!(state.lines[0], "Hello");
        assert_eq!(state.cursor_col, 2);
    }

    #[test]
    fn test_insert_str_single_line() {
        let mut state = TextAreaState::new("Hello");
        state.move_to_end();
        state.insert_str(" World");
        assert_eq!(state.lines[0], "Hello World");
    }

    #[test]
    fn test_insert_str_multi_line() {
        let mut state = TextAreaState::new("Hello");
        state.move_to_end();
        state.insert_str(" World\nNew Line");
        assert_eq!(state.lines.len(), 2);
        assert_eq!(state.lines[0], "Hello World");
        assert_eq!(state.lines[1], "New Line");
    }

    #[test]
    fn test_insert_newline() {
        let mut state = TextAreaState::new("HelloWorld");
        state.cursor_col = 5;
        state.insert_newline();
        assert_eq!(state.lines.len(), 2);
        assert_eq!(state.lines[0], "Hello");
        assert_eq!(state.lines[1], "World");
        assert_eq!(state.cursor_line, 1);
        assert_eq!(state.cursor_col, 0);
    }

    #[test]
    fn test_insert_newline_at_start() {
        let mut state = TextAreaState::new("Hello");
        state.insert_newline();
        assert_eq!(state.lines.len(), 2);
        assert_eq!(state.lines[0], "");
        assert_eq!(state.lines[1], "Hello");
    }

    #[test]
    fn test_insert_newline_at_end() {
        let mut state = TextAreaState::new("Hello");
        state.move_to_end();
        state.insert_newline();
        assert_eq!(state.lines.len(), 2);
        assert_eq!(state.lines[0], "Hello");
        assert_eq!(state.lines[1], "");
    }

    #[test]
    fn test_insert_tab_spaces() {
        let mut state = TextAreaState::empty();
        state.tab_config = TabConfig::Spaces(4);
        state.insert_tab();
        assert_eq!(state.lines[0], "    ");
    }

    #[test]
    fn test_insert_tab_literal() {
        let mut state = TextAreaState::empty();
        state.tab_config = TabConfig::Literal;
        state.insert_tab();
        assert_eq!(state.lines[0], "\t");
    }

    // ========================================================================
    // Deletion tests
    // ========================================================================

    #[test]
    fn test_delete_char_backward() {
        let mut state = TextAreaState::new("Hello");
        state.move_to_end();
        assert!(state.delete_char_backward());
        assert_eq!(state.lines[0], "Hell");
        assert_eq!(state.cursor_col, 4);
    }

    #[test]
    fn test_delete_char_backward_at_start() {
        let mut state = TextAreaState::new("Hello");
        // Cursor starts at 0, so no need to set it
        assert!(!state.delete_char_backward());
        assert_eq!(state.lines[0], "Hello");
    }

    #[test]
    fn test_delete_char_backward_merges_lines() {
        let mut state = TextAreaState::new("Hello\nWorld");
        state.cursor_line = 1;
        state.cursor_col = 0;
        assert!(state.delete_char_backward());
        assert_eq!(state.lines.len(), 1);
        assert_eq!(state.lines[0], "HelloWorld");
        assert_eq!(state.cursor_col, 5);
    }

    #[test]
    fn test_delete_char_forward() {
        let mut state = TextAreaState::new("Hello");
        state.cursor_col = 0;
        assert!(state.delete_char_forward());
        assert_eq!(state.lines[0], "ello");
    }

    #[test]
    fn test_delete_char_forward_at_end() {
        let mut state = TextAreaState::new("Hello");
        state.move_to_end();
        assert!(!state.delete_char_forward());
        assert_eq!(state.lines[0], "Hello");
    }

    #[test]
    fn test_delete_char_forward_merges_lines() {
        let mut state = TextAreaState::new("Hello\nWorld");
        state.cursor_line = 0;
        state.cursor_col = 5;
        assert!(state.delete_char_forward());
        assert_eq!(state.lines.len(), 1);
        assert_eq!(state.lines[0], "HelloWorld");
    }

    #[test]
    fn test_delete_word_backward() {
        let mut state = TextAreaState::new("Hello World");
        state.move_to_end();
        assert!(state.delete_word_backward());
        assert_eq!(state.lines[0], "Hello ");
    }

    #[test]
    fn test_delete_word_backward_from_start() {
        let mut state = TextAreaState::new("Hello");
        // Cursor starts at 0
        assert!(!state.delete_word_backward());
    }

    #[test]
    fn test_delete_line() {
        let mut state = TextAreaState::new("Line 1\nLine 2\nLine 3");
        state.cursor_line = 1;
        state.cursor_col = 0;
        state.delete_line();
        assert_eq!(state.lines.len(), 2);
        assert_eq!(state.lines[0], "Line 1");
        assert_eq!(state.lines[1], "Line 3");
    }

    #[test]
    fn test_delete_line_single() {
        let mut state = TextAreaState::new("Hello");
        state.delete_line();
        assert_eq!(state.lines.len(), 1);
        assert!(state.lines[0].is_empty());
    }

    #[test]
    fn test_delete_to_line_start() {
        let mut state = TextAreaState::new("Hello World");
        state.cursor_col = 6;
        state.delete_to_line_start();
        assert_eq!(state.lines[0], "World");
        assert_eq!(state.cursor_col, 0);
    }

    #[test]
    fn test_delete_to_line_end() {
        let mut state = TextAreaState::new("Hello World");
        state.cursor_col = 5;
        state.delete_to_line_end();
        assert_eq!(state.lines[0], "Hello");
    }

    // ========================================================================
    // Cursor movement tests
    // ========================================================================

    #[test]
    fn test_move_left() {
        let mut state = TextAreaState::new("Hello");
        state.move_to_end();
        state.move_left();
        assert_eq!(state.cursor_col, 4);
    }

    #[test]
    fn test_move_left_wraps_line() {
        let mut state = TextAreaState::new("Hello\nWorld");
        state.cursor_line = 1;
        state.cursor_col = 0;
        state.move_left();
        assert_eq!(state.cursor_line, 0);
        assert_eq!(state.cursor_col, 5);
    }

    #[test]
    fn test_move_left_at_start() {
        let mut state = TextAreaState::new("Hello");
        state.cursor_col = 0;
        state.move_left();
        assert_eq!(state.cursor_col, 0);
    }

    #[test]
    fn test_move_right() {
        let mut state = TextAreaState::new("Hello");
        state.cursor_col = 0;
        state.move_right();
        assert_eq!(state.cursor_col, 1);
    }

    #[test]
    fn test_move_right_wraps_line() {
        let mut state = TextAreaState::new("Hello\nWorld");
        state.cursor_line = 0;
        state.cursor_col = 5;
        state.move_right();
        assert_eq!(state.cursor_line, 1);
        assert_eq!(state.cursor_col, 0);
    }

    #[test]
    fn test_move_right_at_end() {
        let mut state = TextAreaState::new("Hello");
        state.move_to_end();
        state.move_right();
        assert_eq!(state.cursor_col, 5); // Should stay at end
    }

    #[test]
    fn test_move_line_start() {
        let mut state = TextAreaState::new("Hello");
        state.move_line_start();
        assert_eq!(state.cursor_col, 0);
    }

    #[test]
    fn test_move_line_end() {
        let mut state = TextAreaState::new("Hello");
        state.cursor_col = 0;
        state.move_line_end();
        assert_eq!(state.cursor_col, 5);
    }

    #[test]
    fn test_move_up() {
        let mut state = TextAreaState::new("Line 1\nLine 2\nLine 3");
        state.cursor_line = 2; // Start at last line
        state.move_up();
        assert_eq!(state.cursor_line, 1);
    }

    #[test]
    fn test_move_up_clamps_column() {
        let mut state = TextAreaState::new("AB\nCDEFG");
        state.cursor_line = 1; // Start on second line (CDEFG)
        state.cursor_col = 5;
        state.move_up();
        assert_eq!(state.cursor_line, 0);
        assert_eq!(state.cursor_col, 2); // Clamped to line length
    }

    #[test]
    fn test_move_down() {
        let mut state = TextAreaState::new("Line 1\nLine 2\nLine 3");
        state.cursor_line = 0;
        state.move_down();
        assert_eq!(state.cursor_line, 1);
    }

    #[test]
    fn test_move_down_at_last_line() {
        let mut state = TextAreaState::new("Hello");
        state.move_down();
        assert_eq!(state.cursor_line, 0);
    }

    #[test]
    fn test_move_word_left() {
        let mut state = TextAreaState::new("Hello World Test");
        state.move_to_end(); // Start at end of text
        state.move_word_left();
        assert_eq!(state.cursor_col, 12); // Start of "Test"
    }

    #[test]
    fn test_move_word_right() {
        let mut state = TextAreaState::new("Hello World Test");
        state.cursor_col = 0;
        state.move_word_right();
        assert_eq!(state.cursor_col, 6); // After "Hello "
    }

    #[test]
    fn test_move_page_up() {
        let mut state = TextAreaState::new("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
        state.visible_height = 3;
        state.cursor_line = 9; // Start at last line
        state.move_page_up();
        assert_eq!(state.cursor_line, 6);
    }

    #[test]
    fn test_move_page_down() {
        let mut state = TextAreaState::new("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
        state.cursor_line = 0;
        state.visible_height = 3;
        state.move_page_down();
        assert_eq!(state.cursor_line, 3);
    }

    #[test]
    fn test_move_to_start() {
        let mut state = TextAreaState::new("Hello\nWorld");
        state.move_to_start();
        assert_eq!(state.cursor_line, 0);
        assert_eq!(state.cursor_col, 0);
    }

    #[test]
    fn test_move_to_end() {
        let mut state = TextAreaState::new("Hello\nWorld");
        state.cursor_line = 0;
        state.cursor_col = 0;
        state.move_to_end();
        assert_eq!(state.cursor_line, 1);
        assert_eq!(state.cursor_col, 5);
    }

    // ========================================================================
    // Content helpers tests
    // ========================================================================

    #[test]
    fn test_text() {
        let state = TextAreaState::new("Hello\nWorld");
        assert_eq!(state.text(), "Hello\nWorld");
    }

    #[test]
    fn test_set_text() {
        let mut state = TextAreaState::new("Old");
        state.set_text("New\nContent");
        assert_eq!(state.lines.len(), 2);
        assert_eq!(state.lines[0], "New");
        assert_eq!(state.lines[1], "Content");
        assert_eq!(state.cursor_line, 1);
        assert_eq!(state.cursor_col, 7);
    }

    #[test]
    fn test_clear() {
        let mut state = TextAreaState::new("Hello\nWorld");
        state.clear();
        assert!(state.is_empty());
        assert_eq!(state.cursor_line, 0);
        assert_eq!(state.cursor_col, 0);
    }

    #[test]
    fn test_line_count() {
        let state = TextAreaState::new("A\nB\nC");
        assert_eq!(state.line_count(), 3);
    }

    #[test]
    fn test_current_line() {
        let mut state = TextAreaState::new("Hello\nWorld");
        state.cursor_line = 0;
        assert_eq!(state.current_line(), "Hello");
    }

    #[test]
    fn test_is_empty() {
        let state = TextAreaState::empty();
        assert!(state.is_empty());

        let state = TextAreaState::new("Hi");
        assert!(!state.is_empty());
    }

    #[test]
    fn test_len() {
        let state = TextAreaState::new("Hi\nWorld");
        // "Hi" (2) + "\n" (1) + "World" (5) = 8
        assert_eq!(state.len(), 8);
    }

    #[test]
    fn test_text_before_after_cursor() {
        let mut state = TextAreaState::new("Hello World");
        state.cursor_col = 5;
        assert_eq!(state.text_before_cursor(), "Hello");
        assert_eq!(state.text_after_cursor(), " World");
    }

    // ========================================================================
    // Unicode handling tests
    // ========================================================================

    #[test]
    fn test_unicode_handling() {
        let mut state = TextAreaState::new("你好");
        state.move_to_end();
        assert_eq!(state.cursor_col, 2);

        state.move_left();
        assert_eq!(state.cursor_col, 1);

        state.insert_char('世');
        assert_eq!(state.lines[0], "你世好");
    }

    #[test]
    fn test_emoji_handling() {
        let mut state = TextAreaState::new("Hi 👋");
        assert_eq!(state.len(), 4);

        state.move_to_end();
        state.delete_char_backward();
        assert_eq!(state.lines[0], "Hi ");
    }

    // ========================================================================
    // Disabled state tests
    // ========================================================================

    #[test]
    fn test_disabled_no_insert() {
        let mut state = TextAreaState::new("Hello");
        state.enabled = false;
        state.insert_char('!');
        assert_eq!(state.lines[0], "Hello");
    }

    #[test]
    fn test_disabled_no_delete() {
        let mut state = TextAreaState::new("Hello");
        state.enabled = false;
        assert!(!state.delete_char_backward());
        assert_eq!(state.lines[0], "Hello");
    }

    #[test]
    fn test_disabled_no_newline() {
        let mut state = TextAreaState::new("Hello");
        state.enabled = false;
        state.insert_newline();
        assert_eq!(state.lines.len(), 1);
    }

    // ========================================================================
    // Scroll tests
    // ========================================================================

    #[test]
    fn test_scroll_to_cursor_down() {
        let mut state = TextAreaState::new("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
        state.visible_height = 3;
        state.cursor_line = 8;
        state.scroll_to_cursor();
        assert_eq!(state.scroll_y, 6);
    }

    #[test]
    fn test_scroll_to_cursor_up() {
        let mut state = TextAreaState::new("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
        state.visible_height = 3;
        state.scroll_y = 5;
        state.cursor_line = 2;
        state.scroll_to_cursor();
        assert_eq!(state.scroll_y, 2);
    }

    #[test]
    fn test_scroll_up() {
        let mut state = TextAreaState::new("1\n2\n3");
        state.scroll_y = 2;
        state.scroll_up();
        assert_eq!(state.scroll_y, 1);
    }

    #[test]
    fn test_scroll_down() {
        let mut state = TextAreaState::new("1\n2\n3\n4\n5");
        state.visible_height = 2;
        state.scroll_down();
        assert_eq!(state.scroll_y, 1);
    }

    // ========================================================================
    // Style tests
    // ========================================================================

    #[test]
    fn test_style_default() {
        let style = TextAreaStyle::default();
        assert_eq!(style.focused_border, Color::Yellow);
        assert_eq!(style.text_fg, Color::White);
        assert!(!style.show_line_numbers);
    }

    #[test]
    fn test_style_builder() {
        let style = TextAreaStyle::default()
            .focused_border(Color::Cyan)
            .text_fg(Color::Green)
            .show_line_numbers(true);

        assert_eq!(style.focused_border, Color::Cyan);
        assert_eq!(style.text_fg, Color::Green);
        assert!(style.show_line_numbers);
    }

    // ========================================================================
    // Tab config tests
    // ========================================================================

    #[test]
    fn test_tab_config_default() {
        let config = TabConfig::default();
        assert_eq!(config, TabConfig::Spaces(4));
    }

    #[test]
    fn test_with_tab_config() {
        let state = TextAreaState::empty().with_tab_config(TabConfig::Spaces(2));
        assert_eq!(state.tab_config, TabConfig::Spaces(2));
    }
}
