//! Tab view layout component
//!
//! A tab bar with content area that switches based on selected tab.
//! Supports tabs on any side (top, bottom, left, right) with full keyboard
//! and mouse interaction.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{Tab, TabView, TabViewState, TabViewStyle, TabPosition};
//! use ratatui::{buffer::Buffer, layout::Rect, widgets::Paragraph};
//!
//! // Create tabs
//! let tabs = vec![
//!     Tab::new("General").icon("⚙"),
//!     Tab::new("Network").icon("🌐").badge("3"),
//!     Tab::new("Security"),
//! ];
//!
//! // Create state
//! let mut state = TabViewState::new(tabs.len());
//!
//! // Create style with tabs on the left
//! let style = TabViewStyle::left().tab_width(20);
//!
//! // Create tab view with content renderer
//! let tab_view = TabView::new(&tabs, &state)
//!     .style(style)
//!     .content(|idx, area, buf| {
//!         let text = match idx {
//!             0 => "General settings",
//!             1 => "Network configuration",
//!             _ => "Security options",
//!         };
//!         use ratatui::widgets::Widget;
//!         Paragraph::new(text).render(area, buf);
//!     });
//! ```

use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
};
use unicode_width::UnicodeWidthStr;

use crate::traits::{ClickRegionRegistry, FocusId, Focusable};

/// Position of the tab bar relative to content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabPosition {
    /// Tabs above content (default)
    #[default]
    Top,
    /// Tabs below content
    Bottom,
    /// Tabs on left side of content
    Left,
    /// Tabs on right side of content
    Right,
}

impl TabPosition {
    /// Whether this position has horizontal tabs
    pub fn is_horizontal(&self) -> bool {
        matches!(self, TabPosition::Top | TabPosition::Bottom)
    }

    /// Whether this position has vertical tabs
    pub fn is_vertical(&self) -> bool {
        matches!(self, TabPosition::Left | TabPosition::Right)
    }
}

/// A single tab item
#[derive(Debug, Clone)]
pub struct Tab<'a> {
    /// Tab label text
    pub label: &'a str,
    /// Optional icon before label
    pub icon: Option<&'a str>,
    /// Optional badge (e.g., notification count)
    pub badge: Option<&'a str>,
    /// Whether this tab is enabled
    pub enabled: bool,
}

impl<'a> Tab<'a> {
    /// Create a new tab with the given label
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: None,
            badge: None,
            enabled: true,
        }
    }

    /// Set an icon for the tab
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set a badge for the tab
    pub fn badge(mut self, badge: &'a str) -> Self {
        self.badge = Some(badge);
        self
    }

    /// Set whether the tab is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Calculate the display width of this tab
    pub fn display_width(&self) -> usize {
        let mut width = self.label.width();
        if let Some(icon) = self.icon {
            width += icon.width() + 1; // icon + space
        }
        if let Some(badge) = self.badge {
            width += badge.width() + 2; // space + badge + padding
        }
        width + 2 // padding on both sides
    }
}

/// State for the tab view component
#[derive(Debug, Clone)]
pub struct TabViewState {
    /// Currently selected tab index
    pub selected_index: usize,
    /// Total number of tabs
    pub total_tabs: usize,
    /// Scroll offset for overflow tabs
    pub scroll_offset: usize,
    /// Whether the tab bar has focus (vs content)
    pub tab_bar_focused: bool,
    /// Focus ID for focus management
    pub focus_id: FocusId,
    /// Whether this component has focus
    pub focused: bool,
}

impl TabViewState {
    /// Create a new tab view state
    pub fn new(total_tabs: usize) -> Self {
        Self {
            selected_index: 0,
            total_tabs,
            scroll_offset: 0,
            tab_bar_focused: true,
            focus_id: FocusId::default(),
            focused: false,
        }
    }

    /// Create a new tab view state with a specific focus ID
    pub fn with_focus_id(total_tabs: usize, focus_id: FocusId) -> Self {
        Self {
            selected_index: 0,
            total_tabs,
            scroll_offset: 0,
            tab_bar_focused: true,
            focus_id,
            focused: false,
        }
    }

    /// Select the next tab
    pub fn select_next(&mut self) {
        if self.selected_index + 1 < self.total_tabs {
            self.selected_index += 1;
        }
    }

    /// Select the previous tab
    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Select a specific tab by index
    pub fn select(&mut self, index: usize) {
        if index < self.total_tabs {
            self.selected_index = index;
        }
    }

    /// Select the first tab
    pub fn select_first(&mut self) {
        self.selected_index = 0;
    }

    /// Select the last tab
    pub fn select_last(&mut self) {
        if self.total_tabs > 0 {
            self.selected_index = self.total_tabs - 1;
        }
    }

    /// Toggle focus between tab bar and content
    pub fn toggle_focus(&mut self) {
        self.tab_bar_focused = !self.tab_bar_focused;
    }

