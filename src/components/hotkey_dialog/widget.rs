//! Widget implementation for the hotkey dialog.
//!
//! This module provides the rendering logic for the hotkey dialog component.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use super::state::{HotkeyDialogState, HotkeyFocus};
use super::style::HotkeyDialogStyle;
use super::traits::{HotkeyCategory, HotkeyEntryData, HotkeyProvider};

/// A hotkey configuration dialog widget.
///
/// This widget renders a modal dialog for displaying and searching hotkeys.
/// It requires a state object and a provider for the hotkey data.
///
/// # Type Parameters
///
/// - `C`: The category type implementing `HotkeyCategory`
/// - `P`: The provider type implementing `HotkeyProvider<Category = C>`
///
/// # Example
///
/// ```rust,ignore
/// use ratatui_interact::components::hotkey_dialog::{
///     HotkeyDialog, HotkeyDialogState, HotkeyDialogStyle
/// };
///
/// let style = HotkeyDialogStyle::default();
/// let provider = MyHotkeyProvider;
///
/// HotkeyDialog::new(&mut state, &provider, &style)
///     .render(frame, frame.area());
/// ```
pub struct HotkeyDialog<'a, C: HotkeyCategory, P: HotkeyProvider<Category = C>> {
    state: &'a mut HotkeyDialogState<C>,
    provider: &'a P,
    style: &'a HotkeyDialogStyle,
}

impl<'a, C: HotkeyCategory, P: HotkeyProvider<Category = C>> HotkeyDialog<'a, C, P> {
    /// Create a new hotkey dialog widget.
    pub fn new(
        state: &'a mut HotkeyDialogState<C>,
        provider: &'a P,
        style: &'a HotkeyDialogStyle,
    ) -> Self {
        Self {
            state,
            provider,
            style,
        }
    }

