//! Breadcrumb Demo
//!
//! Interactive demo showing the Breadcrumb navigation component:
//! - Multiple style presets (default, slash, chevron, arrow)
//! - Keyboard navigation (arrows, Home/End, Enter)
//! - Mouse click support
//! - Ellipsis collapsing for long paths
//! - Dynamic path manipulation (push/pop items)
//!
//! Run with: cargo run --example breadcrumb_demo

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
    components::{
        Breadcrumb, BreadcrumbAction, BreadcrumbItem, BreadcrumbState, BreadcrumbStyle,
        breadcrumb_hovered_index, handle_breadcrumb_key, handle_breadcrumb_mouse,
    },
    events::is_close_key,
    traits::ClickRegion,
};

/// Which breadcrumb demo is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedDemo {
    Default,
    Slash,
    Chevron,
    Arrow,
    Dynamic,
}

impl FocusedDemo {
    fn next(&self) -> Self {
        match self {
            FocusedDemo::Default => FocusedDemo::Slash,
            FocusedDemo::Slash => FocusedDemo::Chevron,
            FocusedDemo::Chevron => FocusedDemo::Arrow,
            FocusedDemo::Arrow => FocusedDemo::Dynamic,
            FocusedDemo::Dynamic => FocusedDemo::Default,
        }
    }

    fn prev(&self) -> Self {
        match self {
            FocusedDemo::Default => FocusedDemo::Dynamic,
            FocusedDemo::Slash => FocusedDemo::Default,
            FocusedDemo::Chevron => FocusedDemo::Slash,
            FocusedDemo::Arrow => FocusedDemo::Chevron,
            FocusedDemo::Dynamic => FocusedDemo::Arrow,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            FocusedDemo::Default => "Default Style ( > )",
            FocusedDemo::Slash => "Slash Style ( / )",
            FocusedDemo::Chevron => "Chevron Style ( › )",
            FocusedDemo::Arrow => "Arrow Style ( → )",
            FocusedDemo::Dynamic => "Dynamic Path",
        }
    }
}

/// Application state
struct App {
    /// Breadcrumb states for each demo
    default_state: BreadcrumbState,
    slash_state: BreadcrumbState,
    chevron_state: BreadcrumbState,
    arrow_state: BreadcrumbState,
    dynamic_state: BreadcrumbState,

    /// Which demo is focused
    focused: FocusedDemo,

    /// Status message
    message: String,

    /// Should quit
    should_quit: bool,

    /// Stored areas for mouse handling
    demo_areas: [Rect; 5],

    /// Click regions from last render
    click_regions: Vec<ClickRegion<BreadcrumbAction>>,

    /// Hovered item index
    hovered_index: Option<usize>,
}

impl App {
    fn new() -> Self {
        // Create sample breadcrumb items
        let short_path = vec![
            BreadcrumbItem::new("home", "Home").icon("🏠"),
            BreadcrumbItem::new("docs", "Documents"),
            BreadcrumbItem::new("work", "Work"),
        ];

        let long_path = vec![
            BreadcrumbItem::new("home", "Home").icon("🏠"),
            BreadcrumbItem::new("users", "Users"),
            BreadcrumbItem::new("john", "John"),
            BreadcrumbItem::new("projects", "Projects"),
            BreadcrumbItem::new("webapp", "WebApp"),
            BreadcrumbItem::new("src", "src"),
            BreadcrumbItem::new("components", "Components"),
        ];

        let mut default_state = BreadcrumbState::new(long_path.clone());
        default_state.focused = true;

        Self {
            default_state,
            slash_state: BreadcrumbState::new(short_path.clone()),
            chevron_state: BreadcrumbState::new(long_path.clone()),
            arrow_state: BreadcrumbState::new(long_path.clone()),
            dynamic_state: BreadcrumbState::new(vec![
                BreadcrumbItem::new("root", "Root").icon("📁"),
            ]),
            focused: FocusedDemo::Default,
            message: "Tab to switch demos. Arrow keys to navigate. Enter to select. +/- to modify dynamic path.".to_string(),
            should_quit: false,
            demo_areas: [Rect::default(); 5],
            click_regions: Vec::new(),
            hovered_index: None,
        }
    }

    fn focus_next(&mut self) {
        self.update_focus_state(false);
        self.focused = self.focused.next();
        self.update_focus_state(true);
    }

    fn focus_prev(&mut self) {
        self.update_focus_state(false);
        self.focused = self.focused.prev();
        self.update_focus_state(true);
    }

