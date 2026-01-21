//! MenuBar Demo
//!
//! Interactive demo showing menu bar features:
//! - Traditional File/Edit/View/Help style menus
//! - Dropdown menus with items, separators, and shortcuts
//! - Keyboard navigation (arrows, Enter, Escape)
//! - Mouse interaction (click to open, hover to switch)
//! - Submenus with nested items
//! - Disabled items and menus
//!
//! Run with: cargo run --example menu_bar_demo

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

use ratatui_interact::components::{
    Menu, MenuBar, MenuBarAction, MenuBarItem, MenuBarState, MenuBarStyle,
    Orientation, SplitPaneAction, SplitPaneState,
    calculate_menu_bar_height, handle_menu_bar_key, handle_menu_bar_mouse,
    handle_split_pane_mouse,
};
use ratatui_interact::events::is_close_key;

use ratatui_interact::traits::{ClickRegion, ClickRegionRegistry};
use ratatui_interact::components::MenuBarClickTarget;

/// Application state
struct App {
    /// Menu bar state
    menu_state: MenuBarState,
    /// Split pane state
    split_state: SplitPaneState,
    /// Last action message
    last_action: String,
    /// Action history
    action_history: Vec<String>,
    /// Current style index
    style_index: usize,
    /// Should quit
    should_quit: bool,
    /// Stored bar area for mouse handling
    bar_area: Rect,
    /// Stored dropdown area for mouse handling
    dropdown_area: Option<Rect>,
    /// Stored click regions for mouse handling
    click_regions: Vec<ClickRegion<MenuBarClickTarget>>,
    /// Split pane click regions
    split_regions: ClickRegionRegistry<SplitPaneAction>,
    /// Content area for split pane rendering
    content_area: Rect,
}

impl App {
    fn new() -> Self {
        let mut menu_state = MenuBarState::new();
        menu_state.focused = true;
        Self {
            menu_state,
            split_state: SplitPaneState::new(60), // 60% for content, 40% for status
            last_action: "Press Alt+F or click a menu to begin".to_string(),
            action_history: Vec::new(),
            style_index: 0,
            should_quit: false,
            bar_area: Rect::default(),
            dropdown_area: None,
            click_regions: Vec::new(),
            split_regions: ClickRegionRegistry::new(),
            content_area: Rect::default(),
        }
    }

    fn cycle_style(&mut self) {
        self.style_index = (self.style_index + 1) % 3;
        let style_name = match self.style_index {
            0 => "Default (Dark)",
            1 => "Light",
            _ => "Minimal",
        };
        self.add_action(format!("Style changed to: {}", style_name));
    }

    fn current_style(&self) -> MenuBarStyle {
        match self.style_index {
            0 => MenuBarStyle::default(),
            1 => MenuBarStyle::light(),
            _ => MenuBarStyle::minimal(),
        }
    }

    fn add_action(&mut self, action: String) {
        self.last_action = action.clone();
        self.action_history.push(action);
        // Keep only last 10 actions
        if self.action_history.len() > 10 {
            self.action_history.remove(0);
        }
    }

