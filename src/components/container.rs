//! Container component - Popup dialogs with focus management
//!
//! Provides a generic popup dialog container that manages child components,
//! handles Tab navigation, and supports mouse click interactions.
//!
//! # Example
//!
//! ```rust,ignore
//! use tui_extension::components::{DialogConfig, DialogState, PopupDialog};
//! use tui_extension::traits::ContainerAction;
//!
//! // Create dialog configuration
//! let config = DialogConfig::new("Settings")
//!     .width_percent(50)
//!     .height_percent(40)
//!     .buttons(vec![
//!         ("Cancel".to_string(), ContainerAction::Close),
//!         ("Save".to_string(), ContainerAction::Submit),
//!     ]);
//!
//! // Create dialog state
//! let mut state = DialogState::new(MyContent::default());
//! state.show();
//!
//! // Render in your draw function
//! let mut dialog = PopupDialog::new(&config, &mut state, |frame, area, content| {
//!     // Render your content here
//! });
//! dialog.render(frame);
//! ```

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::{
    state::FocusManager,
    traits::{ClickRegionRegistry, ContainerAction, EventResult},
};

/// Focus targets within a dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogFocusTarget {
    /// A child component by index.
    Child(usize),
    /// A dialog button by index.
    Button(usize),
    /// The close button (if present).
    Close,
}

/// State for a dialog.
#[derive(Debug, Clone)]
pub struct DialogState<T> {
    /// Child component state.
    pub children: T,
    /// Focus manager for Tab navigation.
    pub focus: FocusManager<DialogFocusTarget>,
    /// Click regions registry.
    pub click_regions: ClickRegionRegistry<DialogFocusTarget>,
    /// Whether the dialog is visible.
    pub visible: bool,
}

impl<T: Default> Default for DialogState<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> DialogState<T> {
    /// Create a new dialog state.
    pub fn new(children: T) -> Self {
        Self {
            children,
            focus: FocusManager::new(),
            click_regions: ClickRegionRegistry::new(),
            visible: false,
        }
    }

    /// Show the dialog.
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the dialog.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Toggle dialog visibility.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Check if dialog is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Register a child for focus navigation.
    pub fn register_child(&mut self, index: usize) {
        self.focus.register(DialogFocusTarget::Child(index));
    }

    /// Register a button for focus navigation.
    pub fn register_button(&mut self, index: usize) {
        self.focus.register(DialogFocusTarget::Button(index));
    }

    /// Get the currently focused target.
    pub fn current_focus(&self) -> Option<&DialogFocusTarget> {
        self.focus.current()
    }

    /// Check if a child is focused.
    pub fn is_child_focused(&self, index: usize) -> bool {
        self.focus.is_focused(&DialogFocusTarget::Child(index))
    }

    /// Check if a button is focused.
    pub fn is_button_focused(&self, index: usize) -> bool {
        self.focus.is_focused(&DialogFocusTarget::Button(index))
    }
}

/// Configuration for a popup dialog.
#[derive(Debug, Clone)]
pub struct DialogConfig {
    /// Dialog title.
    pub title: String,
    /// Width as percentage of screen (0-100).
    pub width_percent: u16,
    /// Height as percentage of screen (0-100).
    pub height_percent: u16,
    /// Minimum width in columns.
    pub min_width: u16,
    /// Minimum height in rows.
    pub min_height: u16,
    /// Maximum width in columns.
    pub max_width: u16,
    /// Maximum height in rows.
    pub max_height: u16,
    /// Border color.
    pub border_color: Color,
    /// Border color when focused.
    pub focused_border_color: Color,
    /// Close dialog on Escape.
    pub close_on_escape: bool,
    /// Close dialog when clicking outside.
    pub close_on_outside_click: bool,
    /// Dialog buttons (label, action).
    pub buttons: Vec<(String, ContainerAction)>,
}

impl Default for DialogConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            width_percent: 60,
            height_percent: 50,
            min_width: 40,
            min_height: 10,
            max_width: 120,
            max_height: 40,
            border_color: Color::Blue,
            focused_border_color: Color::Cyan,
            close_on_escape: true,
            close_on_outside_click: true,
            buttons: vec![
                ("Cancel".to_string(), ContainerAction::Close),
                ("OK".to_string(), ContainerAction::Submit),
            ],
        }
    }
}

impl DialogConfig {
    /// Create a new dialog configuration with title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set the width percentage.
    pub fn width_percent(mut self, percent: u16) -> Self {
        self.width_percent = percent.min(100);
        self
    }

