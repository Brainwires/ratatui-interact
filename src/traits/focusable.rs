//! Focusable trait for keyboard navigation
//!
//! Components implementing this trait can receive keyboard focus
//! and participate in Tab navigation.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::traits::{FocusId, Focusable};
//! use ratatui::style::{Color, Modifier, Style};
//!
//! struct MyWidget {
//!     focus_id: FocusId,
//!     focused: bool,
//! }
//!
//! impl Focusable for MyWidget {
//!     fn focus_id(&self) -> FocusId {
//!         self.focus_id
//!     }
//!
//!     fn is_focused(&self) -> bool {
//!         self.focused
//!     }
//!
//!     fn set_focused(&mut self, focused: bool) {
//!         self.focused = focused;
//!     }
//!
//!     fn focused_style(&self) -> Style {
//!         Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
//!     }
//!
//!     fn unfocused_style(&self) -> Style {
//!         Style::default().fg(Color::Gray)
//!     }
//! }
//! ```

use ratatui::style::{Color, Modifier, Style};

/// A unique identifier for focusable elements.
///
/// Used to track which element has focus and for Tab navigation ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FocusId(pub u32);

impl FocusId {
    /// Create a new focus ID with the given value.
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the inner ID value.
    pub fn id(&self) -> u32 {
        self.0
    }
}

impl From<u32> for FocusId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<usize> for FocusId {
    fn from(id: usize) -> Self {
        Self(id as u32)
    }
}

/// Trait for components that can receive keyboard focus.
///
/// Components implementing this trait can:
/// - Receive and lose focus
/// - Provide different styles for focused/unfocused states
/// - Participate in Tab navigation with tab ordering
/// - Be conditionally focusable (enabled/disabled state)
pub trait Focusable {
    /// Returns the unique focus ID for this component.
    fn focus_id(&self) -> FocusId;

    /// Returns true if this component currently has focus.
    fn is_focused(&self) -> bool;

    /// Set the focus state of this component.
    fn set_focused(&mut self, focused: bool);

    /// Returns the style to use when this component has focus.
    ///
    /// Default implementation returns yellow foreground with bold modifier.
    fn focused_style(&self) -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    /// Returns the style to use when this component does not have focus.
    ///
    /// Default implementation returns white foreground.
    fn unfocused_style(&self) -> Style {
        Style::default().fg(Color::White)
    }

    /// Returns the current style based on focus state.
    ///
    /// This is a convenience method that returns `focused_style()` if focused,
    /// otherwise `unfocused_style()`.
    fn current_style(&self) -> Style {
        if self.is_focused() {
            self.focused_style()
        } else {
            self.unfocused_style()
        }
    }

    /// Whether this component can currently receive focus.
    ///
    /// Return `false` for disabled components that should be skipped
    /// during Tab navigation.
    ///
    /// Default implementation returns `true`.
    fn can_focus(&self) -> bool {
        true
    }

    /// Tab order index for this component.
    ///
    /// Lower values come earlier in Tab navigation order.
    /// Components with the same tab order are navigated in registration order.
    ///
    /// Default implementation returns `0`.
    fn tab_order(&self) -> u32 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWidget {
        focus_id: FocusId,
        focused: bool,
        enabled: bool,
        tab_order: u32,
    }

    impl TestWidget {
        fn new(id: u32) -> Self {
            Self {
                focus_id: FocusId::new(id),
                focused: false,
                enabled: true,
                tab_order: 0,
            }
        }
    }

    impl Focusable for TestWidget {
        fn focus_id(&self) -> FocusId {
            self.focus_id
        }

        fn is_focused(&self) -> bool {
            self.focused
        }

        fn set_focused(&mut self, focused: bool) {
            self.focused = focused;
        }

        fn can_focus(&self) -> bool {
            self.enabled
        }

        fn tab_order(&self) -> u32 {
            self.tab_order
        }
    }

    #[test]
    fn test_focus_id_creation() {
        let id = FocusId::new(42);
        assert_eq!(id.id(), 42);

        let id_from_u32: FocusId = 100u32.into();
        assert_eq!(id_from_u32.id(), 100);

        let id_from_usize: FocusId = 200usize.into();
        assert_eq!(id_from_usize.id(), 200);
    }

    #[test]
    fn test_focus_state() {
        let mut widget = TestWidget::new(1);
        assert!(!widget.is_focused());

        widget.set_focused(true);
        assert!(widget.is_focused());

        widget.set_focused(false);
        assert!(!widget.is_focused());
    }

    #[test]
    fn test_current_style() {
        let mut widget = TestWidget::new(1);

        // Unfocused style
        let style = widget.current_style();
        assert_eq!(style, widget.unfocused_style());

        // Focused style
        widget.set_focused(true);
        let style = widget.current_style();
        assert_eq!(style, widget.focused_style());
    }

    #[test]
    fn test_can_focus() {
        let mut widget = TestWidget::new(1);
        assert!(widget.can_focus());

        widget.enabled = false;
        assert!(!widget.can_focus());
    }

    #[test]
    fn test_tab_order() {
        let mut widget = TestWidget::new(1);
        assert_eq!(widget.tab_order(), 0);

        widget.tab_order = 5;
        assert_eq!(widget.tab_order(), 5);
    }
}
