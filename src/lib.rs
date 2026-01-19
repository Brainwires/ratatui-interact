//! # TUI Extension
//!
//! Reusable UI components extending ratatui with focus management and mouse support.
//!
//! This crate provides interactive UI components that integrate with ratatui's
//! widget system while adding:
//!
//! - **Focus Management**: Tab navigation between components
//! - **Click Regions**: Mouse click support with hit-testing
//! - **Composition**: Container-based component hierarchies for dialogs
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use ratatui_interact::prelude::*;
//!
//! // Create component state
//! let mut checkbox_state = CheckBoxState::new(false);
//! let mut input_state = InputState::new("Hello");
//! let button_state = ButtonState::enabled();
//!
//! // Use in your render function
//! fn render(frame: &mut Frame, area: Rect) {
//!     let checkbox = CheckBox::new("Enable", &checkbox_state);
//!     let input = Input::new(&input_state).label("Name");
//!     let button = Button::new("Submit", &button_state);
//!
//!     // Render and get click regions
//!     let cb_region = checkbox.render_stateful(area, frame.buffer_mut());
//!     let input_region = input.render_stateful(frame, input_area);
//!     let btn_region = button.render_stateful(button_area, frame.buffer_mut());
//! }
//! ```
//!
//! ## Components
//!
//! ### CheckBox
//!
//! A toggleable checkbox with customizable symbols:
//!
//! ```rust
//! use ratatui_interact::components::{CheckBox, CheckBoxState, CheckBoxStyle};
//!
//! let mut state = CheckBoxState::new(false);
//! let checkbox = CheckBox::new("Dark mode", &state)
//!     .style(CheckBoxStyle::unicode()); // ☑ / ☐
//!
//! // Toggle on user action
//! state.toggle();
//! ```
//!
//! ### Input
//!
//! A text input field with cursor and editing support:
//!
//! ```rust
//! use ratatui_interact::components::{Input, InputState};
//!
//! let mut state = InputState::new("Initial text");
//!
//! // Edit text
//! state.insert_char('!');
//! state.move_left();
//! state.delete_char_backward();
//! ```
//!
//! ### Button
//!
//! Buttons with multiple display variants:
//!
//! ```rust
//! use ratatui_interact::components::{Button, ButtonState, ButtonVariant, ButtonStyle};
//!
//! let state = ButtonState::enabled();
//!
//! // Different styles
//! let simple = Button::new("OK", &state);
//! let with_icon = Button::new("Save", &state).icon("💾");
//! let block_style = Button::new("Submit", &state).variant(ButtonVariant::Block);
//! let toggle = Button::new("Active", &ButtonState::toggled(true))
//!     .variant(ButtonVariant::Toggle);
//! ```
//!
//! ### PopupDialog
//!
//! A container for popup dialogs with focus management:
//!
//! ```rust,ignore
//! use ratatui_interact::components::{DialogConfig, DialogState, PopupDialog};
//! use ratatui_interact::traits::ContainerAction;
//!
//! let config = DialogConfig::new("Settings")
//!     .width_percent(50)
//!     .ok_cancel();
//!
//! let mut state = DialogState::new(MyContent::default());
//! state.show();
//!
//! let mut dialog = PopupDialog::new(&config, &mut state, |frame, area, content| {
//!     // Render dialog content
//! });
//! dialog.render(frame);
//!
//! // Handle events
//! match dialog.handle_key(key_event) {
//!     EventResult::Action(ContainerAction::Submit) => { /* save */ }
//!     EventResult::Action(ContainerAction::Close) => { /* cancel */ }
//!     _ => {}
//! }
//! ```
//!
//! ## Focus Management
//!
//! The `FocusManager` handles Tab navigation:
//!
//! ```rust
//! use ratatui_interact::state::FocusManager;
//!
//! #[derive(Clone, PartialEq, Eq, Hash)]
//! enum Element { Name, Email, Submit }
//!
//! let mut focus = FocusManager::new();
//! focus.register(Element::Name);
//! focus.register(Element::Email);
//! focus.register(Element::Submit);
//!
//! // Navigate
//! focus.next(); // Name -> Email
//! focus.prev(); // Email -> Name
//! focus.set(Element::Submit); // Jump to Submit
//! ```
//!
//! ## Click Regions
//!
//! Track clickable areas with `ClickRegionRegistry`:
//!
//! ```rust
//! use ratatui_interact::traits::ClickRegionRegistry;
//! use ratatui::layout::Rect;
//!
//! let mut registry: ClickRegionRegistry<&str> = ClickRegionRegistry::new();
//!
//! // Register during render
//! registry.clear();
//! registry.register(Rect::new(0, 0, 10, 1), "button1");
//! registry.register(Rect::new(15, 0, 10, 1), "button2");
//!
//! // Check clicks during event handling
//! if let Some(clicked) = registry.handle_click(5, 0) {
//!     println!("Clicked: {}", clicked);
//! }
//! ```

