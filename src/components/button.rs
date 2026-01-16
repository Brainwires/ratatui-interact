//! Button component - Various button views
//!
//! Supports single-line, multi-line (block), icon+text, and toggle button styles.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{Button, ButtonState, ButtonVariant};
//!
//! let state = ButtonState::enabled();
//!
//! // Single line button
//! let button = Button::new("Submit", &state)
//!     .variant(ButtonVariant::SingleLine);
//!
//! // Icon button
//! let save_btn = Button::new("Save", &state)
//!     .icon("💾");
//!
//! // Toggle button
//! let mut toggle_state = ButtonState::enabled();
//! toggle_state.toggled = true;
//! let toggle = Button::new("Dark Mode", &toggle_state)
//!     .variant(ButtonVariant::Toggle);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::traits::{ClickRegion, FocusId};

/// Actions a button can emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonAction {
    /// Button was clicked/activated.
    Click,
}

/// State for a button.
#[derive(Debug, Clone)]
pub struct ButtonState {
    /// Whether the button has focus.
    pub focused: bool,
    /// Whether the button is currently pressed.
    pub pressed: bool,
    /// Whether the button is enabled.
    pub enabled: bool,
    /// For toggle buttons: whether the button is toggled on.
    pub toggled: bool,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            focused: false,
            pressed: false,
            enabled: true,
            toggled: false,
        }
    }
}

impl ButtonState {
    /// Create an enabled button state.
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Create a disabled button state.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Create a toggled-on button state.
    pub fn toggled(toggled: bool) -> Self {
        Self {
            toggled,
            enabled: true,
            ..Default::default()
        }
    }

    /// Set the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Set the pressed state.
    pub fn set_pressed(&mut self, pressed: bool) {
        self.pressed = pressed;
    }

    /// Set the enabled state.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Toggle the toggled state.
    pub fn toggle(&mut self) {
        if self.enabled {
            self.toggled = !self.toggled;
        }
    }
}

/// Button style variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// Single line button: `[ Text ]`
    #[default]
    SingleLine,
    /// Multi-line block button with border.
    Block,
    /// Icon with text: `🔍 Search`
    IconText,
    /// Toggle button (highlighted when toggled).
    Toggle,
    /// Minimal style - just text, changes color on focus.
    Minimal,
}

/// Button styling.
#[derive(Debug, Clone)]
pub struct ButtonStyle {
    /// The button variant.
    pub variant: ButtonVariant,
    /// Foreground color when focused.
    pub focused_fg: Color,
    /// Background color when focused.
    pub focused_bg: Color,
    /// Foreground color when unfocused.
    pub unfocused_fg: Color,
    /// Background color when unfocused.
    pub unfocused_bg: Color,
    /// Foreground color when disabled.
    pub disabled_fg: Color,
    /// Foreground color when pressed.
    pub pressed_fg: Color,
    /// Background color when pressed.
    pub pressed_bg: Color,
    /// Foreground color when toggled.
    pub toggled_fg: Color,
    /// Background color when toggled.
    pub toggled_bg: Color,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            variant: ButtonVariant::SingleLine,
            focused_fg: Color::Black,
            focused_bg: Color::Yellow,
            unfocused_fg: Color::White,
            unfocused_bg: Color::DarkGray,
            disabled_fg: Color::DarkGray,
            pressed_fg: Color::Black,
            pressed_bg: Color::White,
            toggled_fg: Color::Black,
            toggled_bg: Color::Green,
        }
    }
}

impl ButtonStyle {
    /// Create a style for a specific variant.
    pub fn new(variant: ButtonVariant) -> Self {
        Self {
            variant,
            ..Default::default()
        }
    }

    /// Set the variant.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set focused colors.
    pub fn focused(mut self, fg: Color, bg: Color) -> Self {
        self.focused_fg = fg;
        self.focused_bg = bg;
        self
    }

    /// Set unfocused colors.
    pub fn unfocused(mut self, fg: Color, bg: Color) -> Self {
        self.unfocused_fg = fg;
        self.unfocused_bg = bg;
        self
    }

    /// Set toggled colors.
    pub fn toggled(mut self, fg: Color, bg: Color) -> Self {
        self.toggled_fg = fg;
        self.toggled_bg = bg;
        self
    }

    /// Primary button style (prominent).
    pub fn primary() -> Self {
        Self {
            focused_fg: Color::White,
            focused_bg: Color::Blue,
            unfocused_fg: Color::White,
            unfocused_bg: Color::Rgb(50, 100, 200),
            ..Default::default()
        }
    }

