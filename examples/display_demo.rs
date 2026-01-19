//! Display Components Demo
//!
//! Interactive demo showing display/viewer components:
//! - Progress: Bar with percentage, steps mode, and various styles
//! - StepDisplay: Multi-step workflow with sub-steps and output
//! - ParagraphExt: Scrollable text with word-wrapping
//!
//! Keyboard controls:
//! - Tab: Switch between panels
//! - Up/Down: Scroll content (ParagraphExt) or change progress
//! - Space: Start/advance simulation
//! - r: Reset all
//! - q/Esc: Quit
//!
//! Run with: cargo run --example display_demo

use std::io;
use std::time::{Duration, Instant};

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
        ParagraphExt, Progress, ProgressStyle, Step, StepDisplay, StepDisplayState, StepStatus,
    },
    events::is_close_key,
};

/// Which panel is focused
#[derive(Clone, Copy, PartialEq)]
enum FocusedPanel {
    Progress,
    Steps,
    Paragraph,
}

impl FocusedPanel {
    fn next(self) -> Self {
        match self {
            FocusedPanel::Progress => FocusedPanel::Steps,
            FocusedPanel::Steps => FocusedPanel::Paragraph,
            FocusedPanel::Paragraph => FocusedPanel::Progress,
        }
    }

    fn prev(self) -> Self {
        match self {
            FocusedPanel::Progress => FocusedPanel::Paragraph,
            FocusedPanel::Steps => FocusedPanel::Progress,
            FocusedPanel::Paragraph => FocusedPanel::Steps,
        }
    }
}

/// Application state
struct App {
    /// Currently focused panel
    focused_panel: FocusedPanel,
    /// Should quit
    should_quit: bool,

    // Progress state
    progress_value: f64,
    progress_steps: (usize, usize),
    progress_style_idx: usize,

    // StepDisplay state
    step_state: StepDisplayState,
    simulation_running: bool,
    last_tick: Instant,

    // ParagraphExt state
    paragraph_scroll: u16,
    paragraph_content: Vec<Line<'static>>,
}

