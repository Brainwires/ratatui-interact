//! Split pane layout component
//!
//! A resizable two-pane layout with drag-to-resize divider support.
//! Supports both horizontal (left/right) and vertical (top/bottom) orientations.
//!
//! # Example
//!
//! ```rust,ignore
//! use ratatui_interact::components::{SplitPane, SplitPaneState, SplitPaneStyle, Orientation};
//! use ratatui::prelude::*;
//!
//! let mut state = SplitPaneState::new(50); // 50% split
//! let split_pane = SplitPane::new()
//!     .orientation(Orientation::Horizontal)
//!     .min_size(10)
//!     .divider_char("│");
//!
//! // In render:
//! split_pane.render_with_content(
//!     area,
//!     buf,
//!     &mut state,
//!     |first_area, buf| { /* render first pane */ },
//!     |second_area, buf| { /* render second pane */ },
//!     &mut registry,
//! );
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

use crate::traits::{ClickRegion, ClickRegionRegistry, FocusId, Focusable};

/// Actions that can be triggered by mouse interaction with the split pane
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplitPaneAction {
    /// Click on the first pane (left or top)
    FirstPaneClick,
    /// Click on the second pane (right or bottom)
    SecondPaneClick,
    /// Click/drag on the divider
    DividerDrag,
}

/// Orientation of the split pane
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    /// Horizontal split (left | right)
    #[default]
    Horizontal,
    /// Vertical split (top / bottom)
    Vertical,
}

/// State for the SplitPane component
#[derive(Debug, Clone)]
pub struct SplitPaneState {
    /// Percentage of the first pane (0-100)
    pub split_percent: u16,
    /// Whether the component is focused
    pub focused: bool,
    /// Whether the divider itself is focused (for keyboard resize)
    pub divider_focused: bool,
    /// Whether currently dragging the divider
    pub is_dragging: bool,
    /// Starting position when drag began
    drag_start_pos: u16,
    /// Split percentage when drag began
    drag_start_percent: u16,
    /// Total size of the split area (cached from last render)
    total_size: u16,
    /// Focus ID for focus management
    pub focus_id: FocusId,
}

impl SplitPaneState {
    /// Create a new SplitPaneState with the given initial split percentage
    pub fn new(split_percent: u16) -> Self {
        Self {
            split_percent: split_percent.clamp(0, 100),
            focused: false,
            divider_focused: false,
            is_dragging: false,
            drag_start_pos: 0,
            drag_start_percent: 0,
            total_size: 0,
            focus_id: FocusId::default(),
        }
    }

    /// Create a new SplitPaneState with 50/50 split
    pub fn half() -> Self {
        Self::new(50)
    }

    /// Start dragging the divider
    pub fn start_drag(&mut self, pos: u16) {
        self.is_dragging = true;
        self.drag_start_pos = pos;
        self.drag_start_percent = self.split_percent;
    }

    /// Update the split position during drag
    pub fn update_drag(&mut self, pos: u16, min_percent: u16, max_percent: u16) {
        if !self.is_dragging || self.total_size == 0 {
            return;
        }

        let delta = (pos as i32) - (self.drag_start_pos as i32);
        let percent_delta = (delta * 100) / (self.total_size as i32);
        let new_percent =
            ((self.drag_start_percent as i32) + percent_delta).clamp(min_percent as i32, max_percent as i32) as u16;

        self.split_percent = new_percent;
    }

    /// End dragging the divider
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
    }

    /// Adjust split percentage by delta (for keyboard control)
    pub fn adjust_split(&mut self, delta: i16, min_percent: u16, max_percent: u16) {
        let new_percent = ((self.split_percent as i16) + delta).clamp(min_percent as i16, max_percent as i16) as u16;
        self.split_percent = new_percent;
    }

    /// Set the split percentage directly
    pub fn set_split_percent(&mut self, percent: u16) {
        self.split_percent = percent.clamp(0, 100);
    }

    /// Get the current split percentage
    pub fn split_percent(&self) -> u16 {
        self.split_percent
    }

    /// Check if currently dragging
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Update total size (called during render or manually)
    pub fn set_total_size(&mut self, size: u16) {
        self.total_size = size;
    }
}

impl Default for SplitPaneState {
    fn default() -> Self {
        Self::half()
    }
}

impl Focusable for SplitPaneState {
    fn focus_id(&self) -> FocusId {
        self.focus_id
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if !focused {
            self.divider_focused = false;
        }
    }

    fn focused_style(&self) -> Style {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    }

    fn unfocused_style(&self) -> Style {
        Style::default().fg(Color::White)
    }
}

