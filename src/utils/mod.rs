//! Utility functions for TUI rendering
//!
//! This module provides common utility functions used across TUI components:
//!
//! - [`ansi`] - ANSI escape code parsing and conversion to ratatui styles
//! - [`display`] - String manipulation for display (truncation, padding, cleaning)

pub mod ansi;
pub mod display;

pub use ansi::{parse_ansi_to_spans, render_markdown_to_lines};
pub use display::{clean_for_display, format_size, pad_to_width, truncate_to_width};
