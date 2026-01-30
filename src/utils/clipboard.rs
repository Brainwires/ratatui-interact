//! Clipboard utilities
//!
//! Provides cross-platform clipboard operations for copy/paste functionality.
//! This module is gated behind the `clipboard` feature flag since the `arboard`
//! crate requires system libraries.
//!
//! # Example
//!
//! ```rust,ignore
//! use ratatui_interact::utils::{copy_to_clipboard, get_from_clipboard, ClipboardResult};
//!
//! // Copy text to clipboard
//! match copy_to_clipboard("Hello, world!") {
//!     ClipboardResult::Success => println!("Copied!"),
//!     ClipboardResult::Error(e) => println!("Failed: {}", e),
//!     ClipboardResult::NotAvailable => println!("Clipboard not available"),
//! }
//!
//! // Paste from clipboard
//! match get_from_clipboard() {
//!     Ok(text) => println!("Pasted: {}", text),
//!     Err(result) => println!("Failed: {:?}", result),
//! }
//! ```
//!
//! # Feature Flag
//!
//! Enable clipboard support by adding the `clipboard` feature:
//!
//! ```toml
//! [dependencies]
//! ratatui-interact = { version = "0.4", features = ["clipboard"] }
//! ```

/// Result of a clipboard operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardResult {
    /// Operation succeeded
    Success,
    /// Operation failed with an error message
    Error(String),
    /// Clipboard functionality is not available
    ///
    /// This occurs when the `clipboard` feature is not enabled,
    /// or when the system clipboard cannot be accessed.
    NotAvailable,
}

impl ClipboardResult {
    /// Check if the operation was successful
    pub fn is_success(&self) -> bool {
        matches!(self, ClipboardResult::Success)
    }

    /// Check if the operation failed
    pub fn is_error(&self) -> bool {
        matches!(self, ClipboardResult::Error(_))
    }

    /// Check if clipboard is not available
    pub fn is_not_available(&self) -> bool {
        matches!(self, ClipboardResult::NotAvailable)
    }

    /// Get the error message if this is an error
    pub fn error_message(&self) -> Option<&str> {
        match self {
            ClipboardResult::Error(msg) => Some(msg),
            _ => None,
        }
    }
}

impl std::fmt::Display for ClipboardResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardResult::Success => write!(f, "Success"),
            ClipboardResult::Error(e) => write!(f, "Error: {}", e),
            ClipboardResult::NotAvailable => write!(f, "Clipboard not available"),
        }
    }
}

/// Copy text to the system clipboard
///
/// # Arguments
/// * `text` - The text to copy
///
/// # Returns
/// * `ClipboardResult::Success` if the text was copied successfully
/// * `ClipboardResult::Error(message)` if the copy failed
/// * `ClipboardResult::NotAvailable` if clipboard is not available
///
/// # Example
///
/// ```rust,ignore
/// use ratatui_interact::utils::{copy_to_clipboard, ClipboardResult};
///
/// let result = copy_to_clipboard("Hello, clipboard!");
/// if result.is_success() {
///     println!("Text copied!");
/// }
/// ```
#[cfg(feature = "clipboard")]
pub fn copy_to_clipboard(text: &str) -> ClipboardResult {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => match clipboard.set_text(text) {
            Ok(()) => ClipboardResult::Success,
            Err(e) => ClipboardResult::Error(e.to_string()),
        },
        Err(e) => ClipboardResult::Error(format!("Failed to access clipboard: {}", e)),
    }
}

#[cfg(not(feature = "clipboard"))]
pub fn copy_to_clipboard(_text: &str) -> ClipboardResult {
    ClipboardResult::NotAvailable
}

/// Get text from the system clipboard
///
/// # Returns
/// * `Ok(String)` with the clipboard contents if successful
/// * `Err(ClipboardResult::Error(message))` if reading failed
/// * `Err(ClipboardResult::NotAvailable)` if clipboard is not available
///
/// # Example
///
/// ```rust,ignore
/// use ratatui_interact::utils::get_from_clipboard;
///
/// match get_from_clipboard() {
///     Ok(text) => println!("Clipboard: {}", text),
///     Err(e) => eprintln!("Failed: {}", e),
/// }
/// ```
#[cfg(feature = "clipboard")]
pub fn get_from_clipboard() -> Result<String, ClipboardResult> {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => match clipboard.get_text() {
            Ok(text) => Ok(text),
            Err(e) => Err(ClipboardResult::Error(e.to_string())),
        },
        Err(e) => Err(ClipboardResult::Error(format!(
            "Failed to access clipboard: {}",
            e
        ))),
    }
}

