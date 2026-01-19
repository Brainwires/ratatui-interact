//! UI Components
//!
//! This module provides reusable interactive UI components that extend ratatui.
//!
//! # Components
//!
//! ## Interactive Components
//! - [`CheckBox`] - Toggleable checkbox with label
//! - [`Input`] - Text input field with cursor
//! - [`Button`] - Various button styles
//! - [`PopupDialog`] - Container for popup dialogs
//!
//! ## Display Components
//! - [`ParagraphExt`] - Extended paragraph with word-wrapping and scrolling
//! - [`Toast`] - Toast notifications with auto-dismiss
//! - [`Progress`] - Progress bar with label and percentage
//! - [`MarqueeText`] - Scrolling text for long content in limited space
//!
//! ## Navigation Components
//! - [`ListPicker`] - Scrollable list with selection
//! - [`TreeView`] - Collapsible tree view with selection
//! - [`FileExplorer`] - File browser with multi-select
//!
//! ## Viewer Components
//! - [`LogViewer`] - Scrollable log viewer with search
//! - [`StepDisplay`] - Multi-step progress display

pub mod button;
pub mod checkbox;
pub mod container;
pub mod file_explorer;
pub mod input;
pub mod list_picker;
pub mod log_viewer;
pub mod marquee;
pub mod paragraph_ext;
pub mod progress;
pub mod step_display;
pub mod toast;
pub mod tree_view;

pub use button::{Button, ButtonAction, ButtonState, ButtonStyle, ButtonVariant};
pub use checkbox::{CheckBox, CheckBoxAction, CheckBoxState, CheckBoxStyle};
pub use container::{DialogConfig, DialogFocusTarget, DialogState, PopupDialog};
pub use file_explorer::{EntryType, FileEntry, FileExplorer, FileExplorerState, FileExplorerStyle};
pub use input::{Input, InputAction, InputState, InputStyle};
pub use list_picker::{ListPicker, ListPickerState, ListPickerStyle, key_hints_footer};
pub use log_viewer::{LogViewer, LogViewerState, LogViewerStyle, SearchState};
pub use paragraph_ext::ParagraphExt;
pub use progress::{Progress, ProgressStyle};
pub use step_display::{
    Step, StepDisplay, StepDisplayState, StepDisplayStyle, StepStatus, SubStep,
    calculate_height as step_display_height,
};
pub use toast::{Toast, ToastState, ToastStyle};
pub use tree_view::{FlatNode, TreeNode, TreeStyle, TreeView, TreeViewState, get_selected_id};
pub use marquee::{MarqueeMode, MarqueeState, MarqueeStyle, MarqueeText, ScrollDir, bounce_marquee, continuous_marquee};
