//! Context Menu component - Right-click popup menu
//!
//! A context menu component that displays a popup menu at a specified position
//! with support for actions, separators, and nested submenus.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{
//!     ContextMenu, ContextMenuState, ContextMenuStyle, ContextMenuItem,
//!     handle_context_menu_key, handle_context_menu_mouse,
//! };
//! use ratatui::layout::Rect;
//!
//! // Create menu items
//! let items = vec![
//!     ContextMenuItem::action("copy", "Copy").shortcut("Ctrl+C"),
//!     ContextMenuItem::action("paste", "Paste").shortcut("Ctrl+V"),
//!     ContextMenuItem::separator(),
//!     ContextMenuItem::action("delete", "Delete").icon("🗑"),
//! ];
//!
//! // Create state and open at position
//! let mut state = ContextMenuState::new();
//! state.open_at(10, 5);
//!
//! // Create context menu widget
//! let menu = ContextMenu::new(&items, &state);
//!
//! // Render and handle events (see handle_context_menu_key, handle_context_menu_mouse)
//! ```

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use crate::traits::ClickRegion;

/// Actions a context menu can emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextMenuAction {
    /// Menu was opened.
    Open,
    /// Menu was closed.
    Close,
    /// An action item was selected (item ID).
    Select(String),
    /// A submenu was opened (parent index).
    SubmenuOpen(usize),
    /// A submenu was closed.
    SubmenuClose,
    /// Highlight changed (new index).
    HighlightChange(usize),
}

/// A single item in a context menu.
#[derive(Debug, Clone)]
pub enum ContextMenuItem {
    /// A clickable action item.
    Action {
        /// Unique identifier for this action.
        id: String,
        /// Display label.
        label: String,
        /// Optional icon (emoji or character).
        icon: Option<String>,
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
        /// Optional icon.
        icon: Option<String>,
        /// Child menu items.
        items: Vec<ContextMenuItem>,
        /// Whether the submenu is enabled.
        enabled: bool,
    },
}

impl ContextMenuItem {
    /// Create a new action item.
    pub fn action(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Action {
            id: id.into(),
            label: label.into(),
            icon: None,
            shortcut: None,
            enabled: true,
        }
    }

    /// Create a separator.
    pub fn separator() -> Self {
        Self::Separator
    }

    /// Create a submenu.
    pub fn submenu(label: impl Into<String>, items: Vec<ContextMenuItem>) -> Self {
        Self::Submenu {
            label: label.into(),
            icon: None,
            items,
            enabled: true,
        }
    }

    /// Add an icon to this item.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        match &mut self {
            Self::Action { icon: i, .. } => *i = Some(icon.into()),
            Self::Submenu { icon: i, .. } => *i = Some(icon.into()),
            Self::Separator => {}
        }
        self
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

    /// Get the icon for this item.
    pub fn get_icon(&self) -> Option<&str> {
        match self {
            Self::Action { icon, .. } => icon.as_deref(),
            Self::Submenu { icon, .. } => icon.as_deref(),
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
    pub fn submenu_items(&self) -> Option<&[ContextMenuItem]> {
        if let Self::Submenu { items, .. } = self {
            Some(items)
        } else {
            None
        }
    }
}

/// State for a context menu.
#[derive(Debug, Clone)]
pub struct ContextMenuState {
    /// Whether the menu is currently open.
    pub is_open: bool,
    /// Anchor position (x, y) where menu appears.
    pub anchor_position: (u16, u16),
    /// Currently highlighted item index.
    pub highlighted_index: usize,
    /// Scroll offset for long menus.
    pub scroll_offset: u16,
    /// Index of active submenu (if any).
    pub active_submenu: Option<usize>,
    /// State for active submenu (boxed to avoid infinite size).
    pub submenu_state: Option<Box<ContextMenuState>>,
}

impl Default for ContextMenuState {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMenuState {
    /// Create a new context menu state.
    pub fn new() -> Self {
        Self {
            is_open: false,
            anchor_position: (0, 0),
            highlighted_index: 0,
            scroll_offset: 0,
            active_submenu: None,
            submenu_state: None,
        }
    }

    /// Open the menu at the specified position.
    pub fn open_at(&mut self, x: u16, y: u16) {
        self.is_open = true;
        self.anchor_position = (x, y);
        self.highlighted_index = 0;
        self.scroll_offset = 0;
        self.close_submenu();
    }

    /// Close the menu.
    pub fn close(&mut self) {
        self.is_open = false;
        self.close_submenu();
    }

    /// Move highlight to previous selectable item.
    pub fn highlight_prev(&mut self, items: &[ContextMenuItem]) {
        if items.is_empty() {
            return;
        }

        let mut new_index = self.highlighted_index;
        loop {
            if new_index == 0 {
                break;
            }
            new_index -= 1;
            if items.get(new_index).is_some_and(|i| i.is_selectable()) {
                self.highlighted_index = new_index;
                break;
            }
        }
    }

    /// Move highlight to next selectable item.
    pub fn highlight_next(&mut self, items: &[ContextMenuItem]) {
        if items.is_empty() {
            return;
        }

        let mut new_index = self.highlighted_index;
        loop {
            new_index += 1;
            if new_index >= items.len() {
                break;
            }
            if items.get(new_index).is_some_and(|i| i.is_selectable()) {
                self.highlighted_index = new_index;
                break;
            }
        }
    }

    /// Move to first selectable item.
    pub fn highlight_first(&mut self, items: &[ContextMenuItem]) {
        for (i, item) in items.iter().enumerate() {
            if item.is_selectable() {
                self.highlighted_index = i;
                self.scroll_offset = 0;
                break;
            }
        }
    }

    /// Move to last selectable item.
    pub fn highlight_last(&mut self, items: &[ContextMenuItem]) {
        for (i, item) in items.iter().enumerate().rev() {
            if item.is_selectable() {
                self.highlighted_index = i;
                break;
            }
        }
    }

    /// Open submenu at the highlighted index.
    pub fn open_submenu(&mut self) {
        self.active_submenu = Some(self.highlighted_index);
        let mut submenu_state = ContextMenuState::new();
        submenu_state.is_open = true;
        self.submenu_state = Some(Box::new(submenu_state));
    }

    /// Close any open submenu.
    pub fn close_submenu(&mut self) {
        self.active_submenu = None;
        self.submenu_state = None;
    }

    /// Check if a submenu is open.
    pub fn has_open_submenu(&self) -> bool {
        self.active_submenu.is_some()
    }

    /// Ensure highlighted item is visible in viewport.
    pub fn ensure_visible(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }
        if self.highlighted_index < self.scroll_offset as usize {
            self.scroll_offset = self.highlighted_index as u16;
        } else if self.highlighted_index >= self.scroll_offset as usize + viewport_height {
            self.scroll_offset = (self.highlighted_index - viewport_height + 1) as u16;
        }
    }
}

/// Style configuration for context menu.
#[derive(Debug, Clone)]
pub struct ContextMenuStyle {
    /// Background color for the menu.
    pub background: Color,
    /// Border color.
    pub border: Color,
    /// Normal item foreground color.
    pub normal_fg: Color,
    /// Highlighted item background.
    pub highlight_bg: Color,
    /// Highlighted item foreground.
    pub highlight_fg: Color,
    /// Disabled item foreground.
    pub disabled_fg: Color,
    /// Shortcut text color.
    pub shortcut_fg: Color,
    /// Separator color.
    pub separator_fg: Color,
    /// Minimum menu width.
    pub min_width: u16,
    /// Maximum menu width.
    pub max_width: u16,
    /// Maximum visible items before scrolling.
    pub max_visible_items: u16,
    /// Horizontal padding inside menu.
    pub padding: u16,
    /// Submenu indicator (e.g., "▶").
    pub submenu_indicator: &'static str,
    /// Separator character.
    pub separator_char: char,
}

impl Default for ContextMenuStyle {
    fn default() -> Self {
        Self {
            background: Color::Rgb(40, 40, 40),
            border: Color::Rgb(80, 80, 80),
            normal_fg: Color::White,
            highlight_bg: Color::Rgb(60, 100, 180),
            highlight_fg: Color::White,
            disabled_fg: Color::DarkGray,
            shortcut_fg: Color::Rgb(140, 140, 140),
            separator_fg: Color::Rgb(80, 80, 80),
            min_width: 15,
            max_width: 50,
            max_visible_items: 15,
            padding: 1,
            submenu_indicator: "▶",
            separator_char: '─',
        }
    }
}

impl From<&crate::theme::Theme> for ContextMenuStyle {
    fn from(theme: &crate::theme::Theme) -> Self {
        let p = &theme.palette;
        Self {
            background: p.surface,
            border: p.separator,
            normal_fg: p.text,
            highlight_bg: p.menu_highlight_bg,
            highlight_fg: p.menu_highlight_fg,
            disabled_fg: p.text_disabled,
            shortcut_fg: p.text_muted,
            separator_fg: p.separator,
            min_width: 15,
            max_width: 50,
            max_visible_items: 15,
            padding: 1,
            submenu_indicator: "▶",
            separator_char: '─',
        }
    }
}

impl ContextMenuStyle {
    /// Create a light theme style.
    pub fn light() -> Self {
        Self {
            background: Color::Rgb(250, 250, 250),
            border: Color::Rgb(180, 180, 180),
            normal_fg: Color::Rgb(30, 30, 30),
            highlight_bg: Color::Rgb(0, 120, 215),
            highlight_fg: Color::White,
            disabled_fg: Color::Rgb(160, 160, 160),
            shortcut_fg: Color::Rgb(100, 100, 100),
            separator_fg: Color::Rgb(200, 200, 200),
            ..Default::default()
        }
    }

    /// Create a minimal style.
    pub fn minimal() -> Self {
        Self {
            background: Color::Reset,
            border: Color::Gray,
            normal_fg: Color::White,
            highlight_bg: Color::Blue,
            highlight_fg: Color::White,
            disabled_fg: Color::DarkGray,
            shortcut_fg: Color::Gray,
            separator_fg: Color::DarkGray,
            ..Default::default()
        }
    }

    /// Set minimum width.
    pub fn min_width(mut self, width: u16) -> Self {
        self.min_width = width;
        self
    }

    /// Set maximum width.
    pub fn max_width(mut self, width: u16) -> Self {
        self.max_width = width;
        self
    }

    /// Set maximum visible items.
    pub fn max_visible_items(mut self, count: u16) -> Self {
        self.max_visible_items = count;
        self
    }

    /// Set the submenu indicator.
    pub fn submenu_indicator(mut self, indicator: &'static str) -> Self {
        self.submenu_indicator = indicator;
        self
    }

    /// Set the highlight colors.
    pub fn highlight(mut self, fg: Color, bg: Color) -> Self {
        self.highlight_fg = fg;
        self.highlight_bg = bg;
        self
    }
}

/// Context menu widget.
///
/// A popup menu that appears at a specified position, typically triggered
/// by a right-click event.
pub struct ContextMenu<'a> {
    items: &'a [ContextMenuItem],
    state: &'a ContextMenuState,
    style: ContextMenuStyle,
    /// Exact x position (used for submenus, which are placed by the parent).
    forced_x: Option<u16>,
}

