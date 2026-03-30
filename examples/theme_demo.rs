//! Theme Demo
//!
//! Interactive demo showing the theme system:
//! - Press 't' to toggle between dark and light themes
//! - Shows Button, Input, CheckBox, Progress widgets
//!   all styled from a single Theme
//!
//! Run with: cargo run --example theme_demo

use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use ratatui_interact::{
    components::{
        Button, ButtonState, CheckBox, CheckBoxState, Input, InputState, Progress, SpinnerStyle,
    },
    events::{is_activate_key, is_backtab, is_close_key, is_left_click, is_tab},
    state::FocusManager,
    theme::Theme,
    traits::ClickRegionRegistry,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Focus {
    Button1,
    Button2,
    CheckBox1,
    CheckBox2,
    Input,
}

struct App {
    theme: Theme,
    is_dark: bool,
    focus: FocusManager<Focus>,
    click_regions: ClickRegionRegistry<Focus>,
    button1_state: ButtonState,
    button2_state: ButtonState,
    checkbox1_state: CheckBoxState,
    checkbox2_state: CheckBoxState,
    input_state: InputState,
    progress_value: f64,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let mut focus = FocusManager::new();
        focus.register(Focus::Button1);
        focus.register(Focus::Button2);
        focus.register(Focus::CheckBox1);
        focus.register(Focus::CheckBox2);
        focus.register(Focus::Input);

        Self {
            theme: Theme::dark(),
            is_dark: true,
            focus,
            click_regions: ClickRegionRegistry::new(),
            button1_state: ButtonState::enabled(),
            button2_state: ButtonState::enabled(),
            checkbox1_state: CheckBoxState::new(true),
            checkbox2_state: CheckBoxState::new(false),
            input_state: InputState::new("Hello, themes!"),
            progress_value: 0.65,
            should_quit: false,
        }
    }

    fn toggle_theme(&mut self) {
        self.is_dark = !self.is_dark;
        self.theme = if self.is_dark {
            Theme::dark()
        } else {
            Theme::light()
        };
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Ok(event) = event::read() {
            match event {
                Event::Key(key) => {
                    if is_close_key(&key) || key.code == KeyCode::Char('q') {
                        app.should_quit = true;
                    } else if key.code == KeyCode::Char('t') {
                        app.toggle_theme();
                    } else if is_tab(&key) {
                        app.focus.next();
                    } else if is_backtab(&key) {
                        app.focus.prev();
                    } else if is_activate_key(&key) {
                        if app.focus.is_focused(&Focus::CheckBox1) {
                            app.checkbox1_state.toggle();
                        } else if app.focus.is_focused(&Focus::CheckBox2) {
                            app.checkbox2_state.toggle();
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    if is_left_click(&mouse) {
                        if let Some(id) = app.click_regions.handle_click(mouse.column, mouse.row) {
                            app.focus.set(*id);
                            match id {
                                Focus::CheckBox1 => app.checkbox1_state.toggle(),
                                Focus::CheckBox2 => app.checkbox2_state.toggle(),
                                _ => {}
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
    app.click_regions.clear();

    let theme = &app.theme;
    let p = &theme.palette;
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(2), // Buttons
            Constraint::Length(2), // Checkboxes
            Constraint::Length(3), // Input
            Constraint::Length(3), // Progress
            Constraint::Length(3), // Theme info
            Constraint::Min(0),    // Spacer
            Constraint::Length(3), // Help
        ])
        .split(area);

    // Title
    let title_text = format!(
        " Theme Demo - Current: {} (press 't' to toggle) ",
        app.theme.name
    );
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(p.primary).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(p.border)),
        );
    f.render_widget(title, chunks[0]);

    // Buttons
    let button_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Min(0),
        ])
        .split(chunks[1]);

    app.button1_state.focused = app.focus.is_focused(&Focus::Button1);
    app.button2_state.focused = app.focus.is_focused(&Focus::Button2);

    let btn1 = Button::new("OK", &app.button1_state).theme(theme);
    btn1.render_with_registry(
        button_row[0],
        f.buffer_mut(),
        &mut app.click_regions,
        Focus::Button1,
    );

    let btn2 = Button::new("Cancel", &app.button2_state).theme(theme);
    btn2.render_with_registry(
        button_row[1],
        f.buffer_mut(),
        &mut app.click_regions,
        Focus::Button2,
    );

    // Checkboxes
    let cb_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Min(0),
        ])
        .split(chunks[2]);

    app.checkbox1_state
        .set_focused(app.focus.is_focused(&Focus::CheckBox1));
    app.checkbox2_state
        .set_focused(app.focus.is_focused(&Focus::CheckBox2));

    let cb1 = CheckBox::new("Dark mode", &app.checkbox1_state).theme(theme);
    let _region1 = cb1.render_stateful(cb_row[0], f.buffer_mut());

    let cb2 = CheckBox::new("Notifications", &app.checkbox2_state).theme(theme);
    let _region2 = cb2.render_stateful(cb_row[1], f.buffer_mut());

    // Input
    app.input_state.focused = app.focus.is_focused(&Focus::Input);
    let input = Input::new(&app.input_state)
        .label("Name")
        .placeholder("Enter your name...")
        .theme(theme);
    let _region = input.render_stateful(f, chunks[3]);

    // Progress
    let progress = Progress::new(app.progress_value)
        .label("Loading")
        .theme(theme);
    progress.render(chunks[4], f.buffer_mut());

    // Theme info - show that all styles derive from the same theme
    let spinner_style: SpinnerStyle = theme.style();
    let info = Paragraph::new(format!(
        "All widgets above use the {:?} theme. SpinnerStyle spinner_fg={:?}",
        app.theme.name, spinner_style.spinner_style.fg
    ))
    .style(Style::default().fg(p.text_dim));
    f.render_widget(info, chunks[5]);

    // Help
    let help_lines = vec![Line::from(vec![
        Span::styled("t", Style::default().fg(p.primary)),
        Span::styled(": Toggle theme  ", Style::default().fg(p.text)),
        Span::styled("Tab", Style::default().fg(p.primary)),
        Span::styled(": Next  ", Style::default().fg(p.text)),
        Span::styled("Shift+Tab", Style::default().fg(p.primary)),
        Span::styled(": Prev  ", Style::default().fg(p.text)),
        Span::styled("q/Esc", Style::default().fg(p.primary)),
        Span::styled(": Quit", Style::default().fg(p.text)),
    ])];
    let help = Paragraph::new(help_lines).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(p.border)),
    );
    f.render_widget(help, chunks[7]);
}
