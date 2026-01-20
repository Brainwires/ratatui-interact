//! MenuBar component - Horizontal menu bar with dropdown menus
//!
//! A traditional desktop-style menu bar (File, Edit, View, Help) with support
//! for dropdown menus, keyboard navigation, and mouse interaction.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{
//!     MenuBar, MenuBarState, MenuBarStyle, MenuBarItem, Menu,
//!     handle_menu_bar_key, handle_menu_bar_mouse,
//! };
//! use ratatui::layout::Rect;
//!
//! // Create menus
//! let menus = vec![
//!     Menu::new("File")
//!         .items(vec![
//!             MenuBarItem::action("new", "New").shortcut("Ctrl+N"),
//!             MenuBarItem::action("open", "Open").shortcut("Ctrl+O"),
//!             MenuBarItem::separator(),
//!             MenuBarItem::action("save", "Save").shortcut("Ctrl+S"),
//!             MenuBarItem::action("quit", "Quit").shortcut("Ctrl+Q"),
//!         ]),
//!     Menu::new("Edit")
//!         .items(vec![
//!             MenuBarItem::action("undo", "Undo").shortcut("Ctrl+Z"),
//!             MenuBarItem::action("redo", "Redo").shortcut("Ctrl+Y"),
//!         ]),
//! ];
//!
//! // Create state
//! let mut state = MenuBarState::new();
//!
//! // Create menu bar widget
//! let menu_bar = MenuBar::new(&menus, &state);
//!
//! // Render and handle events (see handle_menu_bar_key, handle_menu_bar_mouse)
//! ```

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::traits::ClickRegion;

/// Actions a menu bar can emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuBarAction {
    /// A menu was opened (menu index).
    MenuOpen(usize),
    /// The active menu was closed.
    MenuClose,
    /// An action item was selected (item ID).
    ItemSelect(String),
    /// Highlight changed (menu index, optional item index within dropdown).
    HighlightChange(usize, Option<usize>),
    /// A submenu was opened (parent menu index, parent item index).
    SubmenuOpen(usize, usize),
    /// A submenu was closed.
    SubmenuClose,
}

/// A single item in a menu dropdown.
#[derive(Debug, Clone)]
pub enum MenuBarItem {
    /// A clickable action item.
    Action {
        /// Unique identifier for this action.
        id: String,
        /// Display label.
        label: String,
        /// Optional keyboard shortcut display.
        shortcut: Option<String>,
        /// Whether the item is enabled.
        enabled: bool,
    },
    /// A visual separator line.
    Separator,
    /// A submenu that opens additional items.
    Submenu {
        /// Display label.
        label: String,
        /// Child menu items.
        items: Vec<MenuBarItem>,
        /// Whether the submenu is enabled.
        enabled: bool,
    },
}

impl MenuBarItem {
    /// Create a new action item.
    pub fn action(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Action {
            id: id.into(),
            label: label.into(),
            shortcut: None,
            enabled: true,
        }
    }

    /// Create a separator.
    pub fn separator() -> Self {
        Self::Separator
    }

    /// Create a submenu.
    pub fn submenu(label: impl Into<String>, items: Vec<MenuBarItem>) -> Self {
        Self::Submenu {
            label: label.into(),
            items,
            enabled: true,
        }
    }

    /// Add a shortcut display to this item.
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        if let Self::Action { shortcut: s, .. } = &mut self {
            *s = Some(shortcut.into());
        }
        self
    }

    /// Set whether this item is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        match &mut self {
            Self::Action { enabled: e, .. } => *e = enabled,
            Self::Submenu { enabled: e, .. } => *e = enabled,
            Self::Separator => {}
        }
        self
    }

    /// Check if this item is selectable (not a separator and enabled).
    pub fn is_selectable(&self) -> bool {
        match self {
            Self::Action { enabled, .. } => *enabled,
            Self::Separator => false,
            Self::Submenu { enabled, .. } => *enabled,
        }
    }

    /// Check if this item has a submenu.
    pub fn has_submenu(&self) -> bool {
        matches!(self, Self::Submenu { .. })
    }

    /// Get the ID if this is an action item.
    pub fn id(&self) -> Option<&str> {
        if let Self::Action { id, .. } = self {
            Some(id)
        } else {
            None
        }
    }

    /// Get the label for this item.
    pub fn label(&self) -> Option<&str> {
        match self {
            Self::Action { label, .. } => Some(label),
            Self::Submenu { label, .. } => Some(label),
            Self::Separator => None,
        }
    }

    /// Get the shortcut for this item.
    pub fn get_shortcut(&self) -> Option<&str> {
        if let Self::Action { shortcut, .. } = self {
            shortcut.as_deref()
        } else {
            None
        }
    }

    /// Check if this item is enabled.
    pub fn is_enabled(&self) -> bool {
        match self {
            Self::Action { enabled, .. } => *enabled,
            Self::Separator => false,
            Self::Submenu { enabled, .. } => *enabled,
        }
    }

    /// Get submenu items if this is a submenu.
    pub fn submenu_items(&self) -> Option<&[MenuBarItem]> {
        if let Self::Submenu { items, .. } = self {
            Some(items)
        } else {
            None
        }
    }
}

/// A top-level menu in the menu bar.
#[derive(Debug, Clone)]
pub struct Menu {
    /// Display label for the menu.
    pub label: String,
    /// Items in this menu's dropdown.
    pub items: Vec<MenuBarItem>,
    /// Whether this menu is enabled.
    pub enabled: bool,
}

impl Menu {
    /// Create a new menu with a label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            items: Vec::new(),
            enabled: true,
        }
    }

    /// Set the items for this menu.
    pub fn items(mut self, items: Vec<MenuBarItem>) -> Self {
        self.items = items;
        self
    }

    /// Set whether this menu is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// State for a menu bar.
#[derive(Debug, Clone)]
pub struct MenuBarState {
    /// Whether any menu is currently open.
    pub is_open: bool,
    /// Index of the currently active/highlighted menu (always set when focused).
    pub active_menu: usize,
    /// Currently highlighted item index within the dropdown (if open).
    pub highlighted_item: Option<usize>,
    /// Scroll offset for long dropdown menus.
    pub scroll_offset: u16,
    /// Whether the menu bar has focus.
    pub focused: bool,
    /// Index of active submenu item (if any).
    pub active_submenu: Option<usize>,
    /// State for active submenu.
    pub submenu_highlighted: Option<usize>,
    /// Submenu scroll offset.
    pub submenu_scroll_offset: u16,
}

