//! Scrollable Content Demo
//!
//! Demonstrates ScrollableContent with View/Copy mode:
//! - Single scrollable content pane with keyboard/mouse navigation
//! - Press 'c' to enter View/Copy mode (exits to terminal for native text selection)
//! - Press 'n' to toggle line numbers
//! - On quit, content is printed to stdout for easy copying
//!
//! Run with: cargo run --example copyable_pane_demo

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
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use ratatui_interact::{
    components::{
        ScrollableContent, ScrollableContentState, ScrollableContentStyle,
        handle_scrollable_content_key, handle_scrollable_content_mouse,
    },
    events::is_close_key,
    utils::{ViewCopyAction, ViewCopyConfig, ViewCopyMode, clear_main_screen},
};

/// Application state
struct App {
    /// Content state
    content: ScrollableContentState,
    /// Raw content (without line numbers)
    raw_content: Vec<String>,
    /// Whether to show line numbers
    show_line_numbers: bool,
    /// Whether to quit
    should_quit: bool,
    /// Last rendered content area
    content_area: Rect,
}

impl App {
    fn new() -> Self {
        let raw = generate_content();
        let mut content = ScrollableContentState::new(format_with_line_numbers(&raw, true));
        content.set_title("Scrollable Content");
        content.set_focused(true);

        Self {
            content,
            raw_content: raw,
            show_line_numbers: true,
            should_quit: false,
            content_area: Rect::default(),
        }
    }

    fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
        self.content.set_lines(format_with_line_numbers(
            &self.raw_content,
            self.show_line_numbers,
        ));
    }

    fn get_display_content(&self) -> Vec<String> {
        format_with_line_numbers(&self.raw_content, self.show_line_numbers)
    }
}

/// Run view/copy mode using the crate's utility
fn run_view_copy_mode(
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    let mut stdout = io::stdout();

    let config = ViewCopyConfig::default()
        .with_header("View/Copy Mode")
        .show_hints(true);

    let mode = ViewCopyMode::enter_with_config(&mut stdout, config)?;
    mode.print_lines(&app.get_display_content())?;

    // Event loop for view/copy mode
    loop {
        match mode.wait_for_input()? {
            ViewCopyAction::Exit => break,
            ViewCopyAction::ToggleLineNumbers => {
                app.toggle_line_numbers();
                mode.clear()?;
                mode.print_lines(&app.get_display_content())?;
            }
            ViewCopyAction::None => {}
        }
    }

    mode.exit(terminal)?;
    Ok(())
}

fn main() -> io::Result<()> {
    // Clear main screen buffer before entering alternate screen
    // (ensures View/Copy mode has a clean buffer)
    clear_main_screen()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    handle_key(&mut app, &key, &mut terminal)?;
                }
                Event::Mouse(mouse) => {
                    handle_scrollable_content_mouse(
                        &mut app.content,
                        &mouse,
                        app.content_area,
                        app.content_area.height as usize,
                    );
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
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Print raw content to stdout on exit (no line numbers)
    for line in &app.raw_content {
        println!("{}", line);
    }

    Ok(())
}

fn handle_key(
    app: &mut App,
    key: &crossterm::event::KeyEvent,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Char('c') => {
            run_view_copy_mode(app, terminal)?;
        }
        KeyCode::Char('n') => {
            app.toggle_line_numbers();
        }
        _ => {
            let _ = handle_scrollable_content_key(
                &mut app.content,
                key,
                app.content_area.height as usize,
            );
        }
    }

    if is_close_key(key) {
        app.should_quit = true;
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(1),    // Content
            Constraint::Length(1), // Help
        ])
        .split(area);

    // Title bar
    let line_hint = if app.show_line_numbers { "n: hide lines" } else { "n: show lines" };
    let title = Line::from(vec![
        Span::styled(" Scrollable Content ", Style::default().fg(Color::Cyan)),
        Span::raw(" | "),
        Span::styled("c: view/copy", Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(line_hint, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled("q: quit", Style::default().fg(Color::Yellow)),
    ]);
    f.render_widget(Paragraph::new(title), chunks[0]);

    // Content
    app.content_area = chunks[1];
    let style = ScrollableContentStyle::default().with_focus_color(Color::Cyan);
    let content = ScrollableContent::new(&app.content).style(style);
    content.render(chunks[1], f.buffer_mut());

    // Help bar
    let help = Line::from(vec![
        Span::styled("↑/↓ j/k", Style::default().fg(Color::DarkGray)),
        Span::raw(": Scroll  "),
        Span::styled("PgUp/PgDn", Style::default().fg(Color::DarkGray)),
        Span::raw(": Page  "),
        Span::styled("Home/End", Style::default().fg(Color::DarkGray)),
        Span::raw(": Top/Bottom"),
    ]);
    f.render_widget(Paragraph::new(help), chunks[2]);
}

/// Generate content - 25 sections of lorem ipsum paragraphs
fn generate_content() -> Vec<String> {
    let paragraphs = vec![
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.".to_string(),
        String::new(),
        "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string(),
        String::new(),
        "Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo.".to_string(),
        String::new(),
        "Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt.".to_string(),
        String::new(),
        "Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit, sed quia non numquam eius modi tempora incidunt ut labore et dolore magnam aliquam quaerat voluptatem.".to_string(),
        String::new(),
    ];

    let mut lines = Vec::new();
    for i in 0..25 {
        lines.push(format!("═══ Section {} ═══", i + 1));
        lines.push(String::new());
        lines.extend(paragraphs.clone());
    }

    lines
}

/// Format content with or without line numbers
fn format_with_line_numbers(lines: &[String], show_numbers: bool) -> Vec<String> {
    if show_numbers {
        let width = lines.len().to_string().len();
        lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if line.is_empty() {
                    String::new()
                } else {
                    format!("{:>width$} │ {}", i + 1, line, width = width)
                }
            })
            .collect()
    } else {
        lines.to_vec()
    }
}
