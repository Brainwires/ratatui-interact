//! Mouse Pointer Indicator Widget
//!
//! A visual indicator that displays at the current mouse cursor position.
//! Useful for debugging mouse interactions or providing visual feedback.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{MousePointer, MousePointerState, MousePointerStyle};
//! use ratatui::buffer::Buffer;
//! use ratatui::layout::Rect;
//!
//! // Create state (disabled by default)
//! let mut state = MousePointerState::default();
//!
//! // Enable and update position from mouse event
//! state.set_enabled(true);
//! state.update_position(10, 5);
//!
//! // Create widget with custom style
//! let pointer = MousePointer::new(&state)
//!     .style(MousePointerStyle::crosshair());
//!
//! // Render to buffer (usually called last to be on top)
//! let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
//! pointer.render(&mut buf);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

/// State for the mouse pointer indicator.
///
/// Tracks whether the pointer is enabled and its current position.
#[derive(Debug, Clone, Default)]
pub struct MousePointerState {
    /// Whether the pointer indicator is visible.
    pub enabled: bool,
    /// Current mouse position (column, row). None if not yet set.
    pub position: Option<(u16, u16)>,
}

impl MousePointerState {
    /// Create a new mouse pointer state.
    ///
    /// By default, the pointer is disabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new mouse pointer state with enabled status.
    pub fn with_enabled(enabled: bool) -> Self {
        Self {
            enabled,
            position: None,
        }
    }

    /// Set whether the pointer indicator is enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Toggle the enabled state.
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Update the mouse position.
    pub fn update_position(&mut self, col: u16, row: u16) {
        self.position = Some((col, row));
    }

    /// Clear the stored position.
    pub fn clear_position(&mut self) {
        self.position = None;
    }

    /// Check if the pointer should be rendered.
    ///
    /// Returns true if enabled and position is set.
    pub fn should_render(&self) -> bool {
        self.enabled && self.position.is_some()
    }
}

/// Style configuration for the mouse pointer indicator.
#[derive(Debug, Clone)]
pub struct MousePointerStyle {
    /// Character to display at the pointer position.
    pub symbol: &'static str,
    /// Foreground color.
    pub fg: Color,
    /// Optional background color.
    pub bg: Option<Color>,
}

impl Default for MousePointerStyle {
    fn default() -> Self {
        Self {
            symbol: "█",
            fg: Color::Yellow,
            bg: None,
        }
    }
}

impl MousePointerStyle {
    /// Create a crosshair style pointer.
    pub fn crosshair() -> Self {
        Self {
            symbol: "┼",
            fg: Color::Cyan,
            bg: None,
        }
    }

    /// Create an arrow style pointer.
    pub fn arrow() -> Self {
        Self {
            symbol: "▶",
            fg: Color::White,
            bg: None,
        }
    }

    /// Create a dot style pointer.
    pub fn dot() -> Self {
        Self {
            symbol: "●",
            fg: Color::Green,
            bg: None,
        }
    }

    /// Create a plus style pointer.
    pub fn plus() -> Self {
        Self {
            symbol: "+",
            fg: Color::Magenta,
            bg: None,
        }
    }

    /// Create a custom style pointer.
    pub fn custom(symbol: &'static str, fg: Color) -> Self {
        Self {
            symbol,
            fg,
            bg: None,
        }
    }

    /// Set the symbol.
    pub fn symbol(mut self, symbol: &'static str) -> Self {
        self.symbol = symbol;
        self
    }

    /// Set the foreground color.
    pub fn fg(mut self, fg: Color) -> Self {
        self.fg = fg;
        self
    }

    /// Set the background color.
    pub fn bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }
}

/// A widget that renders a mouse pointer indicator.
///
/// This widget should be rendered last (on top of other widgets) to ensure
/// the pointer is visible above all other content.
#[derive(Debug, Clone)]
pub struct MousePointer<'a> {
    /// Reference to the pointer state.
    state: &'a MousePointerState,
    /// Style configuration.
    style: MousePointerStyle,
}

impl<'a> MousePointer<'a> {
    /// Create a new mouse pointer widget.
    pub fn new(state: &'a MousePointerState) -> Self {
        Self {
            state,
            style: MousePointerStyle::default(),
        }
    }

    /// Set the style.
    pub fn style(mut self, style: MousePointerStyle) -> Self {
        self.style = style;
        self
    }

    /// Render the pointer to the buffer at the stored position.
    ///
    /// This method renders directly to the buffer without area constraints.
    /// The pointer will only render if enabled and position is set.
    pub fn render(self, buf: &mut Buffer) {
        if !self.state.should_render() {
            return;
        }

        let (col, row) = self.state.position.unwrap();
        self.render_at(buf, col, row);
    }

    /// Render the pointer within a constrained area.
    ///
    /// The pointer will only render if it falls within the given area.
    pub fn render_in_area(self, buf: &mut Buffer, area: Rect) {
        if !self.state.should_render() {
            return;
        }

        let (col, row) = self.state.position.unwrap();

        // Check if position is within the area
        if col >= area.x
            && col < area.x + area.width
            && row >= area.y
            && row < area.y + area.height
        {
            self.render_at(buf, col, row);
        }
    }