    fn handle_action(&mut self, action: MenuBarAction) {
        match action {
            MenuBarAction::ItemSelect(id) => {
                match id.as_str() {
                    "quit" => self.should_quit = true,
                    "new" => self.add_action("Action: New File".to_string()),
                    "open" => self.add_action("Action: Open File".to_string()),
                    "save" => self.add_action("Action: Save File".to_string()),
                    "save_as" => self.add_action("Action: Save As...".to_string()),
                    "close" => self.add_action("Action: Close File".to_string()),
                    "undo" => self.add_action("Action: Undo".to_string()),
                    "redo" => self.add_action("Action: Redo".to_string()),
                    "cut" => self.add_action("Action: Cut".to_string()),
                    "copy" => self.add_action("Action: Copy".to_string()),
                    "paste" => self.add_action("Action: Paste".to_string()),
                    "select_all" => self.add_action("Action: Select All".to_string()),
                    "find" => self.add_action("Action: Find".to_string()),
                    "replace" => self.add_action("Action: Replace".to_string()),
                    "zoom_in" => self.add_action("Action: Zoom In".to_string()),
                    "zoom_out" => self.add_action("Action: Zoom Out".to_string()),
                    "zoom_reset" => self.add_action("Action: Reset Zoom".to_string()),
                    "fullscreen" => self.add_action("Action: Toggle Fullscreen".to_string()),
                    "sidebar" => self.add_action("Action: Toggle Sidebar".to_string()),
                    "terminal" => self.add_action("Action: Toggle Terminal".to_string()),
                    "about" => self.add_action("Action: About - ratatui-interact v0.3.0".to_string()),
                    "docs" => self.add_action("Action: Open Documentation".to_string()),
                    "shortcuts" => self.add_action("Action: Keyboard Shortcuts".to_string()),
                    "check_updates" => self.add_action("Action: Check for Updates".to_string()),
                    "export_pdf" => self.add_action("Action: Export as PDF".to_string()),
                    "export_html" => self.add_action("Action: Export as HTML".to_string()),
                    "export_md" => self.add_action("Action: Export as Markdown".to_string()),
                    _ => self.add_action(format!("Action: {}", id)),
                }
            }
            MenuBarAction::MenuOpen(idx) => {
                self.add_action(format!("Menu opened: {}", get_menu_name(idx)));
            }
            MenuBarAction::MenuClose => {
                self.add_action("Menu closed".to_string());
            }
            MenuBarAction::SubmenuOpen(menu_idx, item_idx) => {
                self.add_action(format!("Submenu opened in menu {} at item {}", menu_idx, item_idx));
            }
            MenuBarAction::SubmenuClose => {
                self.add_action("Submenu closed".to_string());
            }
            MenuBarAction::HighlightChange(menu_idx, item_idx) => {
                if let Some(idx) = item_idx {
                    self.add_action(format!("Highlight: menu {}, item {}", menu_idx, idx));
                }
            }
        }
    }
}

fn get_menu_name(idx: usize) -> &'static str {
    match idx {
        0 => "File",
        1 => "Edit",
        2 => "View",
        3 => "Help",
        _ => "Unknown",
    }
}

