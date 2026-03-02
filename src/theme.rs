//! Color theme system for consistent styling across all widgets.
//!
//! The theme system provides a centralized [`ColorPalette`] with semantic color roles
//! that every widget style can be derived from. Existing APIs are untouched --
//! `*Style::default()` still works identically.
//!
//! # Quick Start
//!
//! ```rust
//! use ratatui_interact::theme::Theme;
//! use ratatui_interact::components::ButtonStyle;
//!
//! // Use the dark theme (matches existing defaults)
//! let theme = Theme::dark();
//! let button_style: ButtonStyle = theme.style();
//!
//! // Or use the light theme
//! let light = Theme::light();
//! let button_style: ButtonStyle = light.style();
//! ```
//!
//! # Applying to Widgets
//!
//! Every widget with a `.style()` method also has a `.theme()` convenience method:
//!
//! ```rust,ignore
//! let button = Button::new("OK", &state).theme(&theme);
//! let input = Input::new(&input_state).theme(&theme);
//! ```

use ratatui::style::Color;

/// A semantic color palette with ~30 color roles.
///
/// Each color has a specific semantic purpose (e.g., `primary` for focus/selection,
/// `error` for error states) rather than a visual description.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "theme-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColorPalette {
    // Primary interaction
    /// Focus/selection accent color.
    pub primary: Color,
    /// Secondary accent color.
    pub secondary: Color,

    // Text
    /// Main text color.
    pub text: Color,
    /// Secondary text, line numbers.
    pub text_dim: Color,
    /// Disabled/inactive text.
    pub text_disabled: Color,
    /// Placeholder text.
    pub text_placeholder: Color,
    /// Shortcuts/hints.
    pub text_muted: Color,

    // Backgrounds
    /// Main background.
    pub bg: Color,
    /// Elevated surface - menus/popups.
    pub surface: Color,
    /// Bar/header background.
    pub surface_raised: Color,

    // Borders
    /// Focused border.
    pub border_focused: Color,
    /// Default border.
    pub border: Color,
    /// Disabled border.
    pub border_disabled: Color,
    /// Accent border - panels.
    pub border_accent: Color,
    /// Dividers/separators.
    pub separator: Color,

    // Selection/highlight
    /// Highlighted foreground.
    pub highlight_fg: Color,
    /// Highlighted background.
    pub highlight_bg: Color,
    /// Menu highlight foreground.
    pub menu_highlight_fg: Color,
    /// Menu highlight background.
    pub menu_highlight_bg: Color,
    /// Pressed foreground.
    pub pressed_fg: Color,
    /// Pressed background.
    pub pressed_bg: Color,

    // Semantic status
    /// Success color.
    pub success: Color,
    /// Warning color.
    pub warning: Color,
    /// Error: Color,
    pub error: Color,
    /// Info color.
    pub info: Color,

    // Diff
    /// Diff addition foreground.
    pub diff_add_fg: Color,
    /// Diff addition background.
    pub diff_add_bg: Color,
    /// Diff deletion foreground.
    pub diff_del_fg: Color,
    /// Diff deletion background.
    pub diff_del_bg: Color,
}

