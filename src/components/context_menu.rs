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
}

impl<'a> ContextMenu<'a> {
    /// Create a new context menu.
    pub fn new(items: &'a [ContextMenuItem], state: &'a ContextMenuState) -> Self {
        Self {
            items,
            state,
            style: ContextMenuStyle::default(),
        }
    }

    /// Set the style.
    pub fn style(mut self, style: ContextMenuStyle) -> Self {
        self.style = style;
        self
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
                    let icon_width = icon.as_ref().map(|i| i.chars().count() + 1).unwrap_or(0);
                    let label_width = label.chars().count() + icon_width;
                    max_label_width = max_label_width.max(label_width as u16);
                    if let Some(s) = shortcut {
                        max_shortcut_width = max_shortcut_width.max(s.chars().count() as u16);
                    }
                }
                ContextMenuItem::Submenu { label, icon, .. } => {
                    let icon_width = icon.as_ref().map(|i| i.chars().count() + 1).unwrap_or(0);
                    // +2 for submenu indicator
                    let label_width = label.chars().count() + icon_width + 2;
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
        let x = if anchor_x + width <= screen.x + screen.width {
            anchor_x
        } else {
            anchor_x.saturating_sub(width)
        };

        let y = if anchor_y + height <= screen.y + screen.height {
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
                    spans.push(Span::styled(
                        " ".repeat(self.style.padding as usize),
                        style,
                    ));

                    // Icon
                    if let Some(ic) = icon {
                        spans.push(Span::styled(format!("{} ", ic), style));
                    }

                    // Label
                    spans.push(Span::styled(label.clone(), style));

                    // Fill space before shortcut
                    let current_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
                    let shortcut_len = shortcut.as_ref().map(|s| s.chars().count()).unwrap_or(0);
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
                    spans.push(Span::styled(
                        " ".repeat(self.style.padding as usize),
                        style,
                    ));

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
                    spans.push(Span::styled(
                        " ".repeat(self.style.padding as usize),
                        style,
                    ));

                    // Icon
                    if let Some(ic) = icon {
                        spans.push(Span::styled(format!("{} ", ic), style));
                    }

                    // Label
                    spans.push(Span::styled(label.clone(), style));

                    // Fill and submenu indicator
                    let current_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
                    let indicator_len = self.style.submenu_indicator.chars().count();
                    let fill_len = (inner.width as usize)
                        .saturating_sub(current_len)
                        .saturating_sub(indicator_len)
                        .saturating_sub(self.style.padding as usize);

                    if fill_len > 0 {
                        spans.push(Span::styled(" ".repeat(fill_len), style));
                    }

                    spans.push(Span::styled(self.style.submenu_indicator, style));

                    // Right padding
                    spans.push(Span::styled(
                        " ".repeat(self.style.padding as usize),
                        style,
                    ));

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
            let submenu_anchor_x = menu_area.x + menu_area.width;
            let submenu_anchor_y =
                menu_area.y + 1 + (submenu_idx as u16).saturating_sub(self.state.scroll_offset);

            let mut adjusted_state = (**submenu_state).clone();
            adjusted_state.anchor_position = (submenu_anchor_x, submenu_anchor_y);

            let adjusted_submenu =
                ContextMenu::new(items, &adjusted_state).style(self.style.clone());

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
    if !state.is_open {
        return None;
    }

    // If submenu is open, delegate to it first
    if let (Some(submenu_idx), Some(submenu_state)) =
        (state.active_submenu, &mut state.submenu_state)
        && let Some(ContextMenuItem::Submenu { items: sub_items, .. }) = items.get(submenu_idx)
    {
        match key.code {
            KeyCode::Left | KeyCode::Esc => {
                state.close_submenu();
                return Some(ContextMenuAction::SubmenuClose);
            }
            _ => {
                if let Some(action) =
                    handle_context_menu_key(key, submenu_state.as_mut(), sub_items)
                {
                    return Some(action);
                }
            }
        }
        return None;
    }

    match key.code {
        KeyCode::Esc => {
            state.close();
            Some(ContextMenuAction::Close)
        }
        KeyCode::Up => {
            state.highlight_prev(items);
            state.ensure_visible(8);
            Some(ContextMenuAction::HighlightChange(state.highlighted_index))
        }
        KeyCode::Down => {
            state.highlight_next(items);
            state.ensure_visible(8);
            Some(ContextMenuAction::HighlightChange(state.highlighted_index))
        }
        KeyCode::Home => {
            state.highlight_first(items);
            Some(ContextMenuAction::HighlightChange(state.highlighted_index))
        }
        KeyCode::End => {
            state.highlight_last(items);
            state.ensure_visible(items.len());
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

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Check if clicked on an item
            for region in item_regions {
                if region.contains(col, row) {
                    match &region.data {
                        ContextMenuAction::Select(id) => {
                            let action_id = id.clone();
                            state.close();
                            return Some(ContextMenuAction::Select(action_id));
                        }
                        ContextMenuAction::SubmenuOpen(idx) => {
                            state.highlighted_index = *idx;
                            state.open_submenu();
                            return Some(ContextMenuAction::SubmenuOpen(*idx));
                        }
                        _ => {}
                    }
                }
            }

            // Check if clicked outside menu
            if !menu_area.intersects(Rect::new(col, row, 1, 1)) {
                state.close();
                return Some(ContextMenuAction::Close);
            }
            None
        }
        MouseEventKind::Moved => {
            // Update highlight on hover
            for region in item_regions.iter() {
                if region.contains(col, row) {
                    // Find the actual item index from the region
                    if let ContextMenuAction::Select(_) | ContextMenuAction::SubmenuOpen(_) =
                        &region.data
                    {
                        // The item_regions index may not match the items index due to separators
                        // We need to find the corresponding item
                        let inner_start_y = menu_area.y + 1; // +1 for border
                        let item_idx = (row - inner_start_y) as usize + state.scroll_offset as usize;

                        if item_idx < item_regions.len() + state.scroll_offset as usize
                            && state.highlighted_index != item_idx
                        {
                            state.highlighted_index = item_idx;
                            return Some(ContextMenuAction::HighlightChange(item_idx));
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
            ContextMenuItem::separator(), // index 0 - not selectable
            ContextMenuItem::action("a", "A"), // index 1
            ContextMenuItem::action("b", "B"), // index 2
            ContextMenuItem::separator(), // index 3 - not selectable
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
        let style = ContextMenuStyle::default()
            .highlight(Color::Red, Color::Blue);

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

        let action =
            handle_context_menu_mouse(&mouse, &mut state, Rect::default(), &[]);

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
        use crossterm::event::KeyModifiers;
        use crate::traits::ClickRegion;

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
        use crossterm::event::KeyModifiers;
        use crate::traits::ClickRegion;

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
        assert_eq!(ContextMenuAction::SubmenuClose, ContextMenuAction::SubmenuClose);
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
        let menu = ContextMenu::new(&items, &state);

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

        let items = vec![ContextMenuItem::submenu(
            "More",
            vec![ContextMenuItem::action("sub", "Sub")],
        )
        .enabled(false)];

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
}