impl<'a> ContextMenu<'a> {
    /// Create a new context menu.
    pub fn new(items: &'a [ContextMenuItem], state: &'a ContextMenuState) -> Self {
        Self {
            items,
            state,
            style: ContextMenuStyle::default(),
            forced_x: None,
        }
    }

    /// Set the style.
    pub fn style(mut self, style: ContextMenuStyle) -> Self {
        self.style = style;
        self
    }

    /// Apply a theme to derive the style.
    pub fn theme(self, theme: &crate::theme::Theme) -> Self {
        self.style(ContextMenuStyle::from(theme))
    }

    /// Calculate the required width for the menu.
    fn calculate_width(&self) -> u16 {
        let mut max_label_width = 0u16;
        let mut max_shortcut_width = 0u16;

        for item in self.items {
            match item {
                ContextMenuItem::Action {
                    label,
                    icon,
                    shortcut,
                    ..
                } => {
                    let icon_width = icon.as_ref().map(|i| i.width() + 1).unwrap_or(0);
                    let label_width = label.width() + icon_width;
                    max_label_width = max_label_width.max(label_width as u16);
                    if let Some(s) = shortcut {
                        max_shortcut_width = max_shortcut_width.max(s.width() as u16);
                    }
                }
                ContextMenuItem::Submenu { label, icon, .. } => {
                    let icon_width = icon.as_ref().map(|i| i.width() + 1).unwrap_or(0);
                    // +2 for submenu indicator
                    let label_width = label.width() + icon_width + 2;
                    max_label_width = max_label_width.max(label_width as u16);
                }
                ContextMenuItem::Separator => {}
            }
        }

        // Total width: padding + label + gap + shortcut + padding + borders
        let content_width = self.style.padding
            + max_label_width
            + if max_shortcut_width > 0 {
                2 + max_shortcut_width
            } else {
                0
            }
            + self.style.padding;

        // Clamp to min/max
        (content_width + 2) // +2 for borders
            .max(self.style.min_width)
            .min(self.style.max_width)
    }

    /// Calculate the required height for the menu.
    fn calculate_height(&self) -> u16 {
        let item_count = self.items.len() as u16;
        let visible = item_count.min(self.style.max_visible_items);
        visible + 2 // +2 for borders
    }

    /// Calculate the menu area based on anchor and screen bounds.
    fn calculate_menu_area(&self, screen: Rect) -> Rect {
        let (anchor_x, anchor_y) = self.state.anchor_position;
        let width = self.calculate_width();
        let height = self.calculate_height();

        // Prefer right-down positioning, flip if needed
        let x = if let Some(x) = self.forced_x {
            x
        } else if anchor_x.saturating_add(width) <= screen.x.saturating_add(screen.width) {
            anchor_x
        } else {
            anchor_x.saturating_sub(width)
        };

        let y = if anchor_y.saturating_add(height) <= screen.y.saturating_add(screen.height) {
            anchor_y
        } else {
            anchor_y.saturating_sub(height)
        };

        // Ensure we stay within screen bounds
        let final_width = width.min(screen.width.saturating_sub(x.saturating_sub(screen.x)));
        let final_height = height.min(screen.height.saturating_sub(y.saturating_sub(screen.y)));

        Rect::new(x, y, final_width, final_height)
    }