/// A named theme with a [`ColorPalette`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "theme-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Theme {
    /// Display name for this theme.
    pub name: String,
    /// The color palette.
    pub palette: ColorPalette,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Create the dark theme, matching the existing hardcoded defaults.
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            palette: ColorPalette {
                primary: Color::Yellow,
                secondary: Color::Cyan,

                text: Color::White,
                text_dim: Color::Gray,
                text_disabled: Color::DarkGray,
                text_placeholder: Color::DarkGray,
                text_muted: Color::Rgb(140, 140, 140),

                bg: Color::Reset,
                surface: Color::Rgb(40, 40, 40),
                surface_raised: Color::Rgb(50, 50, 50),

                border_focused: Color::Yellow,
                border: Color::Gray,
                border_disabled: Color::DarkGray,
                border_accent: Color::Cyan,
                separator: Color::Rgb(80, 80, 80),

                highlight_fg: Color::Black,
                highlight_bg: Color::Yellow,
                menu_highlight_fg: Color::White,
                menu_highlight_bg: Color::Rgb(60, 100, 180),
                pressed_fg: Color::Black,
                pressed_bg: Color::White,

                success: Color::Green,
                warning: Color::Yellow,
                error: Color::Red,
                info: Color::Cyan,

                diff_add_fg: Color::Green,
                diff_add_bg: Color::Rgb(0, 40, 0),
                diff_del_fg: Color::Red,
                diff_del_bg: Color::Rgb(40, 0, 0),
            },
        }
    }

    /// Create a light theme suitable for light terminal backgrounds.
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            palette: ColorPalette {
                primary: Color::Blue,
                secondary: Color::Rgb(0, 128, 128),

                text: Color::Rgb(30, 30, 30),
                text_dim: Color::Rgb(100, 100, 100),
                text_disabled: Color::Rgb(160, 160, 160),
                text_placeholder: Color::Rgb(160, 160, 160),
                text_muted: Color::Rgb(100, 100, 100),

                bg: Color::Reset,
                surface: Color::Rgb(250, 250, 250),
                surface_raised: Color::Rgb(240, 240, 240),

                border_focused: Color::Blue,
                border: Color::Rgb(180, 180, 180),
                border_disabled: Color::Rgb(200, 200, 200),
                border_accent: Color::Rgb(0, 128, 128),
                separator: Color::Rgb(200, 200, 200),

                highlight_fg: Color::White,
                highlight_bg: Color::Blue,
                menu_highlight_fg: Color::White,
                menu_highlight_bg: Color::Rgb(0, 120, 215),
                pressed_fg: Color::White,
                pressed_bg: Color::Rgb(30, 30, 30),

                success: Color::Rgb(0, 128, 0),
                warning: Color::Rgb(200, 150, 0),
                error: Color::Rgb(200, 0, 0),
                info: Color::Rgb(0, 128, 128),

                diff_add_fg: Color::Rgb(0, 128, 0),
                diff_add_bg: Color::Rgb(220, 255, 220),
                diff_del_fg: Color::Rgb(200, 0, 0),
                diff_del_bg: Color::Rgb(255, 220, 220),
            },
        }
    }

    /// Convenience method to derive any widget style from this theme.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ratatui_interact::theme::Theme;
    /// use ratatui_interact::components::ButtonStyle;
    ///
    /// let theme = Theme::dark();
    /// let style: ButtonStyle = theme.style();
    /// ```
    pub fn style<S: for<'a> From<&'a Theme>>(&self) -> S {
        S::from(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{ButtonStyle, CheckBoxStyle, InputStyle};

    #[test]
    fn test_dark_theme_matches_button_default() {
        let theme = Theme::dark();
        let themed: ButtonStyle = theme.style();
        let default = ButtonStyle::default();

        assert_eq!(themed.focused_fg, default.focused_fg);
        assert_eq!(themed.focused_bg, default.focused_bg);
        assert_eq!(themed.unfocused_fg, default.unfocused_fg);
        assert_eq!(themed.unfocused_bg, default.unfocused_bg);
        assert_eq!(themed.disabled_fg, default.disabled_fg);
        assert_eq!(themed.pressed_fg, default.pressed_fg);
        assert_eq!(themed.pressed_bg, default.pressed_bg);
        assert_eq!(themed.toggled_fg, default.toggled_fg);
        assert_eq!(themed.toggled_bg, default.toggled_bg);
    }

    #[test]
    fn test_dark_theme_matches_input_default() {
        let theme = Theme::dark();
        let themed: InputStyle = theme.style();
        let default = InputStyle::default();

        assert_eq!(themed.focused_border, default.focused_border);
        assert_eq!(themed.unfocused_border, default.unfocused_border);
        assert_eq!(themed.disabled_border, default.disabled_border);
        assert_eq!(themed.text_fg, default.text_fg);
        assert_eq!(themed.cursor_fg, default.cursor_fg);
        assert_eq!(themed.placeholder_fg, default.placeholder_fg);
    }

    #[test]
    fn test_dark_theme_matches_checkbox_default() {
        let theme = Theme::dark();
        let themed: CheckBoxStyle = theme.style();
        let default = CheckBoxStyle::default();

        assert_eq!(themed.focused_fg, default.focused_fg);
        assert_eq!(themed.unfocused_fg, default.unfocused_fg);
        assert_eq!(themed.disabled_fg, default.disabled_fg);
        assert_eq!(themed.checked_fg, default.checked_fg);
    }

    #[test]
    fn test_light_theme_differs_from_dark() {
        let dark = Theme::dark();
        let light = Theme::light();

        assert_ne!(dark.palette.text, light.palette.text);
        assert_ne!(dark.palette.primary, light.palette.primary);
        assert_ne!(dark.palette.surface, light.palette.surface);
    }

    #[test]
    fn test_theme_default_is_dark() {
        let default = Theme::default();
        let dark = Theme::dark();
        assert_eq!(default.palette, dark.palette);
    }

    #[test]
    fn test_theme_clone_and_eq() {
        let theme = Theme::dark();
        let cloned = theme.clone();
        assert_eq!(theme, cloned);
    }

    #[test]
    fn test_color_palette_clone_and_eq() {
        let palette = Theme::dark().palette;
        let cloned = palette.clone();
        assert_eq!(palette, cloned);
    }

    #[test]
    fn test_style_generic_method() {
        let theme = Theme::dark();
        let _: ButtonStyle = theme.style();
        let _: InputStyle = theme.style();
        let _: CheckBoxStyle = theme.style();
    }

    #[test]
    fn test_light_theme_produces_valid_styles() {
        let theme = Theme::light();
        let btn: ButtonStyle = theme.style();
        let input: InputStyle = theme.style();
        let cb: CheckBoxStyle = theme.style();

        // Light theme should produce different colors than defaults
        let default_btn = ButtonStyle::default();
        assert_ne!(btn.focused_bg, default_btn.focused_bg);

        // But should still have sensible values (not Reset for fg colors)
        assert_ne!(input.text_fg, Color::Reset);
        assert_ne!(cb.focused_fg, Color::Reset);
    }
}
