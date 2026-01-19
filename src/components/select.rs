//! Select component - Dropdown select box
//!
//! A dropdown select component that renders as a compact closed state and opens
//! a popup overlay with selectable options when activated.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{Select, SelectState, SelectStyle};
//! use ratatui::layout::Rect;
//!
//! // Create options
//! let options = vec!["Option A", "Option B", "Option C"];
//!
//! // Create state
//! let mut state = SelectState::new(options.len());
//!
//! // Create select widget
//! let select = Select::new(&options, &state)
//!     .label("Choose")
//!     .placeholder("Select an option...");
//!
//! // Render and handle events (see handle_select_key, handle_select_mouse)
//! ```

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
    Frame,
};

use crate::traits::{ClickRegion, FocusId};

/// Actions a select component can emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectAction {
    /// Focus the select (from click).
    Focus,
    /// Open the dropdown.
    Open,
    /// Close the dropdown without selection.
    Close,
    /// An option was selected (index).
    Select(usize),
}

/// State for a select component.
#[derive(Debug, Clone)]
pub struct SelectState {
    /// Currently selected index (None if nothing selected).
    pub selected_index: Option<usize>,
    /// Whether the dropdown is open.
    pub is_open: bool,
    /// Whether the select has focus.
    pub focused: bool,
    /// Whether the select is enabled.
    pub enabled: bool,
    /// Highlighted index in dropdown (for keyboard navigation).
    pub highlighted_index: usize,
    /// Scroll offset for long option lists.
    pub scroll_offset: u16,
    /// Total number of options.
    pub total_options: usize,
}

impl Default for SelectState {
    fn default() -> Self {
        Self {
            selected_index: None,
            is_open: false,
            focused: false,
            enabled: true,
            highlighted_index: 0,
            scroll_offset: 0,
            total_options: 0,
        }
    }
}

impl SelectState {
    /// Create a new select state with given number of options.
    pub fn new(total_options: usize) -> Self {
        Self {
            total_options,
            ..Default::default()
        }
    }

    /// Create with a pre-selected index.
    pub fn with_selected(total_options: usize, selected: usize) -> Self {
        let mut state = Self::new(total_options);
        if selected < total_options {
            state.selected_index = Some(selected);
            state.highlighted_index = selected;
        }
        state
    }

    /// Open the dropdown.
    pub fn open(&mut self) {
        if self.enabled {
            self.is_open = true;
            // Start highlight at selected item if any
            if let Some(idx) = self.selected_index {
                self.highlighted_index = idx;
            }
        }
    }

    /// Close the dropdown.
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Toggle dropdown open/closed.
    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    /// Move highlight up.
    pub fn highlight_prev(&mut self) {
        if self.highlighted_index > 0 {
            self.highlighted_index -= 1;
        }
    }

    /// Move highlight down.
    pub fn highlight_next(&mut self) {
        if self.highlighted_index + 1 < self.total_options {
            self.highlighted_index += 1;
        }
    }

    /// Move highlight to first option.
    pub fn highlight_first(&mut self) {
        self.highlighted_index = 0;
        self.scroll_offset = 0;
    }

    /// Move highlight to last option.
    pub fn highlight_last(&mut self) {
        if self.total_options > 0 {
            self.highlighted_index = self.total_options - 1;
        }
    }

    /// Select the currently highlighted option and close.
    pub fn select_highlighted(&mut self) {
        if self.total_options > 0 {
            self.selected_index = Some(self.highlighted_index);
        }
        self.close();
    }

    /// Select a specific index.
    pub fn select(&mut self, index: usize) {
        if index < self.total_options {
            self.selected_index = Some(index);
            self.highlighted_index = index;
        }
        self.close();
    }

    /// Clear the selection.
    pub fn clear_selection(&mut self) {
        self.selected_index = None;
    }

    /// Update total options count.
    pub fn set_total(&mut self, total: usize) {
        self.total_options = total;
        if let Some(idx) = self.selected_index {
            if idx >= total {
                self.selected_index = if total > 0 { Some(total - 1) } else { None };
            }
        }
        if self.highlighted_index >= total && total > 0 {
            self.highlighted_index = total - 1;
        }
    }