pub mod components;
pub mod events;
pub mod state;
pub mod traits;
pub mod utils;

/// Prelude for convenient imports.
///
/// Import everything commonly needed:
///
/// ```rust
/// use ratatui_interact::prelude::*;
/// ```
pub mod prelude {
    // Interactive Components
    pub use crate::components::{
        Button, ButtonAction, ButtonState, ButtonStyle, ButtonVariant, CheckBox, CheckBoxAction,
        CheckBoxState, CheckBoxStyle, DialogConfig, DialogFocusTarget, DialogState, Input,
        InputAction, InputState, InputStyle, PopupDialog,
    };

    // Display Components
    pub use crate::components::{
        ParagraphExt, Progress, ProgressStyle, Toast, ToastState, ToastStyle,
    };

    // Navigation Components
    pub use crate::components::{
        EntryType, FileEntry, FileExplorer, FileExplorerState, FileExplorerStyle, ListPicker,
        ListPickerState, ListPickerStyle, key_hints_footer,
    };

    // Tree Components
    pub use crate::components::{
        FlatNode, TreeNode, TreeStyle, TreeView, TreeViewState, get_selected_id,
    };

    // Layout Components
    pub use crate::components::{
        Orientation, SplitPane, SplitPaneAction, SplitPaneState, SplitPaneStyle,
        handle_split_pane_key, handle_split_pane_mouse,
    };

    // Viewer Components
    pub use crate::components::{
        DiffData, DiffHunk, DiffLine, DiffLineType, DiffViewMode, DiffViewer, DiffViewerAction,
        DiffViewerState, DiffViewerStyle, LogViewer, LogViewerState, LogViewerStyle, SearchState,
        Step, StepDisplay, StepDisplayState, StepDisplayStyle, StepStatus, SubStep,
        handle_diff_viewer_key, handle_diff_viewer_mouse, step_display_height,
    };

    // Utilities
    pub use crate::utils::{
        clean_for_display, format_size, pad_to_width, parse_ansi_to_spans, truncate_to_width,
    };

    // Traits
    pub use crate::traits::{
        ClickRegion, ClickRegionRegistry, Clickable, Container, ContainerAction, EventResult,
        FocusId, Focusable, PopupContainer,
    };

    // State management
    pub use crate::state::FocusManager;

    // Event helpers
    pub use crate::events::{
        get_char, get_mouse_pos, get_scroll, has_alt, has_ctrl, has_shift, is_activate_key,
        is_backspace, is_backtab, is_close_key, is_ctrl_a, is_ctrl_e, is_ctrl_k, is_ctrl_u,
        is_ctrl_w, is_delete, is_end, is_enter, is_home, is_left_click, is_navigation_key,
        is_right_click, is_space, is_tab,
    };
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_prelude_imports() {
        // Verify all prelude items are accessible
        let _: CheckBoxState = CheckBoxState::new(false);
        let _: InputState = InputState::new("");
        let _: ButtonState = ButtonState::enabled();
        let _: FocusManager<usize> = FocusManager::new();
        let _: ClickRegionRegistry<()> = ClickRegionRegistry::new();
    }
}
