//! Hotkey Dialog Demo
//!
//! Interactive demo showing the HotkeyDialog component with:
//! - Category-based navigation
//! - Search filtering across all hotkeys
//! - Mouse support for selection
//! - Scrolling for long lists
//!
//! Run with: cargo run --example hotkey_dialog_demo

use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use ratatui_interact::components::hotkey_dialog::{
    HotkeyCategory, HotkeyDialog, HotkeyDialogAction, HotkeyDialogState, HotkeyDialogStyle,
    HotkeyEntryData, HotkeyProvider, handle_hotkey_dialog_key, handle_hotkey_dialog_mouse,
};

// ============================================================================
// Demo Category Implementation
// ============================================================================

/// Demo hotkey categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum DemoCategory {
    #[default]
    GlobalActions,
    Navigation,
    TextEditing,
    ViewsModes,
    FileOperations,
}

impl HotkeyCategory for DemoCategory {
    fn all() -> &'static [Self] {
        &[
            Self::GlobalActions,
            Self::Navigation,
            Self::TextEditing,
            Self::ViewsModes,
            Self::FileOperations,
        ]
    }

    fn display_name(&self) -> &str {
        match self {
            Self::GlobalActions => "Global Actions",
            Self::Navigation => "Navigation",
            Self::TextEditing => "Text Editing",
            Self::ViewsModes => "Views & Modes",
            Self::FileOperations => "File Operations",
        }
    }

    fn icon(&self) -> &str {
        match self {
            Self::GlobalActions => "G",
            Self::Navigation => "N",
            Self::TextEditing => "T",
            Self::ViewsModes => "V",
            Self::FileOperations => "F",
        }
    }

    fn next(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|c| c == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }

    fn prev(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|c| c == self).unwrap_or(0);
        all[(idx + all.len() - 1) % all.len()]
    }
}

// ============================================================================
// Demo Provider Implementation
// ============================================================================

/// Demo hotkey provider
struct DemoHotkeyProvider;

impl HotkeyProvider for DemoHotkeyProvider {
    type Category = DemoCategory;

    fn entries_for_category(&self, category: Self::Category) -> Vec<HotkeyEntryData> {
        match category {
            DemoCategory::GlobalActions => vec![
                HotkeyEntryData::global("Ctrl+C", "Quit application").fixed(),
                HotkeyEntryData::global("Ctrl+Z", "Undo last action"),
                HotkeyEntryData::global("Ctrl+Y", "Redo last action"),
                HotkeyEntryData::global("Escape", "Cancel current operation"),
                HotkeyEntryData::global("F1", "Open help"),
                HotkeyEntryData::global("F2", "Rename"),
                HotkeyEntryData::global("F5", "Refresh"),
            ],
            DemoCategory::Navigation => vec![
                HotkeyEntryData::new("Up/Down", "Navigate items", "List"),
                HotkeyEntryData::new("PageUp", "Page up", "List"),
                HotkeyEntryData::new("PageDown", "Page down", "List"),
                HotkeyEntryData::new("Home", "Go to first item", "List"),
                HotkeyEntryData::new("End", "Go to last item", "List"),
                HotkeyEntryData::new("Ctrl+Home", "Go to document start", "Editor"),
                HotkeyEntryData::new("Ctrl+End", "Go to document end", "Editor"),
                HotkeyEntryData::new("Tab", "Next field", "Form"),
                HotkeyEntryData::new("Shift+Tab", "Previous field", "Form"),
            ],
            DemoCategory::TextEditing => vec![
                HotkeyEntryData::new("Ctrl+A", "Select all", "Editor"),
                HotkeyEntryData::new("Ctrl+X", "Cut selection", "Editor"),
                HotkeyEntryData::new("Ctrl+C", "Copy selection", "Editor"),
                HotkeyEntryData::new("Ctrl+V", "Paste", "Editor"),
                HotkeyEntryData::new("Ctrl+F", "Find", "Editor"),
                HotkeyEntryData::new("Ctrl+H", "Find and Replace", "Editor"),
                HotkeyEntryData::new("Ctrl+D", "Duplicate line", "Editor"),
                HotkeyEntryData::new("Ctrl+/", "Toggle comment", "Editor"),
                HotkeyEntryData::new("Ctrl+Shift+K", "Delete line", "Editor"),
                HotkeyEntryData::new("Alt+Up", "Move line up", "Editor"),
                HotkeyEntryData::new("Alt+Down", "Move line down", "Editor"),
            ],
            DemoCategory::ViewsModes => vec![
                HotkeyEntryData::new("F10", "Toggle fullscreen", "Window"),
                HotkeyEntryData::new("Ctrl+B", "Toggle sidebar", "Window"),
                HotkeyEntryData::new("Ctrl+`", "Toggle terminal", "Window"),
                HotkeyEntryData::new("Ctrl+Shift+E", "Explorer view", "Window"),
                HotkeyEntryData::new("Ctrl+Shift+G", "Git view", "Window"),
                HotkeyEntryData::new("Ctrl+Shift+D", "Debug view", "Window"),
            ],
            DemoCategory::FileOperations => vec![
                HotkeyEntryData::new("Ctrl+N", "New file", "File"),
                HotkeyEntryData::new("Ctrl+O", "Open file", "File"),
                HotkeyEntryData::new("Ctrl+S", "Save file", "File"),
                HotkeyEntryData::new("Ctrl+Shift+S", "Save as", "File"),
                HotkeyEntryData::new("Ctrl+W", "Close file", "File"),
                HotkeyEntryData::new("Ctrl+Shift+T", "Reopen closed", "File"),
            ],
        }
    }

