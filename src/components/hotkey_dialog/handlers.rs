//! Event handlers for the hotkey dialog.
//!
//! This module provides keyboard and mouse event handling functions
//! that can be used with any application's event system.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::state::{HotkeyDialogState, HotkeyFocus};
use super::traits::HotkeyCategory;

/// Result of handling a hotkey dialog event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotkeyDialogAction {
    /// No action needed
    None,
    /// Close the dialog
    Close,
    /// Entry was selected (for potential future features like rebinding)
    EntrySelected {
        /// The key combination of the selected entry
        key_combination: String,
        /// The action description
        action: String,
        /// The context string
        context: String,
    },
    /// Scroll up by the given amount
    ScrollUp(usize),
    /// Scroll down by the given amount
    ScrollDown(usize),
}

/// Handle a keyboard event for the hotkey dialog.
///
/// Returns a `HotkeyDialogAction` indicating what action should be taken.
///
/// # Example
///
/// ```rust,ignore
/// use crossterm::event::KeyEvent;
/// use ratatui_interact::components::hotkey_dialog::{
///     HotkeyDialogState, HotkeyDialogAction, handle_hotkey_dialog_key
/// };
///
/// let mut state = HotkeyDialogState::<MyCategory>::new();
/// let action = handle_hotkey_dialog_key(&mut state, key_event);
///
/// match action {
///     HotkeyDialogAction::Close => { /* close dialog */ }
///     HotkeyDialogAction::EntrySelected { key_combination, .. } => {
///         println!("Selected: {}", key_combination);
///     }
///     _ => {}
/// }
/// ```
pub fn handle_hotkey_dialog_key<C: HotkeyCategory>(
    state: &mut HotkeyDialogState<C>,
    key: KeyEvent,
) -> HotkeyDialogAction {
    // Escape closes the dialog
    if key.code == KeyCode::Esc {
        // If in search input with text, first clear search, then close
        if state.focus == HotkeyFocus::SearchInput && !state.search_query.is_empty() {
            state.clear_search();
            return HotkeyDialogAction::None;
        }
        return HotkeyDialogAction::Close;
    }

    // Tab cycles through focus areas
    if key.code == KeyCode::Tab {
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            state.focus_prev();
        } else {
            state.focus_next();
        }
        return HotkeyDialogAction::None;
    }

    // BackTab (Shift+Tab) cycles backward
    if key.code == KeyCode::BackTab {
        state.focus_prev();
        return HotkeyDialogAction::None;
    }

    // Handle focus-specific keys
    match state.focus {
        HotkeyFocus::SearchInput => handle_search_input_key(state, key),
        HotkeyFocus::CategoryList => handle_category_list_key(state, key),
        HotkeyFocus::HotkeyList => handle_hotkey_list_key(state, key),
    }
}

/// Handle keyboard input for the search field.
fn handle_search_input_key<C: HotkeyCategory>(
    state: &mut HotkeyDialogState<C>,
    key: KeyEvent,
) -> HotkeyDialogAction {
    match key.code {
        KeyCode::Char(c) => {
            state.insert_char(c);
        }
        KeyCode::Backspace => {
            state.delete_char_backward();
        }
        KeyCode::Delete => {
            state.delete_char_forward();
        }
        KeyCode::Left => {
            state.move_cursor_left();
        }
        KeyCode::Right => {
            state.move_cursor_right();
        }
        KeyCode::Home => {
            state.move_cursor_home();
        }
        KeyCode::End => {
            state.move_cursor_end();
        }
        KeyCode::Enter => {
            // Jump to hotkey list when Enter pressed in search
            state.focus = HotkeyFocus::HotkeyList;
        }
        _ => {}
    }
    HotkeyDialogAction::None
}

/// Handle keyboard input for the category list.
fn handle_category_list_key<C: HotkeyCategory>(
    state: &mut HotkeyDialogState<C>,
    key: KeyEvent,
) -> HotkeyDialogAction {
    match key.code {
        KeyCode::Up => {
            state.prev_category();
        }
        KeyCode::Down => {
            state.next_category();
        }
        KeyCode::Enter | KeyCode::Right => {
            // Select category and move to hotkey list
            state.focus = HotkeyFocus::HotkeyList;
        }
        _ => {}
    }
    HotkeyDialogAction::None
}

/// Handle keyboard input for the hotkey list.
fn handle_hotkey_list_key<C: HotkeyCategory>(
    state: &mut HotkeyDialogState<C>,
    key: KeyEvent,
) -> HotkeyDialogAction {
    match key.code {
        KeyCode::Up => {
            state.prev_hotkey();
        }
        KeyCode::Down => {
            state.next_hotkey();
        }
        KeyCode::PageUp => {
            state.page_up();
        }
        KeyCode::PageDown => {
            state.page_down();
        }
        KeyCode::Left => {
            // Go back to category list
            state.focus = HotkeyFocus::CategoryList;
        }
        KeyCode::Enter => {
            // Return entry selection action (for future rebinding feature)
            // For now, this can be used to show a toast or perform other actions
            return HotkeyDialogAction::EntrySelected {
                key_combination: String::new(), // Caller needs to look up via provider
                action: String::new(),
                context: String::new(),
            };
        }
        _ => {}
    }
    HotkeyDialogAction::None
}

