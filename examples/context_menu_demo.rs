//! Context Menu Demo
//!
//! Interactive demo showing the ContextMenu component:
//! - Right-click to open context menu
//! - Keyboard navigation (Up/Down, Enter, Esc)
//! - Mouse click support
//! - Actions with icons and shortcuts
//! - Disabled items
//! - Submenus
//!
//! Run with: cargo run --example context_menu_demo

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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use ratatui_interact::{
    components::{
        ContextMenu, ContextMenuAction, ContextMenuItem, ContextMenuState, ContextMenuStyle,
        handle_context_menu_key, handle_context_menu_mouse, is_context_menu_trigger,
    },
    events::is_close_key,
    traits::ClickRegion,
};

/// Sample file item for the demo
#[derive(Debug, Clone)]
struct FileItem {
    name: String,
    is_dir: bool,
}

impl FileItem {
    fn new(name: &str, is_dir: bool) -> Self {
        Self {
            name: name.to_string(),
            is_dir,
        }
    }
}

/// Application state
struct App {
    /// File list items
    files: Vec<FileItem>,
    /// List selection state
    list_state: ListState,
    /// Context menu state
    context_menu_state: ContextMenuState,
    /// Context menu items
    context_menu_items: Vec<ContextMenuItem>,
    /// Status message
    message: String,
    /// Should quit
    should_quit: bool,
    /// List area for click detection
    list_area: Rect,
    /// Context menu area (set during render)
    menu_area: Rect,
    /// Context menu click regions
    menu_regions: Vec<ClickRegion<ContextMenuAction>>,
}