    fn search(&self, query: &str) -> Vec<(Self::Category, HotkeyEntryData)> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for category in DemoCategory::all() {
            for entry in self.entries_for_category(*category) {
                if entry.key_combination.to_lowercase().contains(&query_lower)
                    || entry.action.to_lowercase().contains(&query_lower)
                    || entry.context.to_lowercase().contains(&query_lower)
                {
                    results.push((*category, entry));
                }
            }
        }

        results
    }
}

// ============================================================================
// Application State
// ============================================================================

struct App {
    /// Hotkey dialog state
    dialog_state: Option<HotkeyDialogState<DemoCategory>>,
    /// Hotkey provider
    provider: DemoHotkeyProvider,
    /// Dialog style
    style: HotkeyDialogStyle,
    /// Status message
    message: String,
    /// Should quit
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            dialog_state: None,
            provider: DemoHotkeyProvider,
            style: HotkeyDialogStyle::default()
                .title(" Demo Hotkey Configuration "),
            message: String::new(),
            should_quit: false,
        }
    }

    fn show_dialog(&mut self) {
        self.dialog_state = Some(HotkeyDialogState::new());
        self.message = "Hotkey dialog opened".to_string();
    }

    fn close_dialog(&mut self) {
        self.dialog_state = None;
        self.message = "Hotkey dialog closed".to_string();
    }

    fn is_dialog_open(&self) -> bool {
        self.dialog_state.is_some()
    }
}

// ============================================================================
// Main
// ============================================================================

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
                if app.is_dialog_open() {
                    // Handle dialog events
                    if let Some(ref mut state) = app.dialog_state {
                        let action = handle_hotkey_dialog_key(state, key);

                        match action {
                            HotkeyDialogAction::Close => {
                                app.close_dialog();
                            }
                            HotkeyDialogAction::EntrySelected { .. } => {
                                // Get selected entry info
                                if let Some(entry) = state.get_selected_entry(&app.provider) {
                                    app.message = format!(
                                        "Selected: {} - {} [{}]",
                                        entry.key_combination, entry.action, entry.context
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
                    // Main screen events
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::F(1) => {
                            app.show_dialog();
                        }
                        _ => {}
                    }
                }
            }
            Event::Mouse(mouse) => {
                if app.is_dialog_open() {
                    if let Some(ref mut state) = app.dialog_state {
                        let action = handle_hotkey_dialog_mouse(state, mouse);

                        // Scroll actions are handled internally, but we can
                        // also respond to them if needed
                        match action {
                            HotkeyDialogAction::ScrollUp(_) | HotkeyDialogAction::ScrollDown(_) => {
                                // Scroll handled by state
                            }
                            _ => {}
                        }
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

// ============================================================================
// UI Rendering
// ============================================================================

fn ui(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Background
    let background = Block::default()
        .borders(Borders::ALL)
        .title(" Hotkey Dialog Demo ")
        .border_style(Style::default().fg(Color::Blue));
    f.render_widget(background, area);

    // Main content
    let inner = Rect::new(area.x + 2, area.y + 2, area.width - 4, area.height - 4);

    let help_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("H / F1", Style::default().fg(Color::Yellow)),
            Span::raw(": Open Hotkey Dialog  "),
            Span::styled("q / Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            &app.message,
            Style::default().fg(Color::Green),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Features demonstrated:",
            Style::default().fg(Color::Cyan),
        )),
        Line::from("  - Category navigation with Up/Down arrows"),
        Line::from("  - Search filtering with instant results"),
        Line::from("  - Tab between Search, Categories, and Hotkey list"),
        Line::from("  - Mouse click support for selection"),
        Line::from("  - Mouse scroll for long lists"),
        Line::from("  - Customizable styling"),
    ];
    let help = Paragraph::new(help_lines);
    f.render_widget(help, inner);

    // Render dialog if open
    if let Some(ref mut state) = app.dialog_state {
        let dialog = HotkeyDialog::new(state, &app.provider, &app.style);
        dialog.render(f, area);
    }
}