#[cfg(not(feature = "clipboard"))]
pub fn get_from_clipboard() -> Result<String, ClipboardResult> {
    Err(ClipboardResult::NotAvailable)
}

/// Check if clipboard functionality is available
///
/// Returns `true` if the `clipboard` feature is enabled and the system
/// clipboard can be accessed.
///
/// # Example
///
/// ```rust,ignore
/// use ratatui_interact::utils::is_clipboard_available;
///
/// if is_clipboard_available() {
///     println!("Clipboard operations supported");
/// } else {
///     println!("Clipboard not available");
/// }
/// ```
#[cfg(feature = "clipboard")]
pub fn is_clipboard_available() -> bool {
    arboard::Clipboard::new().is_ok()
}

#[cfg(not(feature = "clipboard"))]
pub fn is_clipboard_available() -> bool {
    false
}

/// Copy multiple lines to the clipboard, joining with newlines
///
/// # Arguments
/// * `lines` - Iterator of lines to copy
///
/// # Returns
/// * `ClipboardResult::Success` if the text was copied successfully
/// * `ClipboardResult::Error(message)` if the copy failed
/// * `ClipboardResult::NotAvailable` if clipboard is not available
///
/// # Example
///
/// ```rust,ignore
/// use ratatui_interact::utils::copy_lines_to_clipboard;
///
/// let lines = vec!["Line 1", "Line 2", "Line 3"];
/// copy_lines_to_clipboard(lines.iter().copied());
/// ```
pub fn copy_lines_to_clipboard<'a, I>(lines: I) -> ClipboardResult
where
    I: Iterator<Item = &'a str>,
{
    let text: String = lines.collect::<Vec<_>>().join("\n");
    copy_to_clipboard(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_result_is_success() {
        assert!(ClipboardResult::Success.is_success());
        assert!(!ClipboardResult::Error("test".into()).is_success());
        assert!(!ClipboardResult::NotAvailable.is_success());
    }

    #[test]
    fn test_clipboard_result_is_error() {
        assert!(!ClipboardResult::Success.is_error());
        assert!(ClipboardResult::Error("test".into()).is_error());
        assert!(!ClipboardResult::NotAvailable.is_error());
    }

    #[test]
    fn test_clipboard_result_is_not_available() {
        assert!(!ClipboardResult::Success.is_not_available());
        assert!(!ClipboardResult::Error("test".into()).is_not_available());
        assert!(ClipboardResult::NotAvailable.is_not_available());
    }

    #[test]
    fn test_clipboard_result_error_message() {
        assert_eq!(ClipboardResult::Success.error_message(), None);
        assert_eq!(
            ClipboardResult::Error("test".into()).error_message(),
            Some("test")
        );
        assert_eq!(ClipboardResult::NotAvailable.error_message(), None);
    }

    #[test]
    fn test_clipboard_result_display() {
        assert_eq!(format!("{}", ClipboardResult::Success), "Success");
        assert_eq!(
            format!("{}", ClipboardResult::Error("oops".into())),
            "Error: oops"
        );
        assert_eq!(
            format!("{}", ClipboardResult::NotAvailable),
            "Clipboard not available"
        );
    }

    #[cfg(not(feature = "clipboard"))]
    #[test]
    fn test_clipboard_not_available_without_feature() {
        assert!(!is_clipboard_available());
        assert!(copy_to_clipboard("test").is_not_available());
        assert!(get_from_clipboard().is_err());
    }

    #[test]
    fn test_copy_lines_to_clipboard() {
        let lines = vec!["a", "b", "c"];
        // Just verify it doesn't panic - actual clipboard access may not be available in tests
        let _ = copy_lines_to_clipboard(lines.iter().copied());
    }
}
