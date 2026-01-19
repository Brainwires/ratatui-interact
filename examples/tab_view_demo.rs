//! Tab View Demo
//!
//! Interactive demo showing tab view features:
//! - Tabs on all four sides (top, bottom, left, right)
//! - Keyboard navigation (arrows, numbers, Home/End)
//! - Mouse click support
//! - Content switching based on selected tab
//! - Toggle tab position with 'p' key
//!
//! Run with: cargo run --example tab_view_demo

use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use ratatui_interact::{
    components::{
        Tab, TabPosition, TabView, TabViewAction, TabViewState, TabViewStyle,
        handle_tab_view_key, handle_tab_view_mouse,
    },
    events::{is_close_key, is_left_click},
    traits::ClickRegionRegistry,
};

/// Application state
struct App {
    /// Tab view state
    tab_state: TabViewState,
    /// Click regions
    click_regions: ClickRegionRegistry<TabViewAction>,
    /// Current tab position
    position: TabPosition,
    /// Should quit
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            tab_state: TabViewState::new(5),
            click_regions: ClickRegionRegistry::new(),
            position: TabPosition::Top,
            should_quit: false,
        }
    }

    fn cycle_position(&mut self) {
        self.position = match self.position {
            TabPosition::Top => TabPosition::Right,
            TabPosition::Right => TabPosition::Bottom,
            TabPosition::Bottom => TabPosition::Left,
            TabPosition::Left => TabPosition::Top,
        };
    }

    fn handle_click(&mut self, col: u16, row: u16) {
        let mouse = crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: col,
            row,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        handle_tab_view_mouse(&mut self.tab_state, &self.click_regions, &mouse);
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
    app.tab_state.focused = true;

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if is_close_key(&key) || key.code == KeyCode::Char('q') {
                app.should_quit = true;
            } else if key.code == KeyCode::Char('p') {
                app.cycle_position();
            } else {
                // Let tab view handle the key
                handle_tab_view_key(&mut app.tab_state, &key, app.position);
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
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Tab view
            Constraint::Length(4), // Help
        ])
        .split(area);

    // Title
    let position_name = match app.position {
        TabPosition::Top => "Top",
        TabPosition::Bottom => "Bottom",
        TabPosition::Left => "Left",
        TabPosition::Right => "Right",
    };
    let title = Paragraph::new(format!("Tab View Demo - Position: {}", position_name))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Create tabs
    let tabs = vec![
        Tab::new("General").icon("\u{2699}"),      // Gear icon
        Tab::new("Network").icon("\u{1F310}").badge("3"), // Globe icon
        Tab::new("Security").icon("\u{1F512}"),    // Lock icon
        Tab::new("Display").icon("\u{1F5B5}"),     // Monitor icon
        Tab::new("About").icon("\u{2139}"),        // Info icon
    ];

    // Create style based on position
    let style = match app.position {
        TabPosition::Top => TabViewStyle::top(),
        TabPosition::Bottom => TabViewStyle::bottom(),
        TabPosition::Left => TabViewStyle::left().tab_width(18),
        TabPosition::Right => TabViewStyle::right().tab_width(18),
    };

    // Create tab view with content renderer
    let tab_view = TabView::new(&tabs, &app.tab_state)
        .style(style)
        .content(|idx, area, buf| {
            render_tab_content(idx, area, buf);
        });

    // Render tab view and register click regions
    tab_view.render_with_registry(chunks[1], f.buffer_mut(), &mut app.click_regions);

    // Help text
    let focus_text = if app.tab_state.tab_bar_focused {
        "Tab bar focused"
    } else {
        "Content focused"
    };

    let help_lines = vec![
        Line::from(vec![
            Span::styled("Focus: ", Style::default().fg(Color::Gray)),
            Span::styled(focus_text, Style::default().fg(Color::Green)),
            Span::raw(" | "),
            Span::styled("Selected: ", Style::default().fg(Color::Gray)),
            Span::styled(
                tabs.get(app.tab_state.selected_index).map(|t| t.label).unwrap_or("?"),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("\u{2190}\u{2192}/\u{2191}\u{2193}", Style::default().fg(Color::Yellow)),
            Span::raw(": Navigate  "),
            Span::styled("1-5", Style::default().fg(Color::Yellow)),
            Span::raw(": Select tab  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Focus content  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Focus tabs"),
        ]),
        Line::from(vec![
            Span::styled("p", Style::default().fg(Color::Yellow)),
            Span::raw(": Cycle position  "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit  "),
            Span::styled("Mouse", Style::default().fg(Color::Yellow)),
            Span::raw(": Click tabs"),
        ]),
    ];
    let help = Paragraph::new(help_lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn render_tab_content(idx: usize, area: Rect, buf: &mut ratatui::buffer::Buffer) {
    match idx {
        0 => render_general_tab(area, buf),
        1 => render_network_tab(area, buf),
        2 => render_security_tab(area, buf),
        3 => render_display_tab(area, buf),
        4 => render_about_tab(area, buf),
        _ => {
            let text = Paragraph::new("Unknown tab")
                .style(Style::default().fg(Color::Red));
            text.render(area, buf);
        }
    }
}

fn render_general_tab(area: Rect, buf: &mut ratatui::buffer::Buffer) {
    let text = vec![
        Line::from(Span::styled(
            "General Settings",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Language: ", Style::default().fg(Color::Gray)),
            Span::styled("English", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Theme: ", Style::default().fg(Color::Gray)),
            Span::styled("Dark", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Auto-save: ", Style::default().fg(Color::Gray)),
            Span::styled("Enabled", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("Updates: ", Style::default().fg(Color::Gray)),
            Span::styled("Automatic", Style::default().fg(Color::White)),
        ]),
    ];
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    paragraph.render(area, buf);
}

fn render_network_tab(area: Rect, buf: &mut ratatui::buffer::Buffer) {
    let text = vec![
        Line::from(Span::styled(
            "Network Configuration",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Gray)),
            Span::styled("Connected", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("IP Address: ", Style::default().fg(Color::Gray)),
            Span::styled("192.168.1.100", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Gateway: ", Style::default().fg(Color::Gray)),
            Span::styled("192.168.1.1", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("DNS: ", Style::default().fg(Color::Gray)),
            Span::styled("8.8.8.8, 8.8.4.4", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "3 pending updates available",
            Style::default().fg(Color::Yellow),
        )),
    ];
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    paragraph.render(area, buf);
}

fn render_security_tab(area: Rect, buf: &mut ratatui::buffer::Buffer) {
    let text = vec![
        Line::from(Span::styled(
            "Security Options",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Firewall: ", Style::default().fg(Color::Gray)),
            Span::styled("Active", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("Last scan: ", Style::default().fg(Color::Gray)),
            Span::styled("2 hours ago", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Threats found: ", Style::default().fg(Color::Gray)),
            Span::styled("0", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("2FA: ", Style::default().fg(Color::Gray)),
            Span::styled("Enabled", Style::default().fg(Color::Green)),
        ]),
    ];
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    paragraph.render(area, buf);
}

fn render_display_tab(area: Rect, buf: &mut ratatui::buffer::Buffer) {
    let text = vec![
        Line::from(Span::styled(
            "Display Settings",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Resolution: ", Style::default().fg(Color::Gray)),
            Span::styled("1920x1080", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Refresh Rate: ", Style::default().fg(Color::Gray)),
            Span::styled("60Hz", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Brightness: ", Style::default().fg(Color::Gray)),
            Span::styled("75%", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Night Mode: ", Style::default().fg(Color::Gray)),
            Span::styled("Off", Style::default().fg(Color::Gray)),
        ]),
    ];
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    paragraph.render(area, buf);
}

fn render_about_tab(area: Rect, buf: &mut ratatui::buffer::Buffer) {
    let text = vec![
        Line::from(Span::styled(
            "About",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Application: ", Style::default().fg(Color::Gray)),
            Span::styled("TabView Demo", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Version: ", Style::default().fg(Color::Gray)),
            Span::styled("1.0.0", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Library: ", Style::default().fg(Color::Gray)),
            Span::styled("ratatui-interact", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "A demonstration of the TabView component",
            Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "supporting tabs on all four sides.",
            Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
        )),
    ];
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    paragraph.render(area, buf);
}
