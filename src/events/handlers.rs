//! Event handler helpers
//!
//! Utility functions for common event handling patterns.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

/// Check if a key event is an activation key (Enter or Space).
///
/// Used for activating buttons and checkboxes.
pub fn is_activate_key(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
}

/// Check if a key event is a navigation key.
///
/// Includes Tab, BackTab, and arrow keys.
pub fn is_navigation_key(key: &KeyEvent) -> bool {
    matches!(
        key.code,
        KeyCode::Tab
            | KeyCode::BackTab
            | KeyCode::Up
            | KeyCode::Down
            | KeyCode::Left
            | KeyCode::Right
    )
}

/// Check if a key event is Tab (forward navigation).
pub fn is_tab(key: &KeyEvent) -> bool {
    key.code == KeyCode::Tab && !key.modifiers.contains(KeyModifiers::SHIFT)
}

/// Check if a key event is Shift+Tab or BackTab (backward navigation).
pub fn is_backtab(key: &KeyEvent) -> bool {
    key.code == KeyCode::BackTab
        || (key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::SHIFT))
}

/// Check if a key event is the close/escape key.
pub fn is_close_key(key: &KeyEvent) -> bool {
    key.code == KeyCode::Esc
}

/// Check if a key event is Enter.
pub fn is_enter(key: &KeyEvent) -> bool {
    key.code == KeyCode::Enter
}

/// Check if a key event is Space.
pub fn is_space(key: &KeyEvent) -> bool {
    key.code == KeyCode::Char(' ')
}

/// Check if a key event is Backspace.
pub fn is_backspace(key: &KeyEvent) -> bool {
    key.code == KeyCode::Backspace
}

/// Check if a key event is Delete.
pub fn is_delete(key: &KeyEvent) -> bool {
    key.code == KeyCode::Delete
}

/// Check if a key event is an arrow key.
pub fn is_arrow_key(key: &KeyEvent) -> bool {
    matches!(
        key.code,
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
    )
}

/// Check if a key event is Home.
pub fn is_home(key: &KeyEvent) -> bool {
    key.code == KeyCode::Home
}

/// Check if a key event is End.
pub fn is_end(key: &KeyEvent) -> bool {
    key.code == KeyCode::End
}