    /// Danger/destructive button style.
    pub fn danger() -> Self {
        Self {
            focused_fg: Color::White,
            focused_bg: Color::Red,
            unfocused_fg: Color::White,
            unfocused_bg: Color::Rgb(150, 50, 50),
            ..Default::default()
        }
    }

    /// Success button style.
    pub fn success() -> Self {
        Self {
            focused_fg: Color::White,
            focused_bg: Color::Green,
            unfocused_fg: Color::White,
            unfocused_bg: Color::Rgb(50, 150, 50),
            ..Default::default()
        }
    }
}

/// Button widget.
///
/// A clickable button with various display styles.
pub struct Button<'a> {
    label: &'a str,
    icon: Option<&'a str>,
    state: &'a ButtonState,
    style: ButtonStyle,
    focus_id: FocusId,
    alignment: Alignment,
}

impl<'a> Button<'a> {
    /// Create a new button.
    ///
    /// # Arguments
    ///
    /// * `label` - The button text
    /// * `state` - Reference to the button state
    pub fn new(label: &'a str, state: &'a ButtonState) -> Self {
        Self {
            label,
            icon: None,
            state,
            style: ButtonStyle::default(),
            focus_id: FocusId::default(),
            alignment: Alignment::Center,
        }
    }

    /// Set an icon to display before the label.
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set the button style.
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the button variant.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.style.variant = variant;
        self
    }

    /// Set the focus ID.
    pub fn focus_id(mut self, id: FocusId) -> Self {
        self.focus_id = id;
        self
    }

    /// Set the text alignment.
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Get the current style based on state.
    fn current_style(&self) -> Style {
        if !self.state.enabled {
            Style::default().fg(self.style.disabled_fg)
        } else if self.state.pressed {
            Style::default()
                .fg(self.style.pressed_fg)
                .bg(self.style.pressed_bg)
        } else if self.style.variant == ButtonVariant::Toggle && self.state.toggled {
            Style::default()
                .fg(self.style.toggled_fg)
                .bg(self.style.toggled_bg)
                .add_modifier(Modifier::BOLD)
        } else if self.state.focused {
            Style::default()
                .fg(self.style.focused_fg)
                .bg(self.style.focused_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(self.style.unfocused_fg)
                .bg(self.style.unfocused_bg)
        }
    }

    /// Build the button text.
    fn build_text(&self) -> String {
        match self.style.variant {
            ButtonVariant::SingleLine | ButtonVariant::Toggle => {
                if let Some(icon) = self.icon {
                    format!(" {} {} ", icon, self.label)
                } else {
                    format!(" {} ", self.label)
                }
            }
            ButtonVariant::Block | ButtonVariant::IconText | ButtonVariant::Minimal => {
                if let Some(icon) = self.icon {
                    format!("{} {}", icon, self.label)
                } else {
                    self.label.to_string()
                }
            }
        }
    }

    /// Calculate minimum width for this button.
    pub fn min_width(&self) -> u16 {
        let text = self.build_text();
        let text_len = text.chars().count() as u16;

        match self.style.variant {
            ButtonVariant::Block => text_len + 4, // Border + padding
            _ => text_len,
        }
    }

    /// Calculate minimum height for this button.
    pub fn min_height(&self) -> u16 {
        match self.style.variant {
            ButtonVariant::Block => 3, // Border top + content + border bottom
            _ => 1,
        }
    }

    /// Render the button and return the click region.
    pub fn render_stateful(self, area: Rect, buf: &mut Buffer) -> ClickRegion<ButtonAction> {
        let click_area = match self.style.variant {
            ButtonVariant::Block => area,
            _ => Rect::new(area.x, area.y, self.min_width().min(area.width), 1),
        };

        self.render(area, buf);

        ClickRegion::new(click_area, ButtonAction::Click)
    }
}

