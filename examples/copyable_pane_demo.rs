//! Copyable Pane Demo
//!
//! Interactive demo showing scrollable content with copy mode:
//! - Horizontal split pane with two scrollable content panes
//! - Tab navigation between panes with cyan border highlight
//! - Press F10/Enter to toggle fullscreen for focused pane
//! - Press 'c' to enter View/Copy mode (exits to normal terminal for native selection)
//! - Toast notifications for feedback
//!
//! Three display modes:
//! 1. Normal - Split pane view with borders
//! 2. Fullscreen - Fullscreen within TUI (still has title bar)
//! 3. View/Copy - Exits alternate screen, prints content to terminal scrollback
//!
//! Run with: cargo run --example copyable_pane_demo --features clipboard

use std::io::{self, Write};

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
    widgets::{Block, Borders, Paragraph, Widget},
};

use ratatui_interact::{
    components::{
        Orientation, ScrollableContent, ScrollableContentState,
        ScrollableContentStyle, SplitPane, SplitPaneAction, SplitPaneState, SplitPaneStyle,
        Toast, ToastState, handle_scrollable_content_key, handle_scrollable_content_mouse,
        handle_split_pane_mouse,
    },
    events::is_close_key,
    traits::ClickRegionRegistry,
};

/// Display mode for the content
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisplayMode {
    /// Normal split pane view
    Normal,
    /// Fullscreen within TUI (still has title bar)
    Fullscreen,
}

/// Which pane is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedPane {
    Left,
    Right,
}

impl FocusedPane {
    fn toggle(self) -> Self {
        match self {
            FocusedPane::Left => FocusedPane::Right,
            FocusedPane::Right => FocusedPane::Left,
        }
    }
}

/// Application state
struct App {
    /// Left pane content state
    left_pane: ScrollableContentState,
    /// Right pane content state
    right_pane: ScrollableContentState,
    /// Raw content (without line numbers) for reformatting
    left_raw_content: Vec<String>,
    /// Raw content (without line numbers) for reformatting
    right_raw_content: Vec<String>,
    /// Whether to show line numbers
    show_line_numbers: bool,
    /// Split pane state
    split_state: SplitPaneState,
    /// Click regions for split pane
    split_registry: ClickRegionRegistry<SplitPaneAction>,
    /// Which pane is focused
    focused_pane: FocusedPane,
    /// Current display mode
    display_mode: DisplayMode,
    /// Toast notification state
    toast: ToastState,
    /// Whether to quit
    should_quit: bool,
    /// Last rendered content area (for mouse handling)
    last_left_area: Rect,
    last_right_area: Rect,
}

impl App {
    fn new() -> Self {
        let left_raw = generate_raw_content("Left Pane");
        let right_raw = generate_raw_content("Right Pane");

        let mut left_pane =
            ScrollableContentState::new(format_with_line_numbers(&left_raw, true));
        left_pane.set_title("Left Pane");
        left_pane.set_focused(true);

        let mut right_pane =
            ScrollableContentState::new(format_with_line_numbers(&right_raw, true));
        right_pane.set_title("Right Pane");

        Self {
            left_pane,
            right_pane,
            left_raw_content: left_raw,
            right_raw_content: right_raw,
            show_line_numbers: true,
            split_state: SplitPaneState::half(),
            split_registry: ClickRegionRegistry::new(),
            focused_pane: FocusedPane::Left,
            display_mode: DisplayMode::Normal,
            toast: ToastState::new(),
            should_quit: false,
            last_left_area: Rect::default(),
            last_right_area: Rect::default(),
        }
    }

    fn focused_state(&mut self) -> &mut ScrollableContentState {
        match self.focused_pane {
            FocusedPane::Left => &mut self.left_pane,
            FocusedPane::Right => &mut self.right_pane,
        }
    }

    fn focused_raw_content(&self) -> &[String] {
        match self.focused_pane {
            FocusedPane::Left => &self.left_raw_content,
            FocusedPane::Right => &self.right_raw_content,
        }
    }