    /// Render the context menu and return click regions for items.
    ///
    /// Returns a tuple of (menu_area, item_click_regions).
    pub fn render_stateful(
        &self,
        frame: &mut Frame,
        screen: Rect,
    ) -> (Rect, Vec<ClickRegion<ContextMenuAction>>) {
        let mut regions = Vec::new();

        if !self.state.is_open || self.items.is_empty() {
            return (Rect::default(), regions);
        }

        let menu_area = self.calculate_menu_area(screen);

        // Clear background (overlay)
        frame.render_widget(Clear, menu_area);

        // Render border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.style.border))
            .style(Style::default().bg(self.style.background));

        let inner = block.inner(menu_area);
        frame.render_widget(block, menu_area);

        // Render items
        let visible_count = inner.height as usize;
        let scroll = self.state.scroll_offset as usize;

        for (display_idx, (item_idx, item)) in self
            .items
            .iter()
            .enumerate()
            .skip(scroll)
            .take(visible_count)
            .enumerate()
        {
            let y = inner.y + display_idx as u16;
            let item_area = Rect::new(inner.x, y, inner.width, 1);

            let is_highlighted = item_idx == self.state.highlighted_index;

            match item {
                ContextMenuItem::Separator => {
                    // Render separator line
                    let sep_line: String =
                        std::iter::repeat_n(self.style.separator_char, inner.width as usize)
                            .collect();
                    let para = Paragraph::new(Span::styled(
                        sep_line,
                        Style::default().fg(self.style.separator_fg),
                    ));
                    frame.render_widget(para, item_area);
                }
                ContextMenuItem::Action {
                    label,
                    icon,
                    shortcut,
                    enabled,
                    id,
                } => {
                    let (fg, bg) = if !enabled {
                        (self.style.disabled_fg, self.style.background)
                    } else if is_highlighted {
                        (self.style.highlight_fg, self.style.highlight_bg)
                    } else {
                        (self.style.normal_fg, self.style.background)
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
                    spans.push(Span::styled(" ".repeat(self.style.padding as usize), style));

                    // Icon
                    if let Some(ic) = icon {
                        spans.push(Span::styled(format!("{} ", ic), style));
                    }

                    // Label
                    spans.push(Span::styled(label.clone(), style));

                    // Fill space before shortcut
                    let current_len: usize = spans.iter().map(|s| s.content.width()).sum();
                    let shortcut_len = shortcut.as_ref().map(|s| s.width()).unwrap_or(0);
                    let fill_len = (inner.width as usize)
                        .saturating_sub(current_len)
                        .saturating_sub(shortcut_len)
                        .saturating_sub(self.style.padding as usize);

                    if fill_len > 0 {
                        spans.push(Span::styled(" ".repeat(fill_len), style));
                    }

                    // Shortcut
                    if let Some(sc) = shortcut {
                        spans.push(Span::styled(sc.clone(), shortcut_style));
                    }

                    // Right padding
                    spans.push(Span::styled(" ".repeat(self.style.padding as usize), style));

                    let para = Paragraph::new(Line::from(spans));
                    frame.render_widget(para, item_area);

                    // Register click region
                    if *enabled {
                        regions.push(ClickRegion::new(
                            item_area,
                            ContextMenuAction::Select(id.clone()),
                        ));
                    }
                }
                ContextMenuItem::Submenu {
                    label,
                    icon,
                    enabled,
                    ..
                } => {
                    let (fg, bg) = if !enabled {
                        (self.style.disabled_fg, self.style.background)
                    } else if is_highlighted {
                        (self.style.highlight_fg, self.style.highlight_bg)
                    } else {
                        (self.style.normal_fg, self.style.background)
                    };

                    let style = Style::default().fg(fg).bg(bg);

                    let mut spans = Vec::new();

                    // Padding
                    spans.push(Span::styled(" ".repeat(self.style.padding as usize), style));

                    // Icon
                    if let Some(ic) = icon {
                        spans.push(Span::styled(format!("{} ", ic), style));
                    }

                    // Label
                    spans.push(Span::styled(label.clone(), style));

                    // Fill and submenu indicator
                    let current_len: usize = spans.iter().map(|s| s.content.width()).sum();
                    let indicator_len = self.style.submenu_indicator.width();
                    let fill_len = (inner.width as usize)
                        .saturating_sub(current_len)
                        .saturating_sub(indicator_len)
                        .saturating_sub(self.style.padding as usize);

                    if fill_len > 0 {
                        spans.push(Span::styled(" ".repeat(fill_len), style));
                    }

                    spans.push(Span::styled(self.style.submenu_indicator, style));

                    // Right padding
                    spans.push(Span::styled(" ".repeat(self.style.padding as usize), style));

                    let para = Paragraph::new(Line::from(spans));
                    frame.render_widget(para, item_area);

                    // Register click region for submenu
                    if *enabled {
                        regions.push(ClickRegion::new(
                            item_area,
                            ContextMenuAction::SubmenuOpen(item_idx),
                        ));
                    }
                }
            }
        }

        // Render submenu if open
        if let (Some(submenu_idx), Some(submenu_state)) =
            (self.state.active_submenu, &self.state.submenu_state)
            && let Some(ContextMenuItem::Submenu { items, .. }) = self.items.get(submenu_idx)
        {
            // Position submenu to the right of the parent item
            let submenu_anchor_x = menu_area.x.saturating_add(menu_area.width);
            let submenu_anchor_y = menu_area
                .y
                .saturating_add(1)
                .saturating_add((submenu_idx as u16).saturating_sub(self.state.scroll_offset));

            let mut adjusted_state = (**submenu_state).clone();
            adjusted_state.anchor_position = (submenu_anchor_x, submenu_anchor_y);

            let mut adjusted_submenu =
                ContextMenu::new(items, &adjusted_state).style(self.style.clone());

            // If it does not fit on the right, place it to the left of the
            // parent menu (not on top of it). If neither side fits, use
            // the side with more room (the submenu is clipped).
            let sub_width = adjusted_submenu.calculate_width();
            let screen_right = screen.x.saturating_add(screen.width);
            let room_right = screen_right.saturating_sub(submenu_anchor_x);
            let room_left = menu_area.x.saturating_sub(screen.x);
            adjusted_submenu.forced_x = Some(if sub_width <= room_right {
                submenu_anchor_x
            } else if sub_width <= room_left || room_left > room_right {
                menu_area.x.saturating_sub(sub_width).max(screen.x)
            } else {
                submenu_anchor_x
            });

            // Submenu regions are appended after the parent's; the mouse
            // handler hit-tests them first.
            let (_, submenu_regions) = adjusted_submenu.render_stateful(frame, screen);
            regions.extend(submenu_regions);
        }

        (menu_area, regions)
    }
}

/// Handle keyboard events for context menu.
///
/// Returns `Some(ContextMenuAction)` if an action was triggered, `None` otherwise.
///
/// Scrolling assumes the default [`ContextMenuStyle::max_visible_items`]; use
/// [`handle_context_menu_key_with_style`] if you render with a custom value.
///
/// # Key Bindings
///
/// - `Esc` - Close menu
/// - `Up` - Move highlight up
/// - `Down` - Move highlight down
/// - `Enter`, `Space` - Select highlighted item
/// - `Right` - Open submenu (if item has one)
/// - `Left` - Close submenu (if one is open)
/// - `Home` - Move to first item
/// - `End` - Move to last item
pub fn handle_context_menu_key(
    key: &KeyEvent,
    state: &mut ContextMenuState,
    items: &[ContextMenuItem],
) -> Option<ContextMenuAction> {
    handle_context_menu_key_with_style(key, state, items, &ContextMenuStyle::default())
}

/// Handle keyboard events for context menu using the given style.
///
/// Like [`handle_context_menu_key`], but keeps the highlighted item visible
/// according to `style.max_visible_items`.
pub fn handle_context_menu_key_with_style(
    key: &KeyEvent,
    state: &mut ContextMenuState,
    items: &[ContextMenuItem],
    style: &ContextMenuStyle,
) -> Option<ContextMenuAction> {
    if !state.is_open {
        return None;
    }

    // If submenu is open, delegate to it first
    if let (Some(submenu_idx), Some(submenu_state)) =
        (state.active_submenu, &mut state.submenu_state)
        && let Some(ContextMenuItem::Submenu {
            items: sub_items, ..
        }) = items.get(submenu_idx)
    {
        match key.code {
            KeyCode::Left | KeyCode::Esc => {
                state.close_submenu();
                return Some(ContextMenuAction::SubmenuClose);
            }
            _ => {
                if let Some(action) = handle_context_menu_key_with_style(
                    key,
                    submenu_state.as_mut(),
                    sub_items,
                    style,
                ) {
                    // Selecting an item anywhere closes the whole menu
                    if matches!(action, ContextMenuAction::Select(_)) {
                        state.close();
                    }
                    return Some(action);
                }
            }
        }
        return None;
    }

    let viewport = (style.max_visible_items as usize).min(items.len());

    match key.code {
        KeyCode::Esc => {
            state.close();
            Some(ContextMenuAction::Close)
        }
        KeyCode::Up => {
            state.highlight_prev(items);
            state.ensure_visible(viewport);
            Some(ContextMenuAction::HighlightChange(state.highlighted_index))
        }
        KeyCode::Down => {
            state.highlight_next(items);
            state.ensure_visible(viewport);
            Some(ContextMenuAction::HighlightChange(state.highlighted_index))
        }
        KeyCode::Home => {
            state.highlight_first(items);
            state.ensure_visible(viewport);
            Some(ContextMenuAction::HighlightChange(state.highlighted_index))
        }
        KeyCode::End => {
            state.highlight_last(items);
            state.ensure_visible(viewport);
            Some(ContextMenuAction::HighlightChange(state.highlighted_index))
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(item) = items.get(state.highlighted_index) {
                match item {
                    ContextMenuItem::Action { id, enabled, .. } if *enabled => {
                        let action_id = id.clone();
                        state.close();
                        Some(ContextMenuAction::Select(action_id))
                    }
                    ContextMenuItem::Submenu { enabled, .. } if *enabled => {
                        state.open_submenu();
                        Some(ContextMenuAction::SubmenuOpen(state.highlighted_index))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        KeyCode::Right => {
            if let Some(item) = items.get(state.highlighted_index)
                && item.has_submenu()
                && item.is_enabled()
            {
                state.open_submenu();
                return Some(ContextMenuAction::SubmenuOpen(state.highlighted_index));
            }
            None
        }
        KeyCode::Left => {
            // Close current menu level (handled by parent)
            None
        }
        _ => None,
    }
}

/// Handle mouse events for context menu.
///
/// Returns `Some(ContextMenuAction)` if an action was triggered, `None` otherwise.
///
/// # Arguments
///
/// * `mouse` - The mouse event
/// * `state` - Mutable reference to context menu state
/// * `menu_area` - The rendered menu area
/// * `item_regions` - Click regions from `render_stateful`
pub fn handle_context_menu_mouse(
    mouse: &MouseEvent,
    state: &mut ContextMenuState,
    menu_area: Rect,
    item_regions: &[ClickRegion<ContextMenuAction>],
) -> Option<ContextMenuAction> {
    if !state.is_open {
        return None;
    }

    let col = mouse.column;
    let row = mouse.row;

    // Submenu regions are rendered on top of (and registered after) the
    // parent's, so hit-test from the last region backwards.
    let hit = item_regions
        .iter()
        .enumerate()
        .rev()
        .find(|(_, r)| r.contains(col, row));

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Check if clicked on an item
            if let Some((region_idx, region)) = hit {
                match &region.data {
                    ContextMenuAction::Select(id) => {
                        let action_id = id.clone();
                        state.close();
                        return Some(ContextMenuAction::Select(action_id));
                    }
                    ContextMenuAction::SubmenuOpen(idx) => {
                        let level = region_level(item_regions, region_idx, menu_area);
                        let target = state_at_level(state, level);
                        target.highlighted_index = *idx;
                        target.open_submenu();
                        return Some(ContextMenuAction::SubmenuOpen(*idx));
                    }
                    _ => {}
                }
            }

            // Check if clicked outside menu (and outside any open submenu)
            let point = Rect::new(col, row, 1, 1);
            let in_submenu = submenu_bounds(item_regions, menu_area)
                .is_some_and(|bounds| bounds.intersects(point));
            if !menu_area.intersects(point) && !in_submenu {
                state.close();
                return Some(ContextMenuAction::Close);
            }
            None
        }
        MouseEventKind::Moved => {
            // Update highlight on hover
            let (region_idx, region) = hit?;
            if !matches!(
                region.data,
                ContextMenuAction::Select(_) | ContextMenuAction::SubmenuOpen(_)
            ) {
                return None;
            }

            let level = region_level(item_regions, region_idx, menu_area);
            // Row of the first visible item of the menu level that was hit
            let inner_start_y = if level == 0 {
                Some(menu_area.y.saturating_add(1)) // +1 for border
            } else if level == 1 {
                // The submenu is anchored at its parent item's row; below it
                // its first item is right after the top border. Otherwise it
                // was flipped upwards: fall back to its topmost item region.
                state.active_submenu.map(|parent_idx| {
                    let anchor_y = menu_area
                        .y
                        .saturating_add(1)
                        .saturating_add((parent_idx as u16).saturating_sub(state.scroll_offset));
                    if row > anchor_y {
                        anchor_y + 1
                    } else {
                        level_top(item_regions, region.area, menu_area)
                    }
                })
            } else {
                Some(level_top(item_regions, region.area, menu_area))
            };

            let target = state_at_level(state, level);
            let item_idx = row
                .checked_sub(inner_start_y?)
                .map(|offset| offset as usize + target.scroll_offset as usize)?;

            if target.highlighted_index != item_idx {
                target.highlighted_index = item_idx;
                return Some(ContextMenuAction::HighlightChange(item_idx));
            }
            None
        }
        _ => None,
    }
}

/// Menu nesting level of the region at `region_idx`: 0 for the root menu,
/// 1 for its submenu, and so on.
///
/// Regions inside `menu_area` belong to the root menu; the remaining regions
/// are grouped by column (each menu's items share x and width), in the
/// order they were rendered.
fn region_level(
    regions: &[ClickRegion<ContextMenuAction>],
    region_idx: usize,
    menu_area: Rect,
) -> usize {
    let target = regions[region_idx].area;
    if menu_area.contains(target.as_position()) {
        return 0;
    }
    let mut columns: Vec<(u16, u16)> = Vec::new();
    for region in regions {
        if menu_area.contains(region.area.as_position()) {
            continue;
        }
        let key = (region.area.x, region.area.width);
        if !columns.contains(&key) {
            columns.push(key);
        }
    }
    columns
        .iter()
        .position(|&k| k == (target.x, target.width))
        .map_or(1, |p| p + 1)
}

/// Topmost row of the item regions in the same menu column as `area`.
fn level_top(regions: &[ClickRegion<ContextMenuAction>], area: Rect, menu_area: Rect) -> u16 {
    regions
        .iter()
        .filter(|r| {
            r.area.x == area.x
                && r.area.width == area.width
                && !menu_area.contains(r.area.as_position())
        })
        .map(|r| r.area.y)
        .min()
        .unwrap_or(area.y)
}

/// Bounding box (including borders) of all submenu item regions.
fn submenu_bounds(regions: &[ClickRegion<ContextMenuAction>], menu_area: Rect) -> Option<Rect> {
    regions
        .iter()
        .filter(|r| !menu_area.contains(r.area.as_position()))
        .map(|r| {
            Rect::new(
                r.area.x.saturating_sub(1),
                r.area.y.saturating_sub(1),
                r.area.width.saturating_add(2),
                r.area.height.saturating_add(2),
            )
        })
        .reduce(|a, b| a.union(b))
}

/// The state of the open (sub)menu at nesting `level` (or the deepest one).
fn state_at_level(state: &mut ContextMenuState, level: usize) -> &mut ContextMenuState {
    let mut current = state;
    for _ in 0..level {
        if current.submenu_state.is_none() {
            break;
        }
        current = current.submenu_state.as_deref_mut().unwrap();
    }
    current
}

/// Check if a mouse event is a context menu trigger (right-click).
pub fn is_context_menu_trigger(mouse: &MouseEvent) -> bool {
    matches!(mouse.kind, MouseEventKind::Down(MouseButton::Right))
}

/// Calculate the height needed for a context menu.
pub fn calculate_menu_height(item_count: usize, max_visible: u16) -> u16 {
    let visible = (item_count as u16).min(max_visible);
    visible + 2 // +2 for borders
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_item_action() {
        let item = ContextMenuItem::action("copy", "Copy")
            .icon("📋")
            .shortcut("Ctrl+C");

        assert!(item.is_selectable());
        assert!(!item.has_submenu());
        assert_eq!(item.id(), Some("copy"));
        assert_eq!(item.label(), Some("Copy"));
        assert_eq!(item.get_icon(), Some("📋"));
        assert_eq!(item.get_shortcut(), Some("Ctrl+C"));
    }

    #[test]
    fn test_context_menu_item_separator() {
        let item = ContextMenuItem::separator();

        assert!(!item.is_selectable());
        assert!(!item.has_submenu());
        assert_eq!(item.label(), None);
    }

    #[test]
    fn test_context_menu_item_submenu() {
        let items = vec![ContextMenuItem::action("sub1", "Sub Item 1")];
        let item = ContextMenuItem::submenu("More", items).icon("▶");

        assert!(item.is_selectable());
        assert!(item.has_submenu());
        assert_eq!(item.label(), Some("More"));
        assert!(item.submenu_items().is_some());
    }

    #[test]
    fn test_context_menu_item_disabled() {
        let item = ContextMenuItem::action("delete", "Delete").enabled(false);

        assert!(!item.is_selectable());
        assert!(!item.is_enabled());
    }

    #[test]
    fn test_context_menu_state_open_close() {
        let mut state = ContextMenuState::new();

        assert!(!state.is_open);

        state.open_at(10, 5);
        assert!(state.is_open);
        assert_eq!(state.anchor_position, (10, 5));
        assert_eq!(state.highlighted_index, 0);

        state.close();
        assert!(!state.is_open);
    }

    #[test]
    fn test_context_menu_state_navigation() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);

        let items = vec![
            ContextMenuItem::action("a", "A"),
            ContextMenuItem::separator(),
            ContextMenuItem::action("b", "B"),
            ContextMenuItem::action("c", "C"),
        ];

        // Start at index 0
        assert_eq!(state.highlighted_index, 0);

        // Move down (should skip separator)
        state.highlight_next(&items);
        assert_eq!(state.highlighted_index, 2); // Skipped separator at 1

        // Move down again
        state.highlight_next(&items);
        assert_eq!(state.highlighted_index, 3);

        // Move up
        state.highlight_prev(&items);
        assert_eq!(state.highlighted_index, 2);

        // Move up again (should skip separator)
        state.highlight_prev(&items);
        assert_eq!(state.highlighted_index, 0);
    }

    #[test]
    fn test_context_menu_state_submenu() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);
        state.highlighted_index = 2;