    /// Ensure the selected tab is visible within the viewport
    pub fn ensure_visible(&mut self, visible_count: usize) {
        if visible_count == 0 {
            return;
        }

        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_count {
            self.scroll_offset = self.selected_index - visible_count + 1;
        }
    }

    /// Update the total number of tabs
    pub fn set_total(&mut self, total: usize) {
        self.total_tabs = total;
        if self.selected_index >= total && total > 0 {
            self.selected_index = total - 1;
        }
    }
}

impl Default for TabViewState {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Focusable for TabViewState {
    fn focus_id(&self) -> FocusId {
        self.focus_id
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

/// Style configuration for the tab view
#[derive(Debug, Clone)]
pub struct TabViewStyle {
    /// Position of the tab bar
    pub position: TabPosition,
    /// Style for selected tab
    pub selected_style: Style,
    /// Style for normal (unselected) tabs
    pub normal_style: Style,
    /// Style for focused tab (when component has focus)
    pub focused_style: Style,
    /// Style for disabled tabs
    pub disabled_style: Style,
    /// Style for badge text
    pub badge_style: Style,
    /// Style for the content area border
    pub content_border_style: Style,
    /// Tab divider character(s)
    pub divider: &'static str,
    /// Fixed width for vertical tabs (None = auto)
    pub tab_width: Option<u16>,
    /// Height for horizontal tabs
    pub tab_height: u16,
    /// Whether to show border around content
    pub bordered_content: bool,
    /// Whether to show selection indicator
    pub show_indicator: bool,
    /// Selection indicator character
    pub indicator: &'static str,
    /// Scroll left indicator
    pub scroll_left: &'static str,
    /// Scroll right indicator
    pub scroll_right: &'static str,
    /// Scroll up indicator
    pub scroll_up: &'static str,
    /// Scroll down indicator
    pub scroll_down: &'static str,
}

impl Default for TabViewStyle {
    fn default() -> Self {
        Self {
            position: TabPosition::Top,
            selected_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            normal_style: Style::default().fg(Color::White),
            focused_style: Style::default()
                .fg(Color::Yellow)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            disabled_style: Style::default().fg(Color::DarkGray),
            badge_style: Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
            content_border_style: Style::default().fg(Color::Cyan),
            divider: " │ ",
            tab_width: None,
            tab_height: 1,
            bordered_content: true,
            show_indicator: true,
            indicator: "▸",
            scroll_left: "◀",
            scroll_right: "▶",
            scroll_up: "▲",
            scroll_down: "▼",
        }
    }
}

impl From<&crate::theme::Theme> for TabViewStyle {
    fn from(theme: &crate::theme::Theme) -> Self {
        let p = &theme.palette;
        Self {
            position: TabPosition::Top,
            selected_style: Style::default().fg(p.primary).add_modifier(Modifier::BOLD),
            normal_style: Style::default().fg(p.text),
            focused_style: Style::default()
                .fg(p.primary)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            disabled_style: Style::default().fg(p.text_disabled),
            badge_style: Style::default()
                .fg(p.highlight_fg)
                .bg(p.error)
                .add_modifier(Modifier::BOLD),
            content_border_style: Style::default().fg(p.border_accent),
            divider: " │ ",
            tab_width: None,
            tab_height: 1,
            bordered_content: true,
            show_indicator: true,
            indicator: "▸",
            scroll_left: "◀",
            scroll_right: "▶",
            scroll_up: "▲",
            scroll_down: "▼",
        }
    }
}

impl TabViewStyle {
    /// Create a style with tabs on top
    pub fn top() -> Self {
        Self::default()
    }

    /// Create a style with tabs on bottom
    pub fn bottom() -> Self {
        Self {
            position: TabPosition::Bottom,
            ..Default::default()
        }
    }

    /// Create a style with tabs on left
    pub fn left() -> Self {
        Self {
            position: TabPosition::Left,
            tab_width: Some(16),
            divider: "",
            ..Default::default()
        }
    }

    /// Create a style with tabs on right
    pub fn right() -> Self {
        Self {
            position: TabPosition::Right,
            tab_width: Some(16),
            divider: "",
            ..Default::default()
        }
    }

    /// Create a minimal style (no borders, simple)
    pub fn minimal() -> Self {
        Self {
            bordered_content: false,
            show_indicator: false,
            divider: "  ",
            ..Default::default()
        }
    }

    /// Set the tab position
    pub fn position(mut self, position: TabPosition) -> Self {
        self.position = position;
        self
    }

    /// Set the tab width for vertical tabs
    pub fn tab_width(mut self, width: u16) -> Self {
        self.tab_width = Some(width);
        self
    }

    /// Set the tab height for horizontal tabs
    pub fn tab_height(mut self, height: u16) -> Self {
        self.tab_height = height;
        self
    }

    /// Set whether content is bordered
    pub fn bordered_content(mut self, bordered: bool) -> Self {
        self.bordered_content = bordered;
        self
    }

    /// Set selected tab style
    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Set normal tab style
    pub fn normal_style(mut self, style: Style) -> Self {
        self.normal_style = style;
        self
    }

    /// Set the divider between tabs
    pub fn divider(mut self, divider: &'static str) -> Self {
        self.divider = divider;
        self
    }
}

/// Actions that can be triggered by clicking on the tab view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabViewAction {
    /// A specific tab was clicked
    TabClick(usize),
    /// Scroll to previous tabs
    ScrollPrev,
    /// Scroll to next tabs
    ScrollNext,
}

/// Default content renderer type
type DefaultContentRenderer = fn(usize, Rect, &mut Buffer);

/// Tab view widget
///
/// A layout component with a tab bar and content area.
pub struct TabView<'a, F = DefaultContentRenderer>
where
    F: Fn(usize, Rect, &mut Buffer),
{
    tabs: &'a [Tab<'a>],
    state: &'a TabViewState,
    style: TabViewStyle,
    content_renderer: Option<F>,
}

impl<'a> TabView<'a, DefaultContentRenderer> {
    /// Create a new tab view with the given tabs and state
    pub fn new(tabs: &'a [Tab<'a>], state: &'a TabViewState) -> Self {
        Self {
            tabs,
            state,
            style: TabViewStyle::default(),
            content_renderer: None,
        }
    }
}

