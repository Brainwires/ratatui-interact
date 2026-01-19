//! TextArea Demo
//!
//! Interactive demo showing multi-line text area features:
//! - Multi-line text editing with cursor
//! - Vertical navigation (arrows, PageUp/PageDown, Ctrl+Home/End)
//! - Line numbers and current line highlighting
//! - Scrolling for large content
//!
//! Run with: cargo run --example textarea_demo

use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use ratatui_interact::{
    components::{TabConfig, TextArea, TextAreaState, TextAreaStyle},
    events::{
        get_char, has_ctrl, is_backspace, is_close_key, is_ctrl_a, is_ctrl_e, is_ctrl_k, is_ctrl_u,
        is_ctrl_w, is_delete, is_end, is_enter, is_home, is_left_click, is_tab,
    },
    traits::ClickRegionRegistry,
};

/// Application state
struct App {
    /// Text area state
    textarea: TextAreaState,
    /// Click regions
    click_regions: ClickRegionRegistry<()>,
    /// Show line numbers
    show_line_numbers: bool,
    /// Should quit
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let initial_text = r#"Welcome to the TextArea demo!

This is a multi-line text editor component.
You can type, delete, and navigate through text.

Features:
- Multi-line editing
- Cursor movement (arrows, Home/End)
- Page Up/Down navigation
- Ctrl+Home/End for document start/end
- Word navigation (Ctrl+Left/Right)
- Line deletion (Ctrl+D)
- Unicode and emoji support: 你好 👋 🎉

Try these shortcuts:
- Ctrl+A: Line start
- Ctrl+E: Line end
- Ctrl+U: Delete to line start
- Ctrl+K: Delete to line end
- Ctrl+W: Delete word backward
- Ctrl+D: Delete current line
- Ctrl+L: Toggle line numbers

Press Esc to quit."#;

        let mut textarea = TextAreaState::new(initial_text);
        textarea.focused = true;
        textarea.tab_config = TabConfig::Spaces(4);

        Self {
            textarea,
            click_regions: ClickRegionRegistry::new(),
            show_line_numbers: true,
            should_quit: false,
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

        match event::read()? {
            Event::Key(key) => {
                if is_close_key(&key) {
                    app.should_quit = true;
                } else if is_enter(&key) {
                    app.textarea.insert_newline();
                } else if is_tab(&key) {
                    app.textarea.insert_tab();
                } else if is_backspace(&key) {
                    app.textarea.delete_char_backward();
                } else if is_delete(&key) {
                    app.textarea.delete_char_forward();
                } else if is_home(&key) {
                    if has_ctrl(&key) {
                        app.textarea.move_to_start();
                    } else {
                        app.textarea.move_line_start();
                    }
                } else if is_end(&key) {
                    if has_ctrl(&key) {
                        app.textarea.move_to_end();
                    } else {
                        app.textarea.move_line_end();
                    }
                } else if is_ctrl_a(&key) {
                    app.textarea.move_line_start();
                } else if is_ctrl_e(&key) {
                    app.textarea.move_line_end();
                } else if is_ctrl_u(&key) {
                    app.textarea.delete_to_line_start();
                } else if is_ctrl_k(&key) {
                    app.textarea.delete_to_line_end();
                } else if is_ctrl_w(&key) {
                    app.textarea.delete_word_backward();
                } else if key.code == KeyCode::Char('d') && has_ctrl(&key) {
                    app.textarea.delete_line();
                } else if key.code == KeyCode::Char('l') && has_ctrl(&key) {
                    // Toggle line numbers with Ctrl+L
                    app.show_line_numbers = !app.show_line_numbers;
                } else if key.code == KeyCode::Left {
                    if has_ctrl(&key) {
                        app.textarea.move_word_left();
                    } else {
                        app.textarea.move_left();
                    }
                } else if key.code == KeyCode::Right {
                    if has_ctrl(&key) {
                        app.textarea.move_word_right();
                    } else {
                        app.textarea.move_right();
                    }
                } else if key.code == KeyCode::Up {
                    app.textarea.move_up();
                } else if key.code == KeyCode::Down {
                    app.textarea.move_down();
                } else if key.code == KeyCode::PageUp {
                    app.textarea.move_page_up();
                } else if key.code == KeyCode::PageDown {
                    app.textarea.move_page_down();
                } else if let Some(c) = get_char(&key) {
                    app.textarea.insert_char(c);
                }
            }
            Event::Mouse(mouse) => {
                if is_left_click(&mouse) {
                    if app.click_regions.handle_click(mouse.column, mouse.row).is_some() {
                        app.textarea.focused = true;
                    }
                }
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
    // Clear click regions
    app.click_regions.clear();

    let area = f.area();

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // TextArea
            Constraint::Length(3), // Status
            Constraint::Length(6), // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new("TextArea Demo - Multi-line Text Editor")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // TextArea
    let textarea_area = chunks[1];

    let style = TextAreaStyle::default()
        .focused_border(Color::Cyan)
        .cursor_fg(Color::Yellow)
        .line_number_fg(Color::DarkGray)
        .current_line_bg(Some(Color::Rgb(40, 40, 50)))
        .show_line_numbers(app.show_line_numbers);

    let textarea = TextArea::new()
        .label("Editor")
        .placeholder("Start typing...")
        .style(style);

    let region = textarea.render_stateful(f, textarea_area, &mut app.textarea);
    app.click_regions.register(region.area, ());

    // Status bar
    let status_text = format!(
        "Line: {} / {} | Col: {} | Lines: {} | Chars: {} | Line Numbers: {}",
        app.textarea.cursor_line + 1,
        app.textarea.line_count(),
        app.textarea.cursor_col + 1,
        app.textarea.line_count(),
        app.textarea.len(),
        if app.show_line_numbers { "ON" } else { "OFF" }
    );
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .block(Block::default());
    f.render_widget(status, chunks[2]);

    // Help text
    let help_lines = vec![
        Line::from(vec![
            Span::styled("Arrows", Style::default().fg(Color::Yellow)),
            Span::raw(": Move cursor  "),
            Span::styled("PgUp/PgDn", Style::default().fg(Color::Yellow)),
            Span::raw(": Page scroll  "),
            Span::styled("Home/End", Style::default().fg(Color::Yellow)),
            Span::raw(": Line start/end"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+Home/End", Style::default().fg(Color::Yellow)),
            Span::raw(": Doc start/end  "),
            Span::styled("Ctrl+Left/Right", Style::default().fg(Color::Yellow)),
            Span::raw(": Word nav  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": New line"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+U/K", Style::default().fg(Color::Yellow)),
            Span::raw(": Del to start/end  "),
            Span::styled("Ctrl+W", Style::default().fg(Color::Yellow)),
            Span::raw(": Del word  "),
            Span::styled("Ctrl+D", Style::default().fg(Color::Yellow)),
            Span::raw(": Del line"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+L", Style::default().fg(Color::Yellow)),
            Span::raw(": Toggle line numbers  "),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Insert spaces  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];
    let help = Paragraph::new(help_lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[3]);
}