    fn toggle_focus(&mut self) {
        self.focused_pane = self.focused_pane.toggle();
        self.left_pane
            .set_focused(self.focused_pane == FocusedPane::Left);
        self.right_pane
            .set_focused(self.focused_pane == FocusedPane::Right);
    }

    fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
        self.left_pane.set_lines(format_with_line_numbers(
            &self.left_raw_content,
            self.show_line_numbers,
        ));
        self.right_pane.set_lines(format_with_line_numbers(
            &self.right_raw_content,
            self.show_line_numbers,
        ));

        if self.show_line_numbers {
            self.toast.show("Line numbers enabled", 2000);
        } else {
            self.toast.show("Line numbers disabled", 2000);
        }
    }

    fn enter_fullscreen(&mut self) {
        self.display_mode = DisplayMode::Fullscreen;
        self.focused_state().set_fullscreen(true);
    }

    fn exit_fullscreen_to_normal(&mut self) {
        self.display_mode = DisplayMode::Normal;
        self.left_pane.set_fullscreen(false);
        self.right_pane.set_fullscreen(false);
    }
}

/// Run view/copy mode - exits alternate screen, prints content, waits for exit key
fn run_view_copy_mode(
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    let mut stdout = io::stdout();

    // Leave alternate screen and disable mouse capture
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;

    // Clear any leftover content from before entering TUI
    print!("\x1B[3J\x1B[2J\x1B[H");
    let _ = stdout.flush();

    // Print content to terminal scrollback
    print_content(app);

    // Re-enable raw mode to catch exit keys
    enable_raw_mode()?;

    // Simple event loop - just exit key and line number toggle
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('c') | KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('n') => {
                        // Toggle line numbers and reprint
                        app.show_line_numbers = !app.show_line_numbers;
                        app.left_pane.set_lines(format_with_line_numbers(
                            &app.left_raw_content,
                            app.show_line_numbers,
                        ));
                        app.right_pane.set_lines(format_with_line_numbers(
                            &app.right_raw_content,
                            app.show_line_numbers,
                        ));
                        // Must disable raw mode for println to work correctly
                        disable_raw_mode()?;
                        // Clear before reprinting
                        print!("\x1B[3J\x1B[2J\x1B[H");
                        let _ = io::stdout().flush();
                        print_content(app);
                        enable_raw_mode()?;
                    }
                    _ => {}
                }
            }
        }
    }

    // Re-enter alternate screen and enable mouse capture
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Clear terminal to force full redraw
    terminal.clear()?;

    Ok(())
}

/// Print the focused pane content to stdout
fn print_content(app: &App) {
    let content = format_with_line_numbers(app.focused_raw_content(), app.show_line_numbers);
    let pane_name = match app.focused_pane {
        FocusedPane::Left => "Left Pane",
        FocusedPane::Right => "Right Pane",
    };

    // Clear scrollback, screen, and move cursor to top
    print!("\x1B[3J\x1B[2J\x1B[H");
    println!("=== {} - View/Copy Mode ===", pane_name);
    println!("Press 'c', 'q', or Esc to exit | 'n' to toggle line numbers");
    println!("{}", "─".repeat(60));
    println!();

    // Print all content
    for line in &content {
        println!("{}", line);
    }

    println!();
    println!("{}", "─".repeat(60));
    println!("Press 'c', 'q', or Esc to exit | 'n' to toggle line numbers");

    // Flush to ensure output is displayed
    let _ = io::stdout().flush();
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

        // Clear expired toasts
        app.toast.clear_if_expired();

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    handle_key(&mut app, &key, &mut terminal)?;
                }
                Event::Mouse(mouse) => {
                    handle_mouse(&mut app, &mouse);
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

    Ok(())
}

