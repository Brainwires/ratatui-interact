//! Focus Manager - Handles Tab navigation between focusable elements
//!
//! The `FocusManager` tracks which element has focus and handles
//! Tab/Shift+Tab navigation between elements.
//!
//! # Example
//!
//! ```rust
//! use tui_extension::state::FocusManager;
//!
//! #[derive(Clone, PartialEq, Eq, Hash, Debug)]
//! enum DialogElement {
//!     NameInput,
//!     EmailInput,
//!     SubmitButton,
//!     CancelButton,
//! }
//!
//! let mut focus = FocusManager::new();
//!
//! // Register elements in Tab order
//! focus.register(DialogElement::NameInput);
//! focus.register(DialogElement::EmailInput);
//! focus.register(DialogElement::SubmitButton);
//! focus.register(DialogElement::CancelButton);
//!
//! // First element is auto-focused
//! assert_eq!(focus.current(), Some(&DialogElement::NameInput));
//!
//! // Tab to next
//! focus.next();
//! assert_eq!(focus.current(), Some(&DialogElement::EmailInput));
//!
//! // Shift+Tab to previous
//! focus.prev();
//! assert_eq!(focus.current(), Some(&DialogElement::NameInput));
//!
//! // Tab wraps around
//! focus.last();
//! focus.next();
//! assert_eq!(focus.current(), Some(&DialogElement::NameInput));
//! ```

use std::hash::Hash;

/// Focus manager for Tab navigation.
///
/// Manages a list of focusable elements and tracks which one currently has focus.
/// Elements are navigated in registration order.
///
/// # Type Parameters
///
/// * `T` - The type used to identify focusable elements. Must implement
///   `Clone`, `Eq`, and `Hash`. Commonly an enum or integer type.
#[derive(Debug, Clone)]
pub struct FocusManager<T: Clone + Eq + Hash = usize> {
    /// Ordered list of focusable elements (by registration order).
    elements: Vec<T>,
    /// Current focus index.
    current_index: Option<usize>,
}

