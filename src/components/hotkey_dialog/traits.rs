//! Trait definitions for the hotkey dialog component.
//!
//! This module provides traits that allow applications to implement their own
//! hotkey categories and content while using the generic dialog widget.

/// A category for organizing hotkeys.
///
/// Implement this trait on your application's category enum to provide
/// category-based navigation in the hotkey dialog.
///
/// # Example
///
/// ```rust
/// use ratatui_interact::components::hotkey_dialog::HotkeyCategory;
///
/// #[derive(Clone, Copy, PartialEq, Eq, Default)]
/// enum MyCategory {
///     #[default]
///     Global,
///     Navigation,
///     Editing,
/// }
///
/// impl HotkeyCategory for MyCategory {
///     fn all() -> &'static [Self] {
///         &[Self::Global, Self::Navigation, Self::Editing]
///     }
///
///     fn display_name(&self) -> &str {
///         match self {
///             Self::Global => "Global",
///             Self::Navigation => "Navigation",
///             Self::Editing => "Editing",
///         }
///     }
///
///     fn next(&self) -> Self {
///         let all = Self::all();
///         let idx = all.iter().position(|c| c == self).unwrap_or(0);
///         all[(idx + 1) % all.len()]
///     }
///
///     fn prev(&self) -> Self {
///         let all = Self::all();
///         let idx = all.iter().position(|c| c == self).unwrap_or(0);
///         all[(idx + all.len() - 1) % all.len()]
///     }
/// }
/// ```
pub trait HotkeyCategory: Clone + Copy + PartialEq + Default + 'static {
    /// Get all categories in display order.
    fn all() -> &'static [Self];

    /// Get the display name for this category.
    fn display_name(&self) -> &str;

    /// Get an optional icon/emoji for this category.
    /// Defaults to empty string.
    fn icon(&self) -> &str {
        ""
    }

    /// Get the next category (wraps around).
    fn next(&self) -> Self;

    /// Get the previous category (wraps around).
    fn prev(&self) -> Self;
}

/// Generic representation of a hotkey entry.
///
/// This struct holds the display data for a single hotkey, independent
/// of any application-specific context types.
#[derive(Debug, Clone)]
pub struct HotkeyEntryData {
    /// The keyboard shortcut (e.g., "Ctrl+C", "F1")
    pub key_combination: String,
    /// Description of what the hotkey does
    pub action: String,
    /// Context description (e.g., "Global", "Normal Mode")
    pub context: String,
    /// Whether this is a global hotkey (affects styling)
    pub is_global: bool,
    /// Whether this hotkey can be customized
    pub is_customizable: bool,
}

impl HotkeyEntryData {
    /// Create a new hotkey entry.
    pub fn new(
        key_combination: impl Into<String>,
        action: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            key_combination: key_combination.into(),
            action: action.into(),
            context: context.into(),
            is_global: false,
            is_customizable: true,
        }
    }

    /// Create a global hotkey entry.
    pub fn global(key_combination: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            key_combination: key_combination.into(),
            action: action.into(),
            context: "Global".to_string(),
            is_global: true,
            is_customizable: true,
        }
    }

    /// Mark this entry as non-customizable.
    pub fn fixed(mut self) -> Self {
        self.is_customizable = false;
        self
    }

    /// Mark this entry as global.
    pub fn with_global(mut self, is_global: bool) -> Self {
        self.is_global = is_global;
        self
    }
}

/// Provider of hotkey entries for the dialog.
///
/// Implement this trait to supply hotkey data to the dialog widget.
/// This allows full control over how hotkeys are stored and retrieved.
///
/// # Example
///
/// ```rust,ignore
/// use ratatui_interact::components::hotkey_dialog::{HotkeyEntryData, HotkeyProvider};
///
/// struct MyHotkeyProvider;
///
/// impl HotkeyProvider for MyHotkeyProvider {
///     type Category = MyCategory;
///
///     fn entries_for_category(&self, category: Self::Category) -> Vec<HotkeyEntryData> {
///         match category {
///             MyCategory::Global => vec![
///                 HotkeyEntryData::global("Ctrl+C", "Quit application"),
///                 HotkeyEntryData::global("F1", "Show help"),
///             ],
///             // ...other categories
///         }
///     }
///
///     fn search(&self, query: &str) -> Vec<(Self::Category, HotkeyEntryData)> {
///         // Search implementation
///     }
/// }
/// ```
pub trait HotkeyProvider {
    /// The category type this provider uses.
    type Category: HotkeyCategory;

    /// Get all hotkey entries for a specific category.
    fn entries_for_category(&self, category: Self::Category) -> Vec<HotkeyEntryData>;

    /// Search across all categories for matching entries.
    ///
    /// Returns tuples of (category, entry) for each match.
    /// The search should be case-insensitive and match against:
    /// - Key combination
    /// - Action description
    /// - Context string
    fn search(&self, query: &str) -> Vec<(Self::Category, HotkeyEntryData)>;

    /// Get the total number of hotkey entries.
    fn total_count(&self) -> usize {
        Self::Category::all()
            .iter()
            .map(|c| self.entries_for_category(*c).len())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Eq, Default)]
    enum TestCategory {
        #[default]
        General,
        Advanced,
    }

    impl HotkeyCategory for TestCategory {
        fn all() -> &'static [Self] {
            &[Self::General, Self::Advanced]
        }

        fn display_name(&self) -> &str {
            match self {
                Self::General => "General",
                Self::Advanced => "Advanced",
            }
        }

        fn next(&self) -> Self {
            match self {
                Self::General => Self::Advanced,
                Self::Advanced => Self::General,
            }
        }

        fn prev(&self) -> Self {
            match self {
                Self::General => Self::Advanced,
                Self::Advanced => Self::General,
            }
        }
    }

    #[test]
    fn test_category_navigation() {
        let cat = TestCategory::General;
        assert_eq!(cat.next(), TestCategory::Advanced);
        assert_eq!(cat.next().prev(), TestCategory::General);
    }

    #[test]
    fn test_entry_creation() {
        let entry = HotkeyEntryData::new("Ctrl+S", "Save file", "Normal");
        assert_eq!(entry.key_combination, "Ctrl+S");
        assert_eq!(entry.action, "Save file");
        assert!(!entry.is_global);
        assert!(entry.is_customizable);

        let global = HotkeyEntryData::global("Ctrl+C", "Quit");
        assert!(global.is_global);

        let fixed = HotkeyEntryData::new("Ctrl+C", "Quit", "Global").fixed();
        assert!(!fixed.is_customizable);
    }
}