fn handle_key(
    app: &mut App,
    key: &crossterm::event::KeyEvent,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    // Handle based on display mode
    match app.display_mode {
        DisplayMode::Fullscreen => {
            match key.code {
                KeyCode::Esc => {
                    app.exit_fullscreen_to_normal();
                    return Ok(());
                }
                KeyCode::Char('q') => {
                    app.should_quit = true;
                    return Ok(());
                }
                KeyCode::Char('c') => {
                    run_view_copy_mode(app, terminal)?;
                    return Ok(());
                }
                KeyCode::Char('n') => {
                    app.toggle_line_numbers();
                    return Ok(());
                }
                _ => {}
            }
            // Pass scroll keys to focused pane
            let visible_height = 20;
            let _ = handle_scrollable_content_key(app.focused_state(), key, visible_height);
            return Ok(());
        }
        DisplayMode::Normal => {
            // Normal mode key handling below
        }
    }

    // Normal mode keys
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
            return Ok(());
        }
        KeyCode::Esc => {
            app.should_quit = true;
            return Ok(());
        }
        KeyCode::Tab | KeyCode::BackTab => {
            app.toggle_focus();
            return Ok(());
        }
        KeyCode::Char('c') => {
            run_view_copy_mode(app, terminal)?;
            return Ok(());
        }
        KeyCode::Char('n') => {
            app.toggle_line_numbers();
            return Ok(());
        }
        KeyCode::F(10) | KeyCode::Enter => {
            app.enter_fullscreen();
            return Ok(());
        }
        _ => {}
    }

    if is_close_key(key) {
        app.should_quit = true;
        return Ok(());
    }

    // Pass to focused pane for scrolling
    let visible_height = 20;
    let _ = handle_scrollable_content_key(app.focused_state(), key, visible_height);

    Ok(())
}

