//! Mouse capture state management
//!
//! Provides utilities for toggling mouse capture at runtime, enabling "copy mode"
//! where the terminal's native text selection is available instead of ratatui
//! capturing mouse events.
//!
//! # Example
//!
//! ```rust,ignore
//! use ratatui_interact::utils::{MouseCaptureState, toggle_mouse_capture};
//! use std::io;
//!
//! let mut stdout = io::stdout();
//! let mut capture_state = MouseCaptureState::new(true); // Start with capture enabled
//!
//! // Toggle mouse capture (e.g., when user presses 'm')
//! toggle_mouse_capture(&mut stdout, &mut capture_state)?;
//!
//! // Check if we're in copy mode (capture disabled)
//! if capture_state.is_copy_mode() {
//!     println!("Select text with your mouse!");
//! }
//! ```

use std::io::{self, Write};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
};

/// State for mouse capture management
///
/// Tracks whether mouse capture is enabled and provides methods to toggle it.
/// When mouse capture is disabled, the terminal allows native text selection
/// (copy mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseCaptureState {
    /// Whether mouse capture is currently enabled
    enabled: bool,
}

impl MouseCaptureState {
    /// Create a new mouse capture state
    ///
    /// # Arguments
    /// * `enabled` - Initial state (true = capturing mouse events)
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Create a new state with mouse capture enabled
    pub fn enabled() -> Self {
        Self::new(true)
    }

    /// Create a new state with mouse capture disabled (copy mode)
    pub fn disabled() -> Self {
        Self::new(false)
    }

    /// Check if mouse capture is currently enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if copy mode is active (mouse capture disabled)
    ///
    /// When in copy mode, the terminal allows native text selection.
    pub fn is_copy_mode(&self) -> bool {
        !self.enabled
    }

    /// Set the mouse capture state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Toggle the mouse capture state and return the new state
    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }
}

impl Default for MouseCaptureState {
    fn default() -> Self {
        Self::enabled()
    }
}

/// Enable mouse capture
///
/// Sends the crossterm EnableMouseCapture command to the terminal.
///
/// # Errors
/// Returns an error if the command fails to execute.
pub fn enable_mouse_capture<W: Write>(writer: &mut W) -> io::Result<()> {
    execute!(writer, EnableMouseCapture)
}

/// Disable mouse capture
///
/// Sends the crossterm DisableMouseCapture command to the terminal.
/// When disabled, the terminal allows native text selection.
///
/// # Errors
/// Returns an error if the command fails to execute.
pub fn disable_mouse_capture<W: Write>(writer: &mut W) -> io::Result<()> {
    execute!(writer, DisableMouseCapture)
}

/// Toggle mouse capture and update state
///
/// Toggles between enabled and disabled mouse capture. When disabled,
/// the terminal allows native text selection (copy mode).
///
/// # Arguments
/// * `writer` - The terminal output writer
/// * `state` - The mouse capture state to update
///
/// # Returns
/// Ok(true) if capture is now enabled, Ok(false) if disabled (copy mode)
///
/// # Errors
/// Returns an error if the terminal command fails.
pub fn toggle_mouse_capture<W: Write>(
    writer: &mut W,
    state: &mut MouseCaptureState,
) -> io::Result<bool> {
    let new_enabled = state.toggle();
    if new_enabled {
        enable_mouse_capture(writer)?;
    } else {
        disable_mouse_capture(writer)?;
    }
    Ok(new_enabled)
}

/// Set mouse capture to a specific state
///
/// # Arguments
/// * `writer` - The terminal output writer
/// * `state` - The mouse capture state to update
/// * `enabled` - Whether to enable (true) or disable (false) capture
///
/// # Errors
/// Returns an error if the terminal command fails.
pub fn set_mouse_capture<W: Write>(
    writer: &mut W,
    state: &mut MouseCaptureState,
    enabled: bool,
) -> io::Result<()> {
    if state.is_enabled() != enabled {
        state.set_enabled(enabled);
        if enabled {
            enable_mouse_capture(writer)?;
        } else {
            disable_mouse_capture(writer)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_capture_state_new() {
        let enabled = MouseCaptureState::new(true);
        assert!(enabled.is_enabled());
        assert!(!enabled.is_copy_mode());

        let disabled = MouseCaptureState::new(false);
        assert!(!disabled.is_enabled());
        assert!(disabled.is_copy_mode());
    }

    #[test]
    fn test_mouse_capture_state_constructors() {
        let enabled = MouseCaptureState::enabled();
        assert!(enabled.is_enabled());

        let disabled = MouseCaptureState::disabled();
        assert!(!disabled.is_enabled());
    }

    #[test]
    fn test_mouse_capture_state_default() {
        let state = MouseCaptureState::default();
        assert!(state.is_enabled());
    }

    #[test]
    fn test_mouse_capture_state_toggle() {
        let mut state = MouseCaptureState::enabled();
        assert!(state.is_enabled());

        let result = state.toggle();
        assert!(!result);
        assert!(!state.is_enabled());
        assert!(state.is_copy_mode());

        let result = state.toggle();
        assert!(result);
        assert!(state.is_enabled());
    }

    #[test]
    fn test_mouse_capture_state_set_enabled() {
        let mut state = MouseCaptureState::enabled();

        state.set_enabled(false);
        assert!(!state.is_enabled());

        state.set_enabled(true);
        assert!(state.is_enabled());
    }

    #[test]
    fn test_enable_mouse_capture() {
        let mut buffer = Vec::new();
        enable_mouse_capture(&mut buffer).unwrap();
        // The buffer should contain escape sequences
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_disable_mouse_capture() {
        let mut buffer = Vec::new();
        disable_mouse_capture(&mut buffer).unwrap();
        // The buffer should contain escape sequences
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_toggle_mouse_capture() {
        let mut buffer = Vec::new();
        let mut state = MouseCaptureState::enabled();

        // Toggle to disabled
        let result = toggle_mouse_capture(&mut buffer, &mut state).unwrap();
        assert!(!result);
        assert!(state.is_copy_mode());

        // Toggle to enabled
        buffer.clear();
        let result = toggle_mouse_capture(&mut buffer, &mut state).unwrap();
        assert!(result);
        assert!(state.is_enabled());
    }

    #[test]
    fn test_set_mouse_capture() {
        let mut buffer = Vec::new();
        let mut state = MouseCaptureState::enabled();

        // Setting to same value should not write anything
        set_mouse_capture(&mut buffer, &mut state, true).unwrap();
        assert!(buffer.is_empty());

        // Setting to different value should write
        set_mouse_capture(&mut buffer, &mut state, false).unwrap();
        assert!(!buffer.is_empty());
        assert!(state.is_copy_mode());
    }
}