    fn update_focus_state(&mut self, focused: bool) {
        match self.focused {
            FocusedDemo::Default => self.default_state.focused = focused,
            FocusedDemo::Slash => self.slash_state.focused = focused,
            FocusedDemo::Chevron => self.chevron_state.focused = focused,
            FocusedDemo::Arrow => self.arrow_state.focused = focused,
            FocusedDemo::Dynamic => self.dynamic_state.focused = focused,
        }
    }

    fn get_focused_state(&mut self) -> &mut BreadcrumbState {
        match self.focused {
            FocusedDemo::Default => &mut self.default_state,
            FocusedDemo::Slash => &mut self.slash_state,
            FocusedDemo::Chevron => &mut self.chevron_state,
            FocusedDemo::Arrow => &mut self.arrow_state,
            FocusedDemo::Dynamic => &mut self.dynamic_state,
        }
    }

    fn update_message(&mut self, action: &BreadcrumbAction) {
        match action {
            BreadcrumbAction::Navigate(id) => {
                self.message = format!("Navigated to: {} (in {} demo)", id, self.focused.label());
            }
            BreadcrumbAction::ExpandEllipsis => {
                let state = self.get_focused_state();
                self.message = if state.expanded {
                    "Expanded ellipsis - showing all items".to_string()
                } else {
                    "Collapsed - hiding middle items".to_string()
                };
            }
        }
    }

    fn push_dynamic_item(&mut self) {
        let count = self.dynamic_state.len();
        let item = BreadcrumbItem::new(format!("item{}", count), format!("Folder {}", count));
        self.dynamic_state.push(item);
        self.message = format!("Added item {} to dynamic path", count);
    }

    fn pop_dynamic_item(&mut self) {
        if self.dynamic_state.len() > 1 {
            if let Some(item) = self.dynamic_state.pop() {
                self.message = format!("Removed '{}' from dynamic path", item.label);
            }
        } else {
            self.message = "Cannot remove the last item".to_string();
        }
    }