impl<T: Clone + Eq + Hash> Default for FocusManager<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Eq + Hash> FocusManager<T> {
    /// Create a new empty focus manager.
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            current_index: None,
        }
    }

    /// Create a new focus manager with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elements: Vec::with_capacity(capacity),
            current_index: None,
        }
    }

    /// Register a focusable element.
    ///
    /// Elements are added to the end of the navigation order.
    /// Duplicate elements are ignored.
    ///
    /// The first registered element is automatically focused.
    pub fn register(&mut self, element: T) {
        if !self.elements.contains(&element) {
            self.elements.push(element);
            // Auto-focus first element
            if self.current_index.is_none() {
                self.current_index = Some(0);
            }
        }
    }

    /// Register multiple elements at once.
    pub fn register_all(&mut self, elements: impl IntoIterator<Item = T>) {
        for element in elements {
            self.register(element);
        }
    }

    /// Clear all registered elements and reset focus.
    pub fn clear(&mut self) {
        self.elements.clear();
        self.current_index = None;
    }

    /// Get the currently focused element.
    ///
    /// Returns `None` if no elements are registered.
    pub fn current(&self) -> Option<&T> {
        self.current_index.and_then(|i| self.elements.get(i))
    }

    /// Get the current focus index.
    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    /// Check if an element is currently focused.
    pub fn is_focused(&self, element: &T) -> bool {
        self.current() == Some(element)
    }

    /// Move focus to the next element.
    ///
    /// Wraps around to the first element after the last.
    pub fn next(&mut self) {
        if self.elements.is_empty() {
            return;
        }

        self.current_index = Some(
            self.current_index
                .map(|i| (i + 1) % self.elements.len())
                .unwrap_or(0),
        );
    }

    /// Move focus to the previous element.
    ///
    /// Wraps around to the last element before the first.
    pub fn prev(&mut self) {
        if self.elements.is_empty() {
            return;
        }

        self.current_index = Some(
            self.current_index
                .map(|i| {
                    if i == 0 {
                        self.elements.len() - 1
                    } else {
                        i - 1
                    }
                })
                .unwrap_or(0),
        );
    }

    /// Set focus to a specific element.
    ///
    /// If the element is not registered, focus is unchanged.
    pub fn set(&mut self, element: T) {
        if let Some(idx) = self.elements.iter().position(|e| *e == element) {
            self.current_index = Some(idx);
        }
    }

    /// Set focus by index.
    ///
    /// If the index is out of bounds, focus is unchanged.
    pub fn set_index(&mut self, index: usize) {
        if index < self.elements.len() {
            self.current_index = Some(index);
        }
    }

    /// Focus the first element.
    pub fn first(&mut self) {
        if !self.elements.is_empty() {
            self.current_index = Some(0);
        }
    }

    /// Focus the last element.
    pub fn last(&mut self) {
        if !self.elements.is_empty() {
            self.current_index = Some(self.elements.len() - 1);
        }
    }

    /// Remove focus (no element focused).
    pub fn unfocus(&mut self) {
        self.current_index = None;
    }

    /// Check if any element has focus.
    pub fn has_focus(&self) -> bool {
        self.current_index.is_some()
    }

    /// Get the number of registered elements.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if no elements are registered.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Get all registered elements.
    pub fn elements(&self) -> &[T] {
        &self.elements
    }

    /// Remove an element from the focus manager.
    ///
    /// If the removed element was focused, focus moves to the next element
    /// (or previous if it was the last).
    pub fn remove(&mut self, element: &T) -> bool {
        if let Some(idx) = self.elements.iter().position(|e| e == element) {
            self.elements.remove(idx);

            // Adjust current index
            if self.elements.is_empty() {
                self.current_index = None;
            } else if let Some(current) = self.current_index {
                if current == idx {
                    // Was focused - stay at same index (now next element)
                    // or move back if we removed the last
                    if current >= self.elements.len() {
                        self.current_index = Some(self.elements.len() - 1);
                    }
                } else if current > idx {
                    // Adjust index for removed element
                    self.current_index = Some(current - 1);
                }
            }

            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    enum TestElement {
        First,
        Second,
        Third,
    }

    #[test]
    fn test_new_manager() {
        let manager: FocusManager<usize> = FocusManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        assert_eq!(manager.current(), None);
        assert!(!manager.has_focus());
    }

    #[test]
    fn test_register_auto_focus() {
        let mut manager = FocusManager::new();
        manager.register(TestElement::First);

        assert_eq!(manager.len(), 1);
        assert!(manager.has_focus());
        assert_eq!(manager.current(), Some(&TestElement::First));
    }

    #[test]
    fn test_register_duplicates_ignored() {
        let mut manager = FocusManager::new();
        manager.register(TestElement::First);
        manager.register(TestElement::First); // Duplicate

        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_register_all() {
        let mut manager = FocusManager::new();
        manager.register_all([TestElement::First, TestElement::Second, TestElement::Third]);

        assert_eq!(manager.len(), 3);
        assert_eq!(manager.current(), Some(&TestElement::First));
    }

    #[test]
    fn test_next_navigation() {
        let mut manager = FocusManager::new();
        manager.register_all([TestElement::First, TestElement::Second, TestElement::Third]);

        assert_eq!(manager.current(), Some(&TestElement::First));

        manager.next();
        assert_eq!(manager.current(), Some(&TestElement::Second));

        manager.next();
        assert_eq!(manager.current(), Some(&TestElement::Third));

        // Wrap around
        manager.next();
        assert_eq!(manager.current(), Some(&TestElement::First));
    }

    #[test]
    fn test_prev_navigation() {
        let mut manager = FocusManager::new();
        manager.register_all([TestElement::First, TestElement::Second, TestElement::Third]);

        // Wrap around from first to last
        manager.prev();
        assert_eq!(manager.current(), Some(&TestElement::Third));

        manager.prev();
        assert_eq!(manager.current(), Some(&TestElement::Second));

        manager.prev();
        assert_eq!(manager.current(), Some(&TestElement::First));
    }

    #[test]
    fn test_set_focus() {
        let mut manager = FocusManager::new();
        manager.register_all([TestElement::First, TestElement::Second, TestElement::Third]);

        manager.set(TestElement::Third);
        assert_eq!(manager.current(), Some(&TestElement::Third));

        manager.set(TestElement::First);
        assert_eq!(manager.current(), Some(&TestElement::First));
    }

    #[test]
    fn test_set_index() {
        let mut manager = FocusManager::new();
        manager.register_all([TestElement::First, TestElement::Second, TestElement::Third]);

        manager.set_index(2);
        assert_eq!(manager.current(), Some(&TestElement::Third));
        assert_eq!(manager.current_index(), Some(2));

        // Out of bounds - no change
        manager.set_index(10);
        assert_eq!(manager.current(), Some(&TestElement::Third));
    }

    #[test]
    fn test_first_last() {
        let mut manager = FocusManager::new();
        manager.register_all([TestElement::First, TestElement::Second, TestElement::Third]);

        manager.last();
        assert_eq!(manager.current(), Some(&TestElement::Third));

        manager.first();
        assert_eq!(manager.current(), Some(&TestElement::First));
    }

    #[test]
    fn test_unfocus() {
        let mut manager = FocusManager::new();
        manager.register(TestElement::First);

        assert!(manager.has_focus());

        manager.unfocus();
        assert!(!manager.has_focus());
        assert_eq!(manager.current(), None);
    }

    #[test]
    fn test_is_focused() {
        let mut manager = FocusManager::new();
        manager.register_all([TestElement::First, TestElement::Second]);

        assert!(manager.is_focused(&TestElement::First));
        assert!(!manager.is_focused(&TestElement::Second));

        manager.next();
        assert!(!manager.is_focused(&TestElement::First));
        assert!(manager.is_focused(&TestElement::Second));
    }

    #[test]
    fn test_clear() {
        let mut manager = FocusManager::new();
        manager.register_all([TestElement::First, TestElement::Second]);

        manager.clear();
        assert!(manager.is_empty());
        assert!(!manager.has_focus());
    }

    #[test]
    fn test_remove_unfocused() {
        let mut manager = FocusManager::new();
        manager.register_all([TestElement::First, TestElement::Second, TestElement::Third]);

        // Remove unfocused element
        let removed = manager.remove(&TestElement::Third);
        assert!(removed);
        assert_eq!(manager.len(), 2);
        assert_eq!(manager.current(), Some(&TestElement::First)); // Focus unchanged
    }

    #[test]
    fn test_remove_focused() {
        let mut manager = FocusManager::new();
        manager.register_all([TestElement::First, TestElement::Second, TestElement::Third]);

        // Remove focused element
        let removed = manager.remove(&TestElement::First);
        assert!(removed);
        assert_eq!(manager.len(), 2);
        assert_eq!(manager.current(), Some(&TestElement::Second)); // Focus moves to next
    }

    #[test]
    fn test_remove_last_focused() {
        let mut manager = FocusManager::new();
        manager.register_all([TestElement::First, TestElement::Second, TestElement::Third]);
        manager.last();

        // Remove last focused element
        let removed = manager.remove(&TestElement::Third);
        assert!(removed);
        assert_eq!(manager.current(), Some(&TestElement::Second)); // Focus moves back
    }

    #[test]
    fn test_empty_navigation() {
        let mut manager: FocusManager<usize> = FocusManager::new();

        // Should not panic on empty manager
        manager.next();
        manager.prev();
        manager.first();
        manager.last();

        assert!(!manager.has_focus());
    }

    #[test]
    fn test_integer_focus_manager() {
        let mut manager: FocusManager<usize> = FocusManager::new();
        manager.register_all([0, 1, 2, 3, 4]);

        assert_eq!(manager.current(), Some(&0));
        manager.next();
        assert_eq!(manager.current(), Some(&1));
    }
}