    /// Ensure highlighted item is visible in viewport.
    pub fn ensure_visible(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }
        if self.highlighted_index < self.scroll_offset as usize {
            self.scroll_offset = self.highlighted_index as u16;
        } else if self.highlighted_index >= self.scroll_offset as usize + viewport_height {
            self.scroll_offset = (self.highlighted_index - viewport_height + 1) as u16;
        }
    }

    /// Get the selected index.
    pub fn selected(&self) -> Option<usize> {
        self.selected_index
    }

    /// Check if an option is selected.
    pub fn has_selection(&self) -> bool {
        self.selected_index.is_some()
    }
}

/// Style configuration for select component.
#[derive(Debug, Clone)]
pub struct SelectStyle {
    /// Border color when focused.
    pub focused_border: Color,
    /// Border color when unfocused.
    pub unfocused_border: Color,
    /// Border color when disabled.
    pub disabled_border: Color,
    /// Text color for selected value.
    pub text_fg: Color,
    /// Placeholder text color.
    pub placeholder_fg: Color,
    /// Dropdown indicator (e.g., "▼").
    pub dropdown_indicator: &'static str,
    /// Highlighted option style in dropdown.
    pub highlight_style: Style,
    /// Normal option style in dropdown.
    pub option_style: Style,
    /// Selected option indicator.
    pub selected_indicator: &'static str,
    /// Unselected option indicator (padding).
    pub unselected_indicator: &'static str,
    /// Dropdown border color.
    pub dropdown_border: Color,
    /// Max visible options in dropdown.
    pub max_visible_options: u16,
}

impl Default for SelectStyle {
    fn default() -> Self {
        Self {
            focused_border: Color::Yellow,
            unfocused_border: Color::Gray,
            disabled_border: Color::DarkGray,
            text_fg: Color::White,
            placeholder_fg: Color::DarkGray,
            dropdown_indicator: "▼",
            highlight_style: Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            option_style: Style::default().fg(Color::White),
            selected_indicator: "✓ ",
            unselected_indicator: "  ",
            dropdown_border: Color::Cyan,
            max_visible_options: 8,
        }
    }
}

impl SelectStyle {
    /// Minimal style without heavy highlighting.
    pub fn minimal() -> Self {
        Self {
            highlight_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            ..Default::default()
        }
    }

    /// Style with arrow indicator.
    pub fn arrow() -> Self {
        Self {
            selected_indicator: "→ ",
            unselected_indicator: "  ",
            ..Default::default()
        }
    }

    /// Style with bracket indicator.
    pub fn bracket() -> Self {
        Self {
            selected_indicator: "[x] ",
            unselected_indicator: "[ ] ",
            ..Default::default()
        }
    }

    /// Set max visible options in dropdown.
    pub fn max_options(mut self, max: u16) -> Self {
        self.max_visible_options = max;
        self
    }

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

    /// Set the dropdown indicator.
    pub fn indicator(mut self, indicator: &'static str) -> Self {
        self.dropdown_indicator = indicator;
        self
    }

    /// Set the highlight style for dropdown options.
    pub fn highlight(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }
}

/// Default render function type for options.
type DefaultRenderFn<T> = fn(&T) -> String;

/// Select widget - dropdown select box.
///
/// A dropdown select component that renders as a compact closed state and opens
/// a popup overlay with selectable options when activated.
pub struct Select<'a, T, F = DefaultRenderFn<T>>
where
    F: Fn(&T) -> String,
{
    options: &'a [T],
    state: &'a SelectState,
    style: SelectStyle,
    placeholder: &'a str,
    label: Option<&'a str>,
    render_option: F,
    focus_id: FocusId,
}

impl<'a, T: std::fmt::Display> Select<'a, T, DefaultRenderFn<T>> {
    /// Create a new select widget with default option rendering.
    pub fn new(options: &'a [T], state: &'a SelectState) -> Self {
        Self {
            options,
            state,
            style: SelectStyle::default(),
            placeholder: "Please select an option",
            label: None,
            render_option: |opt| opt.to_string(),
            focus_id: FocusId::default(),
        }
    }
}

