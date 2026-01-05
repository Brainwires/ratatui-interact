//! CheckBox component - Toggleable checkbox with label
//!
//! Supports keyboard focus, mouse clicks, and customizable styling.
//!
//! # Example
//!
//! ```rust
//! use tui_extension::components::{CheckBox, CheckBoxState, CheckBoxStyle};
//! use ratatui::{buffer::Buffer, layout::Rect};
//!
//! let mut state = CheckBoxState::new(false);
//! let checkbox = CheckBox::new("Enable notifications", &state)
//!     .style(CheckBoxStyle::unicode());
//!
//! // Toggle when activated
//! state.toggle();
//! assert!(state.checked);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::traits::{ClickRegion, FocusId};

/// Actions a checkbox can emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckBoxAction {
    /// Toggle the checkbox state.
    Toggle,
}

/// State for a checkbox.
#[derive(Debug, Clone)]
pub struct CheckBoxState {
    /// Whether the checkbox is checked.
    pub checked: bool,
    /// Whether the checkbox has focus.
    pub focused: bool,
    /// Whether the checkbox is enabled (can be toggled).
    pub enabled: bool,
}

impl Default for CheckBoxState {
    fn default() -> Self {
        Self {
            checked: false,
            focused: false,
            enabled: true,
        }
    }
}

impl CheckBoxState {
    /// Create a new checkbox state.
    ///
    /// # Arguments
    ///
    /// * `checked` - Initial checked state
    pub fn new(checked: bool) -> Self {
        Self {
            checked,
            ..Default::default()
        }
    }

    /// Toggle the checkbox state.
    ///
    /// Does nothing if the checkbox is disabled.
    pub fn toggle(&mut self) {
        if self.enabled {
            self.checked = !self.checked;
        }
    }

    /// Set the checked state.
    pub fn set_checked(&mut self, checked: bool) {
        if self.enabled {
            self.checked = checked;
        }
    }

    /// Set the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Set the enabled state.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Configuration for checkbox appearance.
#[derive(Debug, Clone)]
pub struct CheckBoxStyle {
    /// Symbol when checked.
    pub checked_symbol: &'static str,
    /// Symbol when unchecked.
    pub unchecked_symbol: &'static str,
    /// Foreground color when focused.
    pub focused_fg: Color,
    /// Foreground color when unfocused.
    pub unfocused_fg: Color,
    /// Foreground color when disabled.
    pub disabled_fg: Color,
    /// Foreground color when checked (unfocused).
    pub checked_fg: Color,
}

impl Default for CheckBoxStyle {
    fn default() -> Self {
        Self {
            checked_symbol: "[x]",
            unchecked_symbol: "[ ]",
            focused_fg: Color::Yellow,
            unfocused_fg: Color::White,
            disabled_fg: Color::DarkGray,
            checked_fg: Color::Green,
        }
    }
}

impl CheckBoxStyle {
    /// ASCII style with brackets: `[x]` and `[ ]`
    pub fn ascii() -> Self {
        Self::default()
    }

    /// Unicode box style: `☑` and `☐`
    pub fn unicode() -> Self {
        Self {
            checked_symbol: "☑",
            unchecked_symbol: "☐",
            ..Default::default()
        }
    }

    /// Unicode checkmark style: `✓` and `○`
    pub fn checkmark() -> Self {
        Self {
            checked_symbol: "✓",
            unchecked_symbol: "○",
            ..Default::default()
        }
    }

    /// Custom symbols.
    pub fn custom(checked: &'static str, unchecked: &'static str) -> Self {
        Self {
            checked_symbol: checked,
            unchecked_symbol: unchecked,
            ..Default::default()
        }
    }

    /// Set the focused foreground color.
    pub fn focused_fg(mut self, color: Color) -> Self {
        self.focused_fg = color;
        self
    }

    /// Set the unfocused foreground color.
    pub fn unfocused_fg(mut self, color: Color) -> Self {
        self.unfocused_fg = color;
        self
    }

    /// Set the disabled foreground color.
    pub fn disabled_fg(mut self, color: Color) -> Self {
        self.disabled_fg = color;
        self
    }

    /// Set the checked foreground color.
    pub fn checked_fg(mut self, color: Color) -> Self {
        self.checked_fg = color;
        self
    }
}

/// CheckBox widget.
///
/// A toggleable checkbox with a label that supports focus styling
/// and mouse click regions.
pub struct CheckBox<'a> {
    label: &'a str,
    state: &'a CheckBoxState,
    style: CheckBoxStyle,
    focus_id: FocusId,
}

impl<'a> CheckBox<'a> {
    /// Create a new checkbox.
    ///
    /// # Arguments
    ///
    /// * `label` - The text label displayed next to the checkbox
    /// * `state` - Reference to the checkbox state
    pub fn new(label: &'a str, state: &'a CheckBoxState) -> Self {
        Self {
            label,
            state,
            style: CheckBoxStyle::default(),
            focus_id: FocusId::default(),
        }
    }

    /// Set the checkbox style.
    pub fn style(mut self, style: CheckBoxStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the focus ID.
    pub fn focus_id(mut self, id: FocusId) -> Self {
        self.focus_id = id;
        self
    }

    /// Build the display line for this checkbox.
    fn build_line(&self) -> Line<'a> {
        let symbol = if self.state.checked {
            self.style.checked_symbol
        } else {
            self.style.unchecked_symbol
        };

        let fg_color = if !self.state.enabled {
            self.style.disabled_fg
        } else if self.state.focused {
            self.style.focused_fg
        } else if self.state.checked {
            self.style.checked_fg
        } else {
            self.style.unfocused_fg
        };

        let mut style = Style::default().fg(fg_color);
        if self.state.focused && self.state.enabled {
            style = style.add_modifier(Modifier::BOLD);
        }

        Line::from(vec![
            Span::styled(symbol, style),
            Span::styled(" ", style),
            Span::styled(self.label, style),
        ])
    }

