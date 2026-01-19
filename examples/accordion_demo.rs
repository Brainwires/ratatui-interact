//! Accordion Demo
//!
//! Interactive demo showing accordion features:
//! - Single expand mode (FAQ panel - only one section at a time)
//! - Multiple expand mode (Settings panel - multiple sections simultaneously)
//! - Keyboard navigation (Up/Down to move, Enter/Space to toggle)
//! - Mouse click support
//! - Different styling options
//!
//! Run with: cargo run --example accordion_demo

use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use ratatui_interact::{
    components::{Accordion, AccordionMode, AccordionState, AccordionStyle, handle_accordion_key},
    events::is_close_key,
};

/// FAQ item for single-expand demo
#[derive(Debug)]
struct FaqItem {
    id: String,
    question: String,
    answer: String,
}

/// Settings section for multiple-expand demo
#[derive(Debug)]
struct SettingsSection {
    id: String,
    title: String,
    settings: Vec<(String, String)>,
}

/// Which panel is focused
#[derive(Clone, Copy, PartialEq)]
enum FocusedPanel {
    Faq,
    Settings,
}

/// Application state
struct App {
    /// FAQ accordion state (single mode)
    faq_state: AccordionState,
    /// Settings accordion state (multiple mode)
    settings_state: AccordionState,
    /// FAQ items
    faq_items: Vec<FaqItem>,
    /// Settings sections
    settings_sections: Vec<SettingsSection>,
    /// Currently focused panel
    focused_panel: FocusedPanel,
    /// Should quit
    should_quit: bool,
    /// Click regions for FAQ headers (index, area, id)
    faq_click_regions: Vec<(usize, Rect, String)>,
    /// Click regions for Settings headers
    settings_click_regions: Vec<(usize, Rect, String)>,
}

impl App {
    fn new() -> Self {
        let faq_items = vec![
            FaqItem {
                id: "faq1".into(),
                question: "What is ratatui?".into(),
                answer: "Ratatui is a Rust library for building\nrich terminal user interfaces (TUIs)\nand dashboards.".into(),
            },
            FaqItem {
                id: "faq2".into(),
                question: "How do I install it?".into(),
                answer: "Add to your Cargo.toml:\n  ratatui = \"0.29\"\nThen use:\n  use ratatui::prelude::*;".into(),
            },
            FaqItem {
                id: "faq3".into(),
                question: "Where are the docs?".into(),
                answer: "Official documentation:\n  https://docs.rs/ratatui\nGitHub repo:\n  https://github.com/ratatui/ratatui".into(),
            },
            FaqItem {
                id: "faq4".into(),
                question: "Is it production ready?".into(),
                answer: "Yes! Ratatui is actively maintained\nand used in many production apps\nlike gitui, bottom, and more.".into(),
            },
        ];

        let settings_sections = vec![
            SettingsSection {
                id: "display".into(),
                title: "Display".into(),
                settings: vec![
                    ("Theme".into(), "Dark".into()),
                    ("Font Size".into(), "14px".into()),
                    ("Line Height".into(), "1.5".into()),
                ],
            },
            SettingsSection {
                id: "audio".into(),
                title: "Audio".into(),
                settings: vec![
                    ("Volume".into(), "80%".into()),
                    ("Mute".into(), "No".into()),
                    ("Output".into(), "Speakers".into()),
                ],
            },
            SettingsSection {
                id: "network".into(),
                title: "Network".into(),
                settings: vec![
                    ("Proxy".into(), "None".into()),
                    ("Timeout".into(), "30s".into()),
                ],
            },
        ];

        let faq_state = AccordionState::new(faq_items.len()).with_mode(AccordionMode::Single);

        let settings_state = AccordionState::new(settings_sections.len())
            .with_mode(AccordionMode::Multiple)
            .with_expanded(vec!["display".into(), "audio".into()]);

        Self {
            faq_state,
            settings_state,
            faq_items,
            settings_sections,
            focused_panel: FocusedPanel::Faq,
            should_quit: false,
            faq_click_regions: Vec::new(),
            settings_click_regions: Vec::new(),
        }
    }

