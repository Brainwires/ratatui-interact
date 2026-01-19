//! Spinner Demo
//!
//! Interactive demo showing spinner features:
//! - Multiple spinner frame styles (dots, braille, line, etc.)
//! - Custom colors and labels
//! - Label positioning (before/after)
//! - Start/stop functionality
//!
//! Run with: cargo run --example spinner_demo

use std::io;
use std::time::Duration;

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

use ratatui_interact::components::{Spinner, SpinnerFrames, SpinnerState, SpinnerStyle};
use ratatui_interact::events::is_close_key;

/// All available spinner frames
const SPINNER_TYPES: &[SpinnerFrames] = &[
    SpinnerFrames::Dots,
    SpinnerFrames::Braille,
    SpinnerFrames::Line,
    SpinnerFrames::Circle,
    SpinnerFrames::Box,
    SpinnerFrames::Arrow,
    SpinnerFrames::Bounce,
    SpinnerFrames::Grow,
    SpinnerFrames::Clock,
    SpinnerFrames::Moon,
    SpinnerFrames::Ascii,
    SpinnerFrames::Toggle,
];

/// Application state
struct App {
    /// Spinner states for each type
    spinner_states: Vec<SpinnerState>,
    /// Currently selected spinner
    selected: usize,
    /// Whether spinners are running
    running: bool,
    /// Should quit
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let spinner_states = SPINNER_TYPES
            .iter()
            .map(|frames| SpinnerState::for_frames(*frames))
            .collect();

        Self {
            spinner_states,
            selected: 0,
            running: true,
            should_quit: false,
        }
    }

    fn toggle_running(&mut self) {
        self.running = !self.running;
        for state in &mut self.spinner_states {
            if self.running {
                state.start();
            } else {
                state.stop();
            }
        }
    }

    fn tick_all(&mut self) {
        for (i, state) in self.spinner_states.iter_mut().enumerate() {
            let frame_count = SPINNER_TYPES[i].frames().len();
            state.tick_with_frames(frame_count);
        }
    }

    fn select_next(&mut self) {
        self.selected = (self.selected + 1) % SPINNER_TYPES.len();
    }

    fn select_prev(&mut self) {
        self.selected = self.selected.checked_sub(1).unwrap_or(SPINNER_TYPES.len() - 1);
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

        // Poll with timeout for animation
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if is_close_key(&key) || key.code == KeyCode::Char('q') {
                    app.should_quit = true;
                } else if key.code == KeyCode::Char(' ') {
                    app.toggle_running();
                } else if key.code == KeyCode::Up || key.code == KeyCode::Char('k') {
                    app.select_prev();
                } else if key.code == KeyCode::Down || key.code == KeyCode::Char('j') {
                    app.select_next();
                }
            }
        }

        // Tick all spinners
        app.tick_all();

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
    let area = f.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Spinners
            Constraint::Length(5), // Help + selected info
        ])
        .split(area);

    // Title
    let status = if app.running { "Running" } else { "Paused" };
    let title = Paragraph::new(format!("Spinner Demo - {} Styles | Status: {}", SPINNER_TYPES.len(), status))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Spinners grid
    render_spinners(f, app, chunks[1]);

    // Help and selected info
    let selected_name = spinner_name(SPINNER_TYPES[app.selected]);
    let selected_num = format!(" ({})", app.selected + 1);
    let help_lines = vec![
        Line::from(vec![
            Span::styled("Selected: ", Style::default().fg(Color::Gray)),
            Span::styled(selected_name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(selected_num),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Space", Style::default().fg(Color::Yellow)),
            Span::raw(": Toggle pause  "),
            Span::styled("Up/Down", Style::default().fg(Color::Yellow)),
            Span::raw(": Select spinner  "),
            Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];
    let help = Paragraph::new(help_lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn render_spinners(f: &mut Frame, app: &mut App, area: Rect) {
    // Split into rows of 3 spinners each
    let rows_count = (SPINNER_TYPES.len() + 2) / 3;
    let row_constraints: Vec<Constraint> = (0..rows_count)
        .map(|_| Constraint::Length(3))
        .chain(std::iter::once(Constraint::Min(0)))
        .collect();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);

    for (row_idx, row_chunk) in rows.iter().enumerate().take(rows_count) {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(*row_chunk);

        for col_idx in 0..3 {
            let spinner_idx = row_idx * 3 + col_idx;
            if spinner_idx < SPINNER_TYPES.len() {
                render_single_spinner(f, app, cols[col_idx], spinner_idx);
            }
        }
    }
}

fn render_single_spinner(f: &mut Frame, app: &App, area: Rect, idx: usize) {
    let frames = SPINNER_TYPES[idx];
    let state = &app.spinner_states[idx];
    let name = spinner_name(frames);

    let is_selected = idx == app.selected;

    // Create block with selection highlight
    let block_style = if is_selected {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(block_style)
        .title(format!(" {} ", name));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Render spinner with label
    let spinner_style = if is_selected {
        SpinnerStyle::new(frames).color(Color::Yellow)
    } else {
        SpinnerStyle::new(frames)
    };

    let spinner = Spinner::new(state)
        .style(spinner_style)
        .label(name);

    f.render_widget(spinner, Rect::new(inner.x + 1, inner.y, inner.width.saturating_sub(2), 1));
}

fn spinner_name(frames: SpinnerFrames) -> &'static str {
    match frames {
        SpinnerFrames::Dots => "Dots",
        SpinnerFrames::Braille => "Braille",
        SpinnerFrames::Line => "Line",
        SpinnerFrames::Circle => "Circle",
        SpinnerFrames::Box => "Box",
        SpinnerFrames::Arrow => "Arrow",
        SpinnerFrames::Bounce => "Bounce",
        SpinnerFrames::Grow => "Grow",
        SpinnerFrames::Clock => "Clock",
        SpinnerFrames::Moon => "Moon",
        SpinnerFrames::Ascii => "ASCII",
        SpinnerFrames::Toggle => "Toggle",
    }
}
