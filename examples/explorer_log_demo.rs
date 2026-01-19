//! File Inspector Demo
//!
//! Combined demo showcasing FileExplorer, LogViewer, and Toast working together.
//!
//! Features:
//! - Left panel: FileExplorer for browsing files
//! - Right panel: LogViewer showing activity log
//! - Toast notifications for user feedback
//! - Tab navigation between panels
//! - Mouse click support
//!
//! Run with: cargo run --example explorer_log_demo --features filesystem

use std::io;
use std::path::PathBuf;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
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
        FileExplorer, FileExplorerState, LogViewer, LogViewerState, Toast, ToastState,
        file_explorer::{FileExplorerMode, draw_search_bar},
    },
    events::{is_backtab, is_close_key, is_left_click, is_tab},
    state::FocusManager,
    traits::ClickRegionRegistry,
};

/// Focus target identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FocusTarget {
    FileExplorer,
    LogViewer,
}

/// Application state
struct App {
    /// File explorer state
    explorer: FileExplorerState,
    /// Log viewer state
    log: LogViewerState,
    /// Toast notification state
    toast: ToastState,
    /// Focus manager
    focus: FocusManager<FocusTarget>,
    /// Click regions
    click_regions: ClickRegionRegistry<FocusTarget>,
    /// Should quit
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        // Get current directory
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));

        // Initialize file explorer
        let mut explorer = FileExplorerState::new(current_dir.clone());
        #[cfg(feature = "filesystem")]
        let _ = explorer.load_entries();

        // Initialize log viewer with startup messages
        let log_content = vec![
            format!("[INFO] File Inspector started"),
            format!("[INFO] Current directory: {}", current_dir.display()),
            format!("[INFO] {} entries loaded", explorer.entries.len()),
            String::new(),
            "[INFO] Navigation:".to_string(),
            "  Tab        - Switch panels".to_string(),
            "  Arrow keys - Navigate".to_string(),
            "  Enter      - Open directory".to_string(),
            "  Space      - Toggle selection".to_string(),
            "  /          - Search".to_string(),
            "  .          - Toggle hidden files".to_string(),
            "  g/G        - Go to top/bottom".to_string(),
            "  q/Esc      - Quit".to_string(),
        ];
        let log = LogViewerState::new(log_content);

        // Initialize toast
        let mut toast = ToastState::new();
        toast.show("Welcome to File Inspector!", 3000);

        // Initialize focus manager
        let mut focus = FocusManager::new();
        focus.register(FocusTarget::FileExplorer);
        focus.register(FocusTarget::LogViewer);

        Self {
            explorer,
            log,
            toast,
            focus,
            click_regions: ClickRegionRegistry::new(),
            should_quit: false,
        }
    }

    fn log_action(&mut self, message: &str) {
        self.log.append(message.to_string());
        // Auto-scroll to bottom when new entries are added
        self.log.go_to_bottom();
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Global keys
        if is_close_key(&key) || key.code == KeyCode::Char('q') {
            self.should_quit = true;
            return;
        }

        if is_tab(&key) {
            self.focus.next();
            return;
        }

        if is_backtab(&key) {
            self.focus.prev();
            return;
        }

        // Panel-specific keys
        match self.focus.current() {
            Some(FocusTarget::FileExplorer) => self.handle_explorer_key(key),
            Some(FocusTarget::LogViewer) => self.handle_log_key(key),
            None => {}
        }
    }

    fn handle_explorer_key(&mut self, key: KeyEvent) {
        match self.explorer.mode {
            FileExplorerMode::Browse => self.handle_explorer_browse_key(key),
            FileExplorerMode::Search => self.handle_explorer_search_key(key),
        }
    }

    fn handle_explorer_browse_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.explorer.cursor_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.explorer.cursor_down();
            }
            KeyCode::Enter => {
                if let Some(entry) = self.explorer.current_entry() {
                    if entry.is_dir() {
                        let path = entry.path.clone();
                        let name = entry.name.clone();
                        self.explorer.enter_directory(path);
                        self.log_action(&format!("[INFO] Entered: {}", name));
                        self.toast.show(format!("Entering: {}", name), 2000);
                    }
                }
            }
            KeyCode::Backspace => {
                self.explorer.go_up();
                self.log_action("[INFO] Navigated to parent directory");
            }
            KeyCode::Char(' ') => {
                if let Some(entry) = self.explorer.current_entry() {
                    let name = entry.name.clone();
                    let was_selected = self.explorer.selected_files.contains(&entry.path);
                    self.explorer.toggle_selection();

                    if was_selected {
                        self.log_action(&format!("[INFO] Deselected: {}", name));
                        self.toast.show(format!("Deselected: {}", name), 1500);
                    } else {
                        self.log_action(&format!("[INFO] Selected: {}", name));
                        self.toast.show(format!("Selected: {}", name), 1500);
                    }
                }
            }
            KeyCode::Char('/') => {
                self.explorer.start_search();
                self.toast.show("Search mode", 1500);
            }
            KeyCode::Char('.') => {
                self.explorer.toggle_hidden();
                let status = if self.explorer.show_hidden {
                    "ON"
                } else {
                    "OFF"
                };
                self.log_action(&format!("[INFO] Hidden files: {}", status));
                self.toast.show(format!("Hidden files: {}", status), 1500);
            }
            KeyCode::Char('a') => {
                self.explorer.select_all();
                let count = self.explorer.selected_files.len();
                self.log_action(&format!("[INFO] Selected all ({} files)", count));
                self.toast.show(format!("Selected {} files", count), 1500);
            }
            KeyCode::Char('n') => {
                self.explorer.select_none();
                self.log_action("[INFO] Cleared selection");
                self.toast.show("Selection cleared", 1500);
            }
            KeyCode::Char('g') => {
                self.explorer.cursor_index = 0;
                self.explorer.scroll = 0;
            }
            KeyCode::Char('G') => {
                let count = self.explorer.visible_count();
                if count > 0 {
                    self.explorer.cursor_index = count - 1;
                }
            }
            _ => {}
        }
    }

    fn handle_explorer_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.explorer.cancel_search();
            }
            KeyCode::Enter => {
                self.explorer.mode = FileExplorerMode::Browse;
                let matches = self
                    .explorer
                    .filtered_indices
                    .as_ref()
                    .map(|i| i.len())
                    .unwrap_or(0);
                self.log_action(&format!("[INFO] Search complete: {} matches", matches));
            }
            KeyCode::Backspace => {
                self.explorer.search_query.pop();
                self.explorer.update_filter();
            }
            KeyCode::Char(c) => {
                self.explorer.search_query.push(c);
                self.explorer.update_filter();
            }
            _ => {}
        }
    }

    fn handle_log_key(&mut self, key: KeyEvent) {
        if self.log.search.active {
            self.handle_log_search_key(key);
        } else {
            self.handle_log_browse_key(key);
        }
    }

    fn handle_log_browse_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.log.scroll_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.log.scroll_down();
            }
            KeyCode::PageUp => {
                self.log.page_up();
            }
            KeyCode::PageDown => {
                self.log.page_down();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.log.scroll_left();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.log.scroll_right();
            }
            KeyCode::Char('g') => {
                self.log.go_to_top();
            }
            KeyCode::Char('G') => {
                self.log.go_to_bottom();
            }
            KeyCode::Char('/') => {
                self.log.start_search();
            }
            KeyCode::Char('n') => {
                self.log.next_match();
            }
            KeyCode::Char('N') => {
                self.log.prev_match();
            }
            _ => {}
        }
    }

    fn handle_log_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.log.cancel_search();
            }
            KeyCode::Enter => {
                self.log.search.active = false;
            }
            KeyCode::Backspace => {
                self.log.search.query.pop();
                self.log.update_search();
            }
            KeyCode::Char(c) => {
                self.log.search.query.push(c);
                self.log.update_search();
            }
            _ => {}
        }
    }

    fn handle_click(&mut self, col: u16, row: u16) {
        if let Some(&target) = self.click_regions.handle_click(col, row) {
            self.focus.set(target);
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

        // Clear expired toasts
        app.toast.clear_if_expired();

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    app.handle_key(key);
                }
                Event::Mouse(mouse) => {
                    if is_left_click(&mouse) {
                        app.handle_click(mouse.column, mouse.row);
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
    // Clear click regions
    app.click_regions.clear();

    let area = f.area();

    // Main layout: title, content, footer
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Content
            Constraint::Length(2), // Footer
        ])
        .split(area);

    // Title
    let title = Paragraph::new("File Inspector Demo")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, main_chunks[0]);

    // Content: two panels side by side
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    // File Explorer panel
    // Update visible height for scrolling
    let explorer_inner_height = content_chunks[0].height.saturating_sub(5) as usize;
    app.explorer.ensure_visible(explorer_inner_height);

    let explorer = FileExplorer::new(&app.explorer);
    f.render_widget(explorer, content_chunks[0]);

    // Register click region for file explorer
    app.click_regions
        .register(content_chunks[0], FocusTarget::FileExplorer);

    // Draw search bar overlay if in search mode
    if app.explorer.mode == FileExplorerMode::Search {
        let search_area = Rect::new(
            content_chunks[0].x,
            content_chunks[0].y + content_chunks[0].height.saturating_sub(3),
            content_chunks[0].width,
            2,
        );
        draw_search_bar(f, &app.explorer.search_query, search_area);
    }

    // Log Viewer panel
    // Update visible height for log viewer
    let log_inner_height = content_chunks[1].height.saturating_sub(4) as usize;
    app.log.visible_height = log_inner_height;

    let log_viewer = LogViewer::new(&app.log)
        .title("Activity Log")
        .show_line_numbers(true);
    f.render_widget(log_viewer, content_chunks[1]);

    // Register click region for log viewer
    app.click_regions
        .register(content_chunks[1], FocusTarget::LogViewer);

    // Footer with help
    let focus_indicator = match app.focus.current() {
        Some(FocusTarget::FileExplorer) => "[Explorer]",
        Some(FocusTarget::LogViewer) => "[Log]",
        None => "",
    };

    let help_line = Line::from(vec![
        Span::styled(
            focus_indicator,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(":Switch "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(":Open "),
        Span::styled("Space", Style::default().fg(Color::Yellow)),
        Span::raw(":Select "),
        Span::styled("/", Style::default().fg(Color::Yellow)),
        Span::raw(":Search "),
        Span::styled(".", Style::default().fg(Color::Yellow)),
        Span::raw(":Hidden "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(":Quit"),
    ]);
    let help = Paragraph::new(help_line);
    f.render_widget(help, main_chunks[2]);

    // Toast notification (rendered last, on top)
    if let Some(message) = app.toast.get_message() {
        let toast = Toast::new(message).auto_style();
        toast.render_with_clear(area, f.buffer_mut());
    }
}
