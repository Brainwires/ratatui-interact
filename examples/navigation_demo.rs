//! Navigation Components Demo
//!
//! Interactive demo showing navigation components:
//! - ListPicker: Scrollable list selection with custom rendering
//! - TreeView: Hierarchical tree navigation with expand/collapse
//!
//! Keyboard controls:
//! - Tab: Switch between panels
//! - Up/Down: Navigate items
//! - Enter/Space: Select item / Toggle tree node
//! - Left/Right: Collapse/Expand tree nodes
//! - Home/End: Jump to first/last item
//! - q/Esc: Quit
//!
//! Run with: cargo run --example navigation_demo

use std::io;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use ratatui_interact::{
    components::{
        ListPicker, ListPickerState, ListPickerStyle, TreeNode, TreeStyle, TreeView, TreeViewState,
        get_selected_id, key_hints_footer,
    },
    events::is_close_key,
};

/// Item for the list picker
#[derive(Debug, Clone)]
struct MenuItem {
    name: String,
    description: String,
    category: &'static str,
}

impl std::fmt::Display for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Data for tree nodes
#[derive(Debug, Clone)]
struct FileEntry {
    name: String,
    size: Option<u64>,
    is_dir: bool,
}

/// Which panel is focused
#[derive(Clone, Copy, PartialEq)]
enum FocusedPanel {
    List,
    Tree,
}

impl FocusedPanel {
    fn toggle(self) -> Self {
        match self {
            FocusedPanel::List => FocusedPanel::Tree,
            FocusedPanel::Tree => FocusedPanel::List,
        }
    }
}

/// Application state
struct App {
    /// Currently focused panel
    focused_panel: FocusedPanel,
    /// Should quit
    should_quit: bool,

    // ListPicker state
    list_items: Vec<MenuItem>,
    list_state: ListPickerState,
    selected_item: Option<String>,

    // TreeView state
    tree_nodes: Vec<TreeNode<FileEntry>>,
    tree_state: TreeViewState,
    selected_node: Option<String>,
}

impl App {
    fn new() -> Self {
        // Create list items
        let list_items = vec![
            MenuItem {
                name: "New File".into(),
                description: "Create a new empty file".into(),
                category: "File",
            },
            MenuItem {
                name: "Open File".into(),
                description: "Open an existing file".into(),
                category: "File",
            },
            MenuItem {
                name: "Save".into(),
                description: "Save the current file".into(),
                category: "File",
            },
            MenuItem {
                name: "Save As".into(),
                description: "Save with a new name".into(),
                category: "File",
            },
            MenuItem {
                name: "Cut".into(),
                description: "Cut selection to clipboard".into(),
                category: "Edit",
            },
            MenuItem {
                name: "Copy".into(),
                description: "Copy selection to clipboard".into(),
                category: "Edit",
            },
            MenuItem {
                name: "Paste".into(),
                description: "Paste from clipboard".into(),
                category: "Edit",
            },
            MenuItem {
                name: "Find".into(),
                description: "Search in document".into(),
                category: "Edit",
            },
            MenuItem {
                name: "Replace".into(),
                description: "Find and replace text".into(),
                category: "Edit",
            },
            MenuItem {
                name: "Settings".into(),
                description: "Open preferences".into(),
                category: "Tools",
            },
            MenuItem {
                name: "Extensions".into(),
                description: "Manage extensions".into(),
                category: "Tools",
            },
            MenuItem {
                name: "About".into(),
                description: "About this application".into(),
                category: "Help",
            },
        ];

        let list_state = ListPickerState::new(list_items.len());

        // Create tree nodes (file system like structure)
        let tree_nodes = vec![
            TreeNode::new(
                "src",
                FileEntry {
                    name: "src".into(),
                    size: None,
                    is_dir: true,
                },
            )
            .with_children(vec![
                TreeNode::new(
                    "src/main.rs",
                    FileEntry {
                        name: "main.rs".into(),
                        size: Some(1024),
                        is_dir: false,
                    },
                ),
                TreeNode::new(
                    "src/lib.rs",
                    FileEntry {
                        name: "lib.rs".into(),
                        size: Some(2048),
                        is_dir: false,
                    },
                ),
                TreeNode::new(
                    "src/components",
                    FileEntry {
                        name: "components".into(),
                        size: None,
                        is_dir: true,
                    },
                )
                .with_children(vec![
                    TreeNode::new(
                        "src/components/mod.rs",
                        FileEntry {
                            name: "mod.rs".into(),
                            size: Some(512),
                            is_dir: false,
                        },
                    ),
                    TreeNode::new(
                        "src/components/button.rs",
                        FileEntry {
                            name: "button.rs".into(),
                            size: Some(3072),
                            is_dir: false,
                        },
                    ),
                    TreeNode::new(
                        "src/components/input.rs",
                        FileEntry {
                            name: "input.rs".into(),
                            size: Some(4096),
                            is_dir: false,
                        },
                    ),
                ]),
                TreeNode::new(
                    "src/utils",
                    FileEntry {
                        name: "utils".into(),
                        size: None,
                        is_dir: true,
                    },
                )
                .with_children(vec![
                    TreeNode::new(
                        "src/utils/mod.rs",
                        FileEntry {
                            name: "mod.rs".into(),
                            size: Some(256),
                            is_dir: false,
                        },
                    ),
                    TreeNode::new(
                        "src/utils/helpers.rs",
                        FileEntry {
                            name: "helpers.rs".into(),
                            size: Some(1536),
                            is_dir: false,
                        },
                    ),
                ]),
            ]),
            TreeNode::new(
                "tests",
                FileEntry {
                    name: "tests".into(),
                    size: None,
                    is_dir: true,
                },
            )
            .with_children(vec![TreeNode::new(
                "tests/integration.rs",
                FileEntry {
                    name: "integration.rs".into(),
                    size: Some(2560),
                    is_dir: false,
                },
            )]),
            TreeNode::new(
                "Cargo.toml",
                FileEntry {
                    name: "Cargo.toml".into(),
                    size: Some(512),
                    is_dir: false,
                },
            ),
            TreeNode::new(
                "README.md",
                FileEntry {
                    name: "README.md".into(),
                    size: Some(1280),
                    is_dir: false,
                },
            ),
        ];

        let tree_state = TreeViewState::new();

        Self {
            focused_panel: FocusedPanel::List,
            should_quit: false,
            list_items,
            list_state,
            selected_item: None,
            tree_nodes,
            tree_state,
            selected_node: None,
        }
    }