/// Style configuration for the SplitPane component
#[derive(Debug, Clone)]
pub struct SplitPaneStyle {
    /// Style for the divider when not focused
    pub divider_style: Style,
    /// Style for the divider when focused
    pub divider_focused_style: Style,
    /// Style for the divider when being dragged
    pub divider_dragging_style: Style,
    /// Style for the divider when hovered
    pub divider_hover_style: Style,
    /// Character used for the divider (vertical orientation: ─, horizontal: │)
    pub divider_char: Option<&'static str>,
    /// Width/height of the divider in cells (default: 1)
    pub divider_size: u16,
    /// Show a grab indicator on the divider
    pub show_grab_indicator: bool,
}

impl Default for SplitPaneStyle {
    fn default() -> Self {
        Self {
            divider_style: Style::default().bg(Color::DarkGray),
            divider_focused_style: Style::default().bg(Color::Yellow).fg(Color::Black),
            divider_dragging_style: Style::default().bg(Color::Cyan).fg(Color::Black),
            divider_hover_style: Style::default().bg(Color::Gray),
            divider_char: None, // Auto-select based on orientation
            divider_size: 1,
            show_grab_indicator: true,
        }
    }
}

impl SplitPaneStyle {
    /// Create a minimal style with thin divider
    pub fn minimal() -> Self {
        Self {
            divider_style: Style::default().fg(Color::DarkGray),
            divider_focused_style: Style::default().fg(Color::Yellow),
            divider_dragging_style: Style::default().fg(Color::Cyan),
            divider_hover_style: Style::default().fg(Color::Gray),
            divider_char: None,
            divider_size: 1,
            show_grab_indicator: false,
        }
    }

    /// Create a style with prominent divider
    pub fn prominent() -> Self {
        Self {
            divider_style: Style::default().bg(Color::Blue).fg(Color::White),
            divider_focused_style: Style::default().bg(Color::Yellow).fg(Color::Black),
            divider_dragging_style: Style::default().bg(Color::Green).fg(Color::Black),
            divider_hover_style: Style::default().bg(Color::LightBlue).fg(Color::Black),
            divider_char: None,
            divider_size: 1,
            show_grab_indicator: true,
        }
    }

    /// Set the divider character
    pub fn divider_char(mut self, char: &'static str) -> Self {
        self.divider_char = Some(char);
        self
    }

    /// Set the divider size
    pub fn divider_size(mut self, size: u16) -> Self {
        self.divider_size = size.max(1);
        self
    }
}

/// A resizable split pane component
pub struct SplitPane {
    orientation: Orientation,
    style: SplitPaneStyle,
    min_size: u16,
    min_percent: u16,
    max_percent: u16,
}

impl SplitPane {
    /// Create a new SplitPane
    pub fn new() -> Self {
        Self {
            orientation: Orientation::default(),
            style: SplitPaneStyle::default(),
            min_size: 5,
            min_percent: 10,
            max_percent: 90,
        }
    }