impl Widget for Button<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = self.current_style();
        let text = self.build_text();

        match self.style.variant {
            ButtonVariant::SingleLine | ButtonVariant::Toggle | ButtonVariant::Minimal => {
                let line = Line::from(Span::styled(text, style));
                let paragraph = Paragraph::new(line).alignment(self.alignment);
                paragraph.render(area, buf);
            }

            ButtonVariant::Block => {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(style);

                let inner = block.inner(area);
                block.render(area, buf);

                let paragraph = Paragraph::new(text)
                    .style(style)
                    .alignment(self.alignment);
                paragraph.render(inner, buf);
            }

            ButtonVariant::IconText => {
                let line = Line::from(Span::styled(text, style));
                let paragraph = Paragraph::new(line);
                paragraph.render(area, buf);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_default() {
        let state = ButtonState::default();
        assert!(!state.focused);
        assert!(!state.pressed);
        assert!(state.enabled);
        assert!(!state.toggled);
    }

    #[test]
    fn test_state_enabled() {
        let state = ButtonState::enabled();
        assert!(state.enabled);
        assert!(!state.focused);
    }

    #[test]
    fn test_state_disabled() {
        let state = ButtonState::disabled();
        assert!(!state.enabled);
    }

    #[test]
    fn test_state_toggled() {
        let state = ButtonState::toggled(true);
        assert!(state.toggled);
        assert!(state.enabled);
    }

    #[test]
    fn test_toggle() {
        let mut state = ButtonState::enabled();
        assert!(!state.toggled);

        state.toggle();
        assert!(state.toggled);

        state.toggle();
        assert!(!state.toggled);
    }

    #[test]
    fn test_toggle_disabled() {
        let mut state = ButtonState::disabled();
        state.toggled = false;

        state.toggle();
        assert!(!state.toggled); // Should not change when disabled
    }

    #[test]
    fn test_button_text_single_line() {
        let state = ButtonState::enabled();
        let button = Button::new("Click", &state).variant(ButtonVariant::SingleLine);

        assert_eq!(button.build_text(), " Click ");
    }

    #[test]
    fn test_button_text_with_icon() {
        let state = ButtonState::enabled();
        let button = Button::new("Save", &state).icon("💾");

        assert_eq!(button.build_text(), " 💾 Save ");
    }

    #[test]
    fn test_button_min_width() {
        let state = ButtonState::enabled();

        let button = Button::new("OK", &state).variant(ButtonVariant::SingleLine);
        assert_eq!(button.min_width(), 4); // " OK "

        let button = Button::new("OK", &state).variant(ButtonVariant::Block);
        assert_eq!(button.min_width(), 6); // "OK" + 4 for border
    }

    #[test]
    fn test_button_min_height() {
        let state = ButtonState::enabled();

        let button = Button::new("OK", &state).variant(ButtonVariant::SingleLine);
        assert_eq!(button.min_height(), 1);

        let button = Button::new("OK", &state).variant(ButtonVariant::Block);
        assert_eq!(button.min_height(), 3);
    }

    #[test]
    fn test_render_stateful() {
        let state = ButtonState::enabled();
        let button = Button::new("Test", &state);

        let area = Rect::new(5, 3, 20, 1);
        let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 10));

        let click_region = button.render_stateful(area, &mut buffer);

        assert_eq!(click_region.area.x, 5);
        assert_eq!(click_region.area.y, 3);
        assert_eq!(click_region.data, ButtonAction::Click);
    }

    #[test]
    fn test_style_presets() {
        let primary = ButtonStyle::primary();
        assert_eq!(primary.focused_bg, Color::Blue);

        let danger = ButtonStyle::danger();
        assert_eq!(danger.focused_bg, Color::Red);

        let success = ButtonStyle::success();
        assert_eq!(success.focused_bg, Color::Green);
    }

    #[test]
    fn test_style_builder() {
        let style = ButtonStyle::default()
            .variant(ButtonVariant::Toggle)
            .focused(Color::White, Color::Cyan)
            .toggled(Color::Black, Color::Magenta);

        assert_eq!(style.variant, ButtonVariant::Toggle);
        assert_eq!(style.focused_fg, Color::White);
        assert_eq!(style.focused_bg, Color::Cyan);
        assert_eq!(style.toggled_fg, Color::Black);
        assert_eq!(style.toggled_bg, Color::Magenta);
    }

    #[test]
    fn test_current_style_states() {
        // Disabled state
        let state = ButtonState::disabled();
        let button = Button::new("Test", &state);
        let style = button.current_style();
        assert_eq!(style.fg, Some(button.style.disabled_fg));

        // Focused state
        let mut state = ButtonState::enabled();
        state.focused = true;
        let button = Button::new("Test", &state);
        let style = button.current_style();
        assert_eq!(style.fg, Some(button.style.focused_fg));
        assert_eq!(style.bg, Some(button.style.focused_bg));

        // Toggled state
        let mut state = ButtonState::enabled();
        state.toggled = true;
        let button = Button::new("Test", &state).variant(ButtonVariant::Toggle);
        let style = button.current_style();
        assert_eq!(style.fg, Some(button.style.toggled_fg));
        assert_eq!(style.bg, Some(button.style.toggled_bg));
    }
}