impl Default for MenuBarState {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuBarState {
    /// Create a new menu bar state.
    pub fn new() -> Self {
        Self {
            is_open: false,
            active_menu: 0,
            highlighted_item: None,
            scroll_offset: 0,
            focused: false,
            active_submenu: None,
            submenu_highlighted: None,
            submenu_scroll_offset: 0,
        }
    }

    /// Open the menu at the given index.
    pub fn open_menu(&mut self, index: usize) {
        self.is_open = true;
        self.active_menu = index;
        self.highlighted_item = None;
        self.scroll_offset = 0;
        self.close_submenu();
    }

    /// Close any open menu.
    pub fn close_menu(&mut self) {
        self.is_open = false;
        self.highlighted_item = None;
        self.scroll_offset = 0;
        self.close_submenu();
    }

    /// Toggle the menu at the given index.
    pub fn toggle_menu(&mut self, index: usize) {
        if self.is_open && self.active_menu == index {
            self.close_menu();
        } else {
            self.open_menu(index);
        }
    }

    /// Move to the next menu in the bar.
    pub fn next_menu(&mut self, menu_count: usize) {
        if menu_count == 0 {
            return;
        }
        self.active_menu = (self.active_menu + 1) % menu_count;
        if self.is_open {
            self.highlighted_item = None;
            self.scroll_offset = 0;
            self.close_submenu();
        }
    }

    /// Move to the previous menu in the bar.
    pub fn prev_menu(&mut self, menu_count: usize) {
        if menu_count == 0 {
            return;
        }
        if self.active_menu == 0 {
            self.active_menu = menu_count - 1;
        } else {
            self.active_menu -= 1;
        }
        if self.is_open {
            self.highlighted_item = None;
            self.scroll_offset = 0;
            self.close_submenu();
        }
    }

    /// Move highlight to the next item in the dropdown.
    pub fn next_item(&mut self, items: &[MenuBarItem]) {
        if items.is_empty() {
            return;
        }

        let current = self.highlighted_item.unwrap_or(0);
        let mut new_index = current;

        loop {
            new_index += 1;
            if new_index >= items.len() {
                // Wrap around to start
                new_index = 0;
            }
            if new_index == current {
                // We've gone full circle
                break;
            }
            if items.get(new_index).is_some_and(|i| i.is_selectable()) {
                self.highlighted_item = Some(new_index);
                break;
            }
        }
    }

    /// Move highlight to the previous item in the dropdown.
    pub fn prev_item(&mut self, items: &[MenuBarItem]) {
        if items.is_empty() {
            return;
        }

        let current = self.highlighted_item.unwrap_or(0);
        let mut new_index = current;

        loop {
            if new_index == 0 {
                new_index = items.len() - 1;
            } else {
                new_index -= 1;
            }
            if new_index == current {
                // We've gone full circle
                break;
            }
            if items.get(new_index).is_some_and(|i| i.is_selectable()) {
                self.highlighted_item = Some(new_index);
                break;
            }
        }
    }

    /// Move to first selectable item.
    pub fn highlight_first(&mut self, items: &[MenuBarItem]) {
        for (i, item) in items.iter().enumerate() {
            if item.is_selectable() {
                self.highlighted_item = Some(i);
                self.scroll_offset = 0;
                break;
            }
        }
    }

    /// Move to last selectable item.
    pub fn highlight_last(&mut self, items: &[MenuBarItem]) {
        for (i, item) in items.iter().enumerate().rev() {
            if item.is_selectable() {
                self.highlighted_item = Some(i);
                break;
            }
        }
    }

    /// Select an item by index.
    pub fn select_item(&mut self, index: usize) {
        self.highlighted_item = Some(index);
    }

    /// Open submenu at the highlighted index.
    pub fn open_submenu(&mut self) {
        if let Some(idx) = self.highlighted_item {
            self.active_submenu = Some(idx);
            self.submenu_highlighted = None;
            self.submenu_scroll_offset = 0;
        }
    }

    /// Close any open submenu.
    pub fn close_submenu(&mut self) {
        self.active_submenu = None;
        self.submenu_highlighted = None;
        self.submenu_scroll_offset = 0;
    }

    /// Check if a submenu is open.
    pub fn has_open_submenu(&self) -> bool {
        self.active_submenu.is_some()
    }

    /// Move to next item in submenu.
    pub fn next_submenu_item(&mut self, items: &[MenuBarItem]) {
        if items.is_empty() {
            return;
        }

        let current = self.submenu_highlighted.unwrap_or(0);
        let mut new_index = current;

        loop {
            new_index += 1;
            if new_index >= items.len() {
                new_index = 0;
            }
            if new_index == current {
                break;
            }
            if items.get(new_index).is_some_and(|i| i.is_selectable()) {
                self.submenu_highlighted = Some(new_index);
                break;
            }
        }
    }

    /// Move to previous item in submenu.
    pub fn prev_submenu_item(&mut self, items: &[MenuBarItem]) {
        if items.is_empty() {
            return;
        }

        let current = self.submenu_highlighted.unwrap_or(0);
        let mut new_index = current;

        loop {
            if new_index == 0 {
                new_index = items.len() - 1;
            } else {
                new_index -= 1;
            }
            if new_index == current {
                break;
            }
            if items.get(new_index).is_some_and(|i| i.is_selectable()) {
                self.submenu_highlighted = Some(new_index);
                break;
            }
        }
    }

    /// Ensure highlighted item is visible in viewport.
    pub fn ensure_visible(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }
        if let Some(idx) = self.highlighted_item {
            if idx < self.scroll_offset as usize {
                self.scroll_offset = idx as u16;
            } else if idx >= self.scroll_offset as usize + viewport_height {
                self.scroll_offset = (idx - viewport_height + 1) as u16;
            }
        }
    }
}

/// Style configuration for menu bar.
#[derive(Debug, Clone)]
pub struct MenuBarStyle {
    /// Background color for the menu bar.
    pub bar_bg: Color,
    /// Foreground color for menu labels.
    pub bar_fg: Color,
    /// Background color for highlighted menu label.
    pub bar_highlight_bg: Color,
    /// Foreground color for highlighted menu label.
    pub bar_highlight_fg: Color,
    /// Background color for dropdown.
    pub dropdown_bg: Color,
    /// Border color for dropdown.
    pub dropdown_border: Color,
    /// Normal item foreground color.
    pub item_fg: Color,
    /// Highlighted item background.
    pub item_highlight_bg: Color,
    /// Highlighted item foreground.
    pub item_highlight_fg: Color,
    /// Shortcut text color.
    pub shortcut_fg: Color,
    /// Disabled item/menu foreground.
    pub disabled_fg: Color,
    /// Separator color.
    pub separator_fg: Color,
    /// Minimum dropdown width.
    pub dropdown_min_width: u16,
    /// Maximum dropdown height (items visible).
    pub dropdown_max_height: u16,
    /// Padding between menu labels.
    pub menu_padding: u16,
    /// Horizontal padding inside dropdown.
    pub dropdown_padding: u16,
    /// Submenu indicator.
    pub submenu_indicator: &'static str,
    /// Separator character.
    pub separator_char: char,
}