    /// Set the height percentage.
    pub fn height_percent(mut self, percent: u16) -> Self {
        self.height_percent = percent.min(100);
        self
    }

    /// Set minimum dimensions.
    pub fn min_size(mut self, width: u16, height: u16) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }

    /// Set maximum dimensions.
    pub fn max_size(mut self, width: u16, height: u16) -> Self {
        self.max_width = width;
        self.max_height = height;
        self
    }

    /// Set the border color.
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Set the focused border color.
    pub fn focused_border_color(mut self, color: Color) -> Self {
        self.focused_border_color = color;
        self
    }

    /// Set close on escape behavior.
    pub fn close_on_escape(mut self, close: bool) -> Self {
        self.close_on_escape = close;
        self
    }

    /// Set close on outside click behavior.
    pub fn close_on_outside_click(mut self, close: bool) -> Self {
        self.close_on_outside_click = close;
        self
    }

    /// Set dialog buttons.
    pub fn buttons(mut self, buttons: Vec<(String, ContainerAction)>) -> Self {
        self.buttons = buttons;
        self
    }

    /// Add a single button.
    pub fn add_button(mut self, label: impl Into<String>, action: ContainerAction) -> Self {
        self.buttons.push((label.into(), action));
        self
    }

    /// Clear all buttons.
    pub fn no_buttons(mut self) -> Self {
        self.buttons.clear();
        self
    }

    /// Set only OK button.
    pub fn ok_only(mut self) -> Self {
        self.buttons = vec![("OK".to_string(), ContainerAction::Close)];
        self
    }

    /// Set OK and Cancel buttons.
    pub fn ok_cancel(mut self) -> Self {
        self.buttons = vec![
            ("Cancel".to_string(), ContainerAction::Close),
            ("OK".to_string(), ContainerAction::Submit),
        ];
        self
    }

    /// Set Yes and No buttons.
    pub fn yes_no(mut self) -> Self {
        self.buttons = vec![
            ("No".to_string(), ContainerAction::Close),
            ("Yes".to_string(), ContainerAction::Submit),
        ];
        self
    }
}

/// Generic popup dialog container.
///
/// Manages rendering, focus, and event handling for a popup dialog.
pub struct PopupDialog<'a, T, F>
where
    F: FnMut(&mut Frame, Rect, &mut T),
{
    config: &'a DialogConfig,
    state: &'a mut DialogState<T>,
    content_renderer: F,
}

