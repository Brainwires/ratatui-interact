//! Container trait for component composition
//!
//! Provides trait-based "inheritance" pattern for containers that manage
//! child components and can be expressed as popup dialogs.
//!
//! # Design Pattern
//!
//! The Container and PopupContainer traits enable composition-based UI design:
//!
//! - `Container` - Base trait for any component that manages children
//! - `PopupContainer` - Extension for popup/modal dialog behavior
//!
//! # Example
//!
//! ```rust,ignore
//! use tui_extension::traits::{Container, PopupContainer, EventResult, ContainerAction};
//! use ratatui::{layout::Rect, Frame};
//! use crossterm::event::{KeyEvent, MouseEvent};
//!
//! struct MyDialog;
//!
//! impl Container for MyDialog {
//!     type State = MyDialogState;
//!
//!     fn render(&self, frame: &mut Frame, area: Rect, state: &Self::State) {
//!         // Render dialog content
//!     }
//!
//!     fn handle_key(&self, key: KeyEvent, state: &mut Self::State) -> EventResult {
//!         EventResult::NotHandled
//!     }
//!
//!     fn handle_mouse(&self, mouse: MouseEvent, state: &mut Self::State) -> EventResult {
//!         EventResult::NotHandled
//!     }
//!
//!     fn preferred_size(&self) -> (u16, u16) {
//!         (60, 20)
//!     }
//! }
//!
//! impl PopupContainer for MyDialog {
//!     // Uses default implementations for popup_area, close_on_escape, etc.
//! }
//! ```

use ratatui::{layout::Rect, Frame};
use crossterm::event::{KeyEvent, MouseEvent};

/// Result of handling an event.
///
/// Used by containers to indicate how an event was processed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventResult {
    /// Event was consumed, no further handling needed.
    Consumed,
    /// Event was not handled, should propagate to parent.
    NotHandled,
    /// Event triggered a specific action.
    Action(ContainerAction),
}

impl EventResult {
    /// Check if the event was consumed (either Consumed or Action).
    pub fn is_consumed(&self) -> bool {
        !matches!(self, EventResult::NotHandled)
    }

    /// Check if the result is an action.
    pub fn is_action(&self) -> bool {
        matches!(self, EventResult::Action(_))
    }

    /// Get the action if this is an Action result.
    pub fn action(&self) -> Option<&ContainerAction> {
        match self {
            EventResult::Action(action) => Some(action),
            _ => None,
        }
    }
}

/// Actions that containers can emit.
///
/// These are standard actions that containers can produce in response
/// to user interactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerAction {
    /// Close the container (dismiss popup/dialog).
    Close,
    /// Submit/confirm the container's contents.
    Submit,
    /// Custom action with string identifier.
    Custom(String),
}

impl ContainerAction {
    /// Create a custom action.
    pub fn custom(name: impl Into<String>) -> Self {
        Self::Custom(name.into())
    }

    /// Check if this is a Close action.
    pub fn is_close(&self) -> bool {
        matches!(self, ContainerAction::Close)
    }

    /// Check if this is a Submit action.
    pub fn is_submit(&self) -> bool {
        matches!(self, ContainerAction::Submit)
    }

    /// Get the custom action name if this is a Custom action.
    pub fn custom_name(&self) -> Option<&str> {
        match self {
            ContainerAction::Custom(name) => Some(name),
            _ => None,
        }
    }
}

/// Trait for container components that manage children.
///
/// Containers are responsible for:
/// - Rendering their content within a given area
/// - Handling keyboard events
/// - Handling mouse events
/// - Reporting their preferred size
pub trait Container {
    /// The type of state this container manages.
    type State;

    /// Render the container and its children.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render into
    /// * `area` - The area allocated to this container
    /// * `state` - The container's state
    fn render(&self, frame: &mut Frame, area: Rect, state: &Self::State);

    /// Handle keyboard events.
    ///
    /// # Arguments
    ///
    /// * `key` - The key event to handle
    /// * `state` - The container's mutable state
    ///
    /// # Returns
    ///
    /// How the event was handled.
    fn handle_key(&self, key: KeyEvent, state: &mut Self::State) -> EventResult;

    /// Handle mouse events.
    ///
    /// # Arguments
    ///
    /// * `mouse` - The mouse event to handle
    /// * `state` - The container's mutable state
    ///
    /// # Returns
    ///
    /// How the event was handled.
    fn handle_mouse(&self, mouse: MouseEvent, state: &mut Self::State) -> EventResult;

    /// Get the preferred size for this container.
    ///
    /// # Returns
    ///
    /// A tuple of (width, height) representing the preferred dimensions.
    fn preferred_size(&self) -> (u16, u16);
}

/// Trait for containers that can be expressed as popup dialogs.
///
/// This extends `Container` with popup-specific behavior like
/// centered positioning and close-on-escape.
pub trait PopupContainer: Container {
    /// Calculate the centered popup area given screen dimensions.
    ///
    /// Default implementation centers the popup based on `preferred_size()`,
    /// with padding from screen edges.
    fn popup_area(&self, screen: Rect) -> Rect {
        let (width, height) = self.preferred_size();
        let width = width.min(screen.width.saturating_sub(4));
        let height = height.min(screen.height.saturating_sub(4));

        let x = (screen.width.saturating_sub(width)) / 2;
        let y = (screen.height.saturating_sub(height)) / 2;

        Rect::new(x, y, width, height)
    }

