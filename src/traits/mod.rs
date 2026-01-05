//! Core traits for TUI components
//!
//! This module provides the foundational traits that enable interactive
//! UI components with focus management and mouse click support.
//!
//! # Traits
//!
//! - [`Focusable`] - For components that can receive keyboard focus
//! - [`Clickable`] - For components that respond to mouse clicks
//! - [`Container`] - For components that manage child components
//! - [`PopupContainer`] - Extension of Container for popup dialogs

mod focusable;
mod clickable;
mod container;

pub use focusable::{FocusId, Focusable};
pub use clickable::{ClickRegion, ClickRegionRegistry, Clickable};
pub use container::{Container, ContainerAction, EventResult, PopupContainer};