impl<'a, F> TabView<'a, F>
where
    F: Fn(usize, Rect, &mut Buffer),
{
    /// Set the style for the tab view
    pub fn style(mut self, style: TabViewStyle) -> Self {
        self.style = style;
        self
    }

    /// Apply a theme to derive the style
    pub fn theme(self, theme: &crate::theme::Theme) -> Self {
        self.style(TabViewStyle::from(theme))
    }

    /// Set the content renderer
    ///
    /// The function receives: (selected_index, content_area, buffer)
    pub fn content<G>(self, renderer: G) -> TabView<'a, G>
    where
        G: Fn(usize, Rect, &mut Buffer),
    {
        TabView {
            tabs: self.tabs,
            state: self.state,
            style: self.style,
            content_renderer: Some(renderer),
        }
    }

    /// Calculate the layout areas for tab bar and content
    fn calculate_layout(&self, area: Rect) -> (Rect, Rect) {
        let (direction, constraints) = match self.style.position {
            TabPosition::Top => (
                Direction::Vertical,
                [
                    Constraint::Length(self.style.tab_height),
                    Constraint::Min(1),
                ],
            ),
            TabPosition::Bottom => (
                Direction::Vertical,
                [
                    Constraint::Min(1),
                    Constraint::Length(self.style.tab_height),
                ],
            ),
            TabPosition::Left => {
                let width = self.style.tab_width.unwrap_or(16);
                (
                    Direction::Horizontal,
                    [Constraint::Length(width), Constraint::Min(1)],
                )
            }
            TabPosition::Right => {
                let width = self.style.tab_width.unwrap_or(16);
                (
                    Direction::Horizontal,
                    [Constraint::Min(1), Constraint::Length(width)],
                )
            }
        };

        let chunks = Layout::default()
            .direction(direction)
            .constraints(constraints)
            .split(area);

        match self.style.position {
            TabPosition::Top | TabPosition::Left => (chunks[0], chunks[1]),
            TabPosition::Bottom | TabPosition::Right => (chunks[1], chunks[0]),
        }
    }

    /// Render the tab bar and return click regions
    fn render_tab_bar(&self, area: Rect, buf: &mut Buffer) -> Vec<(Rect, TabViewAction)> {
        let mut click_regions = Vec::new();

        if self.style.position.is_horizontal() {
            self.render_horizontal_tabs(area, buf, &mut click_regions);
        } else {
            self.render_vertical_tabs(area, buf, &mut click_regions);
        }

        click_regions
    }

    /// Scroll offset clamped to the current number of tabs
    fn effective_scroll(&self) -> usize {
        self.state
            .scroll_offset
            .min(self.tabs.len().saturating_sub(1))
    }

    /// Text of a horizontal tab (including padding / selection indicator)
    fn horizontal_tab_text(&self, idx: usize, tab: &Tab) -> String {
        let mut text = String::new();
        if let Some(icon) = tab.icon {
            text.push_str(icon);
            text.push(' ');
        }
        text.push_str(tab.label);

        if self.state.selected_index == idx && self.style.show_indicator {
            format!("{} {} ", self.style.indicator, text)
        } else {
            format!(" {} ", text)
        }
    }

    /// Rendered width of a horizontal tab (text plus badge, without divider)
    fn horizontal_tab_width(&self, idx: usize, tab: &Tab) -> u16 {
        let mut width = self.horizontal_tab_text(idx, tab).width();
        if let Some(badge) = tab.badge {
            width += badge.width() + 2;
        }
        width.min(u16::MAX as usize) as u16
    }

    /// Render horizontal tabs (top/bottom)
    fn render_horizontal_tabs(
        &self,
        area: Rect,
        buf: &mut Buffer,
        click_regions: &mut Vec<(Rect, TabViewAction)>,
    ) {
        if area.width == 0 || area.height == 0 || self.tabs.is_empty() {
            return;
        }

        let mut x = area.x;
        let y = area.y;
        let right = area.x.saturating_add(area.width);
        let scroll = self.effective_scroll();
        let indicator_width = 2.min(area.width);

        // Check if we need scroll indicators
        let has_overflow = self.calculate_overflow_horizontal(area.width);
        let show_prev = scroll > 0;
        let show_next = has_overflow
            && scroll + self.visible_tabs_horizontal(scroll, area.width) < self.tabs.len();

        // Render scroll-left indicator
        if show_prev {
            let indicator = self.style.scroll_left;
            let indicator_area = Rect::new(x, y, indicator_width, 1);
            buf.set_stringn(
                x,
                y,
                indicator,
                indicator_width as usize,
                Style::default().fg(Color::Yellow),
            );
            click_regions.push((indicator_area, TabViewAction::ScrollPrev));
            x += indicator_width;
        }

        // Tabs must not run into the scroll-right indicator or past the area
        let tabs_right = if show_next {
            right.saturating_sub(indicator_width).max(x)
        } else {
            right
        };

        // Render visible tabs
        let visible_start = scroll;
        let visible_count = self.visible_tabs_horizontal(
            scroll,
            area.width
                .saturating_sub(if show_prev { 2 } else { 0 })
                .saturating_sub(if show_next { 2 } else { 0 }),
        );

        for (idx, tab) in self
            .tabs
            .iter()
            .enumerate()
            .skip(visible_start)
            .take(visible_count)
        {
            if x >= tabs_right {
                break;
            }

            // Remember start position for click region
            let tab_start_x = x;

            // Determine style
            let style = self.get_tab_style(idx, tab.enabled);

            // Render the text using unicode width, clipped to the tab bar
            let text_with_padding = self.horizontal_tab_text(idx, tab);
            x = buf
                .set_stringn(x, y, &text_with_padding, (tabs_right - x) as usize, style)
                .0;

            // Render badge if present (included in click region)
            if let Some(badge) = tab.badge
                && x < tabs_right
            {
                let badge_text = format!(" {} ", badge);
                x = buf
                    .set_stringn(
                        x,
                        y,
                        &badge_text,
                        (tabs_right - x) as usize,
                        self.style.badge_style,
                    )
                    .0;
            }

            // Calculate actual tab width and register click region
            // (disabled tabs are drawn but not clickable)
            let tab_width = x - tab_start_x;
            if tab_width > 0 && tab.enabled {
                let tab_area = Rect::new(tab_start_x, y, tab_width, 1);
                click_regions.push((tab_area, TabViewAction::TabClick(idx)));
            }

            // Render divider (if not last visible) - not part of click region
            if idx + 1 < visible_start + visible_count
                && idx + 1 < self.tabs.len()
                && x < tabs_right
            {
                x = buf
                    .set_stringn(
                        x,
                        y,
                        self.style.divider,
                        (tabs_right - x) as usize,
                        Style::default().fg(Color::DarkGray),
                    )
                    .0;
            }
        }

        // Render scroll-right indicator
        if show_next {
            let indicator = self.style.scroll_right;
            let indicator_x = right.saturating_sub(indicator_width);
            let indicator_area = Rect::new(indicator_x, y, indicator_width, 1);
            buf.set_stringn(
                indicator_x,
                y,
                indicator,
                indicator_width as usize,
                Style::default().fg(Color::Yellow),
            );
            click_regions.push((indicator_area, TabViewAction::ScrollNext));
        }
    }

    /// Render vertical tabs (left/right)
    fn render_vertical_tabs(
        &self,
        area: Rect,
        buf: &mut Buffer,
        click_regions: &mut Vec<(Rect, TabViewAction)>,
    ) {
        if area.width == 0 || area.height == 0 || self.tabs.is_empty() {
            return;
        }

        let x = area.x;
        let mut y = area.y;
        let width = area.width;
        let bottom = area.y.saturating_add(area.height);
        let scroll = self.effective_scroll();

        // Check if we need scroll indicators
        let visible_count = (area.height as usize).min(self.tabs.len());
        let show_prev = scroll > 0;
        let show_next = scroll + visible_count < self.tabs.len();

        // Render scroll-up indicator
        if show_prev {
            let indicator = format!("{:^width$}", self.style.scroll_up, width = width as usize);
            buf.set_stringn(
                x,
                y,
                &indicator,
                width as usize,
                Style::default().fg(Color::Yellow),
            );
            click_regions.push((Rect::new(x, y, width, 1), TabViewAction::ScrollPrev));
            y += 1;
        }

        // Render visible tabs
        let available_height = area
            .height
            .saturating_sub(if show_prev { 1 } else { 0 })
            .saturating_sub(if show_next { 1 } else { 0 });
        let visible_start = scroll;
        let visible_count =
            (available_height as usize).min(self.tabs.len().saturating_sub(visible_start));
        let tabs_bottom = bottom.saturating_sub(if show_next { 1 } else { 0 });

        for (idx, tab) in self
            .tabs
            .iter()
            .enumerate()
            .skip(visible_start)
            .take(visible_count)
        {
            if y >= tabs_bottom {
                break;
            }

            // Build tab text
            let mut text = String::new();
            if self.state.selected_index == idx && self.style.show_indicator {
                text.push_str(self.style.indicator);
                text.push(' ');
            } else {
                text.push_str("  ");
            }
            if let Some(icon) = tab.icon {
                text.push_str(icon);
                text.push(' ');
            }
            text.push_str(tab.label);

            // Add badge
            if let Some(badge) = tab.badge {
                text.push_str(&format!(" ({})", badge));
            }

            // Truncate if too long (by display width)
            let max_len = width as usize;
            let display_text = if text.width() > max_len {
                let mut truncated = String::new();
                let mut used = 0;
                for ch in text.chars() {
                    let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                    if used + w > max_len.saturating_sub(1) {
                        break;
                    }
                    used += w;
                    truncated.push(ch);
                }
                format!("{}…", truncated)
            } else {
                format!("{}{}", text, " ".repeat(max_len - text.width()))
            };

            // Determine style
            let style = self.get_tab_style(idx, tab.enabled);

            let tab_area = Rect::new(x, y, width, 1);
            buf.set_stringn(x, y, &display_text, max_len, style);
            // Disabled tabs are drawn but not clickable
            if tab.enabled {
                click_regions.push((tab_area, TabViewAction::TabClick(idx)));
            }

            y += 1;
        }

        // Render scroll-down indicator
        if show_next {
            let indicator_y = bottom - 1;
            let indicator = format!("{:^width$}", self.style.scroll_down, width = width as usize);
            buf.set_stringn(
                x,
                indicator_y,
                &indicator,
                width as usize,
                Style::default().fg(Color::Yellow),
            );
            click_regions.push((
                Rect::new(x, indicator_y, width, 1),
                TabViewAction::ScrollNext,
            ));
        }
    }

    /// Get the appropriate style for a tab
    fn get_tab_style(&self, idx: usize, enabled: bool) -> Style {
        if !enabled {
            self.style.disabled_style
        } else if idx == self.state.selected_index
            && self.state.focused
            && self.state.tab_bar_focused
        {
            self.style.focused_style
        } else if idx == self.state.selected_index {
            self.style.selected_style
        } else {
            self.style.normal_style
        }
    }

    /// Calculate if horizontal tabs overflow
    fn calculate_overflow_horizontal(&self, available_width: u16) -> bool {
        let divider = self.style.divider.width() as u16;
        let total_width: u16 = self
            .tabs
            .iter()
            .enumerate()
            .map(|(idx, t)| self.horizontal_tab_width(idx, t).saturating_add(divider))
            .fold(0u16, |acc, w| acc.saturating_add(w))
            .saturating_sub(divider); // no divider after the last tab
        total_width > available_width
    }

    /// Calculate how many tabs fit horizontally starting at `scroll`
    fn visible_tabs_horizontal(&self, scroll: usize, available_width: u16) -> usize {
        let divider = self.style.divider.width() as u16;
        let mut width = 0u16;
        let mut count = 0;
        for (idx, tab) in self.tabs.iter().enumerate().skip(scroll) {
            let tab_width = self.horizontal_tab_width(idx, tab);
            let needed = if count == 0 {
                tab_width
            } else {
                tab_width.saturating_add(divider)
            };
            if width.saturating_add(needed) > available_width {
                break;
            }
            width = width.saturating_add(needed);
            count += 1;
        }
        count.max(1)
    }

    /// Render content area
    fn render_content(&self, area: Rect, buf: &mut Buffer) {
        let inner = if self.style.bordered_content {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(self.style.content_border_style);
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if let Some(ref renderer) = self.content_renderer {
            renderer(self.state.selected_index, inner, buf);
        }
    }

    /// Render the tab view and return click regions
    pub fn render_stateful(self, area: Rect, buf: &mut Buffer) -> Vec<(Rect, TabViewAction)> {
        let (tab_area, content_area) = self.calculate_layout(area);

        // Render tab bar
        let click_regions = self.render_tab_bar(tab_area, buf);

        // Render content
        self.render_content(content_area, buf);

        click_regions
    }

    /// Render and register click regions
    pub fn render_with_registry(
        self,
        area: Rect,
        buf: &mut Buffer,
        registry: &mut ClickRegionRegistry<TabViewAction>,
    ) {
        let regions = self.render_stateful(area, buf);
        for (rect, action) in regions {
            registry.register(rect, action);
        }
    }
}

impl<'a, F> Widget for TabView<'a, F>
where
    F: Fn(usize, Rect, &mut Buffer),
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let _ = self.render_stateful(area, buf);
    }
}

