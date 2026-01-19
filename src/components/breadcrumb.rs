//! Breadcrumb component - Hierarchical navigation path display
//!
//! A breadcrumb component that displays hierarchical navigation paths with
//! support for ellipsis collapsing, keyboard/mouse interaction, and customizable styling.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{Breadcrumb, BreadcrumbItem, BreadcrumbState, BreadcrumbStyle};
//! use ratatui::layout::Rect;
//!
//! // Create breadcrumb items
//! let items = vec![
//!     BreadcrumbItem::new("home", "Home"),
//!     BreadcrumbItem::new("settings", "Settings"),
//!     BreadcrumbItem::new("profile", "Profile"),
//! ];
//!
//! // Create state
//! let mut state = BreadcrumbState::new(items);
//!
//! // Create breadcrumb widget with chevron style
//! let breadcrumb = Breadcrumb::new(&state)
//!     .style(BreadcrumbStyle::chevron());
//!
//! // Render and handle events (see handle_breadcrumb_key, handle_breadcrumb_mouse)
//! ```

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::traits::ClickRegion;

/// Actions a breadcrumb component can emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BreadcrumbAction {
    /// Navigate to an item by ID.
    Navigate(String),
    /// Expand collapsed items (show all).
    ExpandEllipsis,
}

/// A single item in the breadcrumb path.
#[derive(Debug, Clone)]
pub struct BreadcrumbItem {
    /// Unique identifier for click actions.
    pub id: String,
    /// Display text.
    pub label: String,
    /// Optional icon prefix (emoji or symbol).
    pub icon: Option<String>,
    /// Can this item be clicked?
    pub enabled: bool,
}

impl BreadcrumbItem {
    /// Create a new breadcrumb item.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this item
    /// * `label` - Display text
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            enabled: true,
        }
    }

    /// Set an icon for this item.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set whether this item is enabled (clickable).
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// State for a breadcrumb component.
#[derive(Debug, Clone)]
pub struct BreadcrumbState {
    /// All breadcrumb items.
    pub items: Vec<BreadcrumbItem>,
    /// Currently selected/highlighted item (for keyboard navigation).
    pub selected_index: Option<usize>,
    /// Whether the component has keyboard focus.
    pub focused: bool,
    /// Whether navigation is enabled.
    pub enabled: bool,
    /// Whether ellipsis is expanded (showing all items).
    pub expanded: bool,
}

impl Default for BreadcrumbState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected_index: None,
            focused: false,
            enabled: true,
            expanded: false,
        }
    }
}

impl BreadcrumbState {
    /// Create a new breadcrumb state with the given items.
    pub fn new(items: Vec<BreadcrumbItem>) -> Self {
        Self {
            items,
            ..Default::default()
        }
    }

    /// Create an empty breadcrumb state.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Select the next item (move right).
    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected_index = Some(match self.selected_index {
            Some(idx) if idx + 1 < self.items.len() => idx + 1,
            Some(idx) => idx, // Stay at end
            None => 0,
        });
    }

    /// Select the previous item (move left).
    pub fn select_prev(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected_index = Some(match self.selected_index {
            Some(idx) if idx > 0 => idx - 1,
            Some(idx) => idx, // Stay at start
            None => self.items.len().saturating_sub(1),
        });
    }

    /// Select an item by index.
    pub fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = Some(index);
        }
    }

    /// Select an item by ID.
    pub fn select_by_id(&mut self, id: &str) {
        if let Some(idx) = self.items.iter().position(|item| item.id == id) {
            self.selected_index = Some(idx);
        }
    }

    /// Select the first item.
    pub fn select_first(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = Some(0);
        }
    }

    /// Select the last item.
    pub fn select_last(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = Some(self.items.len() - 1);
        }
    }

    /// Clear the selection.
    pub fn clear_selection(&mut self) {
        self.selected_index = None;
    }

    /// Push a new item to the end of the path.
    pub fn push(&mut self, item: BreadcrumbItem) {
        self.items.push(item);
    }

    /// Pop the last item from the path.
    pub fn pop(&mut self) -> Option<BreadcrumbItem> {
        let item = self.items.pop();
        // Adjust selection if it was pointing to the removed item
        if let Some(idx) = self.selected_index {
            if idx >= self.items.len() && !self.items.is_empty() {
                self.selected_index = Some(self.items.len() - 1);
            } else if self.items.is_empty() {
                self.selected_index = None;
            }
        }
        item
    }

    /// Clear all items.
    pub fn clear(&mut self) {
        self.items.clear();
        self.selected_index = None;
        self.expanded = false;
    }

    /// Set new items, replacing existing ones.
    pub fn set_items(&mut self, items: Vec<BreadcrumbItem>) {
        self.items = items;
        // Reset selection if it's now out of bounds
        if let Some(idx) = self.selected_index {
            if idx >= self.items.len() {
                self.selected_index = if self.items.is_empty() {
                    None
                } else {
                    Some(self.items.len() - 1)
                };
            }
        }
        self.expanded = false;
    }

    /// Toggle expanded state (show/hide collapsed items).
    pub fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }

    /// Get the currently selected item.
    pub fn selected_item(&self) -> Option<&BreadcrumbItem> {
        self.selected_index.and_then(|idx| self.items.get(idx))
    }

    /// Get the number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Style configuration for breadcrumb component.