impl Default for MenuBarStyle {
    fn default() -> Self {
        Self {
            bar_bg: Color::Rgb(50, 50, 50),
            bar_fg: Color::White,
            bar_highlight_bg: Color::Rgb(70, 70, 70),
            bar_highlight_fg: Color::White,
            dropdown_bg: Color::Rgb(40, 40, 40),
            dropdown_border: Color::Rgb(80, 80, 80),
            item_fg: Color::White,
            item_highlight_bg: Color::Rgb(60, 100, 180),
            item_highlight_fg: Color::White,
            shortcut_fg: Color::Rgb(140, 140, 140),
            disabled_fg: Color::DarkGray,
            separator_fg: Color::Rgb(80, 80, 80),
            dropdown_min_width: 15,
            dropdown_max_height: 15,
            menu_padding: 2,
            dropdown_padding: 1,
            submenu_indicator: "▶",
            separator_char: '─',
        }
    }
}

impl MenuBarStyle {
    /// Create a light theme style.
    pub fn light() -> Self {
        Self {
            bar_bg: Color::Rgb(240, 240, 240),
            bar_fg: Color::Rgb(30, 30, 30),
            bar_highlight_bg: Color::Rgb(200, 200, 200),
            bar_highlight_fg: Color::Rgb(30, 30, 30),
            dropdown_bg: Color::Rgb(250, 250, 250),
            dropdown_border: Color::Rgb(180, 180, 180),
            item_fg: Color::Rgb(30, 30, 30),
            item_highlight_bg: Color::Rgb(0, 120, 215),
            item_highlight_fg: Color::White,
            shortcut_fg: Color::Rgb(100, 100, 100),
            disabled_fg: Color::Rgb(160, 160, 160),
            separator_fg: Color::Rgb(200, 200, 200),
            ..Default::default()
        }
    }

    /// Create a minimal style.
    pub fn minimal() -> Self {
        Self {
            bar_bg: Color::Reset,
            bar_fg: Color::White,
            bar_highlight_bg: Color::Blue,
            bar_highlight_fg: Color::White,
            dropdown_bg: Color::Reset,
            dropdown_border: Color::Gray,
            item_fg: Color::White,
            item_highlight_bg: Color::Blue,
            item_highlight_fg: Color::White,
            shortcut_fg: Color::Gray,
            disabled_fg: Color::DarkGray,
            separator_fg: Color::DarkGray,
            ..Default::default()
        }
    }

    /// Set bar colors.
    pub fn bar_colors(mut self, fg: Color, bg: Color) -> Self {
        self.bar_fg = fg;
        self.bar_bg = bg;
        self
    }

    /// Set bar highlight colors.
    pub fn bar_highlight(mut self, fg: Color, bg: Color) -> Self {
        self.bar_highlight_fg = fg;
        self.bar_highlight_bg = bg;
        self
    }

    /// Set dropdown colors.
    pub fn dropdown_colors(mut self, fg: Color, bg: Color, border: Color) -> Self {
        self.item_fg = fg;
        self.dropdown_bg = bg;
        self.dropdown_border = border;
        self
    }

    /// Set item highlight colors.
    pub fn item_highlight(mut self, fg: Color, bg: Color) -> Self {
        self.item_highlight_fg = fg;
        self.item_highlight_bg = bg;
        self
    }

    /// Set minimum dropdown width.
    pub fn dropdown_min_width(mut self, width: u16) -> Self {
        self.dropdown_min_width = width;
        self
    }

    /// Set maximum dropdown height.
    pub fn dropdown_max_height(mut self, height: u16) -> Self {
        self.dropdown_max_height = height;
        self
    }

    /// Set the submenu indicator.
    pub fn submenu_indicator(mut self, indicator: &'static str) -> Self {
        self.submenu_indicator = indicator;
        self
    }
}

/// Click region identifier for menu bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuBarClickTarget {
    /// A menu label in the bar.
    MenuLabel(usize),
    /// An item in the dropdown.
    DropdownItem(usize),
    /// An item in a submenu.
    SubmenuItem(usize),
}

/// Menu bar widget.
///
/// A horizontal menu bar with dropdown menus.
pub struct MenuBar<'a> {
    menus: &'a [Menu],
    state: &'a MenuBarState,
    style: MenuBarStyle,
}

impl<'a> MenuBar<'a> {
    /// Create a new menu bar.
    pub fn new(menus: &'a [Menu], state: &'a MenuBarState) -> Self {
        Self {
            menus,
            state,
            style: MenuBarStyle::default(),
        }
    }

    /// Set the style.
    pub fn style(mut self, style: MenuBarStyle) -> Self {
        self.style = style;
        self
    }

    /// Calculate the required width for a dropdown.
    fn calculate_dropdown_width(&self, items: &[MenuBarItem]) -> u16 {
        let mut max_label_width = 0u16;
        let mut max_shortcut_width = 0u16;

        for item in items {
            match item {
                MenuBarItem::Action {
                    label, shortcut, ..
                } => {
                    max_label_width = max_label_width.max(label.chars().count() as u16);
                    if let Some(s) = shortcut {
                        max_shortcut_width = max_shortcut_width.max(s.chars().count() as u16);
                    }
                }
                MenuBarItem::Submenu { label, .. } => {
                    // +2 for submenu indicator
                    let label_width = label.chars().count() as u16 + 2;
                    max_label_width = max_label_width.max(label_width);
                }
                MenuBarItem::Separator => {}
            }
        }

        // Total width: padding + label + gap + shortcut + padding + borders
        let content_width = self.style.dropdown_padding
            + max_label_width
            + if max_shortcut_width > 0 {
                2 + max_shortcut_width
            } else {
                0
            }
            + self.style.dropdown_padding;

        (content_width + 2).max(self.style.dropdown_min_width)
    }

    /// Calculate the height for a dropdown.
    fn calculate_dropdown_height(&self, item_count: usize) -> u16 {
        let visible = (item_count as u16).min(self.style.dropdown_max_height);
        visible + 2 // +2 for borders
    }