/// Handle keyboard events for the tab view
///
/// Returns true if the event was handled.
///
/// This handler does not know which tabs are disabled; use
/// [`handle_tab_view_key_with_tabs`] to skip disabled tabs.
pub fn handle_tab_view_key(
    state: &mut TabViewState,
    key: &KeyEvent,
    position: TabPosition,
) -> bool {
    handle_tab_view_key_impl(state, key, position, |_| true)
}

/// Handle keyboard events for the tab view, skipping disabled tabs
///
/// Like [`handle_tab_view_key`], but navigation (arrows, Home/End and number
/// keys) never selects a tab whose [`Tab::enabled`] is `false`.
///
/// Returns true if the event was handled.
pub fn handle_tab_view_key_with_tabs(
    state: &mut TabViewState,
    key: &KeyEvent,
    position: TabPosition,
    tabs: &[Tab],
) -> bool {
    handle_tab_view_key_impl(state, key, position, |idx| {
        tabs.get(idx).is_none_or(|t| t.enabled)
    })
}

fn handle_tab_view_key_impl(
    state: &mut TabViewState,
    key: &KeyEvent,
    position: TabPosition,
    is_enabled: impl Fn(usize) -> bool,
) -> bool {
    let total = state.total_tabs;
    // Handle tab bar navigation based on position
    if state.tab_bar_focused {
        let prev = |state: &mut TabViewState| {
            if let Some(idx) = (0..state.selected_index.min(total))
                .rev()
                .find(|&i| is_enabled(i))
            {
                state.selected_index = idx;
            }
        };
        let next = |state: &mut TabViewState| {
            if let Some(idx) = (state.selected_index + 1..total).find(|&i| is_enabled(i)) {
                state.selected_index = idx;
            }
        };
        match key.code {
            // Horizontal navigation for horizontal tabs
            KeyCode::Left if position.is_horizontal() => {
                prev(state);
                true
            }
            KeyCode::Right if position.is_horizontal() => {
                next(state);
                true
            }
            // Vertical navigation for vertical tabs
            KeyCode::Up if position.is_vertical() => {
                prev(state);
                true
            }
            KeyCode::Down if position.is_vertical() => {
                next(state);
                true
            }
            // Home/End
            KeyCode::Home => {
                if let Some(idx) = (0..total).find(|&i| is_enabled(i)) {
                    state.selected_index = idx;
                } else {
                    state.select_first();
                }
                true
            }
            KeyCode::End => {
                if let Some(idx) = (0..total).rev().find(|&i| is_enabled(i)) {
                    state.selected_index = idx;
                } else {
                    state.select_last();
                }
                true
            }
            // Number keys for direct selection (1-9)
            KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                let idx = (c as usize) - ('1' as usize);
                if idx < total && is_enabled(idx) {
                    state.select(idx);
                }
                true
            }
            // Enter to focus content
            KeyCode::Enter => {
                state.toggle_focus();
                true
            }
            _ => false,
        }
    } else {
        // Content focused - Escape to go back to tab bar
        match key.code {
            KeyCode::Esc => {
                state.toggle_focus();
                true
            }
            _ => false,
        }
    }
}