    fn get_area_index(&self, col: u16, row: u16) -> Option<usize> {
        for (i, area) in self.demo_areas.iter().enumerate() {
            if col >= area.x
                && col < area.x + area.width
                && row >= area.y
                && row < area.y + area.height
            {
                return Some(i);
            }
        }
        None
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
                    } else if key.code == KeyCode::Tab {
                        app.focus_next();
                    } else if key.code == KeyCode::BackTab {
                        app.focus_prev();
                    } else if key.code == KeyCode::Char('+') || key.code == KeyCode::Char('=') {
                        if matches!(app.focused, FocusedDemo::Dynamic) {
                            app.push_dynamic_item();
                        }
                    } else if key.code == KeyCode::Char('-') {
                        if matches!(app.focused, FocusedDemo::Dynamic) {
                            app.pop_dynamic_item();
                        }
                    } else {
                        let state = app.get_focused_state();
                        if let Some(action) = handle_breadcrumb_key(&key, state) {
                            app.update_message(&action);
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    // Update hover state
                    if let crossterm::event::MouseEventKind::Moved = mouse.kind {
                        // Get the state first, then compute hover index
                        let hovered = match app.focused {
                            FocusedDemo::Default => breadcrumb_hovered_index(
                                mouse.column,
                                mouse.row,
                                &app.click_regions,
                                &app.default_state,
                            ),
                            FocusedDemo::Slash => breadcrumb_hovered_index(
                                mouse.column,
                                mouse.row,
                                &app.click_regions,
                                &app.slash_state,
                            ),
                            FocusedDemo::Chevron => breadcrumb_hovered_index(
                                mouse.column,
                                mouse.row,
                                &app.click_regions,
                                &app.chevron_state,
                            ),
                            FocusedDemo::Arrow => breadcrumb_hovered_index(
                                mouse.column,
                                mouse.row,
                                &app.click_regions,
                                &app.arrow_state,
                            ),
                            FocusedDemo::Dynamic => breadcrumb_hovered_index(
                                mouse.column,
                                mouse.row,
                                &app.click_regions,
                                &app.dynamic_state,
                            ),
                        };
                        app.hovered_index = hovered;
                    }

                    // Handle clicks
                    if let crossterm::event::MouseEventKind::Down(
                        crossterm::event::MouseButton::Left,
                    ) = mouse.kind
                    {
                        // Check if clicking on a demo area to focus it
                        if let Some(idx) = app.get_area_index(mouse.column, mouse.row) {
                            let new_focus = match idx {
                                0 => FocusedDemo::Default,
                                1 => FocusedDemo::Slash,
                                2 => FocusedDemo::Chevron,
                                3 => FocusedDemo::Arrow,
                                4 => FocusedDemo::Dynamic,
                                _ => app.focused,
                            };

                            if new_focus != app.focused {
                                app.update_focus_state(false);
                                app.focused = new_focus;
                                app.update_focus_state(true);
                            }
                        }

                        // Handle breadcrumb click
                        let regions = std::mem::take(&mut app.click_regions);
                        let action = match app.focused {
                            FocusedDemo::Default => {
                                handle_breadcrumb_mouse(&mouse, &mut app.default_state, &regions)
                            }
                            FocusedDemo::Slash => {
                                handle_breadcrumb_mouse(&mouse, &mut app.slash_state, &regions)
                            }
                            FocusedDemo::Chevron => {
                                handle_breadcrumb_mouse(&mouse, &mut app.chevron_state, &regions)
                            }
                            FocusedDemo::Arrow => {
                                handle_breadcrumb_mouse(&mouse, &mut app.arrow_state, &regions)
                            }
                            FocusedDemo::Dynamic => {
                                handle_breadcrumb_mouse(&mouse, &mut app.dynamic_state, &regions)
                            }
                        };
                        if let Some(action) = action {
                            app.update_message(&action);
                        }
                        app.click_regions = regions;
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
    let area = f.area();

    // Background
    let background = Block::default()
        .borders(Borders::ALL)
        .title(" Breadcrumb Demo ")
        .border_style(Style::default().fg(Color::Blue));
    f.render_widget(background, area);

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Length(3), // Default breadcrumb
            Constraint::Length(3), // Slash breadcrumb
            Constraint::Length(3), // Chevron breadcrumb
            Constraint::Length(3), // Arrow breadcrumb
            Constraint::Length(3), // Dynamic breadcrumb
            Constraint::Length(2), // Spacer
            Constraint::Length(2), // Message
            Constraint::Min(0),    // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        "Breadcrumb Component Demo",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));
    f.render_widget(title, chunks[0]);

    app.click_regions.clear();

    // Render each breadcrumb demo
    let demos = [
        (
            FocusedDemo::Default,
            &app.default_state,
            BreadcrumbStyle::default(),
        ),
        (
            FocusedDemo::Slash,
            &app.slash_state,
            BreadcrumbStyle::slash(),
        ),
        (
            FocusedDemo::Chevron,
            &app.chevron_state,
            BreadcrumbStyle::chevron(),
        ),
        (
            FocusedDemo::Arrow,
            &app.arrow_state,
            BreadcrumbStyle::arrow(),
        ),
        (
            FocusedDemo::Dynamic,
            &app.dynamic_state,
            BreadcrumbStyle::minimal(),
        ),
    ];

    for (i, (demo_type, state, style)) in demos.iter().enumerate() {
        let chunk_idx = i + 1;
        let is_focused = app.focused == *demo_type;

        // Create bordered area
        let border_color = if is_focused {
            Color::Yellow
        } else {
            Color::DarkGray
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(format!(" {} ", demo_type.label()));

        let inner = block.inner(chunks[chunk_idx]);
        f.render_widget(block, chunks[chunk_idx]);

        // Store area for mouse hit testing
        app.demo_areas[i] = chunks[chunk_idx];

        // Render breadcrumb
        let hovered = if is_focused { app.hovered_index } else { None };
        let breadcrumb = Breadcrumb::new(state).style(style.clone()).hovered(hovered);

        let regions = breadcrumb.render_stateful(inner, f.buffer_mut());

        // Store click regions for focused demo
        if is_focused {
            app.click_regions = regions;
        }
    }

    // Message
    let message = Paragraph::new(Line::from(vec![Span::styled(
        &app.message,
        Style::default().fg(Color::Yellow),
    )]));
    f.render_widget(message, chunks[7]);

    // Help
    let help_lines = vec![
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(": Next demo  "),
            Span::styled("←/→", Style::default().fg(Color::Cyan)),
            Span::raw(": Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(": Select  "),
            Span::styled("e", Style::default().fg(Color::Cyan)),
            Span::raw(": Expand/Collapse  "),
            Span::styled("q/Esc", Style::default().fg(Color::Cyan)),
            Span::raw(": Quit"),
        ]),
        Line::from(vec![
            Span::styled("Home/End", Style::default().fg(Color::Cyan)),
            Span::raw(": First/Last  "),
            Span::styled("+/-", Style::default().fg(Color::Cyan)),
            Span::raw(": Add/Remove item (Dynamic demo)  "),
            Span::styled("Click", Style::default().fg(Color::Cyan)),
            Span::raw(": Navigate/Expand"),
        ]),
    ];
    let help = Paragraph::new(help_lines);
    f.render_widget(help, chunks[8]);
}