        assert!(!state.has_open_submenu());

        state.open_submenu();
        assert!(state.has_open_submenu());
        assert_eq!(state.active_submenu, Some(2));
        assert!(state.submenu_state.is_some());

        state.close_submenu();
        assert!(!state.has_open_submenu());
        assert!(state.submenu_state.is_none());
    }

    #[test]
    fn test_context_menu_style_default() {
        let style = ContextMenuStyle::default();
        assert_eq!(style.min_width, 15);
        assert_eq!(style.max_width, 50);
        assert_eq!(style.max_visible_items, 15);
        assert_eq!(style.submenu_indicator, "▶");
    }

    #[test]
    fn test_context_menu_style_builders() {
        let style = ContextMenuStyle::default()
            .min_width(20)
            .max_width(60)
            .max_visible_items(10)
            .submenu_indicator("→");

        assert_eq!(style.min_width, 20);
        assert_eq!(style.max_width, 60);
        assert_eq!(style.max_visible_items, 10);
        assert_eq!(style.submenu_indicator, "→");
    }

    #[test]
    fn test_context_menu_style_presets() {
        let light = ContextMenuStyle::light();
        assert_eq!(light.background, Color::Rgb(250, 250, 250));

        let minimal = ContextMenuStyle::minimal();
        assert_eq!(minimal.background, Color::Reset);
    }

    #[test]
    fn test_handle_key_escape() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);

        let items = vec![ContextMenuItem::action("a", "A")];
        let key = KeyEvent::from(KeyCode::Esc);
        let action = handle_context_menu_key(&key, &mut state, &items);

        assert_eq!(action, Some(ContextMenuAction::Close));
        assert!(!state.is_open);
    }

    #[test]
    fn test_handle_key_navigation() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);

        let items = vec![
            ContextMenuItem::action("a", "A"),
            ContextMenuItem::action("b", "B"),
            ContextMenuItem::action("c", "C"),
        ];

        // Down
        let key = KeyEvent::from(KeyCode::Down);
        let action = handle_context_menu_key(&key, &mut state, &items);
        assert_eq!(action, Some(ContextMenuAction::HighlightChange(1)));
        assert_eq!(state.highlighted_index, 1);

        // Up
        let key = KeyEvent::from(KeyCode::Up);
        let action = handle_context_menu_key(&key, &mut state, &items);
        assert_eq!(action, Some(ContextMenuAction::HighlightChange(0)));
        assert_eq!(state.highlighted_index, 0);
    }

    #[test]
    fn test_handle_key_select() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);
        state.highlighted_index = 1;

        let items = vec![
            ContextMenuItem::action("a", "A"),
            ContextMenuItem::action("b", "B"),
        ];

        let key = KeyEvent::from(KeyCode::Enter);
        let action = handle_context_menu_key(&key, &mut state, &items);

        assert_eq!(action, Some(ContextMenuAction::Select("b".to_string())));
        assert!(!state.is_open);
    }

    #[test]
    fn test_is_context_menu_trigger() {
        use crossterm::event::KeyModifiers;

        let right_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Right),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        assert!(is_context_menu_trigger(&right_click));

        let left_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        assert!(!is_context_menu_trigger(&left_click));
    }

    #[test]
    fn test_calculate_menu_height() {
        assert_eq!(calculate_menu_height(5, 15), 7); // 5 + 2
        assert_eq!(calculate_menu_height(20, 15), 17); // 15 + 2 (clamped)
        assert_eq!(calculate_menu_height(0, 15), 2); // 0 + 2
    }

    // Additional comprehensive tests

    #[test]
    fn test_context_menu_item_icon_on_separator() {
        // Icon should not affect separators
        let item = ContextMenuItem::separator().icon("x");
        assert_eq!(item.get_icon(), None);
    }

    #[test]
    fn test_context_menu_item_shortcut_on_submenu() {
        // Shortcut should not affect submenus
        let item = ContextMenuItem::submenu("Menu", vec![]).shortcut("Ctrl+X");
        assert_eq!(item.get_shortcut(), None);
    }

    #[test]
    fn test_context_menu_item_enabled_on_separator() {
        // Enabled should not affect separators (always false)
        let item = ContextMenuItem::separator().enabled(true);
        assert!(!item.is_enabled());
    }

    #[test]
    fn test_context_menu_item_submenu_items() {
        let sub_items = vec![
            ContextMenuItem::action("a", "A"),
            ContextMenuItem::action("b", "B"),
        ];
        let item = ContextMenuItem::submenu("Menu", sub_items);
        let items = item.submenu_items().unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_context_menu_item_action_no_submenu_items() {
        let item = ContextMenuItem::action("test", "Test");
        assert!(item.submenu_items().is_none());
    }

    #[test]
    fn test_context_menu_state_default() {
        let state = ContextMenuState::default();
        assert!(!state.is_open);
        assert_eq!(state.anchor_position, (0, 0));
        assert_eq!(state.highlighted_index, 0);
        assert_eq!(state.scroll_offset, 0);
        assert!(state.active_submenu.is_none());
        assert!(state.submenu_state.is_none());
    }

    #[test]
    fn test_context_menu_state_open_resets_state() {
        let mut state = ContextMenuState::new();
        state.highlighted_index = 5;
        state.scroll_offset = 10;
        state.open_submenu();

        state.open_at(20, 30);

        assert!(state.is_open);
        assert_eq!(state.anchor_position, (20, 30));
        assert_eq!(state.highlighted_index, 0);
        assert_eq!(state.scroll_offset, 0);
        assert!(!state.has_open_submenu());
    }

    #[test]
    fn test_context_menu_state_highlight_first_last() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);

        let items = vec![
            ContextMenuItem::separator(),      // index 0 - not selectable
            ContextMenuItem::action("a", "A"), // index 1
            ContextMenuItem::action("b", "B"), // index 2
            ContextMenuItem::separator(),      // index 3 - not selectable
            ContextMenuItem::action("c", "C"), // index 4
        ];

        state.highlight_first(&items);
        assert_eq!(state.highlighted_index, 1); // First selectable

        state.highlight_last(&items);
        assert_eq!(state.highlighted_index, 4); // Last selectable
    }

    #[test]
    fn test_context_menu_state_navigation_bounds() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);
        state.highlighted_index = 0;

        let items = vec![
            ContextMenuItem::action("a", "A"),
            ContextMenuItem::action("b", "B"),
        ];

        // Try to go before first
        state.highlight_prev(&items);
        assert_eq!(state.highlighted_index, 0);

        // Go to last
        state.highlighted_index = 1;
        // Try to go past last
        state.highlight_next(&items);
        assert_eq!(state.highlighted_index, 1);
    }

    #[test]
    fn test_context_menu_state_navigation_empty_items() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);
        state.highlighted_index = 5;

        let items: Vec<ContextMenuItem> = vec![];

        state.highlight_next(&items);
        assert_eq!(state.highlighted_index, 5); // Unchanged

        state.highlight_prev(&items);
        assert_eq!(state.highlighted_index, 5); // Unchanged
    }

    #[test]
    fn test_context_menu_state_ensure_visible() {
        let mut state = ContextMenuState::new();
        state.highlighted_index = 15;
        state.scroll_offset = 0;

        state.ensure_visible(10);
        // 15 - 10 + 1 = 6
        assert!(state.scroll_offset >= 6);

        // Scroll back up
        state.highlighted_index = 3;
        state.ensure_visible(10);
        assert!(state.scroll_offset <= 3);
    }

    #[test]
    fn test_context_menu_state_ensure_visible_zero_viewport() {
        let mut state = ContextMenuState::new();
        state.highlighted_index = 10;
        state.scroll_offset = 5;

        // Zero viewport should not change anything
        state.ensure_visible(0);
        assert_eq!(state.scroll_offset, 5);
    }

    #[test]
    fn test_context_menu_style_highlight() {
        let style = ContextMenuStyle::default().highlight(Color::Red, Color::Blue);

        assert_eq!(style.highlight_fg, Color::Red);
        assert_eq!(style.highlight_bg, Color::Blue);
    }

    #[test]
    fn test_handle_key_when_closed() {
        let mut state = ContextMenuState::new();
        assert!(!state.is_open);

        let items = vec![ContextMenuItem::action("a", "A")];
        let key = KeyEvent::from(KeyCode::Down);
        let action = handle_context_menu_key(&key, &mut state, &items);

        assert!(action.is_none());
    }

    #[test]
    fn test_handle_key_space_select() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);

        let items = vec![ContextMenuItem::action("a", "Action A")];

        let key = KeyEvent::from(KeyCode::Char(' '));
        let action = handle_context_menu_key(&key, &mut state, &items);

        assert_eq!(action, Some(ContextMenuAction::Select("a".to_string())));
        assert!(!state.is_open);
    }

    #[test]
    fn test_handle_key_home_end() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);

        let items = vec![
            ContextMenuItem::action("a", "A"),
            ContextMenuItem::action("b", "B"),
            ContextMenuItem::action("c", "C"),
            ContextMenuItem::action("d", "D"),
        ];

        // End
        let key = KeyEvent::from(KeyCode::End);
        let action = handle_context_menu_key(&key, &mut state, &items);
        assert_eq!(action, Some(ContextMenuAction::HighlightChange(3)));
        assert_eq!(state.highlighted_index, 3);

        // Home
        let key = KeyEvent::from(KeyCode::Home);
        let action = handle_context_menu_key(&key, &mut state, &items);
        assert_eq!(action, Some(ContextMenuAction::HighlightChange(0)));
        assert_eq!(state.highlighted_index, 0);
    }

    #[test]
    fn test_handle_key_select_disabled_item() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);

        let items = vec![ContextMenuItem::action("a", "A").enabled(false)];

        let key = KeyEvent::from(KeyCode::Enter);
        let action = handle_context_menu_key(&key, &mut state, &items);

        // Should not select disabled item
        assert!(action.is_none());
        assert!(state.is_open); // Still open
    }

    #[test]
    fn test_handle_key_open_submenu() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);

        let items = vec![ContextMenuItem::submenu(
            "More",
            vec![ContextMenuItem::action("sub", "Sub Action")],
        )];

        // Enter to open submenu
        let key = KeyEvent::from(KeyCode::Enter);
        let action = handle_context_menu_key(&key, &mut state, &items);

        assert_eq!(action, Some(ContextMenuAction::SubmenuOpen(0)));
        assert!(state.has_open_submenu());
    }

    #[test]
    fn test_handle_key_right_arrow_submenu() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);

        let items = vec![ContextMenuItem::submenu(
            "More",
            vec![ContextMenuItem::action("sub", "Sub Action")],
        )];

        // Right arrow to open submenu
        let key = KeyEvent::from(KeyCode::Right);
        let action = handle_context_menu_key(&key, &mut state, &items);

        assert_eq!(action, Some(ContextMenuAction::SubmenuOpen(0)));
        assert!(state.has_open_submenu());
    }

    #[test]
    fn test_handle_key_right_arrow_no_submenu() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);

        let items = vec![ContextMenuItem::action("a", "A")];

        // Right arrow on non-submenu item
        let key = KeyEvent::from(KeyCode::Right);
        let action = handle_context_menu_key(&key, &mut state, &items);

        assert!(action.is_none());
        assert!(!state.has_open_submenu());
    }

    #[test]
    fn test_handle_key_left_arrow() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);

        let items = vec![ContextMenuItem::action("a", "A")];

        // Left arrow (no effect at top level)
        let key = KeyEvent::from(KeyCode::Left);
        let action = handle_context_menu_key(&key, &mut state, &items);

        assert!(action.is_none());
    }

    #[test]
    fn test_handle_key_unknown_key() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);

        let items = vec![ContextMenuItem::action("a", "A")];

        // Unknown key should be ignored
        let key = KeyEvent::from(KeyCode::Char('x'));
        let action = handle_context_menu_key(&key, &mut state, &items);

        assert!(action.is_none());
        assert!(state.is_open);
    }

    #[test]
    fn test_handle_mouse_when_closed() {
        use crossterm::event::KeyModifiers;

        let mut state = ContextMenuState::new();
        assert!(!state.is_open);

        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };

        let action = handle_context_menu_mouse(&mouse, &mut state, Rect::default(), &[]);

        assert!(action.is_none());
    }

    #[test]
    fn test_handle_mouse_click_outside() {
        use crossterm::event::KeyModifiers;

        let mut state = ContextMenuState::new();
        state.open_at(10, 10);

        let menu_area = Rect::new(10, 10, 20, 10);

        // Click outside menu
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 5,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };

        let action = handle_context_menu_mouse(&mouse, &mut state, menu_area, &[]);

        assert_eq!(action, Some(ContextMenuAction::Close));
        assert!(!state.is_open);
    }

    #[test]
    fn test_handle_mouse_click_item() {
        use crate::traits::ClickRegion;
        use crossterm::event::KeyModifiers;

        let mut state = ContextMenuState::new();
        state.open_at(10, 10);

        let menu_area = Rect::new(10, 10, 20, 10);
        let item_area = Rect::new(11, 11, 18, 1);
        let regions = vec![ClickRegion::new(
            item_area,
            ContextMenuAction::Select("test".to_string()),
        )];

        // Click on item
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 15,
            row: 11,
            modifiers: KeyModifiers::NONE,
        };

        let action = handle_context_menu_mouse(&mouse, &mut state, menu_area, &regions);

        assert_eq!(action, Some(ContextMenuAction::Select("test".to_string())));
        assert!(!state.is_open);
    }

    #[test]
    fn test_handle_mouse_click_submenu_item() {
        use crate::traits::ClickRegion;
        use crossterm::event::KeyModifiers;

        let mut state = ContextMenuState::new();
        state.open_at(10, 10);

        let menu_area = Rect::new(10, 10, 20, 10);
        let item_area = Rect::new(11, 11, 18, 1);
        let regions = vec![ClickRegion::new(
            item_area,
            ContextMenuAction::SubmenuOpen(0),
        )];

        // Click on submenu item
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 15,
            row: 11,
            modifiers: KeyModifiers::NONE,
        };

        let action = handle_context_menu_mouse(&mouse, &mut state, menu_area, &regions);

        assert_eq!(action, Some(ContextMenuAction::SubmenuOpen(0)));
        assert!(state.has_open_submenu());
    }

    #[test]
    fn test_context_menu_action_equality() {
        assert_eq!(ContextMenuAction::Open, ContextMenuAction::Open);
        assert_eq!(ContextMenuAction::Close, ContextMenuAction::Close);
        assert_eq!(
            ContextMenuAction::Select("a".to_string()),
            ContextMenuAction::Select("a".to_string())
        );
        assert_ne!(
            ContextMenuAction::Select("a".to_string()),
            ContextMenuAction::Select("b".to_string())
        );
        assert_eq!(
            ContextMenuAction::SubmenuOpen(1),
            ContextMenuAction::SubmenuOpen(1)
        );
        assert_eq!(
            ContextMenuAction::SubmenuClose,
            ContextMenuAction::SubmenuClose
        );
        assert_eq!(
            ContextMenuAction::HighlightChange(5),
            ContextMenuAction::HighlightChange(5)
        );
    }

    #[test]
    fn test_context_menu_item_all_disabled() {
        let items = vec![
            ContextMenuItem::separator(),
            ContextMenuItem::action("a", "A").enabled(false),
            ContextMenuItem::separator(),
        ];

        let mut state = ContextMenuState::new();
        state.open_at(0, 0);
        state.highlighted_index = 1;

        // Navigation should not move to any item since none are selectable
        state.highlight_next(&items);
        assert_eq!(state.highlighted_index, 1); // Unchanged

        state.highlight_prev(&items);
        assert_eq!(state.highlighted_index, 1); // Unchanged
    }

    #[test]
    fn test_context_menu_widget_new() {
        let items = vec![ContextMenuItem::action("test", "Test")];
        let state = ContextMenuState::new();
        let _menu = ContextMenu::new(&items, &state);

        // Verify menu is created (we can't easily test rendering without Frame)
        assert!(!state.is_open);
    }

    #[test]
    fn test_context_menu_widget_style() {
        let items = vec![ContextMenuItem::action("test", "Test")];
        let state = ContextMenuState::new();
        let style = ContextMenuStyle::light();
        let _menu = ContextMenu::new(&items, &state).style(style);
    }

    #[test]
    fn test_is_context_menu_trigger_other_events() {
        use crossterm::event::KeyModifiers;

        // Mouse move
        let mouse_move = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        assert!(!is_context_menu_trigger(&mouse_move));

        // Mouse up
        let mouse_up = MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Right),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        assert!(!is_context_menu_trigger(&mouse_up));

        // Middle click
        let middle_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Middle),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        assert!(!is_context_menu_trigger(&middle_click));

        // Scroll
        let scroll = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        assert!(!is_context_menu_trigger(&scroll));
    }

    #[test]
    fn test_context_menu_submenu_disabled() {
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);

        let items = vec![
            ContextMenuItem::submenu("More", vec![ContextMenuItem::action("sub", "Sub")])
                .enabled(false),
        ];

        // Right arrow on disabled submenu should not open it
        let key = KeyEvent::from(KeyCode::Right);
        let action = handle_context_menu_key(&key, &mut state, &items);

        assert!(action.is_none());
        assert!(!state.has_open_submenu());

        // Enter on disabled submenu should not open it
        let key = KeyEvent::from(KeyCode::Enter);
        let action = handle_context_menu_key(&key, &mut state, &items);

        assert!(action.is_none());
        assert!(!state.has_open_submenu());
    }

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
    }

    fn mouse_at(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column,
            row,
            modifiers: crossterm::event::KeyModifiers::NONE,
        }
    }

    fn render(
        items: &[ContextMenuItem],
        state: &ContextMenuState,
        width: u16,
        height: u16,
    ) -> (Rect, Vec<ClickRegion<ContextMenuAction>>) {
        use ratatui::{Terminal, backend::TestBackend};
        let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
        let mut out = (Rect::default(), Vec::new());
        term.draw(|f| out = ContextMenu::new(items, state).render_stateful(f, f.area()))
            .unwrap();
        out
    }

    fn region_of(regions: &[ClickRegion<ContextMenuAction>], action: ContextMenuAction) -> Rect {
        regions.iter().find(|r| r.data == action).unwrap().area
    }

    #[test]
    fn test_submenu_keyboard_select_closes_menu() {
        let items = vec![ContextMenuItem::submenu(
            "More",
            vec![ContextMenuItem::action("x", "X")],
        )];
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);
        handle_context_menu_key(&press(KeyCode::Right), &mut state, &items);
        let action = handle_context_menu_key(&press(KeyCode::Enter), &mut state, &items);
        assert_eq!(action, Some(ContextMenuAction::Select("x".into())));
        assert!(!state.is_open);
    }

    #[test]
    fn test_hover_on_flipped_submenu_no_panic() {
        let sub: Vec<_> = (0..10)
            .map(|i| ContextMenuItem::action(format!("s{i}"), format!("Sub {i}")))
            .collect();
        let items = vec![
            ContextMenuItem::submenu("More", sub),
            ContextMenuItem::action("a", "A"),
            ContextMenuItem::action("b", "B"),
        ];
        let mut state = ContextMenuState::new();
        state.open_at(5, 10);
        state.open_submenu();
        let (area, regions) = render(&items, &state, 80, 20);
        let s3 = region_of(&regions, ContextMenuAction::Select("s3".into()));
        let action = handle_context_menu_mouse(
            &mouse_at(MouseEventKind::Moved, s3.x + 1, s3.y),
            &mut state,
            area,
            &regions,
        );
        // Hover is routed to the submenu, not the parent
        assert_eq!(action, Some(ContextMenuAction::HighlightChange(3)));
        assert_eq!(state.submenu_state.as_ref().unwrap().highlighted_index, 3);
        assert_eq!(state.highlighted_index, 0);
    }

    #[test]
    fn test_flipped_submenu_does_not_overlap_parent() {
        let items = vec![
            ContextMenuItem::submenu(
                "More",
                vec![
                    ContextMenuItem::action("sub0", "S0"),
                    ContextMenuItem::action("sub1", "S1"),
                ],
            ),
            ContextMenuItem::action("delete", "Delete"),
        ];
        let mut state = ContextMenuState::new();
        state.open_at(20, 2);
        state.open_submenu();
        let (area, regions) = render(&items, &state, 40, 20);
        let sub = region_of(&regions, ContextMenuAction::Select("sub0".into()));
        assert!(
            sub.x + sub.width < area.x,
            "submenu {sub:?} overlaps parent {area:?}"
        );
        let action = handle_context_menu_mouse(
            &mouse_at(MouseEventKind::Down(MouseButton::Left), sub.x + 2, sub.y),
            &mut state,
            area,
            &regions,
        );
        assert_eq!(action, Some(ContextMenuAction::Select("sub0".into())));
        assert!(!state.is_open);
    }

    #[test]
    fn test_submenu_regions_win_when_overlapping() {
        // Too narrow for the submenu on either side: it overlaps the parent,
        // and the visible (submenu) item must win.
        let items = vec![
            ContextMenuItem::submenu(
                "More",
                vec![ContextMenuItem::action("sub0", "Submenu zero")],
            ),
            ContextMenuItem::action("delete", "Delete"),
        ];
        let mut state = ContextMenuState::new();
        state.open_at(4, 0);
        state.open_submenu();
        let (area, regions) = render(&items, &state, 24, 10);
        let sub = region_of(&regions, ContextMenuAction::Select("sub0".into()));
        let action = handle_context_menu_mouse(
            &mouse_at(MouseEventKind::Down(MouseButton::Left), sub.x, sub.y),
            &mut state,
            area,
            &regions,
        );
        assert_eq!(action, Some(ContextMenuAction::Select("sub0".into())));
    }

    #[test]
    fn test_keyboard_scroll_uses_style_height() {
        let items: Vec<_> = (0..12)
            .map(|i| ContextMenuItem::action(format!("i{i}"), format!("Item {i}")))
            .collect();
        let mut state = ContextMenuState::new();
        state.open_at(0, 0);
        for _ in 0..8 {
            handle_context_menu_key(&press(KeyCode::Down), &mut state, &items);
        }
        assert_eq!(state.highlighted_index, 8);
        assert_eq!(state.scroll_offset, 0, "all 12 items fit (max 15)");

        let style = ContextMenuStyle::default().max_visible_items(5);
        handle_context_menu_key_with_style(&press(KeyCode::End), &mut state, &items, &style);
        assert_eq!(state.highlighted_index, 11);
        assert_eq!(state.scroll_offset, 7);
        handle_context_menu_key_with_style(&press(KeyCode::Home), &mut state, &items, &style);
        assert_eq!(state.scroll_offset, 0);
    }
}
