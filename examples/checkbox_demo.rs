//! CheckBox Demo
//!
//! Interactive demo showing checkbox features:
//! - Multiple checkboxes with Tab navigation
//! - Mouse click support
//! - Space to toggle
//! - Different styling options
//!
//! Run with: cargo run --example checkbox_demo

use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use ratatui_interact::{
    components::{CheckBox, CheckBoxState, CheckBoxStyle},
    events::{is_activate_key, is_backtab, is_close_key, is_left_click, is_tab},
    state::FocusManager,
    traits::ClickRegionRegistry,
};

/// Application state
struct App {
    /// Checkbox states
    checkboxes: Vec<(String, CheckBoxState, CheckBoxStyle)>,
    /// Focus manager
    focus: FocusManager<usize>,
    /// Click regions
    click_regions: ClickRegionRegistry<usize>,
    /// Should quit
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let mut focus = FocusManager::new();

        // Create checkboxes with different styles
        let checkboxes = vec![
            (
                "Enable notifications".to_string(),
                CheckBoxState::new(true),
                CheckBoxStyle::default(),
            ),
            (
                "Dark mode".to_string(),
                CheckBoxState::new(false),
                CheckBoxStyle::unicode(),
            ),
            (
                "Auto-save".to_string(),
                CheckBoxState::new(true),
                CheckBoxStyle::checkmark(),
            ),
            (
                "Show hints".to_string(),
                CheckBoxState::new(false),
                CheckBoxStyle::custom("[ON]", "[OFF]"),
            ),
        ];

        // Register all checkboxes for focus
        for i in 0..checkboxes.len() {
            focus.register(i);
        }

        Self {
            checkboxes,
            focus,
            click_regions: ClickRegionRegistry::new(),
            should_quit: false,
        }
    }

    fn toggle_current(&mut self) {
        if let Some(&idx) = self.focus.current() {
            self.checkboxes[idx].1.toggle();
        }
    }

    fn handle_click(&mut self, col: u16, row: u16) {
        if let Some(&idx) = self.click_regions.handle_click(col, row) {
            self.focus.set(idx);
            self.checkboxes[idx].1.toggle();
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

        if let Event::Key(key) = event::read()? {
            if is_close_key(&key) || key.code == KeyCode::Char('q') {
                app.should_quit = true;
            } else if is_tab(&key) {
                app.focus.next();
            } else if is_backtab(&key) {
                app.focus.prev();
            } else if is_activate_key(&key) {
                app.toggle_current();
            }
        } else if let Event::Mouse(mouse) = event::read().unwrap_or(Event::FocusGained) {
            if is_left_click(&mouse) {
                app.handle_click(mouse.column, mouse.row);
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
    app.click_regions.clear();

    let area = f.area();

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Content
            Constraint::Length(3), // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new("CheckBox Demo")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Checkboxes
    let checkbox_area = chunks[1];
    let checkbox_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            app.checkboxes
                .iter()
                .map(|_| Constraint::Length(2))
                .collect::<Vec<_>>(),
        )
        .split(checkbox_area);

    for (idx, (label, state, style)) in app.checkboxes.iter_mut().enumerate() {
        // Set focus state
        state.focused = app.focus.is_focused(&idx);

        let checkbox = CheckBox::new(label, state).style(style.clone());

        let cb_area = Rect::new(
            checkbox_chunks[idx].x + 2,
            checkbox_chunks[idx].y,
            checkbox_chunks[idx].width.saturating_sub(4),
            1,
        );

        let region = checkbox.render_stateful(cb_area, f.buffer_mut());
        app.click_regions.register(region.area, idx);
    }

    // Help text
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(": Next  "),
        Span::styled("Shift+Tab", Style::default().fg(Color::Yellow)),
        Span::raw(": Prev  "),
        Span::styled("Space/Enter", Style::default().fg(Color::Yellow)),
        Span::raw(": Toggle  "),
        Span::styled("Click", Style::default().fg(Color::Yellow)),
        Span::raw(": Toggle  "),
        Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
        Span::raw(": Quit"),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}