fn create_menus() -> Vec<Menu> {
    vec![
        Menu::new("File").items(vec![
            MenuBarItem::action("new", "New").shortcut("Ctrl+N"),
            MenuBarItem::action("open", "Open...").shortcut("Ctrl+O"),
            MenuBarItem::separator(),
            MenuBarItem::action("save", "Save").shortcut("Ctrl+S"),
            MenuBarItem::action("save_as", "Save As...").shortcut("Ctrl+Shift+S"),
            MenuBarItem::separator(),
            MenuBarItem::submenu(
                "Export",
                vec![
                    MenuBarItem::action("export_pdf", "Export as PDF"),
                    MenuBarItem::action("export_html", "Export as HTML"),
                    MenuBarItem::action("export_md", "Export as Markdown"),
                ],
            ),
            MenuBarItem::separator(),
            MenuBarItem::action("close", "Close").shortcut("Ctrl+W"),
            MenuBarItem::action("quit", "Quit").shortcut("Ctrl+Q"),
        ]),
        Menu::new("Edit").items(vec![
            MenuBarItem::action("undo", "Undo").shortcut("Ctrl+Z"),
            MenuBarItem::action("redo", "Redo").shortcut("Ctrl+Y"),
            MenuBarItem::separator(),
            MenuBarItem::action("cut", "Cut").shortcut("Ctrl+X"),
            MenuBarItem::action("copy", "Copy").shortcut("Ctrl+C"),
            MenuBarItem::action("paste", "Paste").shortcut("Ctrl+V").enabled(false), // Disabled example
            MenuBarItem::separator(),
            MenuBarItem::action("select_all", "Select All").shortcut("Ctrl+A"),
            MenuBarItem::separator(),
            MenuBarItem::action("find", "Find...").shortcut("Ctrl+F"),
            MenuBarItem::action("replace", "Replace...").shortcut("Ctrl+H"),
        ]),
        Menu::new("View").items(vec![
            MenuBarItem::submenu(
                "Zoom",
                vec![
                    MenuBarItem::action("zoom_in", "Zoom In").shortcut("Ctrl++"),
                    MenuBarItem::action("zoom_out", "Zoom Out").shortcut("Ctrl+-"),
                    MenuBarItem::action("zoom_reset", "Reset Zoom").shortcut("Ctrl+0"),
                ],
            ),
            MenuBarItem::separator(),
            MenuBarItem::action("fullscreen", "Toggle Fullscreen").shortcut("F11"),
            MenuBarItem::action("sidebar", "Toggle Sidebar").shortcut("Ctrl+B"),
            MenuBarItem::action("terminal", "Toggle Terminal").shortcut("Ctrl+`"),
        ]),
        Menu::new("Help").items(vec![
            MenuBarItem::action("shortcuts", "Keyboard Shortcuts").shortcut("Ctrl+K Ctrl+S"),
            MenuBarItem::action("docs", "Documentation").shortcut("F1"),
            MenuBarItem::separator(),
            MenuBarItem::action("check_updates", "Check for Updates"),
            MenuBarItem::separator(),
            MenuBarItem::action("about", "About"),
        ]),
    ]
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and menus
    let mut app = App::new();
    let menus = create_menus();

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &mut app, &menus))?;

        match event::read()? {
            Event::Key(key) => {
                if is_close_key(&key) && !app.menu_state.is_open {
                    app.should_quit = true;
                } else if key.code == KeyCode::Char('t') && !app.menu_state.is_open {
                    app.cycle_style();
                } else if let Some(action) = handle_menu_bar_key(&key, &mut app.menu_state, &menus)
                {
                    app.handle_action(action);
                }
            }
            Event::Mouse(mouse) => {
                // Handle menu bar mouse events first (higher priority when menu is open)
                if let Some(action) = handle_menu_bar_mouse(
                    &mouse,
                    &mut app.menu_state,
                    app.bar_area,
                    app.dropdown_area,
                    &app.click_regions,
                    &menus,
                ) {
                    app.handle_action(action);
                } else {
                    // Handle split pane mouse events
                    handle_split_pane_mouse(
                        &mut app.split_state,
                        &mouse,
                        Orientation::Horizontal,
                        &app.split_regions,
                        20,
                        80,
                    );
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

fn ui(f: &mut Frame, app: &mut App, menus: &[Menu]) {
    let area = f.area();

    // Create main layout
    let menu_height = calculate_menu_bar_height();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(menu_height), // Menu bar
            Constraint::Min(1),              // Content area (split pane)
            Constraint::Length(5),           // Info panel (reduced height)
        ])
        .split(area);

    // Store content area for split pane calculations
    app.content_area = chunks[1];

    // Clear split regions for this frame
    app.split_regions.clear();

    // Calculate split areas manually
    let split_percent = app.split_state.split_percent();
    let content_area = chunks[1];
    let available_width = content_area.width.saturating_sub(1); // 1 for divider
    let left_width = ((available_width as u32) * (split_percent as u32) / 100) as u16;
    let left_width = left_width.clamp(5, available_width.saturating_sub(5));
    let right_width = available_width.saturating_sub(left_width);

    let left_area = Rect::new(content_area.x, content_area.y, left_width, content_area.height);
    let divider_area = Rect::new(content_area.x + left_width, content_area.y, 1, content_area.height);
    let right_area = Rect::new(
        content_area.x + left_width + 1,
        content_area.y,
        right_width,
        content_area.height,
    );

    // Update split state total size for mouse handling
    app.split_state.set_total_size(content_area.width);

    // Register click regions for split pane
    app.split_regions.register(left_area, SplitPaneAction::FirstPaneClick);
    app.split_regions.register(divider_area, SplitPaneAction::DividerDrag);
    app.split_regions.register(right_area, SplitPaneAction::SecondPaneClick);

    // Render content into left pane (this is where menus will overlay)
    render_content_area(f, app, left_area);

    // Render divider
    let divider_style = Style::default().fg(Color::DarkGray);
    for y in divider_area.y..divider_area.y + divider_area.height {
        f.buffer_mut().set_string(divider_area.x, y, "│", divider_style);
    }

    // Render status panel in right pane
    render_status_panel(f, app, right_area);

    // Render menu bar (on top of content, using full area for dropdown positioning)
    let menu_bar = MenuBar::new(menus, &app.menu_state).style(app.current_style());

    let (bar_area, dropdown_area, click_regions) = menu_bar.render_stateful(f, area);
    app.bar_area = bar_area;
    app.dropdown_area = dropdown_area;
    app.click_regions = click_regions;

    // Render info panel at bottom
    render_info_panel(f, app, chunks[2]);
}

fn render_content_area(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Content Area ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Show action history
    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(
                "Last Action: ",
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                &app.last_action,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::styled("Action History:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ];

    for (i, action) in app.action_history.iter().rev().take(6).enumerate() {
        let color = if i == 0 { Color::White } else { Color::DarkGray };
        lines.push(Line::from(vec![
            Span::styled(format!("  {}. ", app.action_history.len() - i), Style::default().fg(Color::DarkGray)),
            Span::styled(action, Style::default().fg(color)),
        ]));
    }

    if app.action_history.is_empty() {
        lines.push(Line::styled("  (no actions yet)", Style::default().fg(Color::DarkGray)));
    }

    let content = Paragraph::new(lines);
    f.render_widget(content, inner);
}

fn render_info_panel(f: &mut Frame, app: &App, area: Rect) {
    let style_name = match app.style_index {
        0 => "Default (Dark)",
        1 => "Light",
        _ => "Minimal",
    };

    let menu_status = if app.menu_state.is_open {
        format!("Open ({})", get_menu_name(app.menu_state.active_menu))
    } else {
        "Closed".to_string()
    };

    let info_lines = vec![
        Line::from(vec![
            Span::styled("Menu Status: ", Style::default().fg(Color::Gray)),
            Span::styled(&menu_status, Style::default().fg(Color::Yellow)),
            Span::raw("  |  "),
            Span::styled("Style: ", Style::default().fg(Color::Gray)),
            Span::styled(style_name, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Arrows", Style::default().fg(Color::Yellow)),
            Span::raw(": Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Select  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Close menu  "),
            Span::styled("t", Style::default().fg(Color::Yellow)),
            Span::raw(": Cycle themes"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Click menu labels to open, hover to switch between open menus",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Press Esc (when menu closed) or select File > Quit to exit",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];

    let info = Paragraph::new(info_lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(info, area);
}

/// Render the status panel (right side)
fn render_status_panel(f: &mut Frame, app: &App, area: Rect) {
    let style_name = match app.style_index {
        0 => "Default (Dark)",
        1 => "Light",
        _ => "Minimal",
    };

    let menu_status = if app.menu_state.is_open {
        format!("Open ({})", get_menu_name(app.menu_state.active_menu))
    } else {
        "Closed".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Status ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(vec![
            Span::styled("Menu Status: ", Style::default().fg(Color::Gray)),
            Span::styled(&menu_status, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Style: ", Style::default().fg(Color::Gray)),
            Span::styled(style_name, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Active Menu: ", Style::default().fg(Color::Gray)),
            Span::styled(
                get_menu_name(app.menu_state.active_menu),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Highlighted: ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.menu_state
                    .highlighted_item
                    .map(|i| format!("Item {}", i))
                    .unwrap_or_else(|| "None".to_string()),
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Split: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}%", app.split_state.split_percent()),
                Style::default().fg(Color::Blue),
            ),
        ]),
    ];

    let content = Paragraph::new(lines);
    f.render_widget(content, inner);
}
