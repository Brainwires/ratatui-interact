//! Marquee Demo
//!
//! Interactive demo showing marquee text features:
//! - Continuous scrolling mode (loops around)
//! - Bounce mode (scrolls back and forth)
//! - Static mode (truncate with ellipsis)
//! - Pause at edges
//! - Speed controls
//!
//! Run with: cargo run --example marquee_demo

use std::io;
use std::time::{Duration, Instant};

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

use ratatui_interact::components::{MarqueeMode, MarqueeState, MarqueeStyle, MarqueeText};

/// A single marquee display with its configuration
struct MarqueeEntry {
    label: String,
    text: String,
    state: MarqueeState,
    style: MarqueeStyle,
}

/// Application state
struct App {
    /// Marquee entries
    marquees: Vec<MarqueeEntry>,
    /// Is animation paused
    paused: bool,
    /// Last tick time
    last_tick: Instant,
    /// Tick rate in milliseconds
    tick_rate: u64,
    /// Should quit
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let marquees = vec![
            MarqueeEntry {
                label: "Continuous Mode".to_string(),
                text: "Hello World! This is a continuous scrolling marquee that loops around seamlessly.".to_string(),
                state: MarqueeState::new(),
                style: MarqueeStyle::default()
                    .mode(MarqueeMode::Continuous)
                    .text_style(Style::default().fg(Color::Green))
                    .scroll_speed(1)
                    .separator("  >>>  "),
            },
            MarqueeEntry {
                label: "Bounce Mode (File Path)".to_string(),
                text: "/home/user/projects/my-awesome-project/src/components/very_long_filename_example.rs".to_string(),
                state: MarqueeState::new(),
                style: MarqueeStyle::file_path(),
            },
            MarqueeEntry {
                label: "Bounce Mode (Status)".to_string(),
                text: "Processing files... Scanning directory structure... Analyzing dependencies... Building index...".to_string(),
                state: MarqueeState::new(),
                style: MarqueeStyle::default()
                    .mode(MarqueeMode::Bounce)
                    .text_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                    .scroll_speed(2)
                    .pause_at_edge(5),
            },
            MarqueeEntry {
                label: "Static Mode".to_string(),
                text: "This text is too long to fit in the available space and will be truncated with ellipsis".to_string(),
                state: MarqueeState::new(),
                style: MarqueeStyle::default()
                    .mode(MarqueeMode::Static)
                    .text_style(Style::default().fg(Color::Magenta)),
            },
            MarqueeEntry {
                label: "Unicode Support".to_string(),
                text: "日本語テスト \u{1F680} Emoji support \u{2764} 中文测试 한국어 \u{1F389}".to_string(),
                state: MarqueeState::new(),
                style: MarqueeStyle::default()
                    .mode(MarqueeMode::Continuous)
                    .text_style(Style::default().fg(Color::Rgb(255, 165, 0)))
                    .scroll_speed(1),
            },
            MarqueeEntry {
                label: "Fast Scroll".to_string(),
                text: "This marquee scrolls quickly! Watch it zoom across the screen at high speed.".to_string(),
                state: MarqueeState::new(),
                style: MarqueeStyle::default()
                    .mode(MarqueeMode::Continuous)
                    .text_style(Style::default().fg(Color::Red))
                    .scroll_speed(3),
            },
        ];

        Self {
            marquees,
            paused: false,
            last_tick: Instant::now(),
            tick_rate: 100, // milliseconds
            should_quit: false,
        }
    }

    fn tick(&mut self) {
        if self.paused {
            return;
        }

        // We need to calculate text width vs viewport width
        // For now, we'll use a fixed estimate and let the widget handle it
        for entry in &mut self.marquees {
            let text_width = unicode_width::UnicodeWidthStr::width(entry.text.as_str());
            // Assume viewport is about 50 chars for demo (actual width varies)
            entry.state.tick(text_width, 50, &entry.style);
        }
    }

    fn reset_all(&mut self) {
        for entry in &mut self.marquees {
            entry.state.reset();
        }
    }

    fn increase_speed(&mut self) {
        if self.tick_rate > 20 {
            self.tick_rate -= 20;
        }
    }

    fn decrease_speed(&mut self) {
        if self.tick_rate < 500 {
            self.tick_rate += 20;
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

        // Handle tick
        if app.last_tick.elapsed() >= Duration::from_millis(app.tick_rate) {
            app.tick();
            app.last_tick = Instant::now();
        }

        // Poll for events with timeout
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Char(' ') => {
                        app.paused = !app.paused;
                    }
                    KeyCode::Char('r') => {
                        app.reset_all();
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        app.increase_speed();
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        app.decrease_speed();
                    }
                    _ => {}
                }
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
    let area = f.area();

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Content
            Constraint::Length(3), // Status
            Constraint::Length(3), // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Marquee Text Demo")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Marquees content area
    let content_area = chunks[1];
    let marquee_height = 3; // Each marquee needs 3 rows (label + marquee + spacing)
    let marquee_constraints: Vec<Constraint> = app
        .marquees
        .iter()
        .map(|_| Constraint::Length(marquee_height))
        .collect();

    let marquee_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(marquee_constraints)
        .split(content_area);

    for (idx, entry) in app.marquees.iter_mut().enumerate() {
        if idx >= marquee_chunks.len() {
            break;
        }

        let chunk = marquee_chunks[idx];

        // Label
        let label =
            Paragraph::new(format!("{}: ", entry.label)).style(Style::default().fg(Color::White));
        f.render_widget(label, Rect::new(chunk.x, chunk.y, chunk.width, 1));

        // Marquee text area (with border to show the constrained area)
        let marquee_area = Rect::new(
            chunk.x + 2,
            chunk.y + 1,
            chunk.width.saturating_sub(4).min(60), // Constrain width to show scrolling
            1,
        );

        // Draw a subtle background/border to show the viewport
        let border_area = Rect::new(
            marquee_area.x.saturating_sub(1),
            marquee_area.y,
            marquee_area.width + 2,
            1,
        );
        let border_style = Style::default().fg(Color::DarkGray);
        f.buffer_mut()
            .set_string(border_area.x, border_area.y, "[", border_style);
        f.buffer_mut().set_string(
            border_area.x + border_area.width - 1,
            border_area.y,
            "]",
            border_style,
        );

        // Render the marquee
        let marquee = MarqueeText::new(&entry.text, &mut entry.state).style(entry.style.clone());
        f.render_widget(marquee, marquee_area);
    }

    // Status bar
    let status_text = if app.paused {
        format!("PAUSED | Tick rate: {}ms", app.tick_rate)
    } else {
        format!("RUNNING | Tick rate: {}ms", app.tick_rate)
    };
    let status_style = if app.paused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    };
    let status = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(status, chunks[2]);

    // Help text
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Space", Style::default().fg(Color::Yellow)),
        Span::raw(": Pause  "),
        Span::styled("R", Style::default().fg(Color::Yellow)),
        Span::raw(": Reset  "),
        Span::styled("+/-", Style::default().fg(Color::Yellow)),
        Span::raw(": Speed  "),
        Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
        Span::raw(": Quit"),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[3]);
}