    /// Render the dialog to the frame.
    pub fn render(mut self, frame: &mut Frame, _area: Rect) {
        let screen = frame.area();

        // Calculate modal dimensions
        let (x, y, modal_width, modal_height) =
            self.style.calculate_modal_area(screen.width, screen.height);
        let modal_area = Rect::new(x, y, modal_width, modal_height);

        // Clear background
        frame.render_widget(Clear, modal_area);

        // Outer border with title
        let border_color = ratatui::style::Color::Cyan;
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(self.style.title.as_str())
            .title_style(self.style.title_style());

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        // Clear click regions before rendering
        self.state.clear_click_regions();

        // Layout: Search bar | Main content | Footer
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(self.style.search_height),
                Constraint::Min(1),
                Constraint::Length(self.style.footer_height),
            ])
            .split(inner);

        // Render components
        self.render_search_bar(frame, main_chunks[0]);

        // Split main content: Categories | Hotkeys
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(self.style.category_width_percent),
                Constraint::Percentage(100 - self.style.category_width_percent),
            ])
            .split(main_chunks[1]);

        self.render_category_list(frame, content_chunks[0]);
        self.render_hotkey_list(frame, content_chunks[1]);
        self.render_footer(frame, main_chunks[2]);
    }

    /// Render the search bar.
    fn render_search_bar(&mut self, frame: &mut Frame, area: Rect) {
        let is_focused = self.state.focus == HotkeyFocus::SearchInput;
        let border_style = if is_focused {
            self.style.focused_border_style()
        } else {
            self.style.unfocused_border_style()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Search ");

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Build search text with cursor
        let text = if self.state.search_query.is_empty() && !is_focused {
            Line::from(Span::styled(
                &self.style.search_placeholder,
                self.style.placeholder_style(),
            ))
        } else {
            let before = self.state.text_before_cursor();
            let after = self.state.text_after_cursor();

            let mut spans = vec![Span::styled(before.to_string(), self.style.text_style())];

            if is_focused {
                spans.push(Span::styled("|", self.style.cursor_style()));
            }

            spans.push(Span::styled(after.to_string(), self.style.text_style()));

            Line::from(spans)
        };

        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, inner);
    }

    /// Render the category list.
    fn render_category_list(&mut self, frame: &mut Frame, area: Rect) {
        let is_focused = self.state.focus == HotkeyFocus::CategoryList;
        let border_style = if is_focused {
            self.style.focused_border_style()
        } else {
            self.style.unfocused_border_style()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Categories ");

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let categories = C::all();
        let mut lines = Vec::new();

        for (idx, category) in categories.iter().enumerate() {
            let is_selected = *category == self.state.selected_category && !self.state.is_searching();

            let prefix = if is_selected { "> " } else { "  " };
            let icon = category.icon();
            let name = category.display_name();
            let count = self.provider.entries_for_category(*category).len();

            let style = if is_selected {
                self.style.selected_style()
            } else {
                self.style.text_style()
            };

            let count_style = if is_selected {
                self.style.selected_text_style()
            } else {
                self.style.dim_style()
            };

            let line = Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(format!("{} ", icon), style),
                Span::styled(name, style),
                Span::styled(format!(" ({})", count), count_style),
            ]);
            lines.push(line);

            // Register click region
            let row_y = inner.y + idx as u16;
            if row_y < inner.y + inner.height {
                self.state.add_category_click_region(
                    Rect::new(inner.x, row_y, inner.width, 1),
                    *category,
                );
            }
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }

    /// Render the hotkey list.
    fn render_hotkey_list(&mut self, frame: &mut Frame, area: Rect) {
        let is_focused = self.state.focus == HotkeyFocus::HotkeyList;
        let border_style = if is_focused {
            self.style.focused_border_style()
        } else {
            self.style.unfocused_border_style()
        };

        // Title shows category name or "Search Results"
        let title = if self.state.is_searching() {
            let count = self.state.get_search_results(self.provider).len();
            format!(" Search Results ({}) ", count)
        } else {
            format!(
                " {} {} ",
                self.state.selected_category.icon(),
                self.state.selected_category.display_name()
            )
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Get entries to display
        let entries = self.state.get_current_entries(self.provider);
        let total_entries = entries.len();

        // Update cached entry count
        self.state.update_entry_count(total_entries);

        if entries.is_empty() {
            let msg = if self.state.is_searching() {
                "No matching hotkeys found"
            } else {
                "No hotkeys in this category"
            };
            let paragraph = Paragraph::new(Line::from(Span::styled(
                msg,
                self.style.placeholder_style(),
            )));
            frame.render_widget(paragraph, inner);
            return;
        }

        // Calculate column widths
        let max_key_len = entries
            .iter()
            .map(|e| e.key_combination.chars().count())
            .max()
            .unwrap_or(15)
            .max(15);

        // Visible height for scrolling
        let visible_height = inner.height as usize;

        // Ensure selected hotkey is visible
        self.state.ensure_hotkey_visible(visible_height);

        // Build lines with proper formatting
        let lines = self.build_hotkey_lines(&entries, max_key_len, is_focused, inner, visible_height);

        // Apply scroll
        let scroll = self.state.hotkey_scroll.min(total_entries.saturating_sub(1));

        let paragraph = Paragraph::new(lines).scroll((scroll as u16, 0));
        frame.render_widget(paragraph, inner);

        // Render scrollbar if needed
        if total_entries > visible_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("^"))
                .end_symbol(Some("v"));

            let mut scrollbar_state = ScrollbarState::new(total_entries)
                .position(scroll)
                .viewport_content_length(visible_height);

            let scrollbar_area = Rect::new(
                area.x + area.width - 1,
                area.y + 1,
                1,
                area.height.saturating_sub(2),
            );

            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }
    }

    /// Build the lines for the hotkey list.
    fn build_hotkey_lines(
        &mut self,
        entries: &[HotkeyEntryData],
        max_key_len: usize,
        is_focused: bool,
        inner: Rect,
        visible_height: usize,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        for (idx, entry) in entries.iter().enumerate() {
            let is_selected = idx == self.state.selected_hotkey_idx && is_focused;

            // Key combination with fixed width
            let key_padded = format!("{:width$}", entry.key_combination, width = max_key_len);

            // Context indicator
            let context_str = if entry.is_global {
                self.style.global_indicator.clone()
            } else {
                format!("[{}]", entry.context.chars().next().unwrap_or('?'))
            };

            // Styles
            let (key_style, action_style, context_style) = if is_selected {
                (
                    self.style.selected_style(),
                    self.style.selected_text_style(),
                    self.style.selected_text_style(),
                )
            } else {
                let key_style = if entry.is_global {
                    self.style.global_key_style()
                } else {
                    self.style.local_key_style()
                };
                (key_style, self.style.text_style(), self.style.dim_style())
            };

            // Customizable indicator
            let lock_indicator = if entry.is_customizable {
                " "
            } else {
                &self.style.locked_indicator
            };
            let lock_style = if is_selected {
                self.style.selected_text_style()
            } else {
                self.style.locked_style()
            };

            let line = Line::from(vec![
                Span::styled(lock_indicator.to_string(), lock_style),
                Span::styled(" ", action_style),
                Span::styled(key_padded, key_style),
                Span::styled("  ", action_style),
                Span::styled(entry.action.clone(), action_style),
                Span::styled("  ", action_style),
                Span::styled(context_str, context_style),
            ]);
            lines.push(line);

            // Register click region (only for visible entries)
            let row_offset = idx.saturating_sub(self.state.hotkey_scroll);
            if idx >= self.state.hotkey_scroll && row_offset < visible_height {
                let row_y = inner.y + row_offset as u16;
                self.state.add_hotkey_click_region(
                    Rect::new(inner.x, row_y, inner.width, 1),
                    idx,
                );
            }
        }

        lines
    }

    /// Render the footer with key hints and legend.
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let hints = match self.state.focus {
            HotkeyFocus::SearchInput => vec![
                ("Esc", "Clear/Close"),
                ("Tab", "Categories"),
                ("Type", "Filter"),
            ],
            HotkeyFocus::CategoryList => vec![
                ("Up/Dn", "Navigate"),
                ("Tab", "Hotkeys"),
                ("Esc", "Close"),
            ],
            HotkeyFocus::HotkeyList => vec![
                ("Up/Dn", "Navigate"),
                ("PgUp/Dn", "Page"),
                ("Tab", "Search"),
                ("Esc", "Close"),
            ],
        };

        // Legend
        let mut spans = vec![
            Span::styled(
                &self.style.global_indicator,
                Style::default().fg(self.style.global_key_color),
            ),
            Span::styled("=Global ", self.style.dim_style()),
            Span::styled(&self.style.locked_indicator, self.style.locked_style()),
            Span::styled("=Locked  ", self.style.dim_style()),
            Span::raw("|  "),
        ];

        // Key hints
        for (idx, (key, desc)) in hints.iter().enumerate() {
            if idx > 0 {
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(
                *key,
                Style::default().fg(self.style.global_key_color),
            ));
            spans.push(Span::raw(": "));
            spans.push(Span::styled(*desc, Style::default().fg(self.style.dim_color)));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(self.style.unfocused_border_style()),
        );

        frame.render_widget(paragraph, area);
    }
}

/// Convenience function to render a hotkey dialog.
///
/// This is a simpler alternative to creating a `HotkeyDialog` widget manually.
pub fn render_hotkey_dialog<C: HotkeyCategory, P: HotkeyProvider<Category = C>>(
    frame: &mut Frame,
    state: &mut HotkeyDialogState<C>,
    provider: &P,
    style: &HotkeyDialogStyle,
) {
    let dialog = HotkeyDialog::new(state, provider, style);
    dialog.render(frame, frame.area());
}