/// Handle mouse events for the tab view
///
/// Returns the action if a click was handled.
pub fn handle_tab_view_mouse(
    state: &mut TabViewState,
    registry: &ClickRegionRegistry<TabViewAction>,
    mouse: &MouseEvent,
) -> Option<TabViewAction> {
    use crossterm::event::{MouseButton, MouseEventKind};

    if let MouseEventKind::Down(MouseButton::Left) = mouse.kind
        && let Some(action) = registry.handle_click(mouse.column, mouse.row)
    {
        match action {
            TabViewAction::TabClick(idx) => {
                state.select(*idx);
                state.tab_bar_focused = true;
                return Some(*action);
            }
            TabViewAction::ScrollPrev => {
                if state.scroll_offset > 0 {
                    state.scroll_offset -= 1;
                }
                return Some(*action);
            }
            TabViewAction::ScrollNext => {
                if state.scroll_offset + 1 < state.total_tabs {
                    state.scroll_offset += 1;
                }
                return Some(*action);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_creation() {
        let tab = Tab::new("Test").icon("🔧").badge("5").enabled(true);

        assert_eq!(tab.label, "Test");
        assert_eq!(tab.icon, Some("🔧"));
        assert_eq!(tab.badge, Some("5"));
        assert!(tab.enabled);
    }

    #[test]
    fn test_tab_display_width() {
        let simple = Tab::new("Test");
        // " Test " = 6 chars
        assert_eq!(simple.display_width(), 6);

        let with_icon = Tab::new("Test").icon("⚙");
        // " ⚙ Test " = 8 chars
        assert_eq!(with_icon.display_width(), 8);

        let with_badge = Tab::new("Test").badge("3");
        // " Test " + " 3 " = 6 + 3 = 9 chars
        assert_eq!(with_badge.display_width(), 9);
    }

    #[test]
    fn test_state_navigation() {
        let mut state = TabViewState::new(5);
        assert_eq!(state.selected_index, 0);

        state.select_next();
        assert_eq!(state.selected_index, 1);

        state.select_prev();
        assert_eq!(state.selected_index, 0);

        state.select_prev(); // Should not go below 0
        assert_eq!(state.selected_index, 0);

        state.select_last();
        assert_eq!(state.selected_index, 4);

        state.select_next(); // Should not go above total
        assert_eq!(state.selected_index, 4);

        state.select_first();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_state_direct_select() {
        let mut state = TabViewState::new(5);

        state.select(3);
        assert_eq!(state.selected_index, 3);

        state.select(10); // Out of range - should not change
        assert_eq!(state.selected_index, 3);
    }

    #[test]
    fn test_state_focus_toggle() {
        let mut state = TabViewState::new(3);
        assert!(state.tab_bar_focused);

        state.toggle_focus();
        assert!(!state.tab_bar_focused);

        state.toggle_focus();
        assert!(state.tab_bar_focused);
    }

    #[test]
    fn test_ensure_visible() {
        let mut state = TabViewState::new(20);
        state.selected_index = 15;
        state.ensure_visible(10);
        assert!(state.scroll_offset >= 6); // 15 - 10 + 1 = 6
    }

    #[test]
    fn test_tab_position() {
        assert!(TabPosition::Top.is_horizontal());
        assert!(TabPosition::Bottom.is_horizontal());
        assert!(TabPosition::Left.is_vertical());
        assert!(TabPosition::Right.is_vertical());

        assert!(!TabPosition::Top.is_vertical());
        assert!(!TabPosition::Left.is_horizontal());
    }

    #[test]
    fn test_style_presets() {
        let top = TabViewStyle::top();
        assert_eq!(top.position, TabPosition::Top);

        let bottom = TabViewStyle::bottom();
        assert_eq!(bottom.position, TabPosition::Bottom);

        let left = TabViewStyle::left();
        assert_eq!(left.position, TabPosition::Left);
        assert!(left.tab_width.is_some());

        let right = TabViewStyle::right();
        assert_eq!(right.position, TabPosition::Right);
    }

    #[test]
    fn test_focusable_impl() {
        let mut state = TabViewState::with_focus_id(3, FocusId::new(42));

        assert_eq!(state.focus_id().id(), 42);
        assert!(!state.is_focused());

        state.set_focused(true);
        assert!(state.is_focused());
    }

    #[test]
    fn test_tab_view_render() {
        let tabs = vec![Tab::new("Tab 1"), Tab::new("Tab 2"), Tab::new("Tab 3")];
        let state = TabViewState::new(tabs.len());
        let tab_view = TabView::new(&tabs, &state);

        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 10));
        tab_view.render(Rect::new(0, 0, 50, 10), &mut buf);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_key_handling_horizontal() {
        let mut state = TabViewState::new(5);

        // Right arrow moves next
        let key = KeyEvent::new(KeyCode::Right, crossterm::event::KeyModifiers::NONE);
        assert!(handle_tab_view_key(&mut state, &key, TabPosition::Top));
        assert_eq!(state.selected_index, 1);

        // Left arrow moves prev
        let key = KeyEvent::new(KeyCode::Left, crossterm::event::KeyModifiers::NONE);
        assert!(handle_tab_view_key(&mut state, &key, TabPosition::Top));
        assert_eq!(state.selected_index, 0);

        // Home goes to first
        let key = KeyEvent::new(KeyCode::Home, crossterm::event::KeyModifiers::NONE);
        state.select(3);
        assert!(handle_tab_view_key(&mut state, &key, TabPosition::Top));
        assert_eq!(state.selected_index, 0);

        // End goes to last
        let key = KeyEvent::new(KeyCode::End, crossterm::event::KeyModifiers::NONE);
        assert!(handle_tab_view_key(&mut state, &key, TabPosition::Top));
        assert_eq!(state.selected_index, 4);
    }

    #[test]
    fn test_key_handling_vertical() {
        let mut state = TabViewState::new(5);

        // Down arrow moves next in vertical mode
        let key = KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::NONE);
        assert!(handle_tab_view_key(&mut state, &key, TabPosition::Left));
        assert_eq!(state.selected_index, 1);

        // Up arrow moves prev
        let key = KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::NONE);
        assert!(handle_tab_view_key(&mut state, &key, TabPosition::Left));
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_number_key_selection() {
        let mut state = TabViewState::new(5);

        // Press '3' to select tab 3 (index 2)
        let key = KeyEvent::new(KeyCode::Char('3'), crossterm::event::KeyModifiers::NONE);
        assert!(handle_tab_view_key(&mut state, &key, TabPosition::Top));
        assert_eq!(state.selected_index, 2);

        // Press '1' to select tab 1 (index 0)
        let key = KeyEvent::new(KeyCode::Char('1'), crossterm::event::KeyModifiers::NONE);
        assert!(handle_tab_view_key(&mut state, &key, TabPosition::Top));
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_focus_toggle_with_enter() {
        let mut state = TabViewState::new(3);
        assert!(state.tab_bar_focused);

        // Enter toggles to content
        let key = KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE);
        assert!(handle_tab_view_key(&mut state, &key, TabPosition::Top));
        assert!(!state.tab_bar_focused);

        // Escape goes back to tab bar
        let key = KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::NONE);
        assert!(handle_tab_view_key(&mut state, &key, TabPosition::Top));
        assert!(state.tab_bar_focused);
    }

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
    }

    #[test]
    fn test_horizontal_width1_no_panic() {
        let tabs = vec![Tab::new("One"), Tab::new("Two")];
        let mut state = TabViewState::new(2);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 5));
        TabView::new(&tabs, &state).render(Rect::new(0, 0, 1, 5), &mut buf);
        state.scroll_offset = 1;
        TabView::new(&tabs, &state).render(Rect::new(0, 0, 1, 5), &mut buf);
    }

    #[test]
    fn test_vertical_width0_no_panic() {
        let tabs = vec![Tab::new("One")];
        let state = TabViewState::new(1);
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 5));
        TabView::new(&tabs, &state)
            .style(TabViewStyle::left().tab_width(0))
            .render(Rect::new(0, 0, 10, 5), &mut buf);
    }

    #[test]
    fn test_vertical_stale_scroll_no_panic() {
        let tabs = vec![Tab::new("A"), Tab::new("B")];
        let mut state = TabViewState::new(2);
        state.scroll_offset = 3; // e.g. tabs closed after scrolling
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 5));
        TabView::new(&tabs, &state)
            .style(TabViewStyle::left())
            .render(Rect::new(0, 0, 40, 5), &mut buf);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 5));
        TabView::new(&tabs, &state).render(Rect::new(0, 0, 40, 5), &mut buf);
    }

    #[test]
    fn test_scroll_next_is_bounded() {
        let tabs = [Tab::new("A"), Tab::new("B")];
        let mut state = TabViewState::new(tabs.len());
        let mut registry = ClickRegionRegistry::new();
        registry.register(Rect::new(0, 0, 2, 1), TabViewAction::ScrollNext);
        let click = MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        for _ in 0..10 {
            handle_tab_view_mouse(&mut state, &registry, &click);
        }
        assert_eq!(state.scroll_offset, 1);
    }

    #[test]
    fn test_disabled_tab_skipped_by_keys() {
        let tabs = vec![Tab::new("A"), Tab::new("B").enabled(false), Tab::new("C")];
        let mut state = TabViewState::new(3);
        handle_tab_view_key_with_tabs(&mut state, &press(KeyCode::Right), TabPosition::Top, &tabs);
        assert_eq!(state.selected_index, 2);
        handle_tab_view_key_with_tabs(&mut state, &press(KeyCode::Left), TabPosition::Top, &tabs);
        assert_eq!(state.selected_index, 0);
        handle_tab_view_key_with_tabs(
            &mut state,
            &press(KeyCode::Char('2')),
            TabPosition::Top,
            &tabs,
        );
        assert_eq!(state.selected_index, 0);

        let tabs = vec![
            Tab::new("A").enabled(false),
            Tab::new("B"),
            Tab::new("C").enabled(false),
        ];
        handle_tab_view_key_with_tabs(&mut state, &press(KeyCode::End), TabPosition::Top, &tabs);
        assert_eq!(state.selected_index, 1);
        handle_tab_view_key_with_tabs(&mut state, &press(KeyCode::Home), TabPosition::Top, &tabs);
        assert_eq!(state.selected_index, 1);
    }

    #[test]
    fn test_disabled_tab_has_no_click_region() {
        let tabs = vec![Tab::new("A"), Tab::new("B").enabled(false), Tab::new("C")];
        let state = TabViewState::new(3);
        for style in [TabViewStyle::default(), TabViewStyle::left()] {
            let mut buf = Buffer::empty(Rect::new(0, 0, 40, 10));
            let regions = TabView::new(&tabs, &state)
                .style(style)
                .render_stateful(Rect::new(0, 0, 40, 10), &mut buf);
            assert!(
                !regions
                    .iter()
                    .any(|(_, a)| *a == TabViewAction::TabClick(1))
            );
            assert!(
                regions
                    .iter()
                    .any(|(_, a)| *a == TabViewAction::TabClick(2))
            );
        }
    }

    #[test]
    fn test_selected_tab_stays_inside_area() {
        // " label " is display_width, but the selected tab renders "▸ label "
        let tabs = vec![Tab::new("AAAA"), Tab::new("BBBB")];
        let state = TabViewState::new(2);
        let style = TabViewStyle::default().divider("").bordered_content(false);
        let w: u16 = tabs.iter().map(|t| t.display_width() as u16).sum();
        let mut buf = Buffer::empty(Rect::new(0, 0, w + 5, 3));
        TabView::new(&tabs, &state)
            .style(style)
            .render(Rect::new(0, 0, w, 3), &mut buf);
        assert_eq!(buf[(w, 0)].symbol(), " ");
    }
}
