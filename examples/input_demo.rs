//! Input Demo
//!
//! Interactive demo showing input field features:
//! - Text editing with cursor
//! - Tab navigation between fields
//! - Placeholder text
//! - Labels and styling
//!
//! Run with: cargo run --example input_demo

use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use ratatui_interact::{
    components::{Input, InputState},
    events::{
        get_char, is_backspace, is_backtab, is_close_key, is_ctrl_a, is_ctrl_e, is_ctrl_k,
        is_ctrl_u, is_ctrl_w, is_delete, is_end, is_home, is_left_click, is_tab,
    },
    state::FocusManager,
    traits::ClickRegionRegistry,
};

/// Application state
struct App {
    /// Input fields: (label, placeholder, state)
    inputs: Vec<(&'static str, &'static str, InputState)>,
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

        let inputs = vec![
            ("Name", "Enter your name...", InputState::empty()),
            ("Email", "user@example.com", InputState::empty()),
            (
                "Message",
                "Type your message...",
                InputState::new("Hello, World!"),
            ),
        ];

        for i in 0..inputs.len() {
            focus.register(i);
        }

        Self {
            inputs,
            focus,
            click_regions: ClickRegionRegistry::new(),
            should_quit: false,
        }
    }

    fn current_input(&mut self) -> Option<&mut InputState> {
        self.focus.current().map(|&idx| &mut self.inputs[idx].2)
    }

    fn handle_click(&mut self, col: u16, row: u16) {
        if let Some(&idx) = self.click_regions.handle_click(col, row) {
            self.focus.set(idx);
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
            if is_close_key(&key) {
                app.should_quit = true;
            } else if is_tab(&key) {
                app.focus.next();
            } else if is_backtab(&key) {
                app.focus.prev();
            } else if is_backspace(&key) {
                if let Some(input) = app.current_input() {
                    input.delete_char_backward();
                }
            } else if is_delete(&key) {
                if let Some(input) = app.current_input() {
                    input.delete_char_forward();
                }
            } else if is_home(&key) || is_ctrl_a(&key) {
                if let Some(input) = app.current_input() {
                    input.move_home();
                }
            } else if is_end(&key) || is_ctrl_e(&key) {
                if let Some(input) = app.current_input() {
                    input.move_end();
                }
            } else if is_ctrl_u(&key) {
                if let Some(input) = app.current_input() {
                    // Delete to start of line
                    while input.cursor_pos > 0 {
                        input.delete_char_backward();
                    }
                }
            } else if is_ctrl_k(&key) {
                if let Some(input) = app.current_input() {
                    // Delete to end of line
                    while input.cursor_pos < input.len() {
                        input.delete_char_forward();
                    }
                }
            } else if is_ctrl_w(&key) {
                if let Some(input) = app.current_input() {
                    input.delete_word_backward();
                }
            } else if key.code == KeyCode::Left {
                if let Some(input) = app.current_input() {
                    input.move_left();
                }
            } else if key.code == KeyCode::Right {
                if let Some(input) = app.current_input() {
                    input.move_right();
                }
            } else if let Some(c) = get_char(&key) {
                if let Some(input) = app.current_input() {
                    input.insert_char(c);
                }
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
            Constraint::Length(5), // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Input Demo")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Input fields
    let input_area = chunks[1];
    let input_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            app.inputs
                .iter()
                .map(|_| Constraint::Length(3))
                .collect::<Vec<_>>(),
        )
        .split(input_area);

    for (idx, (label, placeholder, state)) in app.inputs.iter_mut().enumerate() {
        // Set focus state
        state.focused = app.focus.is_focused(&idx);

        let input_widget = Input::new(state).label(label).placeholder(placeholder);

        let input_area = Rect::new(
            input_chunks[idx].x + 2,
            input_chunks[idx].y,
            input_chunks[idx].width.saturating_sub(4).min(50),
            3,
        );

        let region = input_widget.render_stateful(f, input_area);
        app.click_regions.register(region.area, idx);
    }

    // Help text
    let help_lines = vec![
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Next field  "),
            Span::styled("Shift+Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Prev field  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
        Line::from(vec![
            Span::styled("←/→", Style::default().fg(Color::Yellow)),
            Span::raw(": Move cursor  "),
            Span::styled("Home/Ctrl+A", Style::default().fg(Color::Yellow)),
            Span::raw(": Start  "),
            Span::styled("End/Ctrl+E", Style::default().fg(Color::Yellow)),
            Span::raw(": End"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+U", Style::default().fg(Color::Yellow)),
            Span::raw(": Clear to start  "),
            Span::styled("Ctrl+K", Style::default().fg(Color::Yellow)),
            Span::raw(": Clear to end  "),
            Span::styled("Ctrl+W", Style::default().fg(Color::Yellow)),
            Span::raw(": Delete word"),
        ]),
    ];
    let help = Paragraph::new(help_lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}