impl<'a, T, F> Select<'a, T, F>
where
    F: Fn(&T) -> String,
{
    /// Set a custom option renderer.
    pub fn render_option<G>(self, render_fn: G) -> Select<'a, T, G>
    where
        G: Fn(&T) -> String,
    {
        Select {
            options: self.options,
            state: self.state,
            style: self.style,
            placeholder: self.placeholder,
            label: self.label,
            render_option: render_fn,
            focus_id: self.focus_id,
        }
    }

    /// Set the placeholder text.
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    /// Set the label (border title).
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set the style.
    pub fn style(mut self, style: SelectStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the focus ID.
    pub fn focus_id(mut self, id: FocusId) -> Self {
        self.focus_id = id;
        self
    }

    /// Render the closed select box and return click region.
    ///
    /// This renders the compact closed state of the select box.
    /// Call `render_dropdown` separately when the dropdown is open.
    pub fn render_stateful(self, frame: &mut Frame, area: Rect) -> ClickRegion<SelectAction> {
        let border_color = if !self.state.enabled {
            self.style.disabled_border
        } else if self.state.focused {
            self.style.focused_border
        } else {
            self.style.unfocused_border
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        if let Some(label) = self.label {
            block = block.title(format!(" {} ", label));
        }

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Build display text
        let display_text = if let Some(idx) = self.state.selected_index {
            if idx < self.options.len() {
                let text = (self.render_option)(&self.options[idx]);
                Span::styled(text, Style::default().fg(self.style.text_fg))
            } else {
                Span::styled(self.placeholder, Style::default().fg(self.style.placeholder_fg))
            }
        } else {
            Span::styled(self.placeholder, Style::default().fg(self.style.placeholder_fg))
        };

        // Add dropdown indicator on the right
        let indicator_color = if self.state.focused {
            self.style.focused_border
        } else {
            self.style.unfocused_border
        };

        let indicator = Span::styled(
            format!(" {}", self.style.dropdown_indicator),
            Style::default().fg(indicator_color),
        );

        let line = Line::from(vec![display_text, indicator]);
        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, inner);

        ClickRegion::new(area, SelectAction::Focus)
    }

    /// Render the dropdown overlay.
    ///
    /// Call this when `state.is_open` is true. Returns click regions for each option.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to
    /// * `anchor` - The area of the closed select box (dropdown positions below this)
    /// * `screen` - The full screen area (for bounds checking)
    pub fn render_dropdown(
        &self,
        frame: &mut Frame,
        anchor: Rect,
        screen: Rect,
    ) -> Vec<ClickRegion<SelectAction>> {
        let mut regions = Vec::new();

        if self.options.is_empty() {
            return regions;
        }

        let visible_count = (self.options.len() as u16).min(self.style.max_visible_options);
        let dropdown_height = visible_count + 2; // +2 for borders

        let dropdown_width = anchor.width;

        // Position dropdown below the anchor, but flip up if not enough space
        let space_below = screen.height.saturating_sub(anchor.y + anchor.height);
        let space_above = anchor.y.saturating_sub(screen.y);

        let (dropdown_y, flip_up) = if space_below >= dropdown_height {
            (anchor.y + anchor.height, false)
        } else if space_above >= dropdown_height {
            (anchor.y.saturating_sub(dropdown_height), true)
        } else {
            // Not enough space either way, use below and clip
            (anchor.y + anchor.height, false)
        };

        let dropdown_area = Rect::new(
            anchor.x,
            dropdown_y,
            dropdown_width,
            dropdown_height.min(if flip_up { space_above } else { space_below }),
        );

        // Clear background
        frame.render_widget(Clear, dropdown_area);

        // Render border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.style.dropdown_border));

        let inner = block.inner(dropdown_area);
        frame.render_widget(block, dropdown_area);

        // Render options
        let actual_visible = inner.height as usize;
        let scroll = self.state.scroll_offset as usize;

        for (i, option) in self
            .options
            .iter()
            .enumerate()
            .skip(scroll)
            .take(actual_visible)
        {
            let y = inner.y + (i - scroll) as u16;
            let option_area = Rect::new(inner.x, y, inner.width, 1);

            let is_highlighted = i == self.state.highlighted_index;
            let is_selected = self.state.selected_index == Some(i);

            let style = if is_highlighted {
                self.style.highlight_style
            } else {
                self.style.option_style
            };

            let prefix = if is_selected {
                self.style.selected_indicator
            } else {
                self.style.unselected_indicator
            };

            let text = format!("{}{}", prefix, (self.render_option)(option));

            // Truncate if too long
            let max_width = inner.width as usize;
            let display_text: String = text.chars().take(max_width).collect();

            let paragraph = Paragraph::new(Span::styled(display_text, style));
            frame.render_widget(paragraph, option_area);

            // Register click region for this option
            regions.push(ClickRegion::new(option_area, SelectAction::Select(i)));
        }

        regions
    }

    /// Render the select box using Buffer (Widget-style rendering).
    ///
    /// This is useful when you need to render without a Frame reference.
    pub fn render_to_buffer(self, area: Rect, buf: &mut Buffer) -> ClickRegion<SelectAction> {
        let border_color = if !self.state.enabled {
            self.style.disabled_border
        } else if self.state.focused {
            self.style.focused_border
        } else {
            self.style.unfocused_border
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        if let Some(label) = self.label {
            block = block.title(format!(" {} ", label));
        }

        let inner = block.inner(area);
        block.render(area, buf);

        // Build display text
        let display_text = if let Some(idx) = self.state.selected_index {
            if idx < self.options.len() {
                let text = (self.render_option)(&self.options[idx]);
                Span::styled(text, Style::default().fg(self.style.text_fg))
            } else {
                Span::styled(self.placeholder, Style::default().fg(self.style.placeholder_fg))
            }
        } else {
            Span::styled(self.placeholder, Style::default().fg(self.style.placeholder_fg))
        };

        let indicator_color = if self.state.focused {
            self.style.focused_border
        } else {
            self.style.unfocused_border
        };

        let indicator = Span::styled(
            format!(" {}", self.style.dropdown_indicator),
            Style::default().fg(indicator_color),
        );

        let line = Line::from(vec![display_text, indicator]);
        let paragraph = Paragraph::new(line);
        paragraph.render(inner, buf);

        ClickRegion::new(area, SelectAction::Focus)
    }
}

/// Handle keyboard events for select component.
///
/// Returns `Some(SelectAction)` if an action was triggered, `None` otherwise.
///
/// # Key Bindings
///
/// When closed:
/// - `Enter`, `Space`, `Down` - Open dropdown
///
/// When open:
/// - `Esc` - Close without selection
/// - `Enter`, `Space` - Select highlighted option
/// - `Up` - Move highlight up
/// - `Down` - Move highlight down
/// - `Home` - Move to first option
/// - `End` - Move to last option
/// - `PageUp` - Move up by 5
/// - `PageDown` - Move down by 5
pub fn handle_select_key(key: &KeyEvent, state: &mut SelectState) -> Option<SelectAction> {
    if !state.enabled {
        return None;
    }

    if state.is_open {
        // Dropdown is open - handle navigation
        match key.code {
            KeyCode::Esc => {
                state.close();
                Some(SelectAction::Close)
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let idx = state.highlighted_index;
                state.select_highlighted();
                Some(SelectAction::Select(idx))
            }
            KeyCode::Up => {
                state.highlight_prev();
                state.ensure_visible(8); // Use default visible count
                None
            }
            KeyCode::Down => {
                state.highlight_next();
                state.ensure_visible(8);
                None
            }
            KeyCode::Home => {
                state.highlight_first();
                None
            }
            KeyCode::End => {
                state.highlight_last();
                state.ensure_visible(8);
                None
            }
            KeyCode::PageUp => {
                for _ in 0..5 {
                    state.highlight_prev();
                }
                state.ensure_visible(8);
                None
            }
            KeyCode::PageDown => {
                for _ in 0..5 {
                    state.highlight_next();
                }
                state.ensure_visible(8);
                None
            }
            _ => None,
        }
    } else {
        // Dropdown is closed
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Down => {
                state.open();
                Some(SelectAction::Open)
            }
            _ => None,
        }
    }
}

/// Handle mouse events for select component.
///
/// Returns `Some(SelectAction)` if an action was triggered, `None` otherwise.
///
/// # Arguments
///
/// * `mouse` - The mouse event
/// * `state` - Mutable reference to select state
/// * `select_area` - The area of the closed select box
/// * `dropdown_regions` - Click regions from `render_dropdown` (empty if closed)
pub fn handle_select_mouse(
    mouse: &MouseEvent,
    state: &mut SelectState,
    select_area: Rect,
    dropdown_regions: &[ClickRegion<SelectAction>],
) -> Option<SelectAction> {
    if !state.enabled {
        return None;
    }

    if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
        let col = mouse.column;
        let row = mouse.row;

        if state.is_open {
            // Check if clicked on an option
            for region in dropdown_regions {
                if region.contains(col, row) {
                    if let SelectAction::Select(idx) = region.data {
                        state.select(idx);
                        return Some(SelectAction::Select(idx));
                    }
                }
            }

            // Check if clicked on the select box itself (toggle/close)
            if col >= select_area.x
                && col < select_area.x + select_area.width
                && row >= select_area.y
                && row < select_area.y + select_area.height
            {
                state.close();
                return Some(SelectAction::Close);
            }

            // Clicked outside - close
            state.close();
            Some(SelectAction::Close)
        } else {
            // Dropdown is closed - check if clicked on select box
            if col >= select_area.x
                && col < select_area.x + select_area.width
                && row >= select_area.y
                && row < select_area.y + select_area.height
            {
                state.open();
                return Some(SelectAction::Open);
            }
            None
        }
    } else {
        None
    }
}