    /// Set the orientation (horizontal or vertical)
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Set the style
    pub fn style(mut self, style: SplitPaneStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the minimum size for each pane in cells
    pub fn min_size(mut self, min_size: u16) -> Self {
        self.min_size = min_size;
        self
    }

    /// Set the minimum percentage for the first pane
    pub fn min_percent(mut self, min_percent: u16) -> Self {
        self.min_percent = min_percent.clamp(0, 100);
        self
    }

    /// Set the maximum percentage for the first pane
    pub fn max_percent(mut self, max_percent: u16) -> Self {
        self.max_percent = max_percent.clamp(0, 100);
        self
    }

    /// Set the divider character
    pub fn divider_char(mut self, char: &'static str) -> Self {
        self.style.divider_char = Some(char);
        self
    }

    /// Calculate the layout areas for the split pane
    ///
    /// Takes a split_percent (0-100) to determine the first pane size.
    pub fn calculate_areas(&self, area: Rect, split_percent: u16) -> (Rect, Rect, Rect) {
        let total_size = match self.orientation {
            Orientation::Horizontal => area.width,
            Orientation::Vertical => area.height,
        };

        let divider_size = self.style.divider_size;
        let available_size = total_size.saturating_sub(divider_size);

        // Calculate first pane size based on percentage
        let first_size = ((available_size as u32) * (split_percent as u32) / 100) as u16;
        let first_size = first_size.clamp(self.min_size, available_size.saturating_sub(self.min_size));

        // Second pane gets the rest
        let second_size = available_size.saturating_sub(first_size);

        match self.orientation {
            Orientation::Horizontal => {
                let first_area = Rect::new(area.x, area.y, first_size, area.height);
                let divider_area = Rect::new(area.x + first_size, area.y, divider_size, area.height);
                let second_area = Rect::new(
                    area.x + first_size + divider_size,
                    area.y,
                    second_size,
                    area.height,
                );
                (first_area, divider_area, second_area)
            }
            Orientation::Vertical => {
                let first_area = Rect::new(area.x, area.y, area.width, first_size);
                let divider_area = Rect::new(area.x, area.y + first_size, area.width, divider_size);
                let second_area = Rect::new(
                    area.x,
                    area.y + first_size + divider_size,
                    area.width,
                    second_size,
                );
                (first_area, divider_area, second_area)
            }
        }
    }

    /// Render the divider
    fn render_divider(&self, state: &SplitPaneState, divider_area: Rect, buf: &mut Buffer) {
        let divider_style = if state.is_dragging {
            self.style.divider_dragging_style
        } else if state.divider_focused {
            self.style.divider_focused_style
        } else {
            self.style.divider_style
        };

        let divider_char = self.style.divider_char.unwrap_or(match self.orientation {
            Orientation::Horizontal => "│",
            Orientation::Vertical => "─",
        });

        match self.orientation {
            Orientation::Horizontal => {
                for y in divider_area.y..divider_area.y + divider_area.height {
                    for x in divider_area.x..divider_area.x + divider_area.width {
                        // Show grab indicator in the middle
                        let char_to_draw = if self.style.show_grab_indicator {
                            let mid_y = divider_area.y + divider_area.height / 2;
                            if y == mid_y {
                                "┃"
                            } else if y == mid_y.saturating_sub(1) || y == mid_y + 1 {
                                "║"
                            } else {
                                divider_char
                            }
                        } else {
                            divider_char
                        };
                        buf.set_string(x, y, char_to_draw, divider_style);
                    }
                }
            }
            Orientation::Vertical => {
                for y in divider_area.y..divider_area.y + divider_area.height {
                    for x in divider_area.x..divider_area.x + divider_area.width {
                        // Show grab indicator in the middle
                        let char_to_draw = if self.style.show_grab_indicator {
                            let mid_x = divider_area.x + divider_area.width / 2;
                            if x == mid_x {
                                "━"
                            } else if x == mid_x.saturating_sub(1) || x == mid_x + 1 {
                                "═"
                            } else {
                                divider_char
                            }
                        } else {
                            divider_char
                        };
                        buf.set_string(x, y, char_to_draw, divider_style);
                    }
                }
            }
        }
    }

    /// Render the split pane with custom content renderers and click region registry
    pub fn render_with_content<F1, F2>(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut SplitPaneState,
        first_pane_renderer: F1,
        second_pane_renderer: F2,
        registry: &mut ClickRegionRegistry<SplitPaneAction>,
    ) where
        F1: FnOnce(Rect, &mut Buffer),
        F2: FnOnce(Rect, &mut Buffer),
    {
        // Update total size in state for drag calculations
        let total_size = match self.orientation {
            Orientation::Horizontal => area.width,
            Orientation::Vertical => area.height,
        };
        state.set_total_size(total_size);

        let (first_area, divider_area, second_area) = self.calculate_areas(area, state.split_percent);

        // Register click regions
        registry.register(first_area, SplitPaneAction::FirstPaneClick);
        registry.register(divider_area, SplitPaneAction::DividerDrag);
        registry.register(second_area, SplitPaneAction::SecondPaneClick);

        // Render content
        first_pane_renderer(first_area, buf);
        second_pane_renderer(second_area, buf);

        // Render divider on top
        self.render_divider(state, divider_area, buf);
    }

    /// Render just the split pane divider (for cases where content is rendered separately)
    pub fn render_divider_only(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut SplitPaneState,
    ) -> (Rect, Rect, Rect) {
        // Update total size in state
        let total_size = match self.orientation {
            Orientation::Horizontal => area.width,
            Orientation::Vertical => area.height,
        };
        state.set_total_size(total_size);

        let (first_area, divider_area, second_area) = self.calculate_areas(area, state.split_percent);
        self.render_divider(state, divider_area, buf);
        (first_area, divider_area, second_area)
    }

    /// Get a simple click region for the divider
    pub fn divider_click_region(&self, area: Rect, split_percent: u16) -> ClickRegion<SplitPaneAction> {
        let (_, divider_area, _) = self.calculate_areas(area, split_percent);
        ClickRegion::new(divider_area, SplitPaneAction::DividerDrag)
    }

    /// Get the orientation
    pub fn get_orientation(&self) -> Orientation {
        self.orientation
    }

    /// Get min_percent
    pub fn get_min_percent(&self) -> u16 {
        self.min_percent
    }

    /// Get max_percent
    pub fn get_max_percent(&self) -> u16 {
        self.max_percent
    }
}

impl Default for SplitPane {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle keyboard input for split pane
///
/// Returns true if the key was handled
pub fn handle_split_pane_key(
    state: &mut SplitPaneState,
    key: &crossterm::event::KeyEvent,
    orientation: Orientation,
    step: i16,
    min_percent: u16,
    max_percent: u16,
) -> bool {
    use crossterm::event::KeyCode;

    if !state.divider_focused {
        return false;
    }

    match key.code {
        KeyCode::Left if orientation == Orientation::Horizontal => {
            state.adjust_split(-step, min_percent, max_percent);
            true
        }
        KeyCode::Right if orientation == Orientation::Horizontal => {
            state.adjust_split(step, min_percent, max_percent);
            true
        }
        KeyCode::Up if orientation == Orientation::Vertical => {
            state.adjust_split(-step, min_percent, max_percent);
            true
        }
        KeyCode::Down if orientation == Orientation::Vertical => {
            state.adjust_split(step, min_percent, max_percent);
            true
        }
        KeyCode::Home => {
            state.set_split_percent(min_percent);
            true
        }
        KeyCode::End => {
            state.set_split_percent(max_percent);
            true
        }
        _ => false,
    }
}

/// Handle mouse input for split pane
///
/// Returns the action triggered, if any
pub fn handle_split_pane_mouse(
    state: &mut SplitPaneState,
    mouse: &crossterm::event::MouseEvent,
    orientation: Orientation,
    registry: &ClickRegionRegistry<SplitPaneAction>,
    min_percent: u16,
    max_percent: u16,
) -> Option<SplitPaneAction> {
    use crossterm::event::{MouseButton, MouseEventKind};

    let pos = match orientation {
        Orientation::Horizontal => mouse.column,
        Orientation::Vertical => mouse.row,
    };

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(&action) = registry.handle_click(mouse.column, mouse.row) {
                if action == SplitPaneAction::DividerDrag {
                    state.start_drag(pos);
                }
                return Some(action);
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            if state.is_dragging {
                state.end_drag();
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if state.is_dragging {
                state.update_drag(pos, min_percent, max_percent);
            }
        }
        _ => {}
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let state = SplitPaneState::new(30);
        assert_eq!(state.split_percent, 30);
        assert!(!state.is_dragging);
        assert!(!state.focused);
    }

    #[test]
    fn test_state_half() {
        let state = SplitPaneState::half();
        assert_eq!(state.split_percent, 50);
    }

    #[test]
    fn test_split_percent_clamping() {
        let state = SplitPaneState::new(150);
        assert_eq!(state.split_percent, 100);

        let mut state2 = SplitPaneState::new(50);
        state2.set_split_percent(200);
        assert_eq!(state2.split_percent, 100);
    }

    #[test]
    fn test_drag_operations() {
        let mut state = SplitPaneState::new(50);
        state.set_total_size(100);

        state.start_drag(50);
        assert!(state.is_dragging);

        state.update_drag(60, 10, 90);
        assert_eq!(state.split_percent, 60);

        state.end_drag();
        assert!(!state.is_dragging);
    }

    #[test]
    fn test_drag_respects_limits() {
        let mut state = SplitPaneState::new(50);
        state.set_total_size(100);

        state.start_drag(50);
        state.update_drag(5, 10, 90);
        assert!(state.split_percent >= 10);

        state.update_drag(95, 10, 90);
        assert!(state.split_percent <= 90);
    }

    #[test]
    fn test_adjust_split() {
        let mut state = SplitPaneState::new(50);

        state.adjust_split(10, 10, 90);
        assert_eq!(state.split_percent, 60);

        state.adjust_split(-20, 10, 90);
        assert_eq!(state.split_percent, 40);
    }

    #[test]
    fn test_calculate_areas_horizontal() {
        let split_pane = SplitPane::new().orientation(Orientation::Horizontal);

        let area = Rect::new(0, 0, 100, 50);
        let (first, divider, second) = split_pane.calculate_areas(area, 50);

        assert_eq!(first.width + divider.width + second.width, area.width);
        assert_eq!(divider.width, 1);
    }

    #[test]
    fn test_calculate_areas_vertical() {
        let split_pane = SplitPane::new().orientation(Orientation::Vertical);

        let area = Rect::new(0, 0, 100, 50);
        let (first, divider, second) = split_pane.calculate_areas(area, 50);

        assert_eq!(first.height + divider.height + second.height, area.height);
        assert_eq!(divider.height, 1);
    }

    #[test]
    fn test_focusable_trait() {
        let mut state = SplitPaneState::new(50);
        assert!(!state.is_focused());

        state.set_focused(true);
        assert!(state.is_focused());

        state.divider_focused = true;
        state.set_focused(false);
        assert!(!state.divider_focused);
    }
}