    /// Internal method to render at a specific position.
    fn render_at(&self, buf: &mut Buffer, col: u16, row: u16) {
        let buf_area = buf.area();

        // Check bounds
        if col >= buf_area.x + buf_area.width || row >= buf_area.y + buf_area.height {
            return;
        }

        // Build style
        let mut cell_style = Style::default().fg(self.style.fg);
        if let Some(bg) = self.style.bg {
            cell_style = cell_style.bg(bg);
        }

        // Set the cell
        buf[(col, row)].set_symbol(self.style.symbol).set_style(cell_style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_default() {
        let state = MousePointerState::default();
        assert!(!state.enabled);
        assert!(state.position.is_none());
        assert!(!state.should_render());
    }

    #[test]
    fn test_state_with_enabled() {
        let state = MousePointerState::with_enabled(true);
        assert!(state.enabled);
        assert!(state.position.is_none());
        assert!(!state.should_render()); // No position yet
    }

    #[test]
    fn test_state_toggle() {
        let mut state = MousePointerState::default();
        assert!(!state.enabled);

        state.toggle();
        assert!(state.enabled);

        state.toggle();
        assert!(!state.enabled);
    }

    #[test]
    fn test_state_position_update() {
        let mut state = MousePointerState::default();
        state.set_enabled(true);

        assert!(state.position.is_none());

        state.update_position(10, 5);
        assert_eq!(state.position, Some((10, 5)));
        assert!(state.should_render());

        state.clear_position();
        assert!(state.position.is_none());
        assert!(!state.should_render());
    }

    #[test]
    fn test_should_render() {
        let mut state = MousePointerState::default();

        // Not enabled, no position
        assert!(!state.should_render());

        // Enabled, no position
        state.set_enabled(true);
        assert!(!state.should_render());

        // Enabled, has position
        state.update_position(5, 5);
        assert!(state.should_render());

        // Not enabled, has position
        state.set_enabled(false);
        assert!(!state.should_render());
    }

    #[test]
    fn test_style_default() {
        let style = MousePointerStyle::default();
        assert_eq!(style.symbol, "█");
        assert_eq!(style.fg, Color::Yellow);
        assert!(style.bg.is_none());
    }

    #[test]
    fn test_style_presets() {
        let crosshair = MousePointerStyle::crosshair();
        assert_eq!(crosshair.symbol, "┼");
        assert_eq!(crosshair.fg, Color::Cyan);

        let arrow = MousePointerStyle::arrow();
        assert_eq!(arrow.symbol, "▶");
        assert_eq!(arrow.fg, Color::White);

        let dot = MousePointerStyle::dot();
        assert_eq!(dot.symbol, "●");
        assert_eq!(dot.fg, Color::Green);

        let plus = MousePointerStyle::plus();
        assert_eq!(plus.symbol, "+");
        assert_eq!(plus.fg, Color::Magenta);
    }

    #[test]
    fn test_style_custom() {
        let custom = MousePointerStyle::custom("X", Color::Red);
        assert_eq!(custom.symbol, "X");
        assert_eq!(custom.fg, Color::Red);
    }

    #[test]
    fn test_style_builder() {
        let style = MousePointerStyle::default()
            .symbol("*")
            .fg(Color::Blue)
            .bg(Color::White);

        assert_eq!(style.symbol, "*");
        assert_eq!(style.fg, Color::Blue);
        assert_eq!(style.bg, Some(Color::White));
    }

    #[test]
    fn test_render_disabled() {
        let state = MousePointerState::default();
        let pointer = MousePointer::new(&state);

        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 10));
        pointer.render(&mut buf);

        // Buffer should be unchanged (all cells should be default)
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(buf[(x, y)].symbol(), " ");
            }
        }
    }

    #[test]
    fn test_render_enabled() {
        let mut state = MousePointerState::default();
        state.set_enabled(true);
        state.update_position(5, 5);

        let pointer = MousePointer::new(&state);

        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 10));
        pointer.render(&mut buf);

        // Check that the pointer was rendered at position (5, 5)
        assert_eq!(buf[(5, 5)].symbol(), "█");
    }

    #[test]
    fn test_render_with_custom_style() {
        let mut state = MousePointerState::default();
        state.set_enabled(true);
        state.update_position(3, 3);

        let pointer = MousePointer::new(&state).style(MousePointerStyle::crosshair());

        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 10));
        pointer.render(&mut buf);

        assert_eq!(buf[(3, 3)].symbol(), "┼");
    }

    #[test]
    fn test_render_out_of_bounds() {
        let mut state = MousePointerState::default();
        state.set_enabled(true);
        state.update_position(100, 100); // Way outside the buffer

        let pointer = MousePointer::new(&state);

        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 10));
        pointer.render(&mut buf);

        // Buffer should be unchanged - no panic
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(buf[(x, y)].symbol(), " ");
            }
        }
    }

    #[test]
    fn test_render_in_area_inside() {
        let mut state = MousePointerState::default();
        state.set_enabled(true);
        state.update_position(5, 5);

        let pointer = MousePointer::new(&state);
        let area = Rect::new(0, 0, 10, 10);

        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 20));
        pointer.render_in_area(&mut buf, area);

        assert_eq!(buf[(5, 5)].symbol(), "█");
    }

    #[test]
    fn test_render_in_area_outside() {
        let mut state = MousePointerState::default();
        state.set_enabled(true);
        state.update_position(15, 15); // Outside the constrained area

        let pointer = MousePointer::new(&state);
        let area = Rect::new(0, 0, 10, 10);

        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 20));
        pointer.render_in_area(&mut buf, area);

        // Position is outside the area, so nothing should be rendered
        assert_eq!(buf[(15, 15)].symbol(), " ");
    }

    #[test]
    fn test_render_at_boundary() {
        let mut state = MousePointerState::default();
        state.set_enabled(true);
        state.update_position(9, 9); // At the edge

        let pointer = MousePointer::new(&state);

        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 10));
        pointer.render(&mut buf);

        assert_eq!(buf[(9, 9)].symbol(), "█");
    }
}
