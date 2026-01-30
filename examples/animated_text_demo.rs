//! Animated Text Demo
//!
//! Interactive demo showing animated text features:
//! - Multiple animation effects (pulse, wave, rainbow, gradient, sparkle)
//! - Custom colors and styles
//! - Start/stop functionality
//! - Configurable wave width
//!
//! Run with: cargo run --example animated_text_demo

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

use ratatui_interact::components::{AnimatedText, AnimatedTextState, AnimatedTextStyle};
use ratatui_interact::events::is_close_key;

/// Demo text samples for each effect
const DEMO_TEXTS: &[(&str, &str)] = &[
    ("Pulse", "Loading your data..."),
    ("Wave", "Processing files in background"),
    ("Rainbow", "Welcome to the rainbow zone!"),
    ("Gradient", "Smooth color transitions"),
    ("Sparkle", "Sparkling notification text"),
];

/// Application state
struct App {
    /// Animation states for each effect
    states: Vec<AnimatedTextState>,
    /// Currently selected effect
    selected: usize,
    /// Whether animations are running
    running: bool,
    /// Wave width for wave effect
    wave_width: usize,
    /// Should quit
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let states = (0..DEMO_TEXTS.len())
            .map(|_| AnimatedTextState::with_interval(50))
            .collect();

        Self {
            states,
            selected: 0,
            running: true,
            wave_width: 5,
            should_quit: false,
        }
    }

    fn toggle_running(&mut self) {
        self.running = !self.running;
        for state in &mut self.states {
            if self.running {
                state.start();
            } else {
                state.stop();
            }
        }
    }

    fn tick_all(&mut self) {
        for (i, state) in self.states.iter_mut().enumerate() {
            let text_width = DEMO_TEXTS[i].1.len();
            state.tick_with_text_width(text_width);
        }
    }

    fn select_next(&mut self) {
        self.selected = (self.selected + 1) % DEMO_TEXTS.len();
    }

    fn select_prev(&mut self) {
        self.selected = self.selected.checked_sub(1).unwrap_or(DEMO_TEXTS.len() - 1);
    }

    fn increase_wave_width(&mut self) {
        self.wave_width = (self.wave_width + 1).min(15);
    }

    fn decrease_wave_width(&mut self) {
        self.wave_width = self.wave_width.saturating_sub(1).max(1);
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
                } else if key.code == KeyCode::Right || key.code == KeyCode::Char('l') {
                    app.increase_wave_width();
                } else if key.code == KeyCode::Left || key.code == KeyCode::Char('h') {
                    app.decrease_wave_width();
                }
            }
        }

        // Tick all animations
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
            Constraint::Min(1),    // Animated texts
            Constraint::Length(6), // Help + info
        ])
        .split(area);

    // Title
    let status = if app.running { "Running" } else { "Paused" };
    let title = Paragraph::new(format!(
        "Animated Text Demo - {} Effects | Status: {} | Wave Width: {}",
        DEMO_TEXTS.len(),
        status,
        app.wave_width
    ))
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Animated texts
    render_animated_texts(f, app, chunks[1]);

    // Help and selected info
    let selected_name = DEMO_TEXTS[app.selected].0;
    let help_lines = vec![
        Line::from(vec![
            Span::styled("Selected: ", Style::default().fg(Color::Gray)),
            Span::styled(
                selected_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" ({})", app.selected + 1),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Space", Style::default().fg(Color::Yellow)),
            Span::raw(": Toggle pause  "),
            Span::styled("Up/Down", Style::default().fg(Color::Yellow)),
            Span::raw(": Select effect"),
        ]),
        Line::from(vec![
            Span::styled("Left/Right", Style::default().fg(Color::Yellow)),
            Span::raw(": Adjust wave width  "),
            Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];
    let help = Paragraph::new(help_lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn render_animated_texts(f: &mut Frame, app: &App, area: Rect) {
    let row_constraints: Vec<Constraint> = (0..DEMO_TEXTS.len())
        .map(|_| Constraint::Length(3))
        .chain(std::iter::once(Constraint::Min(0)))
        .collect();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);

    for (idx, row_chunk) in rows.iter().enumerate().take(DEMO_TEXTS.len()) {
        render_single_effect(f, app, *row_chunk, idx);
    }
}

fn render_single_effect(f: &mut Frame, app: &App, area: Rect, idx: usize) {
    let (name, text) = DEMO_TEXTS[idx];
    let state = &app.states[idx];
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

    // Create the appropriate style for this effect
    let style = match idx {
        0 => {
            // Pulse - cyan to blue
            AnimatedTextStyle::pulse(Color::Cyan, Color::Blue).bold()
        }
        1 => {
            // Wave - white base with yellow highlight
            AnimatedTextStyle::wave(Color::White, Color::Yellow).wave_width(app.wave_width)
        }
        2 => {
            // Rainbow
            AnimatedTextStyle::rainbow()
        }
        3 => {
            // Gradient shift - green to cyan
            AnimatedTextStyle::gradient_shift(Color::Green, Color::Cyan)
        }
        4 => {
            // Sparkle - white with yellow sparkles
            AnimatedTextStyle::sparkle(Color::White, Color::Yellow)
        }
        _ => AnimatedTextStyle::default(),
    };

    let animated_text = AnimatedText::new(text, state).style(style);

    f.render_widget(
        animated_text,
        Rect::new(inner.x + 1, inner.y, inner.width.saturating_sub(2), 1),
    );
}