fn handle_mouse(app: &mut App, mouse: &crossterm::event::MouseEvent) {
    // Handle split pane and scroll
    let _ = handle_split_pane_mouse(
        &mut app.split_state,
        mouse,
        Orientation::Horizontal,
        &app.split_registry,
        10,
        90,
    );

    let visible_height = 20;

    if mouse.column >= app.last_left_area.x
        && mouse.column < app.last_left_area.x + app.last_left_area.width
        && mouse.row >= app.last_left_area.y
        && mouse.row < app.last_left_area.y + app.last_left_area.height
    {
        handle_scrollable_content_mouse(
            &mut app.left_pane,
            mouse,
            app.last_left_area,
            visible_height,
        );
    }

    if mouse.column >= app.last_right_area.x
        && mouse.column < app.last_right_area.x + app.last_right_area.width
        && mouse.row >= app.last_right_area.y
        && mouse.row < app.last_right_area.y + app.last_right_area.height
    {
        handle_scrollable_content_mouse(
            &mut app.right_pane,
            mouse,
            app.last_right_area,
            visible_height,
        );
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let area = f.area();

    match app.display_mode {
        DisplayMode::Fullscreen => {
            render_fullscreen(f, area, app);
            render_toast(f, area, &app.toast);
        }
        DisplayMode::Normal => {
            render_normal(f, area, app);
            render_toast(f, area, &app.toast);
        }
    }
}

/// Render fullscreen mode - still has title bar
fn render_fullscreen(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(1),    // Content
        ])
        .split(area);

    // Title bar with hints
    let line_hint = if app.show_line_numbers {
        "n: hide lines"
    } else {
        "n: show lines"
    };

    let title_line = Line::from(vec![
        Span::styled(
            " FULLSCREEN ",
            Style::default().bg(Color::Cyan).fg(Color::Black),
        ),
        Span::raw("  "),
        Span::styled("c: view/copy mode", Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled(line_hint, Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled("Esc: back", Style::default().fg(Color::Yellow)),
    ]);
    f.render_widget(Paragraph::new(title_line), chunks[0]);

    // Content
    let state = match app.focused_pane {
        FocusedPane::Left => &app.left_pane,
        FocusedPane::Right => &app.right_pane,
    };
    let style = ScrollableContentStyle::default().with_focus_color(Color::Cyan);
    let content = ScrollableContent::new(state).style(style);
    content.render(chunks[1], f.buffer_mut());
}

/// Render normal split pane mode
fn render_normal(f: &mut Frame, area: Rect, app: &mut App) {
    app.split_registry.clear();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Status bar
            Constraint::Min(1),    // Split pane
            Constraint::Length(4), // Help
        ])
        .split(area);

    render_status_bar(f, chunks[0], app);

    // Split pane
    let split_pane = SplitPane::new()
        .orientation(Orientation::Horizontal)
        .style(SplitPaneStyle::minimal())
        .min_percent(10)
        .max_percent(90);

    let (left_area, divider_area, right_area) =
        split_pane.calculate_areas(chunks[1], app.split_state.split_percent());

    app.split_state.set_total_size(chunks[1].width);
    app.last_left_area = left_area;
    app.last_right_area = right_area;

    app.split_registry
        .register(left_area, SplitPaneAction::FirstPaneClick);
    app.split_registry
        .register(divider_area, SplitPaneAction::DividerDrag);
    app.split_registry
        .register(right_area, SplitPaneAction::SecondPaneClick);

    // Render panes
    let left_style = ScrollableContentStyle::default().with_focus_color(Color::Cyan);
    let right_style = ScrollableContentStyle::default().with_focus_color(Color::Cyan);

    let left_content = ScrollableContent::new(&app.left_pane).style(left_style);
    left_content.render(left_area, f.buffer_mut());

    let right_content = ScrollableContent::new(&app.right_pane).style(right_style);
    right_content.render(right_area, f.buffer_mut());

    // Divider
    let divider_style = if app.split_state.is_dragging() {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    for y in divider_area.y..divider_area.y + divider_area.height {
        f.buffer_mut()
            .set_string(divider_area.x, y, "│", divider_style);
    }

    render_help(f, chunks[2], app);
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let focused_text = match app.focused_pane {
        FocusedPane::Left => "Left Pane",
        FocusedPane::Right => "Right Pane",
    };

    let status = Line::from(vec![
        Span::styled(
            " Copyable Pane Demo ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled("Focus: ", Style::default().fg(Color::Gray)),
        Span::styled(focused_text, Style::default().fg(Color::Green)),
    ]);

    let block = Block::default().borders(Borders::BOTTOM);
    f.render_widget(Paragraph::new(status).block(block), area);
}

fn render_help(f: &mut Frame, area: Rect, app: &App) {
    let line_hint = if app.show_line_numbers {
        "n: hide lines"
    } else {
        "n: show lines"
    };

    let help_lines = vec![
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Switch pane  "),
            Span::styled("↑/↓ j/k", Style::default().fg(Color::Yellow)),
            Span::raw(": Scroll  "),
            Span::styled("PgUp/PgDn", Style::default().fg(Color::Yellow)),
            Span::raw(": Page  "),
            Span::styled("Home/End", Style::default().fg(Color::Yellow)),
            Span::raw(": Top/Bottom"),
        ]),
        Line::from(vec![
            Span::styled("c", Style::default().fg(Color::Yellow)),
            Span::raw(": View/Copy mode  "),
            Span::styled(line_hint, Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled("F10/Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Fullscreen  "),
            Span::styled("Esc/q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];

    f.render_widget(
        Paragraph::new(help_lines).block(Block::default().borders(Borders::TOP)),
        area,
    );
}

fn render_toast(f: &mut Frame, area: Rect, toast: &ToastState) {
    if let Some(message) = toast.get_message() {
        let toast_widget = Toast::new(message).auto_style();
        toast_widget.render_with_clear(area, f.buffer_mut());
    }
}

/// Generate raw content (without line numbers)
/// Each paragraph is a single string - terminal handles wrapping
fn generate_raw_content(title: &str) -> Vec<String> {
    let paragraphs = vec![
        format!("=== {} ===", title),
        String::new(),
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
    for i in 0..5 {
        if i > 0 {
            lines.push(format!("--- Section {} ---", i + 1));
            lines.push(String::new());
        }
        lines.extend(paragraphs.clone());
    }

    lines
}

/// Format content with or without line numbers
fn format_with_line_numbers(lines: &[String], show_numbers: bool) -> Vec<String> {
    if show_numbers {
        lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if line.is_empty() {
                    String::new()
                } else {
                    format!("{:3} | {}", i + 1, line)
                }
            })
            .collect()
    } else {
        lines.to_vec()
    }
}
