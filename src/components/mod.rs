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

pub mod checkbox;
pub mod input;
pub mod button;
pub mod container;

pub use checkbox::{CheckBox, CheckBoxAction, CheckBoxState, CheckBoxStyle};
pub use input::{Input, InputAction, InputState, InputStyle};
pub use button::{Button, ButtonAction, ButtonState, ButtonStyle, ButtonVariant};
pub use container::{DialogConfig, DialogFocusTarget, DialogState, PopupDialog};
