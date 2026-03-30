//! Utility functions for TUI rendering
//!
//! This module provides common utility functions used across TUI components:
//!
//! - [`ansi`] - ANSI escape code parsing and conversion to ratatui styles
//! - [`clipboard`] - Clipboard copy/paste operations (requires `clipboard` feature)
//! - [`display`] - String manipulation for display (truncation, padding, cleaning)
//! - [`mouse_capture`] - Mouse capture state management for copy mode
//! - [`view_copy`] - View/Copy mode for native terminal text selection

pub mod ansi;
pub mod clipboard;
pub mod display;
pub mod mouse_capture;
pub mod view_copy;

pub use ansi::{parse_ansi_to_spans, render_markdown_to_lines};
pub use clipboard::{
    ClipboardResult, copy_lines_to_clipboard, copy_to_clipboard, get_from_clipboard,
    is_clipboard_available,
};
pub use display::{clean_for_display, format_size, pad_to_width, truncate_to_width};
pub use mouse_capture::{
    MouseCaptureState, disable_mouse_capture, enable_mouse_capture, set_mouse_capture,
    toggle_mouse_capture,
};
pub use view_copy::{
    ExitStrategy, ViewCopyAction, ViewCopyConfig, ViewCopyMode, clear_main_screen,
};