/// Get the character from a key event if it's a printable character.
pub fn get_char(key: &KeyEvent) -> Option<char> {
    match key.code {
        KeyCode::Char(c) => {
            // Only return if no modifiers except Shift
            if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT {
                Some(c)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Check if a mouse event is a left click.
pub fn is_left_click(mouse: &MouseEvent) -> bool {
    matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
}

/// Check if a mouse event is a right click.
pub fn is_right_click(mouse: &MouseEvent) -> bool {
    matches!(mouse.kind, MouseEventKind::Down(MouseButton::Right))
}

/// Check if a mouse event is a scroll event.
///
/// Returns `Some(direction)` where negative is up, positive is down.
pub fn get_scroll(mouse: &MouseEvent) -> Option<i16> {
    match mouse.kind {
        MouseEventKind::ScrollUp => Some(-1),
        MouseEventKind::ScrollDown => Some(1),
        _ => None,
    }
}

/// Get the mouse position from a mouse event.
pub fn get_mouse_pos(mouse: &MouseEvent) -> (u16, u16) {
    (mouse.column, mouse.row)
}

/// Check if Ctrl modifier is pressed.
pub fn has_ctrl(key: &KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
}

/// Check if Alt modifier is pressed.
pub fn has_alt(key: &KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::ALT)
}

/// Check if Shift modifier is pressed.
pub fn has_shift(key: &KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::SHIFT)
}

/// Check if this is Ctrl+A (select all / move to start).
pub fn is_ctrl_a(key: &KeyEvent) -> bool {
    key.code == KeyCode::Char('a') && has_ctrl(key)
}

/// Check if this is Ctrl+E (move to end).
pub fn is_ctrl_e(key: &KeyEvent) -> bool {
    key.code == KeyCode::Char('e') && has_ctrl(key)
}

/// Check if this is Ctrl+U (delete to start of line).
pub fn is_ctrl_u(key: &KeyEvent) -> bool {
    key.code == KeyCode::Char('u') && has_ctrl(key)
}

/// Check if this is Ctrl+K (delete to end of line).
pub fn is_ctrl_k(key: &KeyEvent) -> bool {
    key.code == KeyCode::Char('k') && has_ctrl(key)
}

/// Check if this is Ctrl+W (delete word backward).
pub fn is_ctrl_w(key: &KeyEvent) -> bool {
    key.code == KeyCode::Char('w') && has_ctrl(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventKind;

    fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }
    }

    #[test]
    fn test_is_activate_key() {
        assert!(is_activate_key(&make_key(KeyCode::Enter, KeyModifiers::NONE)));
        assert!(is_activate_key(&make_key(
            KeyCode::Char(' '),
            KeyModifiers::NONE
        )));
        assert!(!is_activate_key(&make_key(
            KeyCode::Char('a'),
            KeyModifiers::NONE
        )));
    }

    #[test]
    fn test_is_navigation_key() {
        assert!(is_navigation_key(&make_key(KeyCode::Tab, KeyModifiers::NONE)));
        assert!(is_navigation_key(&make_key(
            KeyCode::BackTab,
            KeyModifiers::NONE
        )));
        assert!(is_navigation_key(&make_key(KeyCode::Up, KeyModifiers::NONE)));
        assert!(is_navigation_key(&make_key(
            KeyCode::Down,
            KeyModifiers::NONE
        )));
        assert!(!is_navigation_key(&make_key(
            KeyCode::Enter,
            KeyModifiers::NONE
        )));
    }

    #[test]
    fn test_is_tab_and_backtab() {
        assert!(is_tab(&make_key(KeyCode::Tab, KeyModifiers::NONE)));
        assert!(!is_tab(&make_key(KeyCode::Tab, KeyModifiers::SHIFT)));

        assert!(is_backtab(&make_key(KeyCode::BackTab, KeyModifiers::NONE)));
        assert!(is_backtab(&make_key(KeyCode::Tab, KeyModifiers::SHIFT)));
        assert!(!is_backtab(&make_key(KeyCode::Tab, KeyModifiers::NONE)));
    }

    #[test]
    fn test_get_char() {
        assert_eq!(
            get_char(&make_key(KeyCode::Char('a'), KeyModifiers::NONE)),
            Some('a')
        );
        assert_eq!(
            get_char(&make_key(KeyCode::Char('A'), KeyModifiers::SHIFT)),
            Some('A')
        );
        assert_eq!(
            get_char(&make_key(KeyCode::Char('a'), KeyModifiers::CONTROL)),
            None
        );
        assert_eq!(get_char(&make_key(KeyCode::Enter, KeyModifiers::NONE)), None);
    }

    #[test]
    fn test_modifier_checks() {
        let ctrl_a = make_key(KeyCode::Char('a'), KeyModifiers::CONTROL);
        assert!(has_ctrl(&ctrl_a));
        assert!(!has_alt(&ctrl_a));
        assert!(!has_shift(&ctrl_a));

        let shift_a = make_key(KeyCode::Char('A'), KeyModifiers::SHIFT);
        assert!(!has_ctrl(&shift_a));
        assert!(has_shift(&shift_a));
    }

    #[test]
    fn test_ctrl_shortcuts() {
        assert!(is_ctrl_a(&make_key(
            KeyCode::Char('a'),
            KeyModifiers::CONTROL
        )));
        assert!(is_ctrl_e(&make_key(
            KeyCode::Char('e'),
            KeyModifiers::CONTROL
        )));
        assert!(is_ctrl_u(&make_key(
            KeyCode::Char('u'),
            KeyModifiers::CONTROL
        )));
        assert!(is_ctrl_k(&make_key(
            KeyCode::Char('k'),
            KeyModifiers::CONTROL
        )));
        assert!(is_ctrl_w(&make_key(
            KeyCode::Char('w'),
            KeyModifiers::CONTROL
        )));
    }

    #[test]
    fn test_mouse_helpers() {
        let left_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        assert!(is_left_click(&left_click));
        assert!(!is_right_click(&left_click));
        assert_eq!(get_mouse_pos(&left_click), (10, 5));

        let scroll_up = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(get_scroll(&scroll_up), Some(-1));

        let scroll_down = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(get_scroll(&scroll_down), Some(1));
    }
}