    /// Calculate width needed for this checkbox.
    pub fn width(&self) -> u16 {
        let symbol_len = if self.state.checked {
            self.style.checked_symbol.chars().count()
        } else {
            self.style.unchecked_symbol.chars().count()
        };
        (symbol_len + 1 + self.label.chars().count()) as u16
    }

    /// Render the checkbox and return the click region.
    ///
    /// Use this method when you need to track click regions for mouse handling.
    pub fn render_stateful(self, area: Rect, buf: &mut Buffer) -> ClickRegion<CheckBoxAction> {
        let width = self.width().min(area.width);
        let click_area = Rect::new(area.x, area.y, width, 1);

        let line = self.build_line();
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);

        ClickRegion::new(click_area, CheckBoxAction::Toggle)
    }
}

impl Widget for CheckBox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = self.build_line();
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_default() {
        let state = CheckBoxState::default();
        assert!(!state.checked);
        assert!(!state.focused);
        assert!(state.enabled);
    }

    #[test]
    fn test_state_new() {
        let state = CheckBoxState::new(true);
        assert!(state.checked);
        assert!(!state.focused);
        assert!(state.enabled);
    }

    #[test]
    fn test_toggle() {
        let mut state = CheckBoxState::new(false);
        assert!(!state.checked);

        state.toggle();
        assert!(state.checked);

        state.toggle();
        assert!(!state.checked);
    }

    #[test]
    fn test_toggle_disabled() {
        let mut state = CheckBoxState::new(false);
        state.enabled = false;

        state.toggle();
        assert!(!state.checked); // Should not change when disabled
    }

    #[test]
    fn test_set_checked() {
        let mut state = CheckBoxState::new(false);

        state.set_checked(true);
        assert!(state.checked);

        state.set_checked(false);
        assert!(!state.checked);
    }

    #[test]
    fn test_set_checked_disabled() {
        let mut state = CheckBoxState::new(false);
        state.enabled = false;

        state.set_checked(true);
        assert!(!state.checked); // Should not change when disabled
    }

    #[test]
    fn test_style_default() {
        let style = CheckBoxStyle::default();
        assert_eq!(style.checked_symbol, "[x]");
        assert_eq!(style.unchecked_symbol, "[ ]");
    }

    #[test]
    fn test_style_unicode() {
        let style = CheckBoxStyle::unicode();
        assert_eq!(style.checked_symbol, "☑");
        assert_eq!(style.unchecked_symbol, "☐");
    }

    #[test]
    fn test_style_checkmark() {
        let style = CheckBoxStyle::checkmark();
        assert_eq!(style.checked_symbol, "✓");
        assert_eq!(style.unchecked_symbol, "○");
    }

    #[test]
    fn test_style_custom() {
        let style = CheckBoxStyle::custom("ON", "OFF");
        assert_eq!(style.checked_symbol, "ON");
        assert_eq!(style.unchecked_symbol, "OFF");
    }

    #[test]
    fn test_checkbox_width() {
        let state = CheckBoxState::new(false);
        let checkbox = CheckBox::new("Test", &state);

        // "[ ] Test" = 3 + 1 + 4 = 8
        assert_eq!(checkbox.width(), 8);
    }

    #[test]
    fn test_checkbox_width_unicode() {
        let state = CheckBoxState::new(true);
        let checkbox = CheckBox::new("Test", &state).style(CheckBoxStyle::unicode());

        // "☑ Test" = 1 + 1 + 4 = 6
        assert_eq!(checkbox.width(), 6);
    }

    #[test]
    fn test_render_basic() {
        let state = CheckBoxState::new(true);
        let checkbox = CheckBox::new("Test", &state);

        let area = Rect::new(0, 0, 20, 1);
        let mut buffer = Buffer::empty(area);

        checkbox.render(area, &mut buffer);

        // Check that content was rendered
        let content: String = (0..8)
            .map(|x| buffer[(x, 0)].symbol().to_string())
            .collect();
        assert!(content.contains("[x]"));
    }

    #[test]
    fn test_render_stateful() {
        let state = CheckBoxState::new(false);
        let checkbox = CheckBox::new("Click me", &state);

        let area = Rect::new(5, 3, 20, 1);
        let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 10));

        let click_region = checkbox.render_stateful(area, &mut buffer);

        // Click region should match the checkbox area
        assert_eq!(click_region.area.x, 5);
        assert_eq!(click_region.area.y, 3);
        assert_eq!(click_region.data, CheckBoxAction::Toggle);
    }

    #[test]
    fn test_click_region_detection() {
        let state = CheckBoxState::new(false);
        let checkbox = CheckBox::new("Test", &state);

        let area = Rect::new(10, 5, 20, 1);
        let mut buffer = Buffer::empty(Rect::new(0, 0, 40, 10));

        let click_region = checkbox.render_stateful(area, &mut buffer);

        // Should detect clicks within the checkbox
        assert!(click_region.contains(10, 5));
        assert!(click_region.contains(15, 5));

        // Should not detect clicks outside
        assert!(!click_region.contains(9, 5));
        assert!(!click_region.contains(10, 4));
        assert!(!click_region.contains(10, 6));
    }
}
