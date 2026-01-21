//! Mouse Pointer Demo
//!
//! Interactive demo showing mouse pointer indicator features:
//! - Toggle pointer on/off with 'p' key
//! - Cycle through style presets with 's' key
//! - Display current position coordinates
//! - Demonstrate layered rendering (pointer on top)
//!
//! Run with: cargo run --example mouse_pointer_demo

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

use ratatui_interact::components::{MousePointer, MousePointerState, MousePointerStyle};
use ratatui_interact::events::is_close_key;

/// Available style presets
const STYLE_NAMES: &[&str] = &["Default (Block)", "Crosshair", "Arrow", "Dot", "Plus"];

/// Application state
struct App {
    /// Mouse pointer state
    pointer_state: MousePointerState,
    /// Current style index
    style_index: usize,
    /// Should quit
    should_quit: bool,
    /// Last known position for display
    last_position: Option<(u16, u16)>,
}

impl App {
    fn new() -> Self {
        Self {
            pointer_state: MousePointerState::default(),
            style_index: 0,
            should_quit: false,
            last_position: None,
        }
    }

    fn toggle_pointer(&mut self) {
        self.pointer_state.toggle();
    }

    fn next_style(&mut self) {
        self.style_index = (self.style_index + 1) % STYLE_NAMES.len();
    }

    fn current_style(&self) -> MousePointerStyle {
        match self.style_index {
            0 => MousePointerStyle::default(),
            1 => MousePointerStyle::crosshair(),
            2 => MousePointerStyle::arrow(),
            3 => MousePointerStyle::dot(),
            4 => MousePointerStyle::plus(),
            _ => MousePointerStyle::default(),
        }
    }

    fn update_position(&mut self, col: u16, row: u16) {
        self.pointer_state.update_position(col, row);
        self.last_position = Some((col, row));
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

        match event::read()? {
            Event::Key(key) => {
                if is_close_key(&key) || key.code == KeyCode::Char('q') {
                    app.should_quit = true;
                } else if key.code == KeyCode::Char('p') {
                    app.toggle_pointer();
                } else if key.code == KeyCode::Char('s') {
                    app.next_style();
                }
            }
            Event::Mouse(mouse) => {
                // Update position on any mouse event
                app.update_position(mouse.column, mouse.row);
            }
            _ => {}
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

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Content area
            Constraint::Length(6), // Info panel
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Mouse Pointer Demo")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Content area - render some boxes to show pointer renders on top
    render_content_area(f, chunks[1]);

    // Info panel
    render_info_panel(f, app, chunks[2]);

    // Render mouse pointer LAST (on top of everything)
    let pointer = MousePointer::new(&app.pointer_state).style(app.current_style());
    pointer.render(f.buffer_mut());
}

fn render_content_area(f: &mut Frame, area: Rect) {
    // Create a grid of colored boxes
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    let colors = [
        [Color::Red, Color::Green, Color::Blue],
        [Color::Yellow, Color::Magenta, Color::Cyan],
        [Color::LightRed, Color::LightGreen, Color::LightBlue],
    ];

    for (row_idx, row) in rows.iter().enumerate() {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(*row);

        for (col_idx, col) in cols.iter().enumerate() {
            let color = colors[row_idx][col_idx];
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color))
                .title(format!(" Box {},{} ", row_idx + 1, col_idx + 1));

            let content = Paragraph::new(format!(
                "Move mouse here\nto see pointer\n\nColor: {:?}",
                color
            ))
            .style(Style::default().fg(color))
            .block(block);

            f.render_widget(content, *col);
        }
    }
}

fn render_info_panel(f: &mut Frame, app: &App, area: Rect) {
    let status = if app.pointer_state.enabled {
        Span::styled("ON", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("OFF", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
    };

    let position_text = match app.last_position {
        Some((col, row)) => format!("({}, {})", col, row),
        None => "N/A".to_string(),
    };

    let style_name = STYLE_NAMES[app.style_index];

    let info_lines = vec![
        Line::from(vec![
            Span::styled("Pointer: ", Style::default().fg(Color::Gray)),
            status,
            Span::raw("  |  "),
            Span::styled("Position: ", Style::default().fg(Color::Gray)),
            Span::styled(position_text, Style::default().fg(Color::Yellow)),
            Span::raw("  |  "),
            Span::styled("Style: ", Style::default().fg(Color::Gray)),
            Span::styled(style_name, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("p", Style::default().fg(Color::Yellow)),
            Span::raw(": Toggle pointer  "),
            Span::styled("s", Style::default().fg(Color::Yellow)),
            Span::raw(": Cycle styles  "),
            Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Move mouse around to see the pointer follow the cursor",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];

    let info = Paragraph::new(info_lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(info, area);
}
