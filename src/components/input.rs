//! Input component - Text input field with cursor
//!
//! Supports single-line text input with cursor movement, label display,
//! focus styling, and click-to-focus.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{Input, InputState, InputStyle};
//!
//! let mut state = InputState::new("Hello");
//!
//! // Cursor is at end by default
//! assert_eq!(state.cursor_pos, 5);
//!
//! // Insert character
//! state.insert_char('!');
//! assert_eq!(state.text, "Hello!");
//!
//! // Move cursor
//! state.move_left();
//! state.insert_char(' ');
//! assert_eq!(state.text, "Hello !");
//! ```

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::traits::{ClickRegion, FocusId};

/// Actions an input can emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    /// Focus the input.
    Focus,
}

/// State for an input field.
#[derive(Debug, Clone)]
pub struct InputState {
    /// The text content.
    pub text: String,
    /// Cursor position (character index).
    pub cursor_pos: usize,
    /// Whether the input has focus.
    pub focused: bool,
    /// Whether the input is enabled.
    pub enabled: bool,
    /// Horizontal scroll offset for long text.
    pub scroll_offset: usize,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor_pos: 0,
            focused: false,
            enabled: true,
            scroll_offset: 0,
        }
    }
}

impl InputState {
    /// Create a new input state with initial text.
    ///
    /// Cursor is positioned at the end of the text.
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor_pos = text.chars().count();
        Self {
            text,
            cursor_pos,
            focused: false,
            enabled: true,
            scroll_offset: 0,
        }
    }

    /// Create an empty input state.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Insert a character at cursor position.
    pub fn insert_char(&mut self, c: char) {
        if !self.enabled {
            return;
        }
        let byte_pos = self.char_to_byte_index(self.cursor_pos);
        self.text.insert(byte_pos, c);
        self.cursor_pos += 1;
    }

    /// Insert a string at cursor position.
    pub fn insert_str(&mut self, s: &str) {
        if !self.enabled {
            return;
        }
        let byte_pos = self.char_to_byte_index(self.cursor_pos);
        self.text.insert_str(byte_pos, s);
        self.cursor_pos += s.chars().count();
    }

    /// Delete character before cursor (backspace).
    ///
    /// Returns `true` if a character was deleted.
    pub fn delete_char_backward(&mut self) -> bool {
        if !self.enabled || self.cursor_pos == 0 {
            return false;
        }

        self.cursor_pos -= 1;
        let byte_pos = self.char_to_byte_index(self.cursor_pos);
        if let Some(c) = self.text[byte_pos..].chars().next() {
            self.text
                .replace_range(byte_pos..byte_pos + c.len_utf8(), "");
            return true;
        }
        false
    }

    /// Delete character at cursor (delete key).
    ///
    /// Returns `true` if a character was deleted.
    pub fn delete_char_forward(&mut self) -> bool {
        if !self.enabled {
            return false;
        }

        let byte_pos = self.char_to_byte_index(self.cursor_pos);
        if byte_pos < self.text.len()
            && let Some(c) = self.text[byte_pos..].chars().next()
        {
            self.text
                .replace_range(byte_pos..byte_pos + c.len_utf8(), "");
            return true;
        }
        false
    }

    /// Delete word before cursor.
    ///
    /// Returns `true` if any characters were deleted.
    pub fn delete_word_backward(&mut self) -> bool {
        if !self.enabled || self.cursor_pos == 0 {
            return false;
        }

        let start_pos = self.cursor_pos;

        // Skip trailing whitespace
        while self.cursor_pos > 0 {
            let prev_char = self.char_at(self.cursor_pos - 1);
            if prev_char.map(|c| c.is_whitespace()).unwrap_or(false) {
                self.cursor_pos -= 1;
            } else {
                break;
            }
        }

        // Delete word characters
        while self.cursor_pos > 0 {
            let prev_char = self.char_at(self.cursor_pos - 1);
            if prev_char.map(|c| !c.is_whitespace()).unwrap_or(false) {
                self.delete_char_backward();
            } else {
                break;
            }
        }

        start_pos != self.cursor_pos
    }

    /// Move cursor left by one character.
    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    /// Move cursor right by one character.
    pub fn move_right(&mut self) {
        let max = self.text.chars().count();
        if self.cursor_pos < max {
            self.cursor_pos += 1;
        }
    }

    /// Move cursor to the start of the text.
    pub fn move_home(&mut self) {
        self.cursor_pos = 0;
    }

    /// Move cursor to the end of the text.
    pub fn move_end(&mut self) {
        self.cursor_pos = self.text.chars().count();
    }

    /// Move cursor left by one word.
    pub fn move_word_left(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }

        // Skip whitespace
        while self.cursor_pos > 0 {
            if let Some(c) = self.char_at(self.cursor_pos - 1) {
                if c.is_whitespace() {
                    self.cursor_pos -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Skip word characters
        while self.cursor_pos > 0 {
            if let Some(c) = self.char_at(self.cursor_pos - 1) {
                if !c.is_whitespace() {
                    self.cursor_pos -= 1;
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
        let max = self.text.chars().count();
        if self.cursor_pos >= max {
            return;
        }

        // Skip current word
        while self.cursor_pos < max {
            if let Some(c) = self.char_at(self.cursor_pos) {
                if !c.is_whitespace() {
                    self.cursor_pos += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Skip whitespace
        while self.cursor_pos < max {
            if let Some(c) = self.char_at(self.cursor_pos) {
                if c.is_whitespace() {
                    self.cursor_pos += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Clear the text and reset cursor.
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
        self.scroll_offset = 0;
    }

    /// Set the text content.
    ///
    /// Cursor is moved to the end.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cursor_pos = self.text.chars().count();
        self.scroll_offset = 0;
    }

    /// Get the character at a given index.
    fn char_at(&self, index: usize) -> Option<char> {
        self.text.chars().nth(index)
    }

    /// Convert character index to byte index.
    fn char_to_byte_index(&self, char_idx: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    /// Get text before cursor.
    pub fn text_before_cursor(&self) -> &str {
        let byte_pos = self.char_to_byte_index(self.cursor_pos);
        &self.text[..byte_pos]
    }

    /// Get text after cursor.
    pub fn text_after_cursor(&self) -> &str {
        let byte_pos = self.char_to_byte_index(self.cursor_pos);
        &self.text[byte_pos..]
    }

    /// Check if the input is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Get the length of the text in characters.
    pub fn len(&self) -> usize {
        self.text.chars().count()
    }

    /// Get a reference to the text content.
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Configuration for input appearance.
#[derive(Debug, Clone)]
pub struct InputStyle {
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
}

impl Default for InputStyle {
    fn default() -> Self {
        Self {
            focused_border: Color::Yellow,
            unfocused_border: Color::Gray,
            disabled_border: Color::DarkGray,
            text_fg: Color::White,
            cursor_fg: Color::Yellow,
            placeholder_fg: Color::DarkGray,
        }
    }
}

impl InputStyle {
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
}

/// Input widget.
///
/// A text input field with cursor, label, and placeholder support.
pub struct Input<'a> {
    label: Option<&'a str>,
    placeholder: Option<&'a str>,
    state: &'a InputState,
    style: InputStyle,
    focus_id: FocusId,
    with_border: bool,
}

impl<'a> Input<'a> {
    /// Create a new input widget.
    ///
    /// # Arguments
    ///
    /// * `state` - Reference to the input state
    pub fn new(state: &'a InputState) -> Self {
        Self {
            label: None,
            placeholder: None,
            state,
            style: InputStyle::default(),
            focus_id: FocusId::default(),
            with_border: true,
        }
    }

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

    /// Set the input style.
    pub fn style(mut self, style: InputStyle) -> Self {
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

    /// Render the input and return the click region.
    pub fn render_stateful(self, frame: &mut Frame, area: Rect) -> ClickRegion<InputAction> {
        let border_color = if !self.state.enabled {
            self.style.disabled_border
        } else if self.state.focused {
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

        // Build display text with cursor indicator
        let display_line = if self.state.text.is_empty() {
            if let Some(placeholder) = self.placeholder {
                Line::from(Span::styled(
                    placeholder,
                    Style::default().fg(self.style.placeholder_fg),
                ))
            } else if self.state.focused {
                // Show cursor even when empty
                Line::from(Span::styled("│", Style::default().fg(self.style.cursor_fg)))
            } else {
                Line::from("")
            }
        } else {
            let before = self.state.text_before_cursor();
            let after = self.state.text_after_cursor();

            let mut spans = vec![Span::styled(
                before.to_string(),
                Style::default().fg(self.style.text_fg),
            )];

            if self.state.focused {
                spans.push(Span::styled("│", Style::default().fg(self.style.cursor_fg)));
            }

            spans.push(Span::styled(
                after.to_string(),
                Style::default().fg(self.style.text_fg),
            ));

            Line::from(spans)
        };

        let paragraph = Paragraph::new(display_line);

        if let Some(block) = block {
            frame.render_widget(block, area);
        }
        frame.render_widget(paragraph, inner_area);

        ClickRegion::new(area, InputAction::Focus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_default() {
        let state = InputState::default();
        assert!(state.text.is_empty());
        assert_eq!(state.cursor_pos, 0);
        assert!(!state.focused);
        assert!(state.enabled);
    }

    #[test]
    fn test_state_new() {
        let state = InputState::new("Hello");
        assert_eq!(state.text, "Hello");
        assert_eq!(state.cursor_pos, 5); // At end
    }

    #[test]
    fn test_insert_char() {
        let mut state = InputState::new("Hello");
        state.insert_char('!');
        assert_eq!(state.text, "Hello!");
        assert_eq!(state.cursor_pos, 6);
    }

    #[test]
    fn test_insert_char_middle() {
        let mut state = InputState::new("Hllo");
        state.cursor_pos = 1;
        state.insert_char('e');
        assert_eq!(state.text, "Hello");
        assert_eq!(state.cursor_pos, 2);
    }

    #[test]
    fn test_insert_str() {
        let mut state = InputState::new("Hello");
        state.insert_str(" World");
        assert_eq!(state.text, "Hello World");
    }

    #[test]
    fn test_delete_char_backward() {
        let mut state = InputState::new("Hello");
        assert!(state.delete_char_backward());
        assert_eq!(state.text, "Hell");
        assert_eq!(state.cursor_pos, 4);
    }

    #[test]
    fn test_delete_char_backward_at_start() {
        let mut state = InputState::new("Hello");
        state.cursor_pos = 0;
        assert!(!state.delete_char_backward());
        assert_eq!(state.text, "Hello");
    }

    #[test]
    fn test_delete_char_forward() {
        let mut state = InputState::new("Hello");
        state.cursor_pos = 0;
        assert!(state.delete_char_forward());
        assert_eq!(state.text, "ello");
    }

    #[test]
    fn test_delete_char_forward_at_end() {
        let mut state = InputState::new("Hello");
        assert!(!state.delete_char_forward());
        assert_eq!(state.text, "Hello");
    }

    #[test]
    fn test_move_cursor() {
        let mut state = InputState::new("Hello");
        assert_eq!(state.cursor_pos, 5);

        state.move_left();
        assert_eq!(state.cursor_pos, 4);

        state.move_right();
        assert_eq!(state.cursor_pos, 5);

        state.move_home();
        assert_eq!(state.cursor_pos, 0);

        state.move_end();
        assert_eq!(state.cursor_pos, 5);
    }

    #[test]
    fn test_move_cursor_bounds() {
        let mut state = InputState::new("Hi");

        state.move_home();
        state.move_left(); // Should not go below 0
        assert_eq!(state.cursor_pos, 0);

        state.move_end();
        state.move_right(); // Should not go past end
        assert_eq!(state.cursor_pos, 2);
    }

    #[test]
    fn test_move_word() {
        let mut state = InputState::new("Hello World Test");

        state.move_home();
        state.move_word_right();
        assert_eq!(state.cursor_pos, 6); // After "Hello "

        state.move_word_right();
        assert_eq!(state.cursor_pos, 12); // After "World "

        state.move_word_left();
        assert_eq!(state.cursor_pos, 6); // Back to "World"
    }

    #[test]
    fn test_clear() {
        let mut state = InputState::new("Hello");
        state.clear();
        assert!(state.text.is_empty());
        assert_eq!(state.cursor_pos, 0);
    }

    #[test]
    fn test_set_text() {
        let mut state = InputState::new("Hello");
        state.set_text("World");
        assert_eq!(state.text, "World");
        assert_eq!(state.cursor_pos, 5);
    }

    #[test]
    fn test_text_before_after_cursor() {
        let mut state = InputState::new("Hello");
        state.cursor_pos = 2;

        assert_eq!(state.text_before_cursor(), "He");
        assert_eq!(state.text_after_cursor(), "llo");
    }

    #[test]
    fn test_unicode_handling() {
        let mut state = InputState::new("你好");
        assert_eq!(state.cursor_pos, 2); // 2 characters

        state.move_left();
        assert_eq!(state.cursor_pos, 1);

        state.insert_char('世');
        assert_eq!(state.text, "你世好");
    }

    #[test]
    fn test_emoji_handling() {
        let mut state = InputState::new("Hi 👋");
        assert_eq!(state.len(), 4); // "H", "i", " ", "👋"

        state.delete_char_backward();
        assert_eq!(state.text, "Hi ");
    }

    #[test]
    fn test_disabled_input() {
        let mut state = InputState::new("Hello");
        state.enabled = false;

        state.insert_char('!');
        assert_eq!(state.text, "Hello"); // No change

        assert!(!state.delete_char_backward());
        assert_eq!(state.text, "Hello"); // No change
    }

    #[test]
    fn test_is_empty_and_len() {
        let state = InputState::empty();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);

        let state = InputState::new("Test");
        assert!(!state.is_empty());
        assert_eq!(state.len(), 4);
    }

    #[test]
    fn test_input_style_builder() {
        let style = InputStyle::default()
            .focused_border(Color::Cyan)
            .text_fg(Color::Green);

        assert_eq!(style.focused_border, Color::Cyan);
        assert_eq!(style.text_fg, Color::Green);
    }
}