    /// Whether clicking outside the popup should close it.
    ///
    /// Default returns `true`.
    fn close_on_outside_click(&self) -> bool {
        true
    }

    /// Whether pressing Escape should close the popup.
    ///
    /// Default returns `true`.
    fn close_on_escape(&self) -> bool {
        true
    }

    /// Get the minimum margin from screen edges.
    ///
    /// Default returns 2 (allows for shadows/borders).
    fn screen_margin(&self) -> u16 {
        2
    }

    /// Calculate popup area with custom positioning.
    ///
    /// # Arguments
    ///
    /// * `screen` - The full screen dimensions
    /// * `anchor_x` - Optional x position to anchor near
    /// * `anchor_y` - Optional y position to anchor near
    fn popup_area_anchored(&self, screen: Rect, anchor_x: Option<u16>, anchor_y: Option<u16>) -> Rect {
        let (width, height) = self.preferred_size();
        let margin = self.screen_margin();
        let width = width.min(screen.width.saturating_sub(margin * 2));
        let height = height.min(screen.height.saturating_sub(margin * 2));

        let x = match anchor_x {
            Some(ax) => {
                // Try to position near anchor, but keep on screen
                let ideal = ax.saturating_sub(width / 2);
                ideal.clamp(margin, screen.width.saturating_sub(width + margin))
            }
            None => (screen.width.saturating_sub(width)) / 2,
        };

        let y = match anchor_y {
            Some(ay) => {
                // Try to position below anchor
                let below = ay + 1;
                if below + height <= screen.height.saturating_sub(margin) {
                    below
                } else {
                    // Position above if not enough room below
                    ay.saturating_sub(height + 1).max(margin)
                }
            }
            None => (screen.height.saturating_sub(height)) / 2,
        };

        Rect::new(x, y, width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_result_consumed() {
        assert!(EventResult::Consumed.is_consumed());
        assert!(EventResult::Action(ContainerAction::Close).is_consumed());
        assert!(!EventResult::NotHandled.is_consumed());
    }

    #[test]
    fn test_event_result_action() {
        assert!(!EventResult::Consumed.is_action());
        assert!(!EventResult::NotHandled.is_action());
        assert!(EventResult::Action(ContainerAction::Close).is_action());

        let result = EventResult::Action(ContainerAction::Submit);
        assert_eq!(result.action(), Some(&ContainerAction::Submit));

        assert_eq!(EventResult::Consumed.action(), None);
    }

    #[test]
    fn test_container_action_types() {
        assert!(ContainerAction::Close.is_close());
        assert!(!ContainerAction::Close.is_submit());

        assert!(ContainerAction::Submit.is_submit());
        assert!(!ContainerAction::Submit.is_close());

        let custom = ContainerAction::custom("my_action");
        assert_eq!(custom.custom_name(), Some("my_action"));
        assert!(!custom.is_close());
        assert!(!custom.is_submit());

        assert_eq!(ContainerAction::Close.custom_name(), None);
    }

    struct TestContainer {
        preferred_width: u16,
        preferred_height: u16,
    }

    impl Container for TestContainer {
        type State = ();

        fn render(&self, _frame: &mut Frame, _area: Rect, _state: &Self::State) {}

        fn handle_key(&self, _key: KeyEvent, _state: &mut Self::State) -> EventResult {
            EventResult::NotHandled
        }

        fn handle_mouse(&self, _mouse: MouseEvent, _state: &mut Self::State) -> EventResult {
            EventResult::NotHandled
        }

        fn preferred_size(&self) -> (u16, u16) {
            (self.preferred_width, self.preferred_height)
        }
    }

    impl PopupContainer for TestContainer {}

    #[test]
    fn test_popup_area_centered() {
        let container = TestContainer {
            preferred_width: 40,
            preferred_height: 20,
        };

        let screen = Rect::new(0, 0, 100, 50);
        let area = container.popup_area(screen);

        // Should be centered
        assert_eq!(area.width, 40);
        assert_eq!(area.height, 20);
        assert_eq!(area.x, 30); // (100 - 40) / 2
        assert_eq!(area.y, 15); // (50 - 20) / 2
    }

    #[test]
    fn test_popup_area_constrained() {
        let container = TestContainer {
            preferred_width: 200, // Larger than screen
            preferred_height: 100,
        };

        let screen = Rect::new(0, 0, 80, 24);
        let area = container.popup_area(screen);

        // Should be constrained to screen with padding
        assert_eq!(area.width, 76); // 80 - 4
        assert_eq!(area.height, 20); // 24 - 4
    }

    #[test]
    fn test_popup_defaults() {
        let container = TestContainer {
            preferred_width: 40,
            preferred_height: 20,
        };

        assert!(container.close_on_escape());
        assert!(container.close_on_outside_click());
        assert_eq!(container.screen_margin(), 2);
    }

    #[test]
    fn test_popup_area_anchored() {
        let container = TestContainer {
            preferred_width: 20,
            preferred_height: 10,
        };

        let screen = Rect::new(0, 0, 80, 24);

        // Anchored at center
        let area = container.popup_area_anchored(screen, Some(40), Some(10));
        assert_eq!(area.x, 30); // 40 - 20/2, within bounds
        assert_eq!(area.y, 11); // Below anchor (10 + 1)

        // Anchored near bottom - should flip above
        let area = container.popup_area_anchored(screen, Some(40), Some(20));
        assert_eq!(area.y, 9); // Above anchor (20 - 10 - 1)
    }
}
