//! UI Components
//!
//! This module provides reusable interactive UI components that extend ratatui.
//!
//! # Components
//!
//! - [`CheckBox`] - Toggleable checkbox with label
//! - [`Input`] - Text input field with cursor
//! - [`Button`] - Various button styles
//! - [`PopupDialog`] - Container for popup dialogs
//! - [`ParagraphExt`] - Extended paragraph with word-wrapping and scrolling

pub mod button;
pub mod checkbox;
pub mod container;
pub mod input;
pub mod paragraph_ext;
pub mod toast;

pub use button::{Button, ButtonAction, ButtonState, ButtonStyle, ButtonVariant};
pub use checkbox::{CheckBox, CheckBoxAction, CheckBoxState, CheckBoxStyle};
pub use container::{DialogConfig, DialogFocusTarget, DialogState, PopupDialog};
pub use input::{Input, InputAction, InputState, InputStyle};
pub use paragraph_ext::ParagraphExt;
pub use toast::{Toast, ToastState, ToastStyle};