/// Handle a mouse event for the hotkey dialog.
///
/// Returns a `HotkeyDialogAction` indicating what action should be taken.
///
/// # Example
///
/// ```rust,ignore
/// use crossterm::event::MouseEvent;
/// use ratatui_interact::components::hotkey_dialog::{
///     HotkeyDialogState, HotkeyDialogAction, handle_hotkey_dialog_mouse
/// };
///
/// let mut state = HotkeyDialogState::<MyCategory>::new();
/// let action = handle_hotkey_dialog_mouse(&mut state, mouse_event);
///
/// match action {
///     HotkeyDialogAction::ScrollUp(amount) => { /* handle scroll */ }
///     HotkeyDialogAction::ScrollDown(amount) => { /* handle scroll */ }
///     _ => {}
/// }
/// ```
pub fn handle_hotkey_dialog_mouse<C: HotkeyCategory>(
    state: &mut HotkeyDialogState<C>,
    mouse: MouseEvent,
) -> HotkeyDialogAction {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            state.scroll_hotkeys_up(3);
            HotkeyDialogAction::ScrollUp(3)
        }
        MouseEventKind::ScrollDown => {
            state.scroll_hotkeys_down(3);
            HotkeyDialogAction::ScrollDown(3)
        }
        MouseEventKind::Down(MouseButton::Left) => {
            // Let the state handle click detection
            state.handle_click(mouse.column, mouse.row);
            HotkeyDialogAction::None
        }
        _ => HotkeyDialogAction::None,
    }
}

/// Check if a key event should close the dialog.
///
/// This is a convenience function for applications that want to check
/// for close conditions without fully handling the event.
pub fn is_close_key(key: &KeyEvent) -> bool {
    key.code == KeyCode::Esc
}

/// Check if a key event is a navigation key (Tab/Shift+Tab).
pub fn is_navigation_key(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Tab | KeyCode::BackTab)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
    enum TestCategory {
        #[default]
        First,
        Second,
    }

    impl HotkeyCategory for TestCategory {
        fn all() -> &'static [Self] {
            &[Self::First, Self::Second]
        }

        fn display_name(&self) -> &str {
            match self {
                Self::First => "First",
                Self::Second => "Second",
            }
        }

        fn next(&self) -> Self {
            match self {
                Self::First => Self::Second,
                Self::Second => Self::First,
            }
        }

        fn prev(&self) -> Self {
            match self {
                Self::First => Self::Second,
                Self::Second => Self::First,
            }
        }
    }

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn key_event_shift(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::SHIFT)
    }

    #[test]
    fn test_escape_closes_dialog() {
        let mut state: HotkeyDialogState<TestCategory> = HotkeyDialogState::new();
        let action = handle_hotkey_dialog_key(&mut state, key_event(KeyCode::Esc));
        assert_eq!(action, HotkeyDialogAction::Close);
    }

    #[test]
    fn test_escape_clears_search_first() {
        let mut state: HotkeyDialogState<TestCategory> = HotkeyDialogState::new();
        state.focus = HotkeyFocus::SearchInput;
        state.insert_char('a');

        let action = handle_hotkey_dialog_key(&mut state, key_event(KeyCode::Esc));
        assert_eq!(action, HotkeyDialogAction::None);
        assert!(state.search_query.is_empty());

        // Second escape should close
        let action = handle_hotkey_dialog_key(&mut state, key_event(KeyCode::Esc));
        assert_eq!(action, HotkeyDialogAction::Close);
    }

    #[test]
    fn test_tab_cycles_focus() {
        let mut state: HotkeyDialogState<TestCategory> = HotkeyDialogState::new();
        state.focus = HotkeyFocus::CategoryList;

        handle_hotkey_dialog_key(&mut state, key_event(KeyCode::Tab));
        assert_eq!(state.focus, HotkeyFocus::HotkeyList);

        handle_hotkey_dialog_key(&mut state, key_event(KeyCode::Tab));
        assert_eq!(state.focus, HotkeyFocus::SearchInput);

        handle_hotkey_dialog_key(&mut state, key_event_shift(KeyCode::Tab));
        assert_eq!(state.focus, HotkeyFocus::HotkeyList);
    }

    #[test]
    fn test_category_navigation() {
        let mut state: HotkeyDialogState<TestCategory> = HotkeyDialogState::new();
        state.focus = HotkeyFocus::CategoryList;

        handle_hotkey_dialog_key(&mut state, key_event(KeyCode::Down));
        assert_eq!(state.selected_category, TestCategory::Second);

        handle_hotkey_dialog_key(&mut state, key_event(KeyCode::Up));
        assert_eq!(state.selected_category, TestCategory::First);
    }

    #[test]
    fn test_search_input() {
        let mut state: HotkeyDialogState<TestCategory> = HotkeyDialogState::new();
        state.focus = HotkeyFocus::SearchInput;

        handle_hotkey_dialog_key(&mut state, KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        handle_hotkey_dialog_key(&mut state, KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty()));
        assert_eq!(state.search_query, "ab");

        handle_hotkey_dialog_key(&mut state, key_event(KeyCode::Backspace));
        assert_eq!(state.search_query, "a");
    }
}
