//! Dialog Demo
//!
//! Interactive demo showing PopupDialog with all components:
//! - CheckBox for options
//! - Input for text entry
//! - Focus management and Tab navigation
//! - Mouse click support
//!
//! Run with: cargo run --example dialog_demo

use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use tui_extension::{
    components::{
        CheckBox, CheckBoxState, CheckBoxStyle, DialogConfig, DialogFocusTarget, DialogState,
        Input, InputState, PopupDialog,
    },
    events::{
        get_char, is_activate_key, is_backspace, is_close_key, is_delete, is_end, is_home,
        is_left_click,
    },
    traits::{ClickRegionRegistry, ContainerAction, EventResult},
};

/// Content state for our settings dialog
#[derive(Debug, Default)]
struct SettingsContent {
    /// Username input
    username: InputState,
    /// Email input
    email: InputState,
    /// Dark mode checkbox
    dark_mode: CheckBoxState,
    /// Notifications checkbox
    notifications: CheckBoxState,
    /// Auto-save checkbox
    auto_save: CheckBoxState,
    /// Click regions for children
    click_regions: ClickRegionRegistry<usize>,
}

impl SettingsContent {
    fn new() -> Self {
        Self {
            username: InputState::new("JohnDoe"),
            email: InputState::new("john@example.com"),
            dark_mode: CheckBoxState::new(false),
            notifications: CheckBoxState::new(true),
            auto_save: CheckBoxState::new(true),
            click_regions: ClickRegionRegistry::new(),
        }
    }
}

/// Application state
struct App {
    /// Dialog configuration
    config: DialogConfig,
    /// Dialog state
    dialog_state: DialogState<SettingsContent>,
    /// Main screen message
    message: String,
    /// Should quit
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let config = DialogConfig::new("⚙ Settings")
            .width_percent(60)
            .height_percent(60)
            .min_size(50, 15)
            .ok_cancel();

        let mut dialog_state = DialogState::new(SettingsContent::new());

        // Register focusable children (5 fields)
        for i in 0..5 {
            dialog_state.register_child(i);
        }
        // Register buttons (OK and Cancel)
        dialog_state.register_button(0);
        dialog_state.register_button(1);

        // Start with dialog visible
        dialog_state.show();

