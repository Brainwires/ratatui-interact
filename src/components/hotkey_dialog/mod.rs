//! Hotkey Dialog Component
//!
//! A reusable, generic hotkey configuration dialog for TUI applications.
//!
//! This module provides a complete hotkey dialog implementation that can be
//! customized for any application through trait-based abstraction.
//!
//! # Architecture
//!
//! The hotkey dialog uses three main abstractions:
//!
//! - [`HotkeyCategory`] - Trait for defining hotkey categories (implemented by your enum)
//! - [`HotkeyProvider`] - Trait for providing hotkey data
//! - [`HotkeyEntryData`] - Generic struct representing a single hotkey
//!
//! # Example
//!
//! ```rust,ignore
//! use ratatui_interact::components::hotkey_dialog::{
//!     HotkeyCategory, HotkeyProvider, HotkeyEntryData,
//!     HotkeyDialogState, HotkeyDialogStyle, HotkeyDialog,
//!     handle_hotkey_dialog_key, handle_hotkey_dialog_mouse,
//! };
//!
//! // 1. Define your category enum
//! #[derive(Clone, Copy, PartialEq, Eq, Default)]
//! enum MyCategory {
//!     #[default]
//!     Global,
//!     Navigation,
//!     Editing,
//! }
//!
//! impl HotkeyCategory for MyCategory {
//!     fn all() -> &'static [Self] { &[Self::Global, Self::Navigation, Self::Editing] }
//!     fn display_name(&self) -> &str { /* ... */ }
//!     fn next(&self) -> Self { /* ... */ }
//!     fn prev(&self) -> Self { /* ... */ }
//! }
//!
//! // 2. Implement the provider
//! struct MyProvider;
//!
//! impl HotkeyProvider for MyProvider {
//!     type Category = MyCategory;
//!
//!     fn entries_for_category(&self, category: Self::Category) -> Vec<HotkeyEntryData> {
//!         match category {
//!             MyCategory::Global => vec![
//!                 HotkeyEntryData::global("Ctrl+C", "Quit"),
//!                 HotkeyEntryData::global("F1", "Help"),
//!             ],
//!             // ... other categories
//!         }
//!     }
//!
//!     fn search(&self, query: &str) -> Vec<(Self::Category, HotkeyEntryData)> {
//!         // Search implementation
//!     }
//! }
//!
//! // 3. Use in your application
//! let mut state = HotkeyDialogState::<MyCategory>::new();
//! let provider = MyProvider;
//! let style = HotkeyDialogStyle::default();
//!
//! // Render
//! HotkeyDialog::new(&mut state, &provider, &style).render(frame, area);
//!
//! // Handle events
//! let action = handle_hotkey_dialog_key(&mut state, key_event);
//! ```
//!
//! # Features
//!
//! - **Search filtering**: Type to filter hotkeys across all categories
//! - **Category navigation**: Arrow keys to navigate between categories
//! - **Mouse support**: Click to select categories and hotkeys
//! - **Scrolling**: Page up/down and mouse scroll for long lists
//! - **Customizable styling**: Colors, sizes, and text can be customized
//! - **Focus management**: Tab between search, categories, and hotkey list

mod handlers;
mod state;
mod style;
mod traits;
mod widget;

pub use handlers::{
    handle_hotkey_dialog_key, handle_hotkey_dialog_mouse, is_close_key, is_navigation_key,
    HotkeyDialogAction,
};
pub use state::{CategoryClickRegion, HotkeyClickRegion, HotkeyDialogState, HotkeyFocus};
pub use style::HotkeyDialogStyle;
pub use traits::{HotkeyCategory, HotkeyEntryData, HotkeyProvider};
pub use widget::{render_hotkey_dialog, HotkeyDialog};