    fn toggle_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Faq => FocusedPanel::Settings,
            FocusedPanel::Settings => FocusedPanel::Faq,
        };
    }

    fn handle_key(&mut self, key: &crossterm::event::KeyEvent) {
        if key.code == KeyCode::Tab {
            self.toggle_panel();
            return;
        }

        match self.focused_panel {
            FocusedPanel::Faq => {
                let faq_items = &self.faq_items;
                handle_accordion_key(&mut self.faq_state, key, |idx| {
                    faq_items.get(idx).map(|f| f.id.clone()).unwrap_or_default()
                });
            }
            FocusedPanel::Settings => {
                let settings_sections = &self.settings_sections;
                handle_accordion_key(&mut self.settings_state, key, |idx| {
                    settings_sections
                        .get(idx)
                        .map(|s| s.id.clone())
                        .unwrap_or_default()
                });
            }
        }
    }

    fn handle_mouse_click(&mut self, col: u16, row: u16) {
        // Check FAQ click regions
        for (idx, area, id) in &self.faq_click_regions {
            if col >= area.x
                && col < area.x + area.width
                && row >= area.y
                && row < area.y + area.height
            {
                self.focused_panel = FocusedPanel::Faq;
                self.faq_state.focus(*idx);
                self.faq_state.toggle(id);
                return;
            }
        }

        // Check Settings click regions
        for (idx, area, id) in &self.settings_click_regions {
            if col >= area.x
                && col < area.x + area.width
                && row >= area.y
                && row < area.y + area.height
            {
                self.focused_panel = FocusedPanel::Settings;
                self.settings_state.focus(*idx);
                self.settings_state.toggle(id);
                return;
            }
        }
    }
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Ok(event) = event::read() {
            match event {
                Event::Key(key) => {
                    if is_close_key(&key) || key.code == KeyCode::Char('q') {
                        app.should_quit = true;
                    } else {
                        app.handle_key(&key);
                    }
                }
                Event::Mouse(mouse) => {
                    if let MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse.kind {
                        app.handle_mouse_click(mouse.column, mouse.row);
                    }
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    // Clear click regions
    app.faq_click_regions.clear();
    app.settings_click_regions.clear();

    let area = f.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Content
            Constraint::Length(3), // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Accordion Demo")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Content area - split into two panels
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // FAQ Panel (Single Mode)
    render_faq_panel(f, app, content_chunks[0]);

    // Settings Panel (Multiple Mode)
    render_settings_panel(f, app, content_chunks[1]);

    // Help text
    let focus_text = match app.focused_panel {
        FocusedPanel::Faq => "FAQ",
        FocusedPanel::Settings => "Settings",
    };

    let help_lines = vec![
        Line::from(vec![
            Span::styled("Focus: ", Style::default().fg(Color::Gray)),
            Span::styled(focus_text, Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("\u{2191}/\u{2193}", Style::default().fg(Color::Yellow)),
            Span::raw(": Navigate  "),
            Span::styled("Enter/Space", Style::default().fg(Color::Yellow)),
            Span::raw(": Toggle  "),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Switch panel  "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];
    let help = Paragraph::new(help_lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn render_faq_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Faq;

    let block = Block::default()
        .title(Span::styled(
            " FAQ (Single Mode) ",
            if is_focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        ))
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Calculate content heights for FAQ items
    let content_heights: Vec<u16> = app
        .faq_items
        .iter()
        .map(|item| item.answer.lines().count() as u16)
        .collect();

    // Build click regions for headers
    let mut y = inner.y;
    for (idx, item) in app.faq_items.iter().enumerate() {
        let header_area = Rect::new(inner.x, y, inner.width, 1);
        app.faq_click_regions
            .push((idx, header_area, item.id.clone()));

        y += 1; // header
        if app.faq_state.is_expanded(&item.id) {
            y += content_heights[idx];
        }
    }

    // Create accordion style
    let style = AccordionStyle::default()
        .header_focused_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .content_style(Style::default().fg(Color::Gray))
        .content_indent(2);

    // Create and render accordion
    let accordion = Accordion::new(&app.faq_items, &app.faq_state)
        .id_fn(|item, _| item.id.clone())
        .render_header(|item, _idx, _is_item_focused| Line::raw(item.question.clone()))
        .render_content(|item, _idx, content_area, buf| {
            let paragraph = Paragraph::new(item.answer.as_str())
                .style(Style::default().fg(Color::Gray))
                .wrap(Wrap { trim: false });
            paragraph.render(content_area, buf);
        })
        .content_heights(&content_heights)
        .style(style);

    accordion.render(inner, f.buffer_mut());
}

fn render_settings_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Settings;

    let block = Block::default()
        .title(Span::styled(
            " Settings (Multiple Mode) ",
            if is_focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        ))
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Calculate content heights for settings sections
    let content_heights: Vec<u16> = app
        .settings_sections
        .iter()
        .map(|section| section.settings.len() as u16)
        .collect();

    // Build click regions for headers
    let mut y = inner.y;
    for (idx, section) in app.settings_sections.iter().enumerate() {
        let header_area = Rect::new(inner.x, y, inner.width, 1);
        app.settings_click_regions
            .push((idx, header_area, section.id.clone()));

        y += 1; // header
        if app.settings_state.is_expanded(&section.id) {
            y += content_heights[idx];
        }
    }

    // Create accordion style with different icons
    let style = AccordionStyle::default()
        .header_focused_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .content_style(Style::default().fg(Color::White))
        .icon_style(Style::default().fg(Color::Green))
        .content_indent(3);

    // Create and render accordion
    let accordion = Accordion::new(&app.settings_sections, &app.settings_state)
        .id_fn(|section, _| section.id.clone())
        .render_header(|section, _idx, _is_focused| Line::raw(section.title.clone()))
        .render_content(|section, _idx, content_area, buf| {
            let lines: Vec<Line> = section
                .settings
                .iter()
                .map(|(key, value)| {
                    Line::from(vec![
                        Span::styled(format!("{}: ", key), Style::default().fg(Color::DarkGray)),
                        Span::styled(value.clone(), Style::default().fg(Color::White)),
                    ])
                })
                .collect();
            let paragraph = Paragraph::new(lines);
            paragraph.render(content_area, buf);
        })
        .content_heights(&content_heights)
        .style(style);

    accordion.render(inner, f.buffer_mut());
}