        Self {
            config,
            dialog_state,
            message: "Press 'S' to open Settings dialog".to_string(),
            should_quit: false,
        }
    }

    fn handle_dialog_content_key(&mut self, key_code: KeyCode, key: &crossterm::event::KeyEvent) {
        // Copy focus target to avoid borrow issues
        let focus_target = self.dialog_state.current_focus().cloned();

        if let Some(DialogFocusTarget::Child(idx)) = focus_target {
            let content = &mut self.dialog_state.children;
            match idx {
                0 => {
                    // Username input
                    if let Some(c) = get_char(key) {
                        content.username.insert_char(c);
                    } else if is_backspace(key) {
                        content.username.delete_char_backward();
                    } else if is_delete(key) {
                        content.username.delete_char_forward();
                    } else if key_code == KeyCode::Left {
                        content.username.move_left();
                    } else if key_code == KeyCode::Right {
                        content.username.move_right();
                    } else if is_home(key) {
                        content.username.move_home();
                    } else if is_end(key) {
                        content.username.move_end();
                    }
                }
                1 => {
                    // Email input
                    if let Some(c) = get_char(key) {
                        content.email.insert_char(c);
                    } else if is_backspace(key) {
                        content.email.delete_char_backward();
                    } else if is_delete(key) {
                        content.email.delete_char_forward();
                    } else if key_code == KeyCode::Left {
                        content.email.move_left();
                    } else if key_code == KeyCode::Right {
                        content.email.move_right();
                    } else if is_home(key) {
                        content.email.move_home();
                    } else if is_end(key) {
                        content.email.move_end();
                    }
                }
                2 => {
                    // Dark mode checkbox
                    if is_activate_key(key) {
                        content.dark_mode.toggle();
                    }
                }
                3 => {
                    // Notifications checkbox
                    if is_activate_key(key) {
                        content.notifications.toggle();
                    }
                }
                4 => {
                    // Auto-save checkbox
                    if is_activate_key(key) {
                        content.auto_save.toggle();
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_dialog_click(&mut self, col: u16, row: u16) {
        let clicked_idx = self.dialog_state.children.click_regions.handle_click(col, row).cloned();

        if let Some(idx) = clicked_idx {
            // Set focus to clicked child
            self.dialog_state.focus.set(DialogFocusTarget::Child(idx));

            // Toggle checkboxes on click
            let content = &mut self.dialog_state.children;
            match idx {
                2 => content.dark_mode.toggle(),
                3 => content.notifications.toggle(),
                4 => content.auto_save.toggle(),
                _ => {}
            }
        }
    }

    fn get_settings_summary(&self) -> String {
        let content = &self.dialog_state.children;
        format!(
            "Settings saved! User: {}, Email: {}, Dark: {}, Notify: {}, AutoSave: {}",
            content.username.text(),
            content.email.text(),
            content.dark_mode.checked,
            content.notifications.checked,
            content.auto_save.checked,
        )
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
            if app.dialog_state.is_visible() {
                // Dialog is open - handle dialog events
                let mut dialog = PopupDialog::new(
                    &app.config,
                    &mut app.dialog_state,
                    |_, _, _| {}, // Empty renderer for event handling
                );

                let result = dialog.handle_key(key);
                drop(dialog); // Drop dialog to release borrow

                match result {
                    EventResult::Action(ContainerAction::Submit) => {
                        app.message = app.get_settings_summary();
                    }
                    EventResult::Action(ContainerAction::Close) => {
                        app.message = "Settings cancelled".to_string();
                    }
                    EventResult::NotHandled => {
                        // Handle content-specific keys
                        app.handle_dialog_content_key(key.code, &key);
                    }
                    _ => {}
                }
            } else {
                // Main screen
                if is_close_key(&key) || key.code == KeyCode::Char('q') {
                    app.should_quit = true;
                } else if key.code == KeyCode::Char('s') || key.code == KeyCode::Char('S') {
                    app.dialog_state.show();
                    app.dialog_state.focus.first();
                }
            }
        } else if let Event::Mouse(mouse) = event::read().unwrap_or(Event::FocusGained) {
            if app.dialog_state.is_visible() && is_left_click(&mouse) {
                // Handle dialog mouse events
                let screen = terminal.get_frame().area();
                let mut dialog = PopupDialog::new(&app.config, &mut app.dialog_state, |_, _, _| {});

                let result = dialog.handle_mouse_with_screen(mouse, screen);
                drop(dialog); // Drop dialog to release borrow

                match result {
                    EventResult::Action(ContainerAction::Submit) => {
                        app.message = app.get_settings_summary();
                    }
                    EventResult::Action(ContainerAction::Close) => {
                        app.message = "Settings cancelled".to_string();
                    }
                    EventResult::NotHandled => {
                        // Check content click regions
                        app.handle_dialog_click(mouse.column, mouse.row);
                    }
                    _ => {}
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
        .title(" Dialog Demo ")
        .border_style(Style::default().fg(Color::Blue));
    f.render_widget(background, area);

    // Main content
    let inner = Rect::new(area.x + 2, area.y + 2, area.width - 4, area.height - 4);

    let help_lines = vec![
        Line::from(Span::styled(
            &app.message,
            Style::default().fg(Color::Green),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("S", Style::default().fg(Color::Yellow)),
            Span::raw(": Open Settings  "),
            Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];
    let help = Paragraph::new(help_lines);
    f.render_widget(help, inner);

    // Render dialog if visible
    if app.dialog_state.is_visible() {
        // Render settings dialog content
        render_dialog(f, app);
    }
}

fn render_dialog(f: &mut Frame, app: &mut App) {
    // Compute focus states first
    let focus_states = [
        app.dialog_state.focus.is_focused(&DialogFocusTarget::Child(0)),
        app.dialog_state.focus.is_focused(&DialogFocusTarget::Child(1)),
        app.dialog_state.focus.is_focused(&DialogFocusTarget::Child(2)),
        app.dialog_state.focus.is_focused(&DialogFocusTarget::Child(3)),
        app.dialog_state.focus.is_focused(&DialogFocusTarget::Child(4)),
    ];

    let mut dialog = PopupDialog::new(&app.config, &mut app.dialog_state, |frame, area, content| {
        render_settings_content(frame, area, content, &focus_states);
    });
    dialog.render(f);
}

fn render_settings_content(
    f: &mut Frame,
    area: Rect,
    content: &mut SettingsContent,
    focus_states: &[bool; 5],
) {
    // Clear click regions
    content.click_regions.clear();

    // Layout: inputs then checkboxes
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Username
            Constraint::Length(3), // Email
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Dark mode
            Constraint::Length(1), // Notifications
            Constraint::Length(1), // Auto-save
            Constraint::Min(0),    // Remaining space
        ])
        .split(area);

    // Username input (child 0)
    content.username.focused = focus_states[0];
    let input = Input::new(&content.username).label("Username");
    let region = input.render_stateful(f, chunks[0]);
    content.click_regions.register(region.area, 0);

    // Email input (child 1)
    content.email.focused = focus_states[1];
    let input = Input::new(&content.email).label("Email");
    let region = input.render_stateful(f, chunks[1]);
    content.click_regions.register(region.area, 1);

    // Dark mode checkbox (child 2)
    content.dark_mode.focused = focus_states[2];
    let checkbox =
        CheckBox::new("Dark Mode", &content.dark_mode).style(CheckBoxStyle::unicode());
    let region = checkbox.render_stateful(chunks[3], f.buffer_mut());
    content.click_regions.register(region.area, 2);

    // Notifications checkbox (child 3)
    content.notifications.focused = focus_states[3];
    let checkbox = CheckBox::new("Enable Notifications", &content.notifications)
        .style(CheckBoxStyle::unicode());
    let region = checkbox.render_stateful(chunks[4], f.buffer_mut());
    content.click_regions.register(region.area, 3);

    // Auto-save checkbox (child 4)
    content.auto_save.focused = focus_states[4];
    let checkbox =
        CheckBox::new("Auto-save", &content.auto_save).style(CheckBoxStyle::unicode());
    let region = checkbox.render_stateful(chunks[5], f.buffer_mut());
    content.click_regions.register(region.area, 4);
}
