//! State management for the hotkey dialog.
//!
//! This module contains the state structures for tracking dialog focus,
//! search, scrolling, and selection.

use ratatui::layout::Rect;

use super::traits::{HotkeyCategory, HotkeyEntryData, HotkeyProvider};

/// Focus states within the hotkey dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HotkeyFocus {
    /// Search input field
    SearchInput,
    /// Category list on the left
    #[default]
    CategoryList,
    /// Hotkey list on the right
    HotkeyList,
}

impl HotkeyFocus {
    /// Move to next focus area.
    pub fn next(&self) -> Self {
        match self {
            HotkeyFocus::SearchInput => HotkeyFocus::CategoryList,
            HotkeyFocus::CategoryList => HotkeyFocus::HotkeyList,
            HotkeyFocus::HotkeyList => HotkeyFocus::SearchInput,
        }
    }

    /// Move to previous focus area.
    pub fn prev(&self) -> Self {
        match self {
            HotkeyFocus::SearchInput => HotkeyFocus::HotkeyList,
            HotkeyFocus::CategoryList => HotkeyFocus::SearchInput,
            HotkeyFocus::HotkeyList => HotkeyFocus::CategoryList,
        }
    }
}

/// Click region for category items.
#[derive(Debug, Clone)]
pub struct CategoryClickRegion<C> {
    pub area: Rect,
    pub category: C,
}

/// Click region for hotkey items.
#[derive(Debug, Clone)]
pub struct HotkeyClickRegion {
    pub area: Rect,
    pub index: usize,
}

/// State for the hotkey configuration dialog.
///
/// Generic over the category type `C` which must implement `HotkeyCategory`.
#[derive(Debug, Clone)]
pub struct HotkeyDialogState<C: HotkeyCategory> {
    /// Current search query
    pub search_query: String,
    /// Cursor position in search field
    pub search_cursor_pos: usize,
    /// Currently selected category
    pub selected_category: C,
    /// Scroll offset for category list (if needed)
    pub category_scroll: usize,
    /// Scroll offset for hotkey list
    pub hotkey_scroll: usize,
    /// Selected hotkey index within current view
    pub selected_hotkey_idx: usize,
    /// Currently focused area
    pub focus: HotkeyFocus,
    /// Click regions for categories (populated during render)
    pub category_click_regions: Vec<CategoryClickRegion<C>>,
    /// Click regions for hotkeys (populated during render)
    pub hotkey_click_regions: Vec<HotkeyClickRegion>,
    /// Cached current entries count (updated during render)
    cached_entry_count: usize,
}