/// Calculate the height needed for the select dropdown.
///
/// Useful for layout calculations.
pub fn calculate_dropdown_height(option_count: usize, max_visible: u16) -> u16 {
    let visible = (option_count as u16).min(max_visible);
    visible + 2 // +2 for borders
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_default() {
        let state = SelectState::default();
        assert!(state.selected_index.is_none());
        assert!(!state.is_open);
        assert!(!state.focused);
        assert!(state.enabled);
        assert_eq!(state.highlighted_index, 0);
    }

    #[test]
    fn test_state_new() {
        let state = SelectState::new(5);
        assert_eq!(state.total_options, 5);
        assert!(state.selected_index.is_none());
    }

    #[test]
    fn test_state_with_selected() {
        let state = SelectState::with_selected(5, 2);
        assert_eq!(state.selected_index, Some(2));
        assert_eq!(state.highlighted_index, 2);
    }

    #[test]
    fn test_state_with_selected_out_of_bounds() {
        let state = SelectState::with_selected(5, 10);
        assert!(state.selected_index.is_none());
        assert_eq!(state.highlighted_index, 0);
    }

    #[test]
    fn test_open_close() {
        let mut state = SelectState::new(5);

        state.open();
        assert!(state.is_open);

        state.close();
        assert!(!state.is_open);

        state.toggle();
        assert!(state.is_open);

        state.toggle();
        assert!(!state.is_open);
    }

    #[test]
    fn test_open_disabled() {
        let mut state = SelectState::new(5);
        state.enabled = false;

        state.open();
        assert!(!state.is_open);
    }

    #[test]
    fn test_highlight_navigation() {
        let mut state = SelectState::new(5);

        state.highlight_next();
        assert_eq!(state.highlighted_index, 1);

        state.highlight_next();
        assert_eq!(state.highlighted_index, 2);

        state.highlight_prev();
        assert_eq!(state.highlighted_index, 1);

        state.highlight_first();
        assert_eq!(state.highlighted_index, 0);

        state.highlight_last();
        assert_eq!(state.highlighted_index, 4);
    }

    #[test]
    fn test_highlight_bounds() {
        let mut state = SelectState::new(3);

        // Should not go below 0
        state.highlight_prev();
        assert_eq!(state.highlighted_index, 0);

        // Should not go above total - 1
        state.highlighted_index = 2;
        state.highlight_next();
        assert_eq!(state.highlighted_index, 2);
    }

    #[test]
    fn test_select() {
        let mut state = SelectState::new(5);
        state.is_open = true;

        state.select(2);
        assert_eq!(state.selected_index, Some(2));
        assert_eq!(state.highlighted_index, 2);
        assert!(!state.is_open); // Should close after selection
    }

    #[test]
    fn test_select_highlighted() {
        let mut state = SelectState::new(5);
        state.is_open = true;
        state.highlighted_index = 3;

        state.select_highlighted();
        assert_eq!(state.selected_index, Some(3));
        assert!(!state.is_open);
    }

    #[test]
    fn test_clear_selection() {
        let mut state = SelectState::with_selected(5, 2);
        assert!(state.has_selection());

        state.clear_selection();
        assert!(!state.has_selection());
        assert!(state.selected_index.is_none());
    }

    #[test]
    fn test_set_total() {
        let mut state = SelectState::with_selected(10, 8);
        state.highlighted_index = 9;

        state.set_total(5);
        assert_eq!(state.total_options, 5);
        assert_eq!(state.selected_index, Some(4)); // Clamped
        assert_eq!(state.highlighted_index, 4); // Clamped
    }

    #[test]
    fn test_ensure_visible() {
        let mut state = SelectState::new(20);
        state.highlighted_index = 15;
        state.scroll_offset = 0;

        state.ensure_visible(10);
        assert!(state.scroll_offset >= 6); // 15 - 10 + 1 = 6
    }

    #[test]
    fn test_style_default() {
        let style = SelectStyle::default();
        assert_eq!(style.focused_border, Color::Yellow);
        assert_eq!(style.max_visible_options, 8);
    }

    #[test]
    fn test_style_builders() {
        let style = SelectStyle::minimal();
        assert_eq!(
            style.highlight_style.add_modifier,
            Modifier::BOLD
        );

        let style = SelectStyle::arrow();
        assert_eq!(style.selected_indicator, "→ ");

        let style = SelectStyle::bracket();
        assert_eq!(style.selected_indicator, "[x] ");
    }

    #[test]
    fn test_style_builder_methods() {
        let style = SelectStyle::default()
            .max_options(10)
            .focused_border(Color::Cyan)
            .indicator("↓");

        assert_eq!(style.max_visible_options, 10);
        assert_eq!(style.focused_border, Color::Cyan);
        assert_eq!(style.dropdown_indicator, "↓");
    }

    #[test]
    fn test_handle_key_closed() {
        let mut state = SelectState::new(5);

        // Enter should open
        let key = KeyEvent::from(KeyCode::Enter);
        let action = handle_select_key(&key, &mut state);
        assert_eq!(action, Some(SelectAction::Open));
        assert!(state.is_open);
    }

    #[test]
    fn test_handle_key_open_navigation() {
        let mut state = SelectState::new(5);
        state.open();

        // Down should move highlight
        let key = KeyEvent::from(KeyCode::Down);
        handle_select_key(&key, &mut state);
        assert_eq!(state.highlighted_index, 1);

        // Up should move highlight back
        let key = KeyEvent::from(KeyCode::Up);
        handle_select_key(&key, &mut state);
        assert_eq!(state.highlighted_index, 0);
    }

    #[test]
    fn test_handle_key_open_select() {
        let mut state = SelectState::new(5);
        state.open();
        state.highlighted_index = 2;

        let key = KeyEvent::from(KeyCode::Enter);
        let action = handle_select_key(&key, &mut state);

        assert_eq!(action, Some(SelectAction::Select(2)));
        assert_eq!(state.selected_index, Some(2));
        assert!(!state.is_open);
    }

    #[test]
    fn test_handle_key_open_escape() {
        let mut state = SelectState::new(5);
        state.open();

        let key = KeyEvent::from(KeyCode::Esc);
        let action = handle_select_key(&key, &mut state);

        assert_eq!(action, Some(SelectAction::Close));
        assert!(!state.is_open);
    }

    #[test]
    fn test_handle_key_disabled() {
        let mut state = SelectState::new(5);
        state.enabled = false;

        let key = KeyEvent::from(KeyCode::Enter);
        let action = handle_select_key(&key, &mut state);

        assert!(action.is_none());
        assert!(!state.is_open);
    }

    #[test]
    fn test_calculate_dropdown_height() {
        assert_eq!(calculate_dropdown_height(3, 8), 5); // 3 + 2
        assert_eq!(calculate_dropdown_height(10, 8), 10); // 8 + 2 (clamped)
        assert_eq!(calculate_dropdown_height(0, 8), 2); // 0 + 2
    }

    #[test]
    fn test_click_region_contains() {
        let region = ClickRegion::new(Rect::new(10, 5, 20, 3), SelectAction::Select(0));

        assert!(region.contains(10, 5));
        assert!(region.contains(29, 7));
        assert!(!region.contains(9, 5));
        assert!(!region.contains(30, 5));
    }
}