impl App {
    fn new() -> Self {
        // Create steps for the StepDisplay
        let steps = vec![
            Step::new("Initialize Project").with_sub_steps(vec![
                "Check dependencies",
                "Create directories",
                "Setup config",
            ]),
            Step::new("Build Source").with_sub_steps(vec![
                "Compile libraries",
                "Compile main",
                "Link binaries",
            ]),
            Step::new("Run Tests").with_sub_steps(vec!["Unit tests", "Integration tests"]),
            Step::new("Deploy"),
        ];
        let step_state = StepDisplayState::new(steps);

        // Create content for ParagraphExt
        let paragraph_content = vec![
            Line::from(vec![Span::styled(
                "ParagraphExt Component",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("This is a demonstration of the ParagraphExt widget which provides:"),
            Line::from(""),
            Line::from(vec![
                Span::styled("  - ", Style::default().fg(Color::Yellow)),
                Span::raw("Word-wrapping support for long lines of text"),
            ]),
            Line::from(vec![
                Span::styled("  - ", Style::default().fg(Color::Yellow)),
                Span::raw("Clean rendering without trailing spaces"),
            ]),
            Line::from(vec![
                Span::styled("  - ", Style::default().fg(Color::Yellow)),
                Span::raw("Per-character styling that survives wrapping"),
            ]),
            Line::from(vec![
                Span::styled("  - ", Style::default().fg(Color::Yellow)),
                Span::raw("Vertical scrolling for long content"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Usage: ", Style::default().fg(Color::Green)),
                Span::raw("Use Up/Down arrows to scroll this content when focused."),
            ]),
            Line::from(""),
            Line::from(
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor \
                incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis \
                nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
            ),
            Line::from(""),
            Line::from(
                "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu \
                fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in \
                culpa qui officia deserunt mollit anim id est laborum.",
            ),
            Line::from(""),
            Line::from(vec![
                Span::styled("Styled text: ", Style::default().fg(Color::Magenta)),
                Span::styled("Red ", Style::default().fg(Color::Red)),
                Span::styled("Green ", Style::default().fg(Color::Green)),
                Span::styled("Blue ", Style::default().fg(Color::Blue)),
                Span::styled("Yellow", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from("End of content. Thank you for scrolling!"),
        ];

        Self {
            focused_panel: FocusedPanel::Progress,
            should_quit: false,
            progress_value: 0.0,
            progress_steps: (0, 10),
            progress_style_idx: 0,
            step_state,
            simulation_running: false,
            last_tick: Instant::now(),
            paragraph_scroll: 0,
            paragraph_content,
        }
    }

    fn reset(&mut self) {
        self.progress_value = 0.0;
        self.progress_steps = (0, 10);
        self.simulation_running = false;

        // Reset step state
        let steps = vec![
            Step::new("Initialize Project").with_sub_steps(vec![
                "Check dependencies",
                "Create directories",
                "Setup config",
            ]),
            Step::new("Build Source").with_sub_steps(vec![
                "Compile libraries",
                "Compile main",
                "Link binaries",
            ]),
            Step::new("Run Tests").with_sub_steps(vec!["Unit tests", "Integration tests"]),
            Step::new("Deploy"),
        ];
        self.step_state = StepDisplayState::new(steps);
        self.paragraph_scroll = 0;
    }

    fn tick(&mut self) {
        if !self.simulation_running {
            return;
        }

        let now = Instant::now();
        if now.duration_since(self.last_tick) < Duration::from_millis(400) {
            return;
        }
        self.last_tick = now;

        // Advance progress
        if self.progress_value < 1.0 {
            self.progress_value = (self.progress_value + 0.1).min(1.0);
            self.progress_steps.0 = ((self.progress_value * 10.0) as usize).min(10);
        }

        // Advance step simulation
        self.advance_steps();
    }

    fn advance_steps(&mut self) {
        let current = self.step_state.current_step();
        if current >= self.step_state.steps.len() {
            self.simulation_running = false;
            return;
        }

        // Check step status first
        let step_status = self.step_state.steps[current].status;
        if step_status == StepStatus::Pending {
            self.step_state.start_step(current);
            self.step_state
                .add_output(current, format!("Starting step {}...", current + 1));
            return;
        }

        // Advance sub-steps - gather info without holding borrow
        let sub_steps_info: Vec<(usize, StepStatus, String)> = self.step_state.steps[current]
            .sub_steps
            .iter()
            .enumerate()
            .map(|(idx, sub)| (idx, sub.status, sub.name.clone()))
            .collect();

        let (completed, total) = self.step_state.steps[current].sub_step_progress();

        if !sub_steps_info.is_empty() && completed < total {
            // Find first non-completed sub-step
            for (sub_idx, status, name) in sub_steps_info {
                if status == StepStatus::Pending {
                    self.step_state.start_sub_step(current, sub_idx);
                    self.step_state
                        .add_output(current, format!("  Running: {}", name));
                    return;
                } else if status == StepStatus::Running {
                    self.step_state.complete_sub_step(current, sub_idx);
                    self.step_state
                        .add_output(current, format!("  Completed: {}", name));
                    return;
                }
            }
        }

        // Complete step
        self.step_state
            .add_output(current, "Step completed successfully!");
        self.step_state.complete_step(current);
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Tab => {
                self.focused_panel = self.focused_panel.next();
            }
            KeyCode::BackTab => {
                self.focused_panel = self.focused_panel.prev();
            }
            KeyCode::Char('r') => {
                self.reset();
            }
            KeyCode::Char(' ') => {
                self.simulation_running = !self.simulation_running;
                self.last_tick = Instant::now();
            }
            KeyCode::Up => match self.focused_panel {
                FocusedPanel::Progress => {
                    self.progress_value = (self.progress_value + 0.1).min(1.0);
                    self.progress_steps.0 = (self.progress_steps.0 + 1).min(10);
                }
                FocusedPanel::Steps => {
                    // Scroll step display up
                    self.step_state.scroll = self.step_state.scroll.saturating_sub(1);
                }
                FocusedPanel::Paragraph => {
                    self.paragraph_scroll = self.paragraph_scroll.saturating_sub(1);
                }
            },
            KeyCode::Down => match self.focused_panel {
                FocusedPanel::Progress => {
                    self.progress_value = (self.progress_value - 0.1).max(0.0);
                    self.progress_steps.0 = self.progress_steps.0.saturating_sub(1);
                }
                FocusedPanel::Steps => {
                    // Scroll step display down
                    self.step_state.scroll += 1;
                }
                FocusedPanel::Paragraph => {
                    self.paragraph_scroll += 1;
                }
            },
            KeyCode::Left => {
                if self.focused_panel == FocusedPanel::Progress {
                    self.progress_style_idx = (self.progress_style_idx + 3) % 4;
                }
            }
            KeyCode::Right => {
                if self.focused_panel == FocusedPanel::Progress {
                    self.progress_style_idx = (self.progress_style_idx + 1) % 4;
                }
            }
            _ => {}
        }
    }

    fn get_progress_style(&self) -> ProgressStyle {
        match self.progress_style_idx {
            0 => ProgressStyle::success(),
            1 => ProgressStyle::warning(),
            2 => ProgressStyle::error(),
            _ => ProgressStyle::info(),
        }
    }

    fn get_style_name(&self) -> &'static str {
        match self.progress_style_idx {
            0 => "Success (Green)",
            1 => "Warning (Yellow)",
            2 => "Error (Red)",
            _ => "Info (Cyan)",
        }
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
        terminal.draw(|f| ui(f, &mut app))?;

        // Non-blocking event handling
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if is_close_key(&key) || key.code == KeyCode::Char('q') {
                    app.should_quit = true;
                } else {
                    app.handle_key(key.code);
                }
            }
        }

        // Tick for simulation
        app.tick();

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

fn ui(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Content
            Constraint::Length(4), // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "Display Components Demo",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "Progress + StepDisplay + ParagraphExt",
            Style::default().fg(Color::DarkGray),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Content area - split into three vertical sections
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Progress section
            Constraint::Min(10),    // StepDisplay section
            Constraint::Length(12), // ParagraphExt section
        ])
        .split(chunks[1]);

    // Progress Panel
    render_progress_panel(f, app, content_chunks[0]);

    // StepDisplay Panel
    render_steps_panel(f, app, content_chunks[1]);

    // ParagraphExt Panel
    render_paragraph_panel(f, app, content_chunks[2]);

    // Help text
    let focus_text = match app.focused_panel {
        FocusedPanel::Progress => "Progress",
        FocusedPanel::Steps => "Steps",
        FocusedPanel::Paragraph => "Paragraph",
    };

    let sim_status = if app.simulation_running {
        Span::styled("Running", Style::default().fg(Color::Green))
    } else {
        Span::styled("Paused", Style::default().fg(Color::Yellow))
    };

    let help_lines = vec![
        Line::from(vec![
            Span::styled("Focus: ", Style::default().fg(Color::Gray)),
            Span::styled(focus_text, Style::default().fg(Color::Green)),
            Span::raw("  |  "),
            Span::styled("Simulation: ", Style::default().fg(Color::Gray)),
            sim_status,
        ]),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Switch panel  "),
            Span::styled("Up/Down", Style::default().fg(Color::Yellow)),
            Span::raw(": Adjust/Scroll  "),
            Span::styled("Left/Right", Style::default().fg(Color::Yellow)),
            Span::raw(": Change style  "),
        ]),
        Line::from(vec![
            Span::styled("Space", Style::default().fg(Color::Yellow)),
            Span::raw(": Start/Pause sim  "),
            Span::styled("r", Style::default().fg(Color::Yellow)),
            Span::raw(": Reset  "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];
    let help = Paragraph::new(help_lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}

fn render_progress_panel(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Progress;

    let block = Block::default()
        .title(Span::styled(
            " Progress ",
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

    // Layout for progress bars
    let progress_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Style label
            Constraint::Length(3), // Simple progress
            Constraint::Length(3), // Steps progress
        ])
        .split(inner);

    // Style label
    let style_label = Paragraph::new(vec![Line::from(vec![
        Span::raw("Style: "),
        Span::styled(app.get_style_name(), Style::default().fg(Color::Cyan)),
        Span::raw(" (use Left/Right to change)"),
    ])]);
    f.render_widget(style_label, progress_chunks[0]);

    // Simple progress bar
    let progress1 = Progress::new(app.progress_value)
        .label("Download")
        .style(app.get_progress_style());
    progress1.render(progress_chunks[1], f.buffer_mut());

    // Steps progress bar
    let progress2 = Progress::from_steps(app.progress_steps.0, app.progress_steps.1)
        .label("Processing")
        .style(app.get_progress_style());
    progress2.render(progress_chunks[2], f.buffer_mut());
}

fn render_steps_panel(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Steps;

    let block = Block::default()
        .title(Span::styled(
            " StepDisplay ",
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

    // Render step display
    let step_display = StepDisplay::new(&app.step_state);
    step_display.render(inner, f.buffer_mut());
}

fn render_paragraph_panel(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Paragraph;

    let block = Block::default()
        .title(Span::styled(
            " ParagraphExt ",
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

    // Render paragraph with scrolling
    let paragraph = ParagraphExt::new(app.paragraph_content.clone())
        .scroll(app.paragraph_scroll)
        .width(inner.width);
    paragraph.render(inner, f.buffer_mut());
}