#[derive(Debug, Clone)]
pub struct BreadcrumbStyle {
    /// Separator between items (e.g., " > ", " / ", " › ").
    pub separator: &'static str,
    /// Style for the separator.
    pub separator_style: Style,

    /// Ellipsis string when items are collapsed.
    pub ellipsis: &'static str,
    /// Style for the ellipsis.
    pub ellipsis_style: Style,
    /// Number of items before collapsing occurs.
    pub collapse_threshold: usize,
    /// Number of items to show at start when collapsed.
    pub visible_start: usize,
    /// Number of items to show at end when collapsed.
    pub visible_end: usize,

    /// Style for normal items.
    pub item_style: Style,
    /// Style for keyboard-focused item.
    pub focused_item_style: Style,
    /// Style for currently selected/active item.
    pub selected_item_style: Style,
    /// Style for mouse-hovered item.
    pub hovered_item_style: Style,
    /// Style for disabled items.
    pub disabled_item_style: Style,
    /// Special style for the last item (current location).
    pub last_item_style: Style,

    /// Style for icons.
    pub icon_style: Style,
    /// Separator between icon and label.
    pub icon_separator: &'static str,

    /// Horizontal padding (left, right).
    pub padding: (u16, u16),
}

impl Default for BreadcrumbStyle {
    fn default() -> Self {
        Self {
            separator: " > ",
            separator_style: Style::default().fg(Color::DarkGray),

            ellipsis: "...",
            ellipsis_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            collapse_threshold: 4,
            visible_start: 1,
            visible_end: 2,

            item_style: Style::default().fg(Color::Blue),
            focused_item_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            selected_item_style: Style::default().fg(Color::Black).bg(Color::Yellow),
            hovered_item_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
            disabled_item_style: Style::default().fg(Color::DarkGray),
            last_item_style: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),

            icon_style: Style::default(),
            icon_separator: " ",

            padding: (1, 1),
        }
    }
}

impl BreadcrumbStyle {
    /// Style with slash separator (Unix path style).
    pub fn slash() -> Self {
        Self {
            separator: " / ",
            ..Default::default()
        }
    }

    /// Style with chevron separator (Unicode).
    pub fn chevron() -> Self {
        Self {
            separator: " › ",
            separator_style: Style::default().fg(Color::Gray),
            ..Default::default()
        }
    }

    /// Style with arrow separator (Unicode).
    pub fn arrow() -> Self {
        Self {
            separator: " → ",
            separator_style: Style::default().fg(Color::Gray),
            ..Default::default()
        }
    }

    /// Minimal style with subdued colors.
    pub fn minimal() -> Self {
        Self {
            separator: " / ",
            separator_style: Style::default().fg(Color::DarkGray),
            item_style: Style::default().fg(Color::Gray),
            focused_item_style: Style::default().fg(Color::White),
            selected_item_style: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            last_item_style: Style::default().fg(Color::White),
            ellipsis_style: Style::default().fg(Color::Gray),
            ..Default::default()
        }
    }

    /// Set the separator string.
    pub fn separator(mut self, sep: &'static str) -> Self {
        self.separator = sep;
        self
    }