impl App {
    fn new() -> Self {
        let files = vec![
            FileItem::new("Documents", true),
            FileItem::new("Downloads", true),
            FileItem::new("Pictures", true),
            FileItem::new("readme.txt", false),
            FileItem::new("config.json", false),
            FileItem::new("notes.md", false),
            FileItem::new("report.pdf", false),
        ];

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            files,
            list_state,
            context_menu_state: ContextMenuState::new(),
            context_menu_items: Vec::new(),
            message: "Right-click on an item to open context menu. Use arrow keys to navigate."
                .to_string(),
            should_quit: false,
            list_area: Rect::default(),
            menu_area: Rect::default(),
            menu_regions: Vec::new(),
        }
    }

    /// Build context menu items based on selected item
    fn build_context_menu(&mut self) {
        let selected_idx = self.list_state.selected().unwrap_or(0);
        let selected_file = &self.files[selected_idx];

        let mut items = vec![
            ContextMenuItem::action("open", "Open").icon("📂").shortcut("Enter"),
        ];

        if selected_file.is_dir {
            items.push(
                ContextMenuItem::action("open_new_tab", "Open in New Tab")
                    .icon("📑")
                    .shortcut("Ctrl+T"),
            );
        } else {
            items.push(
                ContextMenuItem::action("edit", "Edit")
                    .icon("✏️")
                    .shortcut("E"),
            );
        }

        items.push(ContextMenuItem::separator());

        items.push(ContextMenuItem::action("copy", "Copy").icon("📋").shortcut("Ctrl+C"));
        items.push(ContextMenuItem::action("cut", "Cut").icon("✂️").shortcut("Ctrl+X"));
        items.push(
            ContextMenuItem::action("paste", "Paste")
                .icon("📄")
                .shortcut("Ctrl+V")
                .enabled(false), // Disabled - nothing in clipboard
        );

        items.push(ContextMenuItem::separator());

        items.push(ContextMenuItem::action("rename", "Rename").icon("📝").shortcut("F2"));
        items.push(
            ContextMenuItem::action("delete", "Delete")
                .icon("🗑️")
                .shortcut("Del"),
        );

        items.push(ContextMenuItem::separator());

        // Submenu example
        let submenu_items = vec![
            ContextMenuItem::action("create_file", "New File").icon("📄"),
            ContextMenuItem::action("create_folder", "New Folder").icon("📁"),
            ContextMenuItem::action("create_link", "Symbolic Link").icon("🔗"),
        ];
        items.push(ContextMenuItem::submenu("Create New", submenu_items).icon("➕"));

        items.push(ContextMenuItem::separator());

        items.push(
            ContextMenuItem::action("properties", "Properties")
                .icon("ℹ️")
                .shortcut("Alt+Enter"),
        );

        self.context_menu_items = items;
    }

    fn handle_action(&mut self, action_id: &str) {
        let selected_idx = self.list_state.selected().unwrap_or(0);
        let file_name = &self.files[selected_idx].name;

        self.message = match action_id {
            "open" => format!("Opening '{}'...", file_name),
            "open_new_tab" => format!("Opening '{}' in new tab...", file_name),
            "edit" => format!("Editing '{}'...", file_name),
            "copy" => format!("Copied '{}' to clipboard", file_name),
            "cut" => format!("Cut '{}' to clipboard", file_name),
            "paste" => "Pasting from clipboard...".to_string(),
            "rename" => format!("Renaming '{}'...", file_name),
            "delete" => format!("Deleting '{}'...", file_name),
            "create_file" => "Creating new file...".to_string(),
            "create_folder" => "Creating new folder...".to_string(),
            "create_link" => "Creating symbolic link...".to_string(),
            "properties" => format!("Showing properties for '{}'...", file_name),
            _ => format!("Unknown action: {}", action_id),
        };
    }

    fn select_next(&mut self) {
        let len = self.files.len();
        if len == 0 {
            return;
        }
        let i = self.list_state.selected().map_or(0, |i| {
            if i >= len - 1 {
                0
            } else {
                i + 1
            }
        });
        self.list_state.select(Some(i));
    }

    fn select_prev(&mut self) {
        let len = self.files.len();
        if len == 0 {
            return;
        }
        let i = self.list_state.selected().map_or(0, |i| {
            if i == 0 {
                len - 1
            } else {
                i - 1
            }
        });
        self.list_state.select(Some(i));
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
                    if app.context_menu_state.is_open {
                        // Handle context menu keys
                        if let Some(action) = handle_context_menu_key(
                            &key,
                            &mut app.context_menu_state,
                            &app.context_menu_items,
                        ) {
                            match action {
                                ContextMenuAction::Select(id) => {
                                    app.handle_action(&id);
                                }
                                ContextMenuAction::Close => {
                                    app.message = "Context menu closed.".to_string();
                                }
                                _ => {}
                            }
                        }
                    } else {
                        // Handle normal keys
                        if is_close_key(&key) || key.code == KeyCode::Char('q') {
                            app.should_quit = true;
                        } else if key.code == KeyCode::Down || key.code == KeyCode::Char('j') {
                            app.select_next();
                        } else if key.code == KeyCode::Up || key.code == KeyCode::Char('k') {
                            app.select_prev();
                        } else if key.code == KeyCode::Enter {
                            let selected_idx = app.list_state.selected().unwrap_or(0);
                            let file_name = &app.files[selected_idx].name;
                            app.message = format!("Opened '{}'", file_name);
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    if app.context_menu_state.is_open {
                        // Handle context menu mouse
                        if let Some(action) = handle_context_menu_mouse(
                            &mouse,
                            &mut app.context_menu_state,
                            app.menu_area,
                            &app.menu_regions,
                        ) {
                            match action {
                                ContextMenuAction::Select(id) => {
                                    app.handle_action(&id);
                                }
                                ContextMenuAction::Close => {
                                    app.message = "Context menu closed.".to_string();
                                }
                                _ => {}
                            }
                        }
                    } else if is_context_menu_trigger(&mouse) {
                        // Right-click - open context menu
                        let col = mouse.column;
                        let row = mouse.row;

                        // Check if click is within list area
                        if col >= app.list_area.x
                            && col < app.list_area.x + app.list_area.width
                            && row >= app.list_area.y
                            && row < app.list_area.y + app.list_area.height
                        {
                            // Calculate which item was clicked
                            let clicked_idx = (row - app.list_area.y - 1) as usize; // -1 for border
                            if clicked_idx < app.files.len() {
                                app.list_state.select(Some(clicked_idx));
                                app.build_context_menu();
                                app.context_menu_state.open_at(col, row);
                                app.message = format!(
                                    "Context menu opened for '{}'",
                                    app.files[clicked_idx].name
                                );
                            }
                        }
                    } else if let crossterm::event::MouseEventKind::Down(
                        crossterm::event::MouseButton::Left,
                    ) = mouse.kind
                    {
                        // Left click - select item
                        let col = mouse.column;
                        let row = mouse.row;

                        if col >= app.list_area.x
                            && col < app.list_area.x + app.list_area.width
                            && row >= app.list_area.y
                            && row < app.list_area.y + app.list_area.height
                        {
                            let clicked_idx = (row - app.list_area.y - 1) as usize;
                            if clicked_idx < app.files.len() {
                                app.list_state.select(Some(clicked_idx));
                            }
                        }
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
        .title(" Context Menu Demo ")
        .border_style(Style::default().fg(Color::Blue));
    f.render_widget(background, area);

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // File list
            Constraint::Length(2), // Message
            Constraint::Length(3), // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        "Context Menu Component Demo",
        Style::default().fg(Color::Cyan),
    )]));
    f.render_widget(title, chunks[0]);

    // File list
    let list_items: Vec<ListItem> = app
        .files
        .iter()
        .map(|file| {
            let icon = if file.is_dir { "📁 " } else { "📄 " };
            let style = if file.is_dir {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(vec![
                Span::raw(icon),
                Span::styled(&file.name, style),
            ]))
        })
        .collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Files ")
                .border_style(Style::default().fg(Color::Gray)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    app.list_area = chunks[1];
    f.render_stateful_widget(list, chunks[1], &mut app.list_state);

    // Message
    let message = Paragraph::new(Line::from(vec![Span::styled(
        &app.message,
        Style::default().fg(Color::Yellow),
    )]));
    f.render_widget(message, chunks[2]);

    // Help
    let help_lines = vec![
        Line::from(vec![
            Span::styled("Right-click", Style::default().fg(Color::Cyan)),
            Span::raw(": Open context menu  "),
            Span::styled("Up/Down", Style::default().fg(Color::Cyan)),
            Span::raw(": Navigate list  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(": Open item  "),
            Span::styled("q/Esc", Style::default().fg(Color::Cyan)),
            Span::raw(": Quit"),
        ]),
        Line::from(vec![
            Span::styled("In menu:", Style::default().fg(Color::Magenta)),
            Span::raw(" "),
            Span::styled("Up/Down", Style::default().fg(Color::Cyan)),
            Span::raw(": Navigate  "),
            Span::styled("Enter/Space", Style::default().fg(Color::Cyan)),
            Span::raw(": Select  "),
            Span::styled("Right", Style::default().fg(Color::Cyan)),
            Span::raw(": Open submenu  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(": Close"),
        ]),
    ];
    let help = Paragraph::new(help_lines);
    f.render_widget(help, chunks[3]);

    // Render context menu overlay (must be last to appear on top)
    if app.context_menu_state.is_open {
        let context_menu = ContextMenu::new(&app.context_menu_items, &app.context_menu_state)
            .style(ContextMenuStyle::default());
        let (menu_area, regions) = context_menu.render_stateful(f, area);
        app.menu_area = menu_area;
        app.menu_regions = regions;
    } else {
        app.menu_regions.clear();
        app.menu_area = Rect::default();
    }
}