    fn visible_tree_count(&self) -> usize {
        let tree = TreeView::new(&self.tree_nodes, &self.tree_state);
        tree.visible_count()
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Tab => {
                self.focused_panel = self.focused_panel.toggle();
            }
            KeyCode::Up => match self.focused_panel {
                FocusedPanel::List => {
                    self.list_state.select_prev();
                    self.list_state.ensure_visible(10);
                }
                FocusedPanel::Tree => {
                    self.tree_state.select_prev();
                    self.tree_state.ensure_visible(10);
                }
            },
            KeyCode::Down => match self.focused_panel {
                FocusedPanel::List => {
                    self.list_state.select_next();
                    self.list_state.ensure_visible(10);
                }
                FocusedPanel::Tree => {
                    let count = self.visible_tree_count();
                    self.tree_state.select_next(count);
                    self.tree_state.ensure_visible(10);
                }
            },
            KeyCode::Home => match self.focused_panel {
                FocusedPanel::List => {
                    self.list_state.select_first();
                    self.list_state.ensure_visible(10);
                }
                FocusedPanel::Tree => {
                    self.tree_state.selected_index = 0;
                    self.tree_state.ensure_visible(10);
                }
            },
            KeyCode::End => match self.focused_panel {
                FocusedPanel::List => {
                    self.list_state.select_last();
                    self.list_state.ensure_visible(10);
                }
                FocusedPanel::Tree => {
                    let count = self.visible_tree_count();
                    if count > 0 {
                        self.tree_state.selected_index = count - 1;
                    }
                    self.tree_state.ensure_visible(10);
                }
            },
            KeyCode::Enter | KeyCode::Char(' ') => match self.focused_panel {
                FocusedPanel::List => {
                    if let Some(item) = self.list_items.get(self.list_state.selected_index) {
                        self.selected_item = Some(item.name.clone());
                    }
                }
                FocusedPanel::Tree => {
                    // Toggle expand/collapse for current selection
                    if let Some(id) = get_selected_id(&self.tree_nodes, &self.tree_state) {
                        // Check if it has children
                        if self.node_has_children(&id) {
                            self.tree_state.toggle_collapsed(&id);
                        } else {
                            self.selected_node = Some(id);
                        }
                    }
                }
            },
            KeyCode::Left => {
                if self.focused_panel == FocusedPanel::Tree {
                    if let Some(id) = get_selected_id(&self.tree_nodes, &self.tree_state) {
                        self.tree_state.collapse(&id);
                    }
                }
            }
            KeyCode::Right => {
                if self.focused_panel == FocusedPanel::Tree {
                    if let Some(id) = get_selected_id(&self.tree_nodes, &self.tree_state) {
                        self.tree_state.expand(&id);
                    }
                }
            }
            _ => {}
        }
    }

    fn node_has_children(&self, id: &str) -> bool {
        self.find_node(&self.tree_nodes, id)
            .is_some_and(|n| n.has_children())
    }

    fn find_node<'a>(
        &self,
        nodes: &'a [TreeNode<FileEntry>],
        id: &str,
    ) -> Option<&'a TreeNode<FileEntry>> {
        for node in nodes {
            if node.id == id {
                return Some(node);
            }
            if let Some(found) = self.find_node(&node.children, id) {
                return Some(found);
            }
        }
        None
    }
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Ok(Event::Key(key)) = event::read() {
            if is_close_key(&key) || key.code == KeyCode::Char('q') {
                app.should_quit = true;
            } else {
                app.handle_key(key.code);
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Content
            Constraint::Length(5), // Status + Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "Navigation Components Demo",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "ListPicker + TreeView",
            Style::default().fg(Color::DarkGray),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Content area - split into two panels
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    // ListPicker Panel
    render_list_panel(f, app, content_chunks[0]);

    // TreeView Panel
    render_tree_panel(f, app, content_chunks[1]);

    // Status + Help
    let status_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(3)])
        .split(chunks[2]);

    // Status line
    let focus_text = match app.focused_panel {
        FocusedPanel::List => "ListPicker",
        FocusedPanel::Tree => "TreeView",
    };

    let selected_info = match app.focused_panel {
        FocusedPanel::List => app
            .selected_item
            .as_ref()
            .map(|s| format!("Selected: {}", s))
            .unwrap_or_else(|| "No selection".to_string()),
        FocusedPanel::Tree => app
            .selected_node
            .as_ref()
            .map(|s| format!("Selected: {}", s))
            .unwrap_or_else(|| "No selection".to_string()),
    };

    let status = Paragraph::new(vec![Line::from(vec![
        Span::styled("Focus: ", Style::default().fg(Color::Gray)),
        Span::styled(focus_text, Style::default().fg(Color::Green)),
        Span::raw("  |  "),
        Span::styled(selected_info, Style::default().fg(Color::Yellow)),
    ])])
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(status, status_chunks[0]);

    // Help
    let help_lines = vec![
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Switch  "),
            Span::styled("Up/Down", Style::default().fg(Color::Yellow)),
            Span::raw(": Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Select/Toggle  "),
            Span::styled("Left/Right", Style::default().fg(Color::Yellow)),
            Span::raw(": Collapse/Expand"),
        ]),
        Line::from(vec![
            Span::styled("Home/End", Style::default().fg(Color::Yellow)),
            Span::raw(": First/Last  "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];
    let help = Paragraph::new(help_lines);
    f.render_widget(help, status_chunks[1]);
}

fn render_list_panel(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::List;

    let block = Block::default()
        .title(Span::styled(
            " Command Palette ",
            if is_focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        ))
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create list picker with custom rendering
    let style = ListPickerStyle::default().bordered(false);

    let footer = key_hints_footer(&[("Enter", "Select")]);

    let picker = ListPicker::new(&app.list_items, &app.list_state)
        .style(style)
        .footer(footer)
        .render_item(|item, _idx, is_selected| {
            let cat_color = match item.category {
                "File" => Color::Blue,
                "Edit" => Color::Green,
                "Tools" => Color::Magenta,
                "Help" => Color::Cyan,
                _ => Color::White,
            };

            vec![
                Line::from(vec![
                    Span::styled(
                        item.name.clone(),
                        if is_selected {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        },
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("[{}]", item.category),
                        Style::default().fg(cat_color),
                    ),
                ]),
                Line::from(vec![Span::styled(
                    format!("  {}", item.description),
                    Style::default().fg(Color::DarkGray),
                )]),
            ]
        });

    picker.render(inner, f.buffer_mut());
}

fn render_tree_panel(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Tree;

    let block = Block::default()
        .title(Span::styled(
            " Project Explorer ",
            if is_focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        ))
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create tree view with custom rendering
    let style = TreeStyle::default();

    let tree = TreeView::new(&app.tree_nodes, &app.tree_state)
        .style(style)
        .render_item(|node, _is_selected| {
            let icon = if node.data.is_dir {
                if app.tree_state.is_collapsed(&node.id) {
                    ""
                } else {
                    ""
                }
            } else {
                match node.data.name.split('.').last() {
                    Some("rs") => "",
                    Some("toml") => "",
                    Some("md") => "",
                    _ => "",
                }
            };

            let size_str = node
                .data
                .size
                .map(|s| format!(" ({}B)", s))
                .unwrap_or_default();

            format!("{} {}{}", icon, node.data.name, size_str)
        });

    tree.render(inner, f.buffer_mut());
}
