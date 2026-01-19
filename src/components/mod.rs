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
//! - [`Select`] - Dropdown select box with popup options
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
//! - [`Accordion`] - Collapsible sections with single or multiple expansion
//!
//! ## Layout Components
//! - [`TabView`] - Tab bar with switchable content panes
//!
//! ## Viewer Components
//! - [`LogViewer`] - Scrollable log viewer with search
//! - [`StepDisplay`] - Multi-step progress display

pub mod accordion;
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
pub mod select;
pub mod step_display;
pub mod tab_view;
pub mod toast;
pub mod tree_view;

pub use accordion::{
    Accordion, AccordionMode, AccordionState, AccordionStyle,
    calculate_height as accordion_height, handle_accordion_key, handle_accordion_mouse,
};
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
pub use select::{
    Select, SelectAction, SelectState, SelectStyle,
    calculate_dropdown_height, handle_select_key, handle_select_mouse,
};
pub use tab_view::{
    Tab, TabPosition, TabView, TabViewAction, TabViewState, TabViewStyle,
    handle_tab_view_key, handle_tab_view_mouse,
};