    /// Set the separator style.
    pub fn separator_style(mut self, style: Style) -> Self {
        self.separator_style = style;
        self
    }

    /// Set the collapse threshold.
    pub fn collapse_threshold(mut self, threshold: usize) -> Self {
        self.collapse_threshold = threshold;
        self
    }

    /// Set visible items when collapsed (start_count, end_count).
    pub fn visible_ends(mut self, start: usize, end: usize) -> Self {
        self.visible_start = start;
        self.visible_end = end;
        self
    }

    /// Set the item style.
    pub fn item_style(mut self, style: Style) -> Self {
        self.item_style = style;
        self
    }

    /// Set the focused item style.
    pub fn focused_item_style(mut self, style: Style) -> Self {
        self.focused_item_style = style;
        self
    }

    /// Set the last item style.
    pub fn last_item_style(mut self, style: Style) -> Self {
        self.last_item_style = style;
        self
    }

    /// Set padding (horizontal, vertical).
    pub fn padding(mut self, left: u16, right: u16) -> Self {
        self.padding = (left, right);
        self
    }
}

/// Represents a visible element in the rendered breadcrumb.
#[derive(Debug, Clone)]
enum VisibleElement {
    /// A regular item with its index.
    Item(usize),
    /// The ellipsis element.
    Ellipsis,
}

/// Breadcrumb widget - hierarchical navigation path display.
///
/// Displays a breadcrumb trail showing the user's current location in a hierarchy.
/// Supports collapsing long paths with ellipsis, keyboard navigation, and mouse clicks.
pub struct Breadcrumb<'a> {
    state: &'a BreadcrumbState,
    style: BreadcrumbStyle,
    /// Index of currently hovered item (for mouse hover effects).
    hovered_index: Option<usize>,
}

impl<'a> Breadcrumb<'a> {
    /// Create a new breadcrumb widget.
    pub fn new(state: &'a BreadcrumbState) -> Self {
        Self {
            state,
            style: BreadcrumbStyle::default(),
            hovered_index: None,
        }
    }

    /// Set the style.
    pub fn style(mut self, style: BreadcrumbStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the hovered item index (for mouse hover effects).
    pub fn hovered(mut self, index: Option<usize>) -> Self {
        self.hovered_index = index;
        self
    }

    /// Determine which elements are visible based on collapse logic.
    fn visible_elements(&self) -> Vec<VisibleElement> {
        let len = self.state.items.len();

        // No collapsing needed
        if len <= self.style.collapse_threshold || self.state.expanded {
            return (0..len).map(VisibleElement::Item).collect();
        }

        let mut elements = Vec::new();

        // Show first visible_start items
        for i in 0..self.style.visible_start.min(len) {
            elements.push(VisibleElement::Item(i));
        }

        // Add ellipsis
        elements.push(VisibleElement::Ellipsis);

        // Show last visible_end items
        let start = len.saturating_sub(self.style.visible_end);
        for i in start..len {
            elements.push(VisibleElement::Item(i));
        }

        elements
    }

    /// Get the style for an item at the given index.
    fn item_style(&self, idx: usize) -> Style {
        let item = &self.state.items[idx];
        let is_last = idx == self.state.items.len() - 1;
        let is_selected = self.state.selected_index == Some(idx);
        let is_hovered = self.hovered_index == Some(idx);
        let is_focused = self.state.focused && is_selected;

        if !item.enabled {
            self.style.disabled_item_style
        } else if is_focused {
            self.style.selected_item_style
        } else if is_hovered {
            self.style.hovered_item_style
        } else if is_selected {
            self.style.focused_item_style
        } else if is_last {
            self.style.last_item_style
        } else {
            self.style.item_style
        }
    }

    /// Render the breadcrumb and return click regions.
    pub fn render_stateful(
        self,
        area: Rect,
        buf: &mut Buffer,
    ) -> Vec<ClickRegion<BreadcrumbAction>> {
        let mut regions = Vec::new();

        if self.state.items.is_empty() {
            return regions;
        }

        let visible = self.visible_elements();
        let mut spans = Vec::new();
        let mut x_offset = area.x + self.style.padding.0;

        // Track positions for click regions
        let mut element_positions: Vec<(VisibleElement, u16, u16)> = Vec::new();

        for (i, element) in visible.iter().enumerate() {
            // Add separator before items (except first)
            if i > 0 {
                let sep_span = Span::styled(self.style.separator, self.style.separator_style);
                let sep_width = self.style.separator.chars().count() as u16;
                spans.push(sep_span);
                x_offset += sep_width;
            }

            match element {
                VisibleElement::Item(idx) => {
                    let item = &self.state.items[*idx];
                    let style = self.item_style(*idx);

                    // Build item text
                    let mut item_text = String::new();
                    if let Some(ref icon) = item.icon {
                        item_text.push_str(icon);
                        item_text.push_str(self.style.icon_separator);
                    }
                    item_text.push_str(&item.label);

                    let item_width = item_text.chars().count() as u16;
                    element_positions.push((element.clone(), x_offset, item_width));

                    spans.push(Span::styled(item_text, style));
                    x_offset += item_width;
                }
                VisibleElement::Ellipsis => {
                    let ellipsis_width = self.style.ellipsis.chars().count() as u16;
                    element_positions.push((element.clone(), x_offset, ellipsis_width));

                    spans.push(Span::styled(self.style.ellipsis, self.style.ellipsis_style));
                    x_offset += ellipsis_width;
                }
            }
        }

        // Create the line and render
        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);

        // Create click regions
        for (element, start_x, width) in element_positions {
            if width == 0 {
                continue;
            }

            let click_area = Rect::new(start_x, area.y, width, 1);

            match element {
                VisibleElement::Item(idx) => {
                    let item = &self.state.items[idx];
                    if item.enabled {
                        regions.push(ClickRegion::new(
                            click_area,
                            BreadcrumbAction::Navigate(item.id.clone()),
                        ));
                    }
                }
                VisibleElement::Ellipsis => {
                    regions.push(ClickRegion::new(
                        click_area,
                        BreadcrumbAction::ExpandEllipsis,
                    ));
                }
            }
        }

        regions
    }

