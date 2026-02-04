//! Button Demo
//!
//! Interactive demo showing button features:
//! - Different button variants (SingleLine, Block, IconText, Toggle, Minimal)
//! - Tab navigation
//! - Mouse click support
//! - Style presets (primary, danger, success)
//!
//! Run with: cargo run --example button_demo

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
    components::{Button, ButtonState, ButtonStyle, ButtonVariant, Toast, ToastState},
    events::{is_activate_key, is_backtab, is_close_key, is_left_click, is_tab},
    state::FocusManager,
    traits::ClickRegionRegistry,
};

/// Button definition
struct ButtonDef {
    label: &'static str,
    icon: Option<&'static str>,
    variant: ButtonVariant,
    style: ButtonStyle,
    state: ButtonState,
}

/// Application state
struct App {
    /// Buttons
    buttons: Vec<ButtonDef>,
    /// Focus manager
    focus: FocusManager<usize>,
    /// Click regions
    click_regions: ClickRegionRegistry<usize>,
    /// Last clicked button
    last_clicked: Option<usize>,
    /// Toast notification state
    toast_state: ToastState,
    /// Should quit
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let mut focus = FocusManager::new();

        let buttons = vec![
            ButtonDef {
                label: "OK",
                icon: None,
                variant: ButtonVariant::SingleLine,
                style: ButtonStyle::default(),
                state: ButtonState::enabled(),
            },
            ButtonDef {
                label: "Cancel",
                icon: None,
                variant: ButtonVariant::SingleLine,
                style: ButtonStyle::default(),
                state: ButtonState::enabled(),
            },
            ButtonDef {
                label: "Save",
                icon: Some("💾"),
                variant: ButtonVariant::SingleLine,
                style: ButtonStyle::primary(),
                state: ButtonState::enabled(),
            },
            ButtonDef {
                label: "Delete",
                icon: Some("🗑"),
                variant: ButtonVariant::SingleLine,
                style: ButtonStyle::danger(),
                state: ButtonState::enabled(),
            },
            ButtonDef {
                label: "Submit",
                icon: None,
                variant: ButtonVariant::Block,
                style: ButtonStyle::success(),
                state: ButtonState::enabled(),
            },
            ButtonDef {
                label: "Dark Mode",
                icon: Some("🌙"),
                variant: ButtonVariant::Toggle,
                style: ButtonStyle::new(ButtonVariant::Toggle),
                state: ButtonState::toggled(false),
            },
            ButtonDef {
                label: "Disabled",
                icon: None,
                variant: ButtonVariant::SingleLine,
                style: ButtonStyle::default(),
                state: ButtonState::disabled(),
            },
        ];

        for i in 0..buttons.len() {
            focus.register(i);
        }

        Self {
            buttons,
            focus,
            click_regions: ClickRegionRegistry::new(),
            last_clicked: None,
            toast_state: ToastState::new(),
            should_quit: false,
        }
    }

    fn activate_current(&mut self) {
        if let Some(&idx) = self.focus.current() {
            self.activate_button(idx);
        }
    }

    fn activate_button(&mut self, idx: usize) {
        if !self.buttons[idx].state.enabled {
            return;
        }

        self.last_clicked = Some(idx);

        // Show toast notification
        let button_label = self.buttons[idx].label;
        let message = if self.buttons[idx].variant == ButtonVariant::Toggle {
            let is_on = !self.buttons[idx].state.toggled;
            format!("'{}' {}", button_label, if is_on { "ON" } else { "OFF" })
        } else {
            format!("'{}' clicked!", button_label)
        };
        self.toast_state.show(message, 1500);

        // Handle toggle buttons
        if self.buttons[idx].variant == ButtonVariant::Toggle {
            self.buttons[idx].state.toggle();
        }
    }

    fn handle_click(&mut self, col: u16, row: u16) {
        if let Some(&idx) = self.click_regions.handle_click(col, row) {
            self.focus.set(idx);
            self.activate_button(idx);
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

        // Read event ONCE and match on it
        match event::read()? {
            Event::Key(key) => {
                if is_close_key(&key) || key.code == KeyCode::Char('q') {
                    app.should_quit = true;
                } else if is_tab(&key) {
                    app.focus.next();
                } else if is_backtab(&key) {
                    app.focus.prev();
                } else if is_activate_key(&key) {
                    app.activate_current();
                }
            }
            Event::Mouse(mouse) => {
                if is_left_click(&mouse) {
                    app.handle_click(mouse.column, mouse.row);
                }
            }
            _ => {}
        }

        // Clear expired toasts
        app.toast_state.clear_if_expired();

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
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Content
            Constraint::Length(4), // Status + Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Button Demo")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Buttons - arrange in rows
    let button_area = chunks[1];
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Row 1: OK, Cancel
            Constraint::Length(2), // Row 2: Save, Delete
            Constraint::Length(4), // Row 3: Submit (block)
            Constraint::Length(2), // Row 4: Toggle, Disabled
            Constraint::Min(0),
        ])
        .split(button_area);

    let mut btn_idx = 0;

    // Row 1: OK, Cancel
    let row1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Min(0),
        ])
        .split(rows[0]);

    for (i, area) in row1.iter().take(2).enumerate() {
        let idx = btn_idx + i;
        render_button(f, app, idx, *area);
    }
    btn_idx += 2;

    // Row 2: Save, Delete
    let row2 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(14),
            Constraint::Length(14),
            Constraint::Min(0),
        ])
        .split(rows[1]);

    for (i, area) in row2.iter().take(2).enumerate() {
        let idx = btn_idx + i;
        render_button(f, app, idx, *area);
    }
    btn_idx += 2;

    // Row 3: Submit (block button)
    let submit_area = Rect::new(rows[2].x, rows[2].y, 20, 3);
    render_button(f, app, btn_idx, submit_area);
    btn_idx += 1;

    // Row 4: Toggle, Disabled
    let row4 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18),
            Constraint::Length(14),
            Constraint::Min(0),
        ])
        .split(rows[3]);

    for (i, area) in row4.iter().take(2).enumerate() {
        let idx = btn_idx + i;
        render_button(f, app, idx, *area);
    }

    // Status and help
    let status_text = if let Some(idx) = app.last_clicked {
        format!("Last clicked: {}", app.buttons[idx].label)
    } else {
        "Click a button!".to_string()
    };

    let help_lines = vec![
        Line::from(Span::styled(status_text, Style::default().fg(Color::Green))),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Next  "),
            Span::styled("Shift+Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Prev  "),
            Span::styled("Space/Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Activate  "),
            Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];
    let help = Paragraph::new(help_lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);

    // Render toast notifications on top
    if let Some(message) = app.toast_state.get_message() {
        let toast = Toast::new(message).auto_style();
        toast.render_with_clear(area, f.buffer_mut());
    }
}

fn render_button(f: &mut Frame, app: &mut App, idx: usize, area: Rect) {
    let btn_def = &mut app.buttons[idx];

    // Set focus state
    btn_def.state.focused = app.focus.is_focused(&idx);

    let mut button = Button::new(btn_def.label, &btn_def.state)
        .variant(btn_def.variant)
        .style(btn_def.style.clone());

    if let Some(icon) = btn_def.icon {
        button = button.icon(icon);
    }

    // Use convenience method - renders and registers click region in one call
    button.render_with_registry(area, f.buffer_mut(), &mut app.click_regions, idx);
}
