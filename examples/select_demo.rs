//! Select Demo
//!
//! Interactive demo showing the Select dropdown component:
//! - Closed state with placeholder
//! - Dropdown popup with keyboard navigation
//! - Mouse click support
//! - Multiple select instances with focus management
//!
//! Run with: cargo run --example select_demo

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
    widgets::{Block, Borders, Paragraph},
};

use ratatui_interact::{
    components::{
        Select, SelectAction, SelectState, SelectStyle, handle_select_key, handle_select_mouse,
    },
    events::is_close_key,
    traits::ClickRegion,
};

/// Which select is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedSelect {
    Color,
    Size,
    Priority,
}

/// Application state
struct App {
    /// Color selection state
    color_state: SelectState,
    /// Size selection state
    size_state: SelectState,
    /// Priority selection state
    priority_state: SelectState,
    /// Which select is focused
    focused: FocusedSelect,
    /// Color options
    colors: Vec<&'static str>,
    /// Size options
    sizes: Vec<&'static str>,
    /// Priority options
    priorities: Vec<&'static str>,
    /// Status message
    message: String,
    /// Should quit
    should_quit: bool,
    /// Stored areas for mouse handling
    color_area: Rect,
    size_area: Rect,
    priority_area: Rect,
    /// Dropdown regions (populated during render)
    dropdown_regions: Vec<ClickRegion<SelectAction>>,
}

impl App {
    fn new() -> Self {
        let colors = vec![
            "Red", "Green", "Blue", "Yellow", "Purple", "Orange", "Cyan", "Magenta",
        ];
        let sizes = vec!["Small", "Medium", "Large", "Extra Large"];
        let priorities = vec!["Low", "Normal", "High", "Critical", "Urgent"];

        let mut color_state = SelectState::new(colors.len());
        color_state.focused = true; // Start with first select focused

        Self {
            color_state,
            size_state: SelectState::new(sizes.len()),
            priority_state: SelectState::with_selected(priorities.len(), 1), // Pre-select "Normal"
            focused: FocusedSelect::Color,
            colors,
            sizes,
            priorities,
            message: "Tab to switch between selects. Enter/Space to open dropdown.".to_string(),
            should_quit: false,
            color_area: Rect::default(),
            size_area: Rect::default(),
            priority_area: Rect::default(),
            dropdown_regions: Vec::new(),
        }
    }

    fn focus_next(&mut self) {
        // Close any open dropdown first
        self.close_all_dropdowns();

        // Update focus states
        self.color_state.focused = false;
        self.size_state.focused = false;
        self.priority_state.focused = false;

        self.focused = match self.focused {
            FocusedSelect::Color => FocusedSelect::Size,
            FocusedSelect::Size => FocusedSelect::Priority,
            FocusedSelect::Priority => FocusedSelect::Color,
        };

        match self.focused {
            FocusedSelect::Color => self.color_state.focused = true,
            FocusedSelect::Size => self.size_state.focused = true,
            FocusedSelect::Priority => self.priority_state.focused = true,
        }
    }

    fn focus_prev(&mut self) {
        self.close_all_dropdowns();

        self.color_state.focused = false;
        self.size_state.focused = false;
        self.priority_state.focused = false;

        self.focused = match self.focused {
            FocusedSelect::Color => FocusedSelect::Priority,
            FocusedSelect::Size => FocusedSelect::Color,
            FocusedSelect::Priority => FocusedSelect::Size,
        };

        match self.focused {
            FocusedSelect::Color => self.color_state.focused = true,
            FocusedSelect::Size => self.size_state.focused = true,
            FocusedSelect::Priority => self.priority_state.focused = true,
        }
    }

    fn close_all_dropdowns(&mut self) {
        self.color_state.close();
        self.size_state.close();
        self.priority_state.close();
    }

    fn any_dropdown_open(&self) -> bool {
        self.color_state.is_open || self.size_state.is_open || self.priority_state.is_open
    }