    /// Calculate the width needed to render the breadcrumb.
    pub fn calculate_width(&self) -> u16 {
        if self.state.items.is_empty() {
            return 0;
        }

        let visible = self.visible_elements();
        let mut width = self.style.padding.0 + self.style.padding.1;

        for (i, element) in visible.iter().enumerate() {
            // Add separator width (except first)
            if i > 0 {
                width += self.style.separator.chars().count() as u16;
            }

            match element {
                VisibleElement::Item(idx) => {
                    let item = &self.state.items[*idx];
                    if let Some(ref icon) = item.icon {
                        width += icon.chars().count() as u16;
                        width += self.style.icon_separator.chars().count() as u16;
                    }
                    width += item.label.chars().count() as u16;
                }
                VisibleElement::Ellipsis => {
                    width += self.style.ellipsis.chars().count() as u16;
                }
            }
        }

        width
    }
}

/// Handle keyboard events for breadcrumb component.
///
/// Returns `Some(BreadcrumbAction)` if an action was triggered, `None` otherwise.
///
/// # Key Bindings
///
/// - `←` / `h` - Select previous item
/// - `→` / `l` - Select next item
/// - `Enter` / `Space` - Activate selected item
/// - `Home` - Select first item
/// - `End` - Select last item
/// - `e` - Expand/collapse ellipsis
pub fn handle_breadcrumb_key(
    key: &KeyEvent,
    state: &mut BreadcrumbState,
) -> Option<BreadcrumbAction> {
    if !state.enabled || state.items.is_empty() {
        return None;
    }

    match key.code {
        KeyCode::Left | KeyCode::Char('h') => {
            state.select_prev();
            None
        }
        KeyCode::Right | KeyCode::Char('l') => {
            state.select_next();
            None
        }
        KeyCode::Home => {
            state.select_first();
            None
        }
        KeyCode::End => {
            state.select_last();
            None
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(item) = state.selected_item() {
                if item.enabled {
                    Some(BreadcrumbAction::Navigate(item.id.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        }
        KeyCode::Char('e') => {
            state.toggle_expanded();
            Some(BreadcrumbAction::ExpandEllipsis)
        }
        _ => None,
    }
}

/// Handle mouse events for breadcrumb component.
///
/// Returns `Some(BreadcrumbAction)` if an action was triggered, `None` otherwise.
///
/// # Arguments
///
/// * `mouse` - The mouse event
/// * `state` - Mutable reference to breadcrumb state
/// * `regions` - Click regions from `render_stateful`
pub fn handle_breadcrumb_mouse(
    mouse: &MouseEvent,
    state: &mut BreadcrumbState,
    regions: &[ClickRegion<BreadcrumbAction>],
) -> Option<BreadcrumbAction> {
    if !state.enabled {
        return None;
    }

    if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
        let col = mouse.column;
        let row = mouse.row;

        for region in regions {
            if region.contains(col, row) {
                match &region.data {
                    BreadcrumbAction::Navigate(id) => {
                        // Update selection to clicked item
                        state.select_by_id(id);
                        return Some(region.data.clone());
                    }
                    BreadcrumbAction::ExpandEllipsis => {
                        state.toggle_expanded();
                        return Some(BreadcrumbAction::ExpandEllipsis);
                    }
                }
            }
        }
    }

    None
}

/// Get the item index at a given mouse position.
///
/// Useful for implementing hover effects.
pub fn get_hovered_index(
    col: u16,
    row: u16,
    regions: &[ClickRegion<BreadcrumbAction>],
    state: &BreadcrumbState,
) -> Option<usize> {
    for region in regions {
        if region.contains(col, row) {
            if let BreadcrumbAction::Navigate(ref id) = region.data {
                return state.items.iter().position(|item| &item.id == id);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breadcrumb_item_creation() {
        let item = BreadcrumbItem::new("home", "Home").icon("🏠").enabled(true);

        assert_eq!(item.id, "home");
        assert_eq!(item.label, "Home");
        assert_eq!(item.icon, Some("🏠".to_string()));
        assert!(item.enabled);
    }

    #[test]
    fn test_breadcrumb_state_navigation() {
        let items = vec![
            BreadcrumbItem::new("a", "A"),
            BreadcrumbItem::new("b", "B"),
            BreadcrumbItem::new("c", "C"),
        ];
        let mut state = BreadcrumbState::new(items);

        assert!(state.selected_index.is_none());

        state.select_next();
        assert_eq!(state.selected_index, Some(0));

        state.select_next();
        assert_eq!(state.selected_index, Some(1));

        state.select_prev();
        assert_eq!(state.selected_index, Some(0));

        state.select_prev();
        assert_eq!(state.selected_index, Some(0)); // Stay at start

        state.select_last();
        assert_eq!(state.selected_index, Some(2));

        state.select_first();
        assert_eq!(state.selected_index, Some(0));
    }

    #[test]
    fn test_breadcrumb_state_select_by_id() {
        let items = vec![
            BreadcrumbItem::new("home", "Home"),
            BreadcrumbItem::new("settings", "Settings"),
            BreadcrumbItem::new("profile", "Profile"),
        ];
        let mut state = BreadcrumbState::new(items);

        state.select_by_id("settings");
        assert_eq!(state.selected_index, Some(1));

        state.select_by_id("nonexistent");
        assert_eq!(state.selected_index, Some(1)); // Unchanged
    }

    #[test]
    fn test_breadcrumb_state_push_pop() {
        let mut state = BreadcrumbState::empty();
        assert!(state.is_empty());

        state.push(BreadcrumbItem::new("a", "A"));
        state.push(BreadcrumbItem::new("b", "B"));
        assert_eq!(state.len(), 2);

        state.select_last();
        assert_eq!(state.selected_index, Some(1));

        let popped = state.pop();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().id, "b");
        assert_eq!(state.selected_index, Some(0)); // Adjusted
    }

    #[test]
    fn test_breadcrumb_state_clear() {
        let items = vec![BreadcrumbItem::new("a", "A"), BreadcrumbItem::new("b", "B")];
        let mut state = BreadcrumbState::new(items);
        state.select(1);

        state.clear();
        assert!(state.is_empty());
        assert!(state.selected_index.is_none());
    }

    #[test]
    fn test_breadcrumb_style_presets() {
        let default = BreadcrumbStyle::default();
        assert_eq!(default.separator, " > ");

        let slash = BreadcrumbStyle::slash();
        assert_eq!(slash.separator, " / ");

        let chevron = BreadcrumbStyle::chevron();
        assert_eq!(chevron.separator, " › ");

        let arrow = BreadcrumbStyle::arrow();
        assert_eq!(arrow.separator, " → ");
    }

    #[test]
    fn test_breadcrumb_style_builder() {
        let style = BreadcrumbStyle::default()
            .separator(" | ")
            .collapse_threshold(5)
            .visible_ends(2, 3)
            .padding(2, 2);

        assert_eq!(style.separator, " | ");
        assert_eq!(style.collapse_threshold, 5);
        assert_eq!(style.visible_start, 2);
        assert_eq!(style.visible_end, 3);
        assert_eq!(style.padding, (2, 2));
    }

    #[test]
    fn test_breadcrumb_collapse_logic() {
        // Create breadcrumb with 6 items (above default threshold of 4)
        let items: Vec<BreadcrumbItem> = (0..6)
            .map(|i| BreadcrumbItem::new(format!("item{}", i), format!("Item {}", i)))
            .collect();
        let state = BreadcrumbState::new(items);
        let breadcrumb = Breadcrumb::new(&state);

        // Default: visible_start=1, visible_end=2
        // Should show: Item0, ..., Item4, Item5
        let visible = breadcrumb.visible_elements();
        assert_eq!(visible.len(), 4); // 1 start + ellipsis + 2 end

        // When expanded, should show all
        let mut expanded_state = state.clone();
        expanded_state.expanded = true;
        let expanded_breadcrumb = Breadcrumb::new(&expanded_state);
        let visible = expanded_breadcrumb.visible_elements();
        assert_eq!(visible.len(), 6);
    }

    #[test]
    fn test_breadcrumb_no_collapse() {
        // Create breadcrumb with 3 items (below threshold)
        let items: Vec<BreadcrumbItem> = (0..3)
            .map(|i| BreadcrumbItem::new(format!("item{}", i), format!("Item {}", i)))
            .collect();
        let state = BreadcrumbState::new(items);
        let breadcrumb = Breadcrumb::new(&state);

        let visible = breadcrumb.visible_elements();
        assert_eq!(visible.len(), 3); // All items shown
    }

    #[test]
    fn test_handle_breadcrumb_key() {
        let items = vec![
            BreadcrumbItem::new("a", "A"),
            BreadcrumbItem::new("b", "B"),
            BreadcrumbItem::new("c", "C"),
        ];
        let mut state = BreadcrumbState::new(items);
        state.focused = true;

        // Right arrow
        let key = KeyEvent::from(KeyCode::Right);
        handle_breadcrumb_key(&key, &mut state);
        assert_eq!(state.selected_index, Some(0));

        // Right arrow again
        handle_breadcrumb_key(&key, &mut state);
        assert_eq!(state.selected_index, Some(1));

        // Enter to navigate
        state.select(1);
        let key = KeyEvent::from(KeyCode::Enter);
        let action = handle_breadcrumb_key(&key, &mut state);
        assert_eq!(action, Some(BreadcrumbAction::Navigate("b".to_string())));
    }

    #[test]
    fn test_handle_breadcrumb_key_disabled() {
        let items = vec![BreadcrumbItem::new("a", "A")];
        let mut state = BreadcrumbState::new(items);
        state.enabled = false;

        let key = KeyEvent::from(KeyCode::Right);
        let action = handle_breadcrumb_key(&key, &mut state);
        assert!(action.is_none());
    }

    #[test]
    fn test_calculate_width() {
        let items = vec![
            BreadcrumbItem::new("home", "Home"),
            BreadcrumbItem::new("settings", "Settings"),
        ];
        let state = BreadcrumbState::new(items);
        let breadcrumb = Breadcrumb::new(&state);

        let width = breadcrumb.calculate_width();
        // "Home" (4) + " > " (3) + "Settings" (8) + padding (1+1) = 17
        assert_eq!(width, 17);
    }

    #[test]
    fn test_click_region_contains() {
        let region = ClickRegion::new(
            Rect::new(10, 5, 20, 1),
            BreadcrumbAction::Navigate("test".to_string()),
        );

        assert!(region.contains(10, 5));
        assert!(region.contains(29, 5));
        assert!(!region.contains(9, 5));
        assert!(!region.contains(30, 5));
    }
}