    /// Calculate dropdown area based on menu position and screen bounds.
    fn calculate_dropdown_area(&self, menu_x: u16, bar_bottom: u16, items: &[MenuBarItem], screen: Rect) -> Rect {
        let width = self.calculate_dropdown_width(items);
        let height = self.calculate_dropdown_height(items.len());

        // Prefer below the menu bar
        let y = bar_bottom;

        // Prefer aligned with menu label, adjust if needed
        let x = if menu_x + width <= screen.x + screen.width {
            menu_x
        } else {
            screen.x + screen.width.saturating_sub(width)
        };

        // Ensure we stay within screen bounds
        let final_width = width.min(screen.width.saturating_sub(x.saturating_sub(screen.x)));
        let final_height = height.min(screen.height.saturating_sub(y.saturating_sub(screen.y)));

        Rect::new(x, y, final_width, final_height)
    }

    /// Render the menu bar and return click regions.
    ///
    /// Returns a tuple of (bar_area, dropdown_area, click_regions).
    pub fn render_stateful(
        &self,
        frame: &mut Frame,
        area: Rect,
    ) -> (Rect, Option<Rect>, Vec<ClickRegion<MenuBarClickTarget>>) {
        let mut regions = Vec::new();

        if area.height == 0 || self.menus.is_empty() {
            return (Rect::default(), None, regions);
        }

        // Render the bar (1 row high)
        let bar_area = Rect::new(area.x, area.y, area.width, 1);

        // Fill bar background
        let bar_style = Style::default().bg(self.style.bar_bg);
        let bar_line = " ".repeat(bar_area.width as usize);
        let bar_para = Paragraph::new(Span::styled(bar_line, bar_style));
        frame.render_widget(bar_para, bar_area);

        // Render menu labels
        let mut x = bar_area.x;
        let mut menu_positions: Vec<(u16, u16)> = Vec::new(); // (x, width) for each menu

        for (idx, menu) in self.menus.iter().enumerate() {
            let label = format!(" {} ", menu.label);
            let label_width = label.chars().count() as u16;

            let is_active = self.state.focused && idx == self.state.active_menu;
            let is_open = self.state.is_open && idx == self.state.active_menu;

            let (fg, bg) = if !menu.enabled {
                (self.style.disabled_fg, self.style.bar_bg)
            } else if is_active || is_open {
                (self.style.bar_highlight_fg, self.style.bar_highlight_bg)
            } else {
                (self.style.bar_fg, self.style.bar_bg)
            };

            let style = Style::default().fg(fg).bg(bg);
            let label_area = Rect::new(x, bar_area.y, label_width, 1);

            let para = Paragraph::new(Span::styled(label.clone(), style));
            frame.render_widget(para, label_area);

            menu_positions.push((x, label_width));

            // Register click region for menu label
            if menu.enabled {
                regions.push(ClickRegion::new(label_area, MenuBarClickTarget::MenuLabel(idx)));
            }

            x += label_width + self.style.menu_padding;
        }

        // Render dropdown if a menu is open
        let dropdown_area = if self.state.is_open {
            if let Some(menu) = self.menus.get(self.state.active_menu) {
                if let Some(&(menu_x, _)) = menu_positions.get(self.state.active_menu) {
                    let screen = frame.area();
                    let dropdown_area = self.calculate_dropdown_area(
                        menu_x,
                        bar_area.y + 1,
                        &menu.items,
                        screen,
                    );

                    // Clear background (overlay)
                    frame.render_widget(Clear, dropdown_area);

                    // Render border
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(self.style.dropdown_border))
                        .style(Style::default().bg(self.style.dropdown_bg));

                    let inner = block.inner(dropdown_area);
                    frame.render_widget(block, dropdown_area);

                    // Render items
                    let visible_count = inner.height as usize;
                    let scroll = self.state.scroll_offset as usize;

                    for (display_idx, (item_idx, item)) in menu
                        .items
                        .iter()
                        .enumerate()
                        .skip(scroll)
                        .take(visible_count)
                        .enumerate()
                    {
                        let y = inner.y + display_idx as u16;
                        let item_area = Rect::new(inner.x, y, inner.width, 1);

                        let is_highlighted = self.state.highlighted_item == Some(item_idx);

                        self.render_menu_item(
                            frame,
                            item,
                            item_area,
                            is_highlighted,
                            &mut regions,
                            item_idx,
                            false,
                        );
                    }

                    // Render submenu if open
                    if let Some(submenu_idx) = self.state.active_submenu {
                        if let Some(MenuBarItem::Submenu { items, .. }) = menu.items.get(submenu_idx) {
                            let submenu_x = dropdown_area.x + dropdown_area.width;
                            let submenu_y = dropdown_area.y + 1 + (submenu_idx as u16).saturating_sub(self.state.scroll_offset);

                            let submenu_width = self.calculate_dropdown_width(items);
                            let submenu_height = self.calculate_dropdown_height(items.len());

                            let screen = frame.area();

                            // Adjust submenu position to stay on screen
                            let final_x = if submenu_x + submenu_width <= screen.x + screen.width {
                                submenu_x
                            } else {
                                dropdown_area.x.saturating_sub(submenu_width)
                            };

                            let submenu_area = Rect::new(
                                final_x,
                                submenu_y.min(screen.y + screen.height - submenu_height),
                                submenu_width,
                                submenu_height,
                            );

                            // Clear and render submenu
                            frame.render_widget(Clear, submenu_area);

                            let block = Block::default()
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(self.style.dropdown_border))
                                .style(Style::default().bg(self.style.dropdown_bg));

                            let sub_inner = block.inner(submenu_area);
                            frame.render_widget(block, submenu_area);

                            let sub_visible = sub_inner.height as usize;
                            let sub_scroll = self.state.submenu_scroll_offset as usize;

                            for (display_idx, (item_idx, item)) in items
                                .iter()
                                .enumerate()
                                .skip(sub_scroll)
                                .take(sub_visible)
                                .enumerate()
                            {
                                let y = sub_inner.y + display_idx as u16;
                                let item_area = Rect::new(sub_inner.x, y, sub_inner.width, 1);

                                let is_highlighted = self.state.submenu_highlighted == Some(item_idx);

                                self.render_menu_item(
                                    frame,
                                    item,
                                    item_area,
                                    is_highlighted,
                                    &mut regions,
                                    item_idx,
                                    true,
                                );
                            }
                        }
                    }

                    Some(dropdown_area)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        (bar_area, dropdown_area, regions)
    }

    /// Render a single menu item.
    fn render_menu_item(
        &self,
        frame: &mut Frame,
        item: &MenuBarItem,
        item_area: Rect,
        is_highlighted: bool,
        regions: &mut Vec<ClickRegion<MenuBarClickTarget>>,
        item_idx: usize,
        is_submenu: bool,
    ) {
        match item {
            MenuBarItem::Separator => {
                let sep_line: String =
                    std::iter::repeat_n(self.style.separator_char, item_area.width as usize).collect();
                let para = Paragraph::new(Span::styled(
                    sep_line,
                    Style::default().fg(self.style.separator_fg).bg(self.style.dropdown_bg),
                ));
                frame.render_widget(para, item_area);
            }
            MenuBarItem::Action {
                label,
                shortcut,
                enabled,
                id,
            } => {
                let (fg, bg) = if !enabled {
                    (self.style.disabled_fg, self.style.dropdown_bg)
                } else if is_highlighted {
                    (self.style.item_highlight_fg, self.style.item_highlight_bg)
                } else {
                    (self.style.item_fg, self.style.dropdown_bg)
                };

                let style = Style::default().fg(fg).bg(bg);
                let shortcut_style = Style::default()
                    .fg(if *enabled {
                        self.style.shortcut_fg
                    } else {
                        self.style.disabled_fg
                    })
                    .bg(bg);

                let mut spans = Vec::new();

                // Padding
                spans.push(Span::styled(
                    " ".repeat(self.style.dropdown_padding as usize),
                    style,
                ));

                // Label
                spans.push(Span::styled(label.clone(), style));

                // Fill space before shortcut
                let current_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
                let shortcut_len = shortcut.as_ref().map(|s| s.chars().count()).unwrap_or(0);
                let fill_len = (item_area.width as usize)
                    .saturating_sub(current_len)
                    .saturating_sub(shortcut_len)
                    .saturating_sub(self.style.dropdown_padding as usize);

                if fill_len > 0 {
                    spans.push(Span::styled(" ".repeat(fill_len), style));
                }

                // Shortcut
                if let Some(sc) = shortcut {
                    spans.push(Span::styled(sc.clone(), shortcut_style));
                }

                // Right padding
                spans.push(Span::styled(
                    " ".repeat(self.style.dropdown_padding as usize),
                    style,
                ));

                let para = Paragraph::new(Line::from(spans));
                frame.render_widget(para, item_area);

                // Register click region
                if *enabled {
                    let target = if is_submenu {
                        MenuBarClickTarget::SubmenuItem(item_idx)
                    } else {
                        MenuBarClickTarget::DropdownItem(item_idx)
                    };
                    regions.push(ClickRegion::new(item_area, target));
                }

                // Silence unused variable warning
                let _ = id;
            }
            MenuBarItem::Submenu { label, enabled, .. } => {
                let (fg, bg) = if !enabled {
                    (self.style.disabled_fg, self.style.dropdown_bg)
                } else if is_highlighted {
                    (self.style.item_highlight_fg, self.style.item_highlight_bg)
                } else {
                    (self.style.item_fg, self.style.dropdown_bg)
                };

                let style = Style::default().fg(fg).bg(bg);

                let mut spans = Vec::new();

                // Padding
                spans.push(Span::styled(
                    " ".repeat(self.style.dropdown_padding as usize),
                    style,
                ));

                // Label
                spans.push(Span::styled(label.clone(), style));

                // Fill and submenu indicator
                let current_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
                let indicator_len = self.style.submenu_indicator.chars().count();
                let fill_len = (item_area.width as usize)
                    .saturating_sub(current_len)
                    .saturating_sub(indicator_len)
                    .saturating_sub(self.style.dropdown_padding as usize);

                if fill_len > 0 {
                    spans.push(Span::styled(" ".repeat(fill_len), style));
                }

                spans.push(Span::styled(self.style.submenu_indicator, style));

                // Right padding
                spans.push(Span::styled(
                    " ".repeat(self.style.dropdown_padding as usize),
                    style,
                ));

                let para = Paragraph::new(Line::from(spans));
                frame.render_widget(para, item_area);

                // Register click region (only for parent dropdown, not for nested submenus)
                if *enabled && !is_submenu {
                    regions.push(ClickRegion::new(item_area, MenuBarClickTarget::DropdownItem(item_idx)));
                }
            }
        }
    }
}

/// Handle keyboard events for menu bar.
///
/// Returns `Some(MenuBarAction)` if an action was triggered, `None` otherwise.
///
/// # Key Bindings
///
/// - `Left/Right` - Navigate between menus
/// - `Up/Down` - Navigate within dropdown (opens menu if closed)
/// - `Enter/Space` - Select item or toggle menu
/// - `Escape` - Close menu
/// - `Home` - Jump to first item
/// - `End` - Jump to last item
pub fn handle_menu_bar_key(
    key: &KeyEvent,
    state: &mut MenuBarState,
    menus: &[Menu],
) -> Option<MenuBarAction> {
    if menus.is_empty() {
        return None;
    }

    // If submenu is open, handle submenu navigation
    if state.has_open_submenu() {
        if let Some(menu) = menus.get(state.active_menu) {
            if let Some(submenu_idx) = state.active_submenu {
                if let Some(MenuBarItem::Submenu { items, .. }) = menu.items.get(submenu_idx) {
                    match key.code {
                        KeyCode::Esc | KeyCode::Left => {
                            state.close_submenu();
                            return Some(MenuBarAction::SubmenuClose);
                        }
                        KeyCode::Up => {
                            state.prev_submenu_item(items);
                            return Some(MenuBarAction::HighlightChange(
                                state.active_menu,
                                state.submenu_highlighted,
                            ));
                        }
                        KeyCode::Down => {
                            state.next_submenu_item(items);
                            return Some(MenuBarAction::HighlightChange(
                                state.active_menu,
                                state.submenu_highlighted,
                            ));
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if let Some(idx) = state.submenu_highlighted {
                                if let Some(item) = items.get(idx) {
                                    if let MenuBarItem::Action { id, enabled, .. } = item {
                                        if *enabled {
                                            let action_id = id.clone();
                                            state.close_menu();
                                            return Some(MenuBarAction::ItemSelect(action_id));
                                        }
                                    }
                                }
                            }
                            return None;
                        }
                        _ => return None,
                    }
                }
            }
        }
    }

    match key.code {
        KeyCode::Left => {
            state.prev_menu(menus.len());
            Some(MenuBarAction::HighlightChange(state.active_menu, None))
        }
        KeyCode::Right => {
            // If on a submenu item, open it
            if state.is_open {
                if let Some(menu) = menus.get(state.active_menu) {
                    if let Some(idx) = state.highlighted_item {
                        if let Some(item) = menu.items.get(idx) {
                            if item.has_submenu() && item.is_enabled() {
                                state.open_submenu();
                                return Some(MenuBarAction::SubmenuOpen(state.active_menu, idx));
                            }
                        }
                    }
                }
            }
            state.next_menu(menus.len());
            Some(MenuBarAction::HighlightChange(state.active_menu, None))
        }
        KeyCode::Down => {
            if state.is_open {
                if let Some(menu) = menus.get(state.active_menu) {
                    state.next_item(&menu.items);
                    state.ensure_visible(8);
                    Some(MenuBarAction::HighlightChange(
                        state.active_menu,
                        state.highlighted_item,
                    ))
                } else {
                    None
                }
            } else {
                state.open_menu(state.active_menu);
                if let Some(menu) = menus.get(state.active_menu) {
                    state.highlight_first(&menu.items);
                }
                Some(MenuBarAction::MenuOpen(state.active_menu))
            }
        }
        KeyCode::Up => {
            if state.is_open {
                if let Some(menu) = menus.get(state.active_menu) {
                    state.prev_item(&menu.items);
                    state.ensure_visible(8);
                    Some(MenuBarAction::HighlightChange(
                        state.active_menu,
                        state.highlighted_item,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if state.is_open {
                if let Some(menu) = menus.get(state.active_menu) {
                    if let Some(idx) = state.highlighted_item {
                        if let Some(item) = menu.items.get(idx) {
                            match item {
                                MenuBarItem::Action { id, enabled, .. } if *enabled => {
                                    let action_id = id.clone();
                                    state.close_menu();
                                    return Some(MenuBarAction::ItemSelect(action_id));
                                }
                                MenuBarItem::Submenu { enabled, .. } if *enabled => {
                                    state.open_submenu();
                                    return Some(MenuBarAction::SubmenuOpen(state.active_menu, idx));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                None
            } else {
                state.open_menu(state.active_menu);
                if let Some(menu) = menus.get(state.active_menu) {
                    state.highlight_first(&menu.items);
                }
                Some(MenuBarAction::MenuOpen(state.active_menu))
            }
        }
        KeyCode::Esc => {
            if state.is_open {
                state.close_menu();
                Some(MenuBarAction::MenuClose)
            } else {
                None
            }
        }
        KeyCode::Home => {
            if state.is_open {
                if let Some(menu) = menus.get(state.active_menu) {
                    state.highlight_first(&menu.items);
                    Some(MenuBarAction::HighlightChange(
                        state.active_menu,
                        state.highlighted_item,
                    ))
                } else {
                    None
                }
            } else {
                state.active_menu = 0;
                Some(MenuBarAction::HighlightChange(0, None))
            }
        }
        KeyCode::End => {
            if state.is_open {
                if let Some(menu) = menus.get(state.active_menu) {
                    state.highlight_last(&menu.items);
                    state.ensure_visible(menu.items.len());
                    Some(MenuBarAction::HighlightChange(
                        state.active_menu,
                        state.highlighted_item,
                    ))
                } else {
                    None
                }
            } else {
                state.active_menu = menus.len().saturating_sub(1);
                Some(MenuBarAction::HighlightChange(state.active_menu, None))
            }
        }
        _ => None,
    }
}

/// Handle mouse events for menu bar.
///
/// Returns `Some(MenuBarAction)` if an action was triggered, `None` otherwise.
///
/// # Arguments
///
/// * `mouse` - The mouse event
/// * `state` - Mutable reference to menu bar state
/// * `bar_area` - The rendered bar area
/// * `dropdown_area` - The rendered dropdown area (if any)
/// * `click_regions` - Click regions from `render_stateful`
/// * `menus` - The menu definitions
pub fn handle_menu_bar_mouse(
    mouse: &MouseEvent,
    state: &mut MenuBarState,
    bar_area: Rect,
    dropdown_area: Option<Rect>,
    click_regions: &[ClickRegion<MenuBarClickTarget>],
    menus: &[Menu],
) -> Option<MenuBarAction> {
    let col = mouse.column;
    let row = mouse.row;

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Check if clicked on a menu label
            for region in click_regions {
                if region.contains(col, row) {
                    match &region.data {
                        MenuBarClickTarget::MenuLabel(idx) => {
                            state.toggle_menu(*idx);
                            if state.is_open {
                                if let Some(menu) = menus.get(*idx) {
                                    state.highlight_first(&menu.items);
                                }
                                return Some(MenuBarAction::MenuOpen(*idx));
                            } else {
                                return Some(MenuBarAction::MenuClose);
                            }
                        }
                        MenuBarClickTarget::DropdownItem(idx) => {
                            if let Some(menu) = menus.get(state.active_menu) {
                                if let Some(item) = menu.items.get(*idx) {
                                    match item {
                                        MenuBarItem::Action { id, enabled, .. } if *enabled => {
                                            let action_id = id.clone();
                                            state.close_menu();
                                            return Some(MenuBarAction::ItemSelect(action_id));
                                        }
                                        MenuBarItem::Submenu { enabled, .. } if *enabled => {
                                            state.highlighted_item = Some(*idx);
                                            state.open_submenu();
                                            return Some(MenuBarAction::SubmenuOpen(state.active_menu, *idx));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        MenuBarClickTarget::SubmenuItem(idx) => {
                            if let Some(menu) = menus.get(state.active_menu) {
                                if let Some(submenu_idx) = state.active_submenu {
                                    if let Some(MenuBarItem::Submenu { items, .. }) = menu.items.get(submenu_idx) {
                                        if let Some(item) = items.get(*idx) {
                                            if let MenuBarItem::Action { id, enabled, .. } = item {
                                                if *enabled {
                                                    let action_id = id.clone();
                                                    state.close_menu();
                                                    return Some(MenuBarAction::ItemSelect(action_id));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Check if clicked outside menu
            let in_bar = bar_area.intersects(Rect::new(col, row, 1, 1));
            let in_dropdown = dropdown_area
                .map(|d| d.intersects(Rect::new(col, row, 1, 1)))
                .unwrap_or(false);

            if state.is_open && !in_bar && !in_dropdown {
                state.close_menu();
                return Some(MenuBarAction::MenuClose);
            }

            None
        }
        MouseEventKind::Moved => {
            // Update highlight on hover
            for region in click_regions {
                if region.contains(col, row) {
                    match &region.data {
                        MenuBarClickTarget::MenuLabel(idx) => {
                            // If a menu is open and we hover over a different menu label, switch to it
                            if state.is_open && state.active_menu != *idx {
                                state.open_menu(*idx);
                                if let Some(menu) = menus.get(*idx) {
                                    state.highlight_first(&menu.items);
                                }
                                return Some(MenuBarAction::MenuOpen(*idx));
                            }
                        }
                        MenuBarClickTarget::DropdownItem(idx) => {
                            if state.highlighted_item != Some(*idx) {
                                state.highlighted_item = Some(*idx);
                                // Close submenu when moving to different item
                                if state.active_submenu.is_some() && state.active_submenu != Some(*idx) {
                                    state.close_submenu();
                                }
                                return Some(MenuBarAction::HighlightChange(
                                    state.active_menu,
                                    Some(*idx),
                                ));
                            }
                        }
                        MenuBarClickTarget::SubmenuItem(idx) => {
                            if state.submenu_highlighted != Some(*idx) {
                                state.submenu_highlighted = Some(*idx);
                                return Some(MenuBarAction::HighlightChange(
                                    state.active_menu,
                                    Some(*idx),
                                ));
                            }
                        }
                    }
                    break;
                }
            }
            None
        }
        _ => None,
    }
}

/// Calculate the height needed for a menu bar (always 1).
pub fn calculate_menu_bar_height() -> u16 {
    1
}

/// Calculate the height needed for a dropdown menu.
pub fn calculate_dropdown_height(item_count: usize, max_visible: u16) -> u16 {
    let visible = (item_count as u16).min(max_visible);
    visible + 2 // +2 for borders
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_bar_item_action() {
        let item = MenuBarItem::action("save", "Save").shortcut("Ctrl+S");

        assert!(item.is_selectable());
        assert!(!item.has_submenu());
        assert_eq!(item.id(), Some("save"));
        assert_eq!(item.label(), Some("Save"));
        assert_eq!(item.get_shortcut(), Some("Ctrl+S"));
    }

    #[test]
    fn test_menu_bar_item_separator() {
        let item = MenuBarItem::separator();

        assert!(!item.is_selectable());
        assert!(!item.has_submenu());
        assert_eq!(item.label(), None);
    }

    #[test]
    fn test_menu_bar_item_submenu() {
        let items = vec![MenuBarItem::action("sub1", "Sub Item 1")];
        let item = MenuBarItem::submenu("More", items);

        assert!(item.is_selectable());
        assert!(item.has_submenu());
        assert_eq!(item.label(), Some("More"));
        assert!(item.submenu_items().is_some());
    }

    #[test]
    fn test_menu_bar_item_disabled() {
        let item = MenuBarItem::action("delete", "Delete").enabled(false);

        assert!(!item.is_selectable());
        assert!(!item.is_enabled());
    }

    #[test]
    fn test_menu_creation() {
        let menu = Menu::new("File")
            .items(vec![
                MenuBarItem::action("new", "New"),
                MenuBarItem::separator(),
                MenuBarItem::action("quit", "Quit"),
            ])
            .enabled(true);

        assert_eq!(menu.label, "File");
        assert_eq!(menu.items.len(), 3);
        assert!(menu.enabled);
    }

    #[test]
    fn test_menu_bar_state_new() {
        let state = MenuBarState::new();

        assert!(!state.is_open);
        assert_eq!(state.active_menu, 0);
        assert_eq!(state.highlighted_item, None);
        assert!(!state.focused);
    }

    #[test]
    fn test_menu_bar_state_open_close() {
        let mut state = MenuBarState::new();

        state.open_menu(1);
        assert!(state.is_open);
        assert_eq!(state.active_menu, 1);
        assert_eq!(state.highlighted_item, None);

        state.close_menu();
        assert!(!state.is_open);
    }

    #[test]
    fn test_menu_bar_state_toggle() {
        let mut state = MenuBarState::new();

        state.toggle_menu(0);
        assert!(state.is_open);
        assert_eq!(state.active_menu, 0);

        state.toggle_menu(0);
        assert!(!state.is_open);

        state.toggle_menu(0);
        assert!(state.is_open);

        // Toggle different menu while open
        state.toggle_menu(1);
        assert!(state.is_open);
        assert_eq!(state.active_menu, 1);
    }

    #[test]
    fn test_menu_bar_state_navigation() {
        let mut state = MenuBarState::new();
        state.active_menu = 0;

        state.next_menu(3);
        assert_eq!(state.active_menu, 1);

        state.next_menu(3);
        assert_eq!(state.active_menu, 2);

        state.next_menu(3);
        assert_eq!(state.active_menu, 0); // Wrap around

        state.prev_menu(3);
        assert_eq!(state.active_menu, 2); // Wrap around

        state.prev_menu(3);
        assert_eq!(state.active_menu, 1);
    }

    #[test]
    fn test_menu_bar_state_item_navigation() {
        let mut state = MenuBarState::new();
        state.open_menu(0);

        let items = vec![
            MenuBarItem::action("a", "A"),
            MenuBarItem::separator(),
            MenuBarItem::action("b", "B"),
            MenuBarItem::action("c", "C"),
        ];

        // Move down (should skip separator)
        state.next_item(&items);
        // With no initial highlighted item, wraps from 0
        assert!(state.highlighted_item.is_some());

        state.highlight_first(&items);
        assert_eq!(state.highlighted_item, Some(0));

        state.next_item(&items);
        assert_eq!(state.highlighted_item, Some(2)); // Skipped separator

        state.next_item(&items);
        assert_eq!(state.highlighted_item, Some(3));

        state.prev_item(&items);
        assert_eq!(state.highlighted_item, Some(2));

        state.prev_item(&items);
        assert_eq!(state.highlighted_item, Some(0));
    }

    #[test]
    fn test_menu_bar_state_submenu() {
        let mut state = MenuBarState::new();
        state.open_menu(0);
        state.highlighted_item = Some(2);

        assert!(!state.has_open_submenu());

        state.open_submenu();
        assert!(state.has_open_submenu());
        assert_eq!(state.active_submenu, Some(2));

        state.close_submenu();
        assert!(!state.has_open_submenu());
    }

    #[test]
    fn test_menu_bar_style_default() {
        let style = MenuBarStyle::default();
        assert_eq!(style.dropdown_min_width, 15);
        assert_eq!(style.dropdown_max_height, 15);
        assert_eq!(style.submenu_indicator, "▶");
    }

    #[test]
    fn test_menu_bar_style_builders() {
        let style = MenuBarStyle::default()
            .dropdown_min_width(20)
            .dropdown_max_height(10)
            .submenu_indicator("→");

        assert_eq!(style.dropdown_min_width, 20);
        assert_eq!(style.dropdown_max_height, 10);
        assert_eq!(style.submenu_indicator, "→");
    }

    #[test]
    fn test_menu_bar_style_presets() {
        let light = MenuBarStyle::light();
        assert_eq!(light.bar_bg, Color::Rgb(240, 240, 240));

        let minimal = MenuBarStyle::minimal();
        assert_eq!(minimal.bar_bg, Color::Reset);
    }

    #[test]
    fn test_handle_key_left_right() {
        let mut state = MenuBarState::new();
        state.focused = true;

        let menus = vec![
            Menu::new("File").items(vec![]),
            Menu::new("Edit").items(vec![]),
            Menu::new("View").items(vec![]),
        ];

        let key = KeyEvent::from(KeyCode::Right);
        let action = handle_menu_bar_key(&key, &mut state, &menus);
        assert_eq!(action, Some(MenuBarAction::HighlightChange(1, None)));
        assert_eq!(state.active_menu, 1);

        let key = KeyEvent::from(KeyCode::Left);
        let action = handle_menu_bar_key(&key, &mut state, &menus);
        assert_eq!(action, Some(MenuBarAction::HighlightChange(0, None)));
        assert_eq!(state.active_menu, 0);
    }

    #[test]
    fn test_handle_key_down_opens_menu() {
        let mut state = MenuBarState::new();
        state.focused = true;

        let menus = vec![Menu::new("File").items(vec![
            MenuBarItem::action("new", "New"),
        ])];

        let key = KeyEvent::from(KeyCode::Down);
        let action = handle_menu_bar_key(&key, &mut state, &menus);

        assert_eq!(action, Some(MenuBarAction::MenuOpen(0)));
        assert!(state.is_open);
    }

    #[test]
    fn test_handle_key_escape_closes() {
        let mut state = MenuBarState::new();
        state.open_menu(0);

        let menus = vec![Menu::new("File").items(vec![])];

        let key = KeyEvent::from(KeyCode::Esc);
        let action = handle_menu_bar_key(&key, &mut state, &menus);

        assert_eq!(action, Some(MenuBarAction::MenuClose));
        assert!(!state.is_open);
    }

    #[test]
    fn test_handle_key_enter_selects_item() {
        let mut state = MenuBarState::new();
        state.open_menu(0);
        state.highlighted_item = Some(0);

        let menus = vec![Menu::new("File").items(vec![
            MenuBarItem::action("new", "New"),
        ])];

        let key = KeyEvent::from(KeyCode::Enter);
        let action = handle_menu_bar_key(&key, &mut state, &menus);

        assert_eq!(action, Some(MenuBarAction::ItemSelect("new".to_string())));
        assert!(!state.is_open);
    }

    #[test]
    fn test_handle_key_enter_opens_submenu() {
        let mut state = MenuBarState::new();
        state.open_menu(0);
        state.highlighted_item = Some(0);

        let menus = vec![Menu::new("File").items(vec![
            MenuBarItem::submenu("Recent", vec![
                MenuBarItem::action("file1", "File 1"),
            ]),
        ])];

        let key = KeyEvent::from(KeyCode::Enter);
        let action = handle_menu_bar_key(&key, &mut state, &menus);

        assert_eq!(action, Some(MenuBarAction::SubmenuOpen(0, 0)));
        assert!(state.has_open_submenu());
    }

    #[test]
    fn test_handle_key_empty_menus() {
        let mut state = MenuBarState::new();
        let menus: Vec<Menu> = vec![];

        let key = KeyEvent::from(KeyCode::Down);
        let action = handle_menu_bar_key(&key, &mut state, &menus);

        assert!(action.is_none());
    }

    #[test]
    fn test_menu_bar_action_equality() {
        assert_eq!(MenuBarAction::MenuOpen(0), MenuBarAction::MenuOpen(0));
        assert_ne!(MenuBarAction::MenuOpen(0), MenuBarAction::MenuOpen(1));
        assert_eq!(MenuBarAction::MenuClose, MenuBarAction::MenuClose);
        assert_eq!(
            MenuBarAction::ItemSelect("test".to_string()),
            MenuBarAction::ItemSelect("test".to_string())
        );
        assert_eq!(
            MenuBarAction::HighlightChange(0, Some(1)),
            MenuBarAction::HighlightChange(0, Some(1))
        );
    }

    #[test]
    fn test_calculate_heights() {
        assert_eq!(calculate_menu_bar_height(), 1);
        assert_eq!(calculate_dropdown_height(5, 15), 7); // 5 + 2
        assert_eq!(calculate_dropdown_height(20, 15), 17); // 15 + 2 (clamped)
    }

    #[test]
    fn test_menu_bar_widget_new() {
        let menus = vec![Menu::new("File").items(vec![])];
        let state = MenuBarState::new();
        let _menu_bar = MenuBar::new(&menus, &state);
    }

    #[test]
    fn test_menu_bar_widget_style() {
        let menus = vec![Menu::new("File").items(vec![])];
        let state = MenuBarState::new();
        let style = MenuBarStyle::light();
        let _menu_bar = MenuBar::new(&menus, &state).style(style);
    }

    #[test]
    fn test_click_target_equality() {
        assert_eq!(
            MenuBarClickTarget::MenuLabel(0),
            MenuBarClickTarget::MenuLabel(0)
        );
        assert_ne!(
            MenuBarClickTarget::MenuLabel(0),
            MenuBarClickTarget::MenuLabel(1)
        );
        assert_eq!(
            MenuBarClickTarget::DropdownItem(0),
            MenuBarClickTarget::DropdownItem(0)
        );
        assert_eq!(
            MenuBarClickTarget::SubmenuItem(0),
            MenuBarClickTarget::SubmenuItem(0)
        );
    }

    #[test]
    fn test_menu_bar_state_ensure_visible() {
        let mut state = MenuBarState::new();
        state.highlighted_item = Some(15);
        state.scroll_offset = 0;

        state.ensure_visible(10);
        assert!(state.scroll_offset >= 6);

        state.highlighted_item = Some(3);
        state.ensure_visible(10);
        assert!(state.scroll_offset <= 3);
    }

    #[test]
    fn test_menu_bar_state_highlight_first_last() {
        let mut state = MenuBarState::new();
        state.open_menu(0);

        let items = vec![
            MenuBarItem::separator(),
            MenuBarItem::action("a", "A"),
            MenuBarItem::action("b", "B"),
            MenuBarItem::separator(),
            MenuBarItem::action("c", "C"),
        ];

        state.highlight_first(&items);
        assert_eq!(state.highlighted_item, Some(1));

        state.highlight_last(&items);
        assert_eq!(state.highlighted_item, Some(4));
    }

    #[test]
    fn test_submenu_navigation() {
        let mut state = MenuBarState::new();
        state.open_menu(0);
        state.highlighted_item = Some(0);
        state.open_submenu();

        let items = vec![
            MenuBarItem::action("a", "A"),
            MenuBarItem::separator(),
            MenuBarItem::action("b", "B"),
        ];

        state.next_submenu_item(&items);
        // Should skip separator
        assert!(state.submenu_highlighted.is_some());

        state.prev_submenu_item(&items);
        assert!(state.submenu_highlighted.is_some());
    }
}