impl<C: HotkeyCategory> Default for HotkeyDialogState<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: HotkeyCategory> HotkeyDialogState<C> {
    /// Create a new hotkey dialog state.
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            search_cursor_pos: 0,
            selected_category: C::default(),
            category_scroll: 0,
            hotkey_scroll: 0,
            selected_hotkey_idx: 0,
            focus: HotkeyFocus::CategoryList,
            category_click_regions: Vec::new(),
            hotkey_click_regions: Vec::new(),
            cached_entry_count: 0,
        }
    }

    /// Move to next category.
    pub fn next_category(&mut self) {
        self.selected_category = self.selected_category.next();
        self.hotkey_scroll = 0;
        self.selected_hotkey_idx = 0;
    }

    /// Move to previous category.
    pub fn prev_category(&mut self) {
        self.selected_category = self.selected_category.prev();
        self.hotkey_scroll = 0;
        self.selected_hotkey_idx = 0;
    }

    /// Move to next hotkey in list.
    pub fn next_hotkey(&mut self) {
        if self.cached_entry_count > 0 {
            self.selected_hotkey_idx = (self.selected_hotkey_idx + 1).min(self.cached_entry_count - 1);
        }
    }

    /// Move to previous hotkey in list.
    pub fn prev_hotkey(&mut self) {
        if self.selected_hotkey_idx > 0 {
            self.selected_hotkey_idx -= 1;
        }
    }

    /// Move hotkey selection by a page (10 items).
    pub fn page_down(&mut self) {
        for _ in 0..10 {
            self.next_hotkey();
        }
    }

    /// Move hotkey selection by a page (10 items).
    pub fn page_up(&mut self) {
        for _ in 0..10 {
            self.prev_hotkey();
        }
    }

    /// Scroll hotkey list down.
    pub fn scroll_hotkeys_down(&mut self, amount: usize) {
        let max_scroll = self.cached_entry_count.saturating_sub(1);
        self.hotkey_scroll = (self.hotkey_scroll + amount).min(max_scroll);
    }

    /// Scroll hotkey list up.
    pub fn scroll_hotkeys_up(&mut self, amount: usize) {
        self.hotkey_scroll = self.hotkey_scroll.saturating_sub(amount);
    }

    /// Move to next focus area.
    pub fn focus_next(&mut self) {
        self.focus = self.focus.next();
    }

    /// Move to previous focus area.
    pub fn focus_prev(&mut self) {
        self.focus = self.focus.prev();
    }

    /// Check if we're in search mode.
    pub fn is_searching(&self) -> bool {
        !self.search_query.is_empty()
    }

    /// Get current entries using the provider.
    pub fn get_current_entries<P: HotkeyProvider<Category = C>>(
        &self,
        provider: &P,
    ) -> Vec<HotkeyEntryData> {
        if self.is_searching() {
            provider
                .search(&self.search_query)
                .into_iter()
                .map(|(_, entry)| entry)
                .collect()
        } else {
            provider.entries_for_category(self.selected_category)
        }
    }

    /// Get search results using the provider.
    pub fn get_search_results<P: HotkeyProvider<Category = C>>(
        &self,
        provider: &P,
    ) -> Vec<(C, HotkeyEntryData)> {
        if self.search_query.is_empty() {
            return vec![];
        }
        provider.search(&self.search_query)
    }

    /// Get the selected entry using the provider.
    pub fn get_selected_entry<P: HotkeyProvider<Category = C>>(
        &self,
        provider: &P,
    ) -> Option<HotkeyEntryData> {
        let entries = self.get_current_entries(provider);
        entries.get(self.selected_hotkey_idx).cloned()
    }

    /// Update the cached entry count (call during render).
    pub fn update_entry_count(&mut self, count: usize) {
        self.cached_entry_count = count;
    }

    /// Insert a character into the search query.
    pub fn insert_char(&mut self, c: char) {
        let byte_pos = self.char_to_byte_index(self.search_cursor_pos);
        self.search_query.insert(byte_pos, c);
        self.search_cursor_pos += 1;
        self.hotkey_scroll = 0;
        self.selected_hotkey_idx = 0;
    }

    /// Delete character before cursor.
    pub fn delete_char_backward(&mut self) -> bool {
        if self.search_cursor_pos == 0 {
            return false;
        }

        self.search_cursor_pos -= 1;
        let byte_pos = self.char_to_byte_index(self.search_cursor_pos);
        if let Some(c) = self.search_query[byte_pos..].chars().next() {
            self.search_query
                .replace_range(byte_pos..byte_pos + c.len_utf8(), "");
            self.selected_hotkey_idx = 0;
            return true;
        }
        false
    }

    /// Delete character at cursor.
    pub fn delete_char_forward(&mut self) -> bool {
        let byte_pos = self.char_to_byte_index(self.search_cursor_pos);
        if byte_pos < self.search_query.len() {
            if let Some(c) = self.search_query[byte_pos..].chars().next() {
                self.search_query
                    .replace_range(byte_pos..byte_pos + c.len_utf8(), "");
                self.selected_hotkey_idx = 0;
                return true;
            }
        }
        false
    }

    /// Move cursor left.
    pub fn move_cursor_left(&mut self) {
        if self.search_cursor_pos > 0 {
            self.search_cursor_pos -= 1;
        }
    }

    /// Move cursor right.
    pub fn move_cursor_right(&mut self) {
        let max = self.search_query.chars().count();
        if self.search_cursor_pos < max {
            self.search_cursor_pos += 1;
        }
    }

    /// Move cursor to start.
    pub fn move_cursor_home(&mut self) {
        self.search_cursor_pos = 0;
    }

    /// Move cursor to end.
    pub fn move_cursor_end(&mut self) {
        self.search_cursor_pos = self.search_query.chars().count();
    }

    /// Clear search query.
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_cursor_pos = 0;
        self.hotkey_scroll = 0;
        self.selected_hotkey_idx = 0;
    }

    /// Convert character index to byte index.
    fn char_to_byte_index(&self, char_idx: usize) -> usize {
        self.search_query
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.search_query.len())
    }

    /// Get text before cursor for rendering.
    pub fn text_before_cursor(&self) -> &str {
        let byte_pos = self.char_to_byte_index(self.search_cursor_pos);
        &self.search_query[..byte_pos]
    }

    /// Get text after cursor for rendering.
    pub fn text_after_cursor(&self) -> &str {
        let byte_pos = self.char_to_byte_index(self.search_cursor_pos);
        &self.search_query[byte_pos..]
    }

    /// Clear click regions (call before render).
    pub fn clear_click_regions(&mut self) {
        self.category_click_regions.clear();
        self.hotkey_click_regions.clear();
    }

    /// Add a click region for a category.
    pub fn add_category_click_region(&mut self, area: Rect, category: C) {
        self.category_click_regions
            .push(CategoryClickRegion { area, category });
    }

    /// Add a click region for a hotkey.
    pub fn add_hotkey_click_region(&mut self, area: Rect, index: usize) {
        self.hotkey_click_regions
            .push(HotkeyClickRegion { area, index });
    }

    /// Handle a click at the given position.
    /// Returns true if something was clicked.
    pub fn handle_click(&mut self, col: u16, row: u16) -> bool {
        // Check category click regions
        for region in &self.category_click_regions {
            if col >= region.area.x
                && col < region.area.x + region.area.width
                && row >= region.area.y
                && row < region.area.y + region.area.height
            {
                self.selected_category = region.category;
                self.hotkey_scroll = 0;
                self.selected_hotkey_idx = 0;
                self.focus = HotkeyFocus::CategoryList;
                return true;
            }
        }

        // Check hotkey click regions
        for region in &self.hotkey_click_regions {
            if col >= region.area.x
                && col < region.area.x + region.area.width
                && row >= region.area.y
                && row < region.area.y + region.area.height
            {
                self.selected_hotkey_idx = region.index;
                self.focus = HotkeyFocus::HotkeyList;
                return true;
            }
        }

        false
    }

    /// Ensure selected hotkey is visible in scroll view.
    pub fn ensure_hotkey_visible(&mut self, visible_height: usize) {
        if self.selected_hotkey_idx < self.hotkey_scroll {
            self.hotkey_scroll = self.selected_hotkey_idx;
        } else if self.selected_hotkey_idx >= self.hotkey_scroll + visible_height {
            self.hotkey_scroll = self.selected_hotkey_idx - visible_height + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
    enum TestCategory {
        #[default]
        First,
        Second,
    }

    impl HotkeyCategory for TestCategory {
        fn all() -> &'static [Self] {
            &[Self::First, Self::Second]
        }

        fn display_name(&self) -> &str {
            match self {
                Self::First => "First",
                Self::Second => "Second",
            }
        }

        fn next(&self) -> Self {
            match self {
                Self::First => Self::Second,
                Self::Second => Self::First,
            }
        }

        fn prev(&self) -> Self {
            match self {
                Self::First => Self::Second,
                Self::Second => Self::First,
            }
        }
    }

    #[test]
    fn test_new_state() {
        let state: HotkeyDialogState<TestCategory> = HotkeyDialogState::new();
        assert!(state.search_query.is_empty());
        assert_eq!(state.focus, HotkeyFocus::CategoryList);
        assert_eq!(state.selected_category, TestCategory::First);
    }

    #[test]
    fn test_category_navigation() {
        let mut state: HotkeyDialogState<TestCategory> = HotkeyDialogState::new();
        assert_eq!(state.selected_category, TestCategory::First);

        state.next_category();
        assert_eq!(state.selected_category, TestCategory::Second);

        state.prev_category();
        assert_eq!(state.selected_category, TestCategory::First);
    }

    #[test]
    fn test_focus_cycling() {
        let mut state: HotkeyDialogState<TestCategory> = HotkeyDialogState::new();
        state.focus = HotkeyFocus::CategoryList;

        state.focus_next();
        assert_eq!(state.focus, HotkeyFocus::HotkeyList);

        state.focus_next();
        assert_eq!(state.focus, HotkeyFocus::SearchInput);

        state.focus_next();
        assert_eq!(state.focus, HotkeyFocus::CategoryList);
    }

    #[test]
    fn test_search_input() {
        let mut state: HotkeyDialogState<TestCategory> = HotkeyDialogState::new();

        state.insert_char('c');
        state.insert_char('t');
        state.insert_char('r');
        state.insert_char('l');

        assert_eq!(state.search_query, "ctrl");
        assert_eq!(state.search_cursor_pos, 4);

        state.delete_char_backward();
        assert_eq!(state.search_query, "ctr");
        assert_eq!(state.search_cursor_pos, 3);
    }

    #[test]
    fn test_cursor_movement() {
        let mut state: HotkeyDialogState<TestCategory> = HotkeyDialogState::new();
        state.search_query = "test".to_string();
        state.search_cursor_pos = 4;

        state.move_cursor_left();
        assert_eq!(state.search_cursor_pos, 3);

        state.move_cursor_home();
        assert_eq!(state.search_cursor_pos, 0);

        state.move_cursor_end();
        assert_eq!(state.search_cursor_pos, 4);
    }

    #[test]
    fn test_is_searching() {
        let mut state: HotkeyDialogState<TestCategory> = HotkeyDialogState::new();
        assert!(!state.is_searching());

        state.insert_char('a');
        assert!(state.is_searching());

        state.clear_search();
        assert!(!state.is_searching());
    }
}