impl<'a, T, F> PopupDialog<'a, T, F>
where
    F: FnMut(&mut Frame, Rect, &mut T),
{
    /// Create a new popup dialog.
    ///
    /// # Arguments
    ///
    /// * `config` - Dialog configuration
    /// * `state` - Dialog state
    /// * `content_renderer` - Closure to render dialog content
    pub fn new(config: &'a DialogConfig, state: &'a mut DialogState<T>, content_renderer: F) -> Self {
        Self {
            config,
            state,
            content_renderer,
        }
    }

    /// Calculate dialog area centered on screen.
    pub fn calculate_area(&self, screen: Rect) -> Rect {
        let width = (screen.width * self.config.width_percent / 100)
            .max(self.config.min_width)
            .min(self.config.max_width)
            .min(screen.width.saturating_sub(4));

        let height = (screen.height * self.config.height_percent / 100)
            .max(self.config.min_height)
            .min(self.config.max_height)
            .min(screen.height.saturating_sub(4));

        let x = (screen.width.saturating_sub(width)) / 2;
        let y = (screen.height.saturating_sub(height)) / 2;

        Rect::new(x, y, width, height)
    }

    /// Render the popup.
    pub fn render(&mut self, frame: &mut Frame) {
        if !self.state.visible {
            return;
        }

        let screen = frame.area();
        let area = self.calculate_area(screen);

        // Clear click regions before rendering
        self.state.click_regions.clear();

        // Clear area behind popup
        frame.render_widget(Clear, area);

        // Render border and title
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.config.focused_border_color))
            .title(format!(" {} ", self.config.title))
            .title_alignment(Alignment::Center);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split inner area for content and buttons
        let button_height = if self.config.buttons.is_empty() { 0 } else { 2 };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(button_height),
            ])
            .split(inner);

        // Render content
        (self.content_renderer)(frame, chunks[0], &mut self.state.children);

        // Render buttons
        if !self.config.buttons.is_empty() {
            self.render_buttons(frame, chunks[1]);
        }
    }

    fn render_buttons(&mut self, frame: &mut Frame, area: Rect) {
        let button_count = self.config.buttons.len();
        if button_count == 0 {
            return;
        }

        let total_button_width: u16 = self.config
            .buttons
            .iter()
            .map(|(label, _)| label.len() as u16 + 4)
            .sum::<u16>()
            + (button_count as u16).saturating_sub(1) * 2;

        let start_x = area.x + (area.width.saturating_sub(total_button_width)) / 2;
        let mut x = start_x;

        for (idx, (label, _action)) in self.config.buttons.iter().enumerate() {
            let is_focused = self.state.is_button_focused(idx);
            let btn_width = label.len() as u16 + 4;
            let btn_area = Rect::new(x, area.y, btn_width, 1);

            let style = if is_focused {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            };

            let button_text = format!(" {} ", label);
            let paragraph = Paragraph::new(Span::styled(button_text, style));
            frame.render_widget(paragraph, btn_area);

            // Register click region
            self.state
                .click_regions
                .register(btn_area, DialogFocusTarget::Button(idx));

            x += btn_width + 2;
        }
    }

    /// Handle keyboard event.
    pub fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        if !self.state.visible {
            return EventResult::NotHandled;
        }

        match key.code {
            KeyCode::Esc if self.config.close_on_escape => {
                self.state.hide();
                EventResult::Action(ContainerAction::Close)
            }
            KeyCode::Tab if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.state.focus.next();
                EventResult::Consumed
            }
            KeyCode::BackTab => {
                self.state.focus.prev();
                EventResult::Consumed
            }
            KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.state.focus.prev();
                EventResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(DialogFocusTarget::Button(idx)) = self.state.focus.current() {
                    if let Some((_, action)) = self.config.buttons.get(*idx) {
                        let action = action.clone();
                        if action.is_close() {
                            self.state.hide();
                        }
                        return EventResult::Action(action);
                    }
                }
                EventResult::NotHandled
            }
            _ => EventResult::NotHandled,
        }
    }

    /// Handle mouse event.
    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> EventResult {
        if !self.state.visible {
            return EventResult::NotHandled;
        }

        let screen = Rect::new(0, 0, 80, 24); // Will be passed from actual screen
        let area = self.calculate_area(screen);

        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            let col = mouse.column;
            let row = mouse.row;

            // Check if click is outside dialog
            if self.config.close_on_outside_click {
                if col < area.x
                    || col >= area.x + area.width
                    || row < area.y
                    || row >= area.y + area.height
                {
                    self.state.hide();
                    return EventResult::Action(ContainerAction::Close);
                }
            }

            // Check click regions
            if let Some(target) = self.state.click_regions.handle_click(col, row) {
                match target {
                    DialogFocusTarget::Button(idx) => {
                        if let Some((_, action)) = self.config.buttons.get(*idx) {
                            let action = action.clone();
                            if action.is_close() {
                                self.state.hide();
                            }
                            return EventResult::Action(action);
                        }
                    }
                    DialogFocusTarget::Child(idx) => {
                        self.state.focus.set(DialogFocusTarget::Child(*idx));
                        return EventResult::Consumed;
                    }
                    DialogFocusTarget::Close => {
                        self.state.hide();
                        return EventResult::Action(ContainerAction::Close);
                    }
                }
            }
        }

        EventResult::NotHandled
    }

    /// Handle mouse event with screen dimensions.
    pub fn handle_mouse_with_screen(&mut self, mouse: MouseEvent, screen: Rect) -> EventResult {
        if !self.state.visible {
            return EventResult::NotHandled;
        }

        let area = self.calculate_area(screen);

        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            let col = mouse.column;
            let row = mouse.row;

            // Check if click is outside dialog
            if self.config.close_on_outside_click {
                if col < area.x
                    || col >= area.x + area.width
                    || row < area.y
                    || row >= area.y + area.height
                {
                    self.state.hide();
                    return EventResult::Action(ContainerAction::Close);
                }
            }

            // Check click regions (same as handle_mouse)
            if let Some(target) = self.state.click_regions.handle_click(col, row) {
                match target {
                    DialogFocusTarget::Button(idx) => {
                        if let Some((_, action)) = self.config.buttons.get(*idx) {
                            let action = action.clone();
                            if action.is_close() {
                                self.state.hide();
                            }
                            return EventResult::Action(action);
                        }
                    }
                    DialogFocusTarget::Child(idx) => {
                        self.state.focus.set(DialogFocusTarget::Child(*idx));
                        return EventResult::Consumed;
                    }
                    DialogFocusTarget::Close => {
                        self.state.hide();
                        return EventResult::Action(ContainerAction::Close);
                    }
                }
            }
        }

        EventResult::NotHandled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_state_default() {
        let state: DialogState<()> = DialogState::default();
        assert!(!state.visible);
        assert!(state.focus.is_empty());
    }

    #[test]
    fn test_dialog_state_visibility() {
        let mut state: DialogState<()> = DialogState::new(());

        assert!(!state.is_visible());

        state.show();
        assert!(state.is_visible());

        state.hide();
        assert!(!state.is_visible());

        state.toggle();
        assert!(state.is_visible());

        state.toggle();
        assert!(!state.is_visible());
    }

    #[test]
    fn test_dialog_state_focus_registration() {
        let mut state: DialogState<()> = DialogState::new(());

        state.register_child(0);
        state.register_child(1);
        state.register_button(0);
        state.register_button(1);

        assert!(state.is_child_focused(0)); // First registered is focused

        state.focus.next();
        assert!(state.is_child_focused(1));

        state.focus.next();
        assert!(state.is_button_focused(0));
    }

    #[test]
    fn test_dialog_config_default() {
        let config = DialogConfig::default();
        assert_eq!(config.width_percent, 60);
        assert_eq!(config.height_percent, 50);
        assert!(config.close_on_escape);
        assert!(config.close_on_outside_click);
        assert_eq!(config.buttons.len(), 2);
    }

    #[test]
    fn test_dialog_config_builder() {
        let config = DialogConfig::new("Test Dialog")
            .width_percent(80)
            .height_percent(60)
            .close_on_escape(false)
            .close_on_outside_click(false);

        assert_eq!(config.title, "Test Dialog");
        assert_eq!(config.width_percent, 80);
        assert_eq!(config.height_percent, 60);
        assert!(!config.close_on_escape);
        assert!(!config.close_on_outside_click);
    }

    #[test]
    fn test_dialog_config_buttons() {
        let config = DialogConfig::new("Test").ok_only();
        assert_eq!(config.buttons.len(), 1);
        assert_eq!(config.buttons[0].0, "OK");

        let config = DialogConfig::new("Test").ok_cancel();
        assert_eq!(config.buttons.len(), 2);

        let config = DialogConfig::new("Test").yes_no();
        assert_eq!(config.buttons.len(), 2);
        assert_eq!(config.buttons[0].0, "No");
        assert_eq!(config.buttons[1].0, "Yes");

        let config = DialogConfig::new("Test").no_buttons();
        assert!(config.buttons.is_empty());
    }

    #[test]
    fn test_dialog_config_custom_buttons() {
        let config = DialogConfig::new("Test")
            .no_buttons()
            .add_button("Apply", ContainerAction::custom("apply"))
            .add_button("Close", ContainerAction::Close);

        assert_eq!(config.buttons.len(), 2);
        assert_eq!(config.buttons[0].0, "Apply");
        assert_eq!(config.buttons[1].1, ContainerAction::Close);
    }

    #[test]
    fn test_calculate_area() {
        let config = DialogConfig::new("Test")
            .width_percent(50)
            .height_percent(50);
        let mut state: DialogState<()> = DialogState::new(());

        let dialog = PopupDialog::new(&config, &mut state, |_, _, _| {});

        let screen = Rect::new(0, 0, 100, 50);
        let area = dialog.calculate_area(screen);

        assert_eq!(area.width, 50); // 50% of 100
        assert_eq!(area.height, 25); // 50% of 50
        assert_eq!(area.x, 25); // Centered: (100 - 50) / 2
        assert_eq!(area.y, 12); // Centered: (50 - 25) / 2
    }

    #[test]
    fn test_calculate_area_constrained() {
        let config = DialogConfig::new("Test")
            .width_percent(100)
            .height_percent(100)
            .max_size(60, 30);
        let mut state: DialogState<()> = DialogState::new(());

        let dialog = PopupDialog::new(&config, &mut state, |_, _, _| {});

        let screen = Rect::new(0, 0, 100, 50);
        let area = dialog.calculate_area(screen);

        // Should be constrained to max size
        assert_eq!(area.width, 60);
        assert_eq!(area.height, 30);
    }

    #[test]
    fn test_dialog_focus_target_equality() {
        assert_eq!(DialogFocusTarget::Child(0), DialogFocusTarget::Child(0));
        assert_ne!(DialogFocusTarget::Child(0), DialogFocusTarget::Child(1));
        assert_ne!(DialogFocusTarget::Child(0), DialogFocusTarget::Button(0));
        assert_eq!(DialogFocusTarget::Close, DialogFocusTarget::Close);
    }
}