    fn get_focused_state(&mut self) -> &mut SelectState {
        match self.focused {
            FocusedSelect::Color => &mut self.color_state,
            FocusedSelect::Size => &mut self.size_state,
            FocusedSelect::Priority => &mut self.priority_state,
        }
    }

    fn update_message(&mut self, action: SelectAction) {
        match action {
            SelectAction::Open => {
                self.message =
                    "Dropdown opened. Use Up/Down to navigate, Enter to select, Esc to close."
                        .to_string();
            }
            SelectAction::Close => {
                self.message = "Dropdown closed.".to_string();
            }
            SelectAction::Select(idx) => {
                let (name, value) = match self.focused {
                    FocusedSelect::Color => ("Color", self.colors[idx]),
                    FocusedSelect::Size => ("Size", self.sizes[idx]),
                    FocusedSelect::Priority => ("Priority", self.priorities[idx]),
                };
                self.message = format!("Selected {}: {}", name, value);
            }
            SelectAction::Focus => {}
        }
    }

    fn get_summary(&self) -> String {
        let color = self
            .color_state
            .selected()
            .map(|i| self.colors[i])
            .unwrap_or("None");
        let size = self
            .size_state
            .selected()
            .map(|i| self.sizes[i])
            .unwrap_or("None");
        let priority = self
            .priority_state
            .selected()
            .map(|i| self.priorities[i])
            .unwrap_or("None");

        format!("Color: {} | Size: {} | Priority: {}", color, size, priority)
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

        if let Event::Key(key) = event::read()? {
            // Check for quit
            if is_close_key(&key) || key.code == KeyCode::Char('q') {
                if app.any_dropdown_open() {
                    app.close_all_dropdowns();
                } else {
                    app.should_quit = true;
                }
            } else if key.code == KeyCode::Tab {
                // Tab navigation between selects
                app.focus_next();
            } else if key.code == KeyCode::BackTab {
                app.focus_prev();
            } else {
                // Handle key for focused select
                let state = app.get_focused_state();
                if let Some(action) = handle_select_key(&key, state) {
                    app.update_message(action);
                }
            }
        } else if let Event::Mouse(mouse) = event::read().unwrap_or(Event::FocusGained) {
            if let crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) =
                mouse.kind
            {
                // Check which select was clicked
                let col = mouse.column;
                let row = mouse.row;

                // Check dropdown regions first (if any dropdown is open)
                let mut handled = false;
                if app.any_dropdown_open() {
                    // Get the necessary values before borrowing mutably
                    let focused = app.focused;
                    let area = match focused {
                        FocusedSelect::Color => app.color_area,
                        FocusedSelect::Size => app.size_area,
                        FocusedSelect::Priority => app.priority_area,
                    };
                    let regions = std::mem::take(&mut app.dropdown_regions);

                    let state = match focused {
                        FocusedSelect::Color => &mut app.color_state,
                        FocusedSelect::Size => &mut app.size_state,
                        FocusedSelect::Priority => &mut app.priority_state,
                    };

                    if let Some(action) = handle_select_mouse(&mouse, state, area, &regions) {
                        app.update_message(action);
                        handled = true;
                    }
                    app.dropdown_regions = regions;
                }

                if !handled {
                    // Check if clicking on a select to focus/open it
                    if col >= app.color_area.x
                        && col < app.color_area.x + app.color_area.width
                        && row >= app.color_area.y
                        && row < app.color_area.y + app.color_area.height
                    {
                        app.close_all_dropdowns();
                        app.color_state.focused = true;
                        app.size_state.focused = false;
                        app.priority_state.focused = false;
                        app.focused = FocusedSelect::Color;
                        app.color_state.open();
                        app.update_message(SelectAction::Open);
                    } else if col >= app.size_area.x
                        && col < app.size_area.x + app.size_area.width
                        && row >= app.size_area.y
                        && row < app.size_area.y + app.size_area.height
                    {
                        app.close_all_dropdowns();
                        app.color_state.focused = false;
                        app.size_state.focused = true;
                        app.priority_state.focused = false;
                        app.focused = FocusedSelect::Size;
                        app.size_state.open();
                        app.update_message(SelectAction::Open);
                    } else if col >= app.priority_area.x
                        && col < app.priority_area.x + app.priority_area.width
                        && row >= app.priority_area.y
                        && row < app.priority_area.y + app.priority_area.height
                    {
                        app.close_all_dropdowns();
                        app.color_state.focused = false;
                        app.size_state.focused = false;
                        app.priority_state.focused = true;
                        app.focused = FocusedSelect::Priority;
                        app.priority_state.open();
                        app.update_message(SelectAction::Open);
                    } else {
                        // Clicked outside - close dropdowns
                        app.close_all_dropdowns();
                    }
                }
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
        .title(" Select Demo ")
        .border_style(Style::default().fg(Color::Blue));
    f.render_widget(background, area);

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Color select
            Constraint::Length(3), // Size select
            Constraint::Length(3), // Priority select
            Constraint::Length(2), // Spacer
            Constraint::Length(1), // Summary
            Constraint::Length(2), // Message
            Constraint::Min(0),    // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        "Select Component Demo",
        Style::default().fg(Color::Cyan),
    )]));
    f.render_widget(title, chunks[0]);

    // Color select
    app.color_area = chunks[1];
    let color_select = Select::new(&app.colors, &app.color_state)
        .label("Color")
        .placeholder("Choose a color...")
        .style(SelectStyle::default());
    color_select.render_stateful(f, chunks[1]);

    // Size select
    app.size_area = chunks[2];
    let size_select = Select::new(&app.sizes, &app.size_state)
        .label("Size")
        .placeholder("Select size...")
        .style(SelectStyle::minimal());
    size_select.render_stateful(f, chunks[2]);

    // Priority select
    app.priority_area = chunks[3];
    let priority_select = Select::new(&app.priorities, &app.priority_state)
        .label("Priority")
        .placeholder("Set priority...")
        .style(SelectStyle::arrow());
    priority_select.render_stateful(f, chunks[3]);

    // Summary
    let summary = Paragraph::new(Line::from(vec![Span::styled(
        app.get_summary(),
        Style::default().fg(Color::Green),
    )]));
    f.render_widget(summary, chunks[5]);

    // Message
    let message = Paragraph::new(Line::from(vec![Span::styled(
        &app.message,
        Style::default().fg(Color::Yellow),
    )]));
    f.render_widget(message, chunks[6]);

    // Help
    let help_lines = vec![
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(": Next select  "),
            Span::styled("Shift+Tab", Style::default().fg(Color::Cyan)),
            Span::raw(": Previous  "),
            Span::styled("Enter/Space", Style::default().fg(Color::Cyan)),
            Span::raw(": Open/Select  "),
            Span::styled("Esc/q", Style::default().fg(Color::Cyan)),
            Span::raw(": Close/Quit"),
        ]),
        Line::from(vec![
            Span::styled("Up/Down", Style::default().fg(Color::Cyan)),
            Span::raw(": Navigate options  "),
            Span::styled("Home/End", Style::default().fg(Color::Cyan)),
            Span::raw(": First/Last  "),
            Span::styled("Click", Style::default().fg(Color::Cyan)),
            Span::raw(": Select/Open"),
        ]),
    ];
    let help = Paragraph::new(help_lines);
    f.render_widget(help, chunks[7]);

    // Render dropdown overlays (must be last to appear on top)
    app.dropdown_regions.clear();

    if app.color_state.is_open {
        let select = Select::new(&app.colors, &app.color_state);
        app.dropdown_regions = select.render_dropdown(f, app.color_area, area);
    } else if app.size_state.is_open {
        let select = Select::new(&app.sizes, &app.size_state).style(SelectStyle::minimal());
        app.dropdown_regions = select.render_dropdown(f, app.size_area, area);
    } else if app.priority_state.is_open {
        let select = Select::new(&app.priorities, &app.priority_state).style(SelectStyle::arrow());
        app.dropdown_regions = select.render_dropdown(f, app.priority_area, area);
    }
}
