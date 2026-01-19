//! Style configuration for the hotkey dialog.
//!
//! This module provides customizable styling options for the hotkey dialog component.

use ratatui::style::{Color, Modifier, Style};

/// Style configuration for the hotkey dialog.
#[derive(Debug, Clone)]
pub struct HotkeyDialogStyle {
    /// Dialog title
    pub title: String,
    /// Border color when focused
    pub border_focused: Color,
    /// Border color when not focused
    pub border_unfocused: Color,
    /// Title color
    pub title_color: Color,
    /// Color for global hotkeys
    pub global_key_color: Color,
    /// Color for non-global hotkeys
    pub local_key_color: Color,
    /// Background color for selected items
    pub selected_bg: Color,
    /// Text color for selected items
    pub selected_fg: Color,
    /// Color for locked/non-customizable indicator
    pub locked_color: Color,
    /// Color for the cursor in search field
    pub cursor_color: Color,
    /// Placeholder text color
    pub placeholder_color: Color,
    /// Default text color
    pub text_color: Color,
    /// Dimmed text color
    pub dim_color: Color,
    /// Width percentage (0-100)
    pub width_percent: u16,
    /// Height percentage (0-100)
    pub height_percent: u16,
    /// Maximum width in columns
    pub max_width: u16,
    /// Maximum height in rows
    pub max_height: u16,
    /// Minimum width in columns
    pub min_width: u16,
    /// Minimum height in rows
    pub min_height: u16,
    /// Category list width percentage (0-100)
    pub category_width_percent: u16,
    /// Search bar height
    pub search_height: u16,
    /// Footer height
    pub footer_height: u16,
    /// Global context indicator (e.g., "[G]")
    pub global_indicator: String,
    /// Locked/non-customizable indicator (e.g., "L")
    pub locked_indicator: String,
    /// Search placeholder text
    pub search_placeholder: String,
}

impl Default for HotkeyDialogStyle {
    fn default() -> Self {
        Self {
            title: " Hotkey Configuration ".to_string(),
            border_focused: Color::Yellow,
            border_unfocused: Color::DarkGray,
            title_color: Color::Yellow,
            global_key_color: Color::Yellow,
            local_key_color: Color::Cyan,
            selected_bg: Color::Yellow,
            selected_fg: Color::Black,
            locked_color: Color::Red,
            cursor_color: Color::Yellow,
            placeholder_color: Color::DarkGray,
            text_color: Color::White,
            dim_color: Color::DarkGray,
            width_percent: 85,
            height_percent: 85,
            max_width: 110,
            max_height: 45,
            min_width: 70,
            min_height: 25,
            category_width_percent: 28,
            search_height: 3,
            footer_height: 2,
            global_indicator: "[G]".to_string(),
            locked_indicator: "L".to_string(),
            search_placeholder: "Type to filter hotkeys...".to_string(),
        }
    }
}

impl HotkeyDialogStyle {
    /// Create a new style with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the dialog title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the border color when focused.
    pub fn border_focused(mut self, color: Color) -> Self {
        self.border_focused = color;
        self
    }

    /// Set the border color when not focused.
    pub fn border_unfocused(mut self, color: Color) -> Self {
        self.border_unfocused = color;
        self
    }

    /// Set the size constraints.
    pub fn size(
        mut self,
        width_percent: u16,
        height_percent: u16,
        max_width: u16,
        max_height: u16,
    ) -> Self {
        self.width_percent = width_percent;
        self.height_percent = height_percent;
        self.max_width = max_width;
        self.max_height = max_height;
        self
    }

    /// Set minimum size constraints.
    pub fn min_size(mut self, min_width: u16, min_height: u16) -> Self {
        self.min_width = min_width;
        self.min_height = min_height;
        self
    }

    /// Set the search placeholder text.
    pub fn search_placeholder(mut self, text: impl Into<String>) -> Self {
        self.search_placeholder = text.into();
        self
    }

    /// Get the style for a focused border.
    pub fn focused_border_style(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    /// Get the style for an unfocused border.
    pub fn unfocused_border_style(&self) -> Style {
        Style::default().fg(self.border_unfocused)
    }

    /// Get the style for the title.
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.title_color)
            .add_modifier(Modifier::BOLD)
    }

    /// Get the style for selected items.
    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.selected_fg)
            .bg(self.selected_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get the style for selected items without bold.
    pub fn selected_text_style(&self) -> Style {
        Style::default().fg(self.selected_fg).bg(self.selected_bg)
    }

    /// Get the style for global hotkeys.
    pub fn global_key_style(&self) -> Style {
        Style::default()
            .fg(self.global_key_color)
            .add_modifier(Modifier::BOLD)
    }

    /// Get the style for local hotkeys.
    pub fn local_key_style(&self) -> Style {
        Style::default()
            .fg(self.local_key_color)
            .add_modifier(Modifier::BOLD)
    }

    /// Get the style for locked indicators.
    pub fn locked_style(&self) -> Style {
        Style::default().fg(self.locked_color)
    }

    /// Get the style for normal text.
    pub fn text_style(&self) -> Style {
        Style::default().fg(self.text_color)
    }

    /// Get the style for dimmed/secondary text.
    pub fn dim_style(&self) -> Style {
        Style::default().fg(self.dim_color)
    }

    /// Get the style for placeholder text.
    pub fn placeholder_style(&self) -> Style {
        Style::default().fg(self.placeholder_color)
    }

    /// Get the style for the cursor.
    pub fn cursor_style(&self) -> Style {
        Style::default().fg(self.cursor_color)
    }

    /// Calculate the modal area dimensions.
    pub fn calculate_modal_area(&self, screen_width: u16, screen_height: u16) -> (u16, u16, u16, u16) {
        let modal_width = (screen_width * self.width_percent / 100)
            .min(self.max_width)
            .max(self.min_width)
            .min(screen_width.saturating_sub(4));

        let modal_height = (screen_height * self.height_percent / 100)
            .min(self.max_height)
            .max(self.min_height)
            .min(screen_height.saturating_sub(4));

        let x = (screen_width.saturating_sub(modal_width)) / 2;
        let y = (screen_height.saturating_sub(modal_height)) / 2;

        (x, y, modal_width, modal_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_style() {
        let style = HotkeyDialogStyle::default();
        assert_eq!(style.width_percent, 85);
        assert_eq!(style.height_percent, 85);
        assert_eq!(style.border_focused, Color::Yellow);
    }

    #[test]
    fn test_builder_pattern() {
        let style = HotkeyDialogStyle::new()
            .title("My Hotkeys")
            .border_focused(Color::Cyan)
            .size(80, 80, 100, 40);

        assert_eq!(style.title, "My Hotkeys");
        assert_eq!(style.border_focused, Color::Cyan);
        assert_eq!(style.width_percent, 80);
    }

    #[test]
    fn test_calculate_modal_area() {
        let style = HotkeyDialogStyle::default();
        let (x, y, w, h) = style.calculate_modal_area(120, 40);

        // 85% of 120 = 102, capped at max_width 110, so 102
        assert!(w <= 110);
        assert!(w >= 70);
        // Should be centered
        assert_eq!(x, (120 - w) / 2);
    }
}
