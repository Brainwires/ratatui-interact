//! State management for TUI components
//!
//! This module provides state management utilities for interactive components.
//!
//! # Components
//!
//! - [`FocusManager`] - Manages keyboard focus and Tab navigation

mod focus;

pub use focus::FocusManager;
