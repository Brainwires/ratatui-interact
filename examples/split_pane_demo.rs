//! Split Pane Demo
//!
//! Interactive demo showing split pane features:
//! - Drag-to-resize divider with mouse
//! - Keyboard-based resize with arrow keys
//! - Horizontal (left/right) and vertical (top/bottom) orientations
//! - Toggle orientation with 'o' key
//! - Nested split panes for complex layouts
//!
//! Run with: cargo run --example split_pane_demo

use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use ratatui_interact::{
    components::{
        Orientation, SplitPane, SplitPaneAction, SplitPaneState, SplitPaneStyle,
        handle_split_pane_key, handle_split_pane_mouse,
    },
    events::is_close_key,
    traits::ClickRegionRegistry,
};

/// Application state
struct App {
    /// Main split pane state
    split_state: SplitPaneState,
    /// Nested split pane state (inside the first pane)
    nested_split_state: SplitPaneState,
    /// Click regions for main split
    main_registry: ClickRegionRegistry<SplitPaneAction>,
    /// Click regions for nested split
    nested_registry: ClickRegionRegistry<SplitPaneAction>,
    /// Current orientation for main split
    orientation: Orientation,
    /// Nested split orientation
    nested_orientation: Orientation,
    /// Which split is focused (main or nested)
    focused_split: FocusedSplit,
    /// Should quit
    should_quit: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum FocusedSplit {
    Main,
    Nested,
}

impl App {
    fn new() -> Self {
        let mut split_state = SplitPaneState::new(50);
        split_state.focused = true;
        split_state.divider_focused = true;

        Self {
            split_state,
            nested_split_state: SplitPaneState::new(50),
            main_registry: ClickRegionRegistry::new(),
            nested_registry: ClickRegionRegistry::new(),
            orientation: Orientation::Horizontal,
            nested_orientation: Orientation::Vertical,
            focused_split: FocusedSplit::Main,
            should_quit: false,
        }
    }

    fn toggle_orientation(&mut self) {
        match self.focused_split {
            FocusedSplit::Main => {
                self.orientation = match self.orientation {
                    Orientation::Horizontal => Orientation::Vertical,
                    Orientation::Vertical => Orientation::Horizontal,
                };
            }
            FocusedSplit::Nested => {
                self.nested_orientation = match self.nested_orientation {
                    Orientation::Horizontal => Orientation::Vertical,
                    Orientation::Vertical => Orientation::Horizontal,
                };
            }
        }
    }

    fn toggle_focus(&mut self) {
        self.focused_split = match self.focused_split {
            FocusedSplit::Main => {
                self.split_state.divider_focused = false;
                self.nested_split_state.divider_focused = true;
                FocusedSplit::Nested
            }
            FocusedSplit::Nested => {
                self.nested_split_state.divider_focused = false;
                self.split_state.divider_focused = true;
                FocusedSplit::Main
            }
        };
    }

    fn reset_split(&mut self) {
        match self.focused_split {
            FocusedSplit::Main => self.split_state.set_split_percent(50),
            FocusedSplit::Nested => self.nested_split_state.set_split_percent(50),
        }
    }

    fn handle_mouse(&mut self, mouse: &crossterm::event::MouseEvent) {
        // Handle main split
        let action = handle_split_pane_mouse(
            &mut self.split_state,
            mouse,
            self.orientation,
            &self.main_registry,
            10,
            90,
        );

        // Handle nested split
        let nested_action = handle_split_pane_mouse(
            &mut self.nested_split_state,
            mouse,
            self.nested_orientation,
            &self.nested_registry,
            10,
            90,
        );

        // Update focus based on where user clicked
        if let Some(action) = action {
            if action == SplitPaneAction::DividerDrag {
                self.focused_split = FocusedSplit::Main;
                self.split_state.divider_focused = true;
                self.nested_split_state.divider_focused = false;
            }
        }
        if let Some(action) = nested_action {
            if action == SplitPaneAction::DividerDrag {
                self.focused_split = FocusedSplit::Nested;
                self.nested_split_state.divider_focused = true;
                self.split_state.divider_focused = false;
            }
        }
    }

    fn handle_key(&mut self, key: &crossterm::event::KeyEvent) {
        let (state, orientation) = match self.focused_split {
            FocusedSplit::Main => (&mut self.split_state, self.orientation),
            FocusedSplit::Nested => (&mut self.nested_split_state, self.nested_orientation),
        };

        handle_split_pane_key(state, key, orientation, 5, 10, 90);
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

        match event::read()? {
            Event::Key(key) => {
                if is_close_key(&key) || key.code == KeyCode::Char('q') {
                    app.should_quit = true;
                } else if key.code == KeyCode::Char('o') {
                    app.toggle_orientation();
                } else if key.code == KeyCode::Tab {
                    app.toggle_focus();
                } else if key.code == KeyCode::Char('r') {
                    app.reset_split();
                } else {
                    app.handle_key(&key);
                }
            }
            Event::Mouse(mouse) => {
                app.handle_mouse(&mouse);
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

fn ui(f: &mut Frame, app: &mut App) {
    // Clear click regions
    app.main_registry.clear();
    app.nested_registry.clear();

    let area = f.area();

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Split pane
            Constraint::Length(5), // Help
        ])
        .split(area);

    // Title
    let orientation_name = match app.orientation {
        Orientation::Horizontal => "Horizontal (Left | Right)",
        Orientation::Vertical => "Vertical (Top / Bottom)",
    };
    let title = Paragraph::new(format!("Split Pane Demo - {}", orientation_name))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Calculate main split areas first
    let main_style = if app.focused_split == FocusedSplit::Main {
        SplitPaneStyle::prominent()
    } else {
        SplitPaneStyle::default()
    };

    let split_pane = SplitPane::new(&app.split_state)
        .orientation(app.orientation)
        .style(main_style.clone())
        .min_percent(10)
        .max_percent(90);

    let (first_area, divider_area, second_area) = split_pane.calculate_areas(chunks[1]);

    // Update total size for drag calculations
    let total_size = match app.orientation {
        Orientation::Horizontal => chunks[1].width,
        Orientation::Vertical => chunks[1].height,
    };
    app.split_state.set_total_size(total_size);

    // Register main click regions
    app.main_registry.register(first_area, SplitPaneAction::FirstPaneClick);
    app.main_registry.register(divider_area, SplitPaneAction::DividerDrag);
    app.main_registry.register(second_area, SplitPaneAction::SecondPaneClick);

    // Render first pane with nested split
    let block1 = Block::default()
        .title(" Pane 1 (with nested split) ")
        .borders(Borders::ALL)
        .border_style(if app.focused_split == FocusedSplit::Nested {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        });
    let inner1 = block1.inner(first_area);
    f.render_widget(block1, first_area);

    // Render nested split pane
    render_nested_split(f, app, inner1);

    // Render main divider
    render_divider(f.buffer_mut(), divider_area, app.orientation, &main_style,
                   app.split_state.is_dragging(), app.split_state.divider_focused);

    // Render second pane
    let block2 = Block::default()
        .title(" Pane 2 ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner2 = block2.inner(second_area);
    f.render_widget(block2, second_area);

    let percent = 100 - app.split_state.split_percent();
    render_pane_content("Main Right", percent, inner2, f.buffer_mut());

    // Help text
    render_help(f, chunks[2], app);
}

fn render_nested_split(f: &mut Frame, app: &mut App, area: Rect) {
    let nested_style = if app.focused_split == FocusedSplit::Nested {
        SplitPaneStyle::prominent()
    } else {
        SplitPaneStyle::minimal()
    };

    let nested_split = SplitPane::new(&app.nested_split_state)
        .orientation(app.nested_orientation)
        .style(nested_style.clone())
        .min_percent(10)
        .max_percent(90);

    let (nested_first, nested_divider, nested_second) = nested_split.calculate_areas(area);

    // Update total size for nested drag calculations
    let nested_total_size = match app.nested_orientation {
        Orientation::Horizontal => area.width,
        Orientation::Vertical => area.height,
    };
    app.nested_split_state.set_total_size(nested_total_size);

    // Register nested click regions
    app.nested_registry.register(nested_first, SplitPaneAction::FirstPaneClick);
    app.nested_registry.register(nested_divider, SplitPaneAction::DividerDrag);
    app.nested_registry.register(nested_second, SplitPaneAction::SecondPaneClick);

    // Render nested content
    render_pane_content("Nested 1", app.nested_split_state.split_percent(), nested_first, f.buffer_mut());
    render_pane_content("Nested 2", 100 - app.nested_split_state.split_percent(), nested_second, f.buffer_mut());

    // Render nested divider
    render_divider(f.buffer_mut(), nested_divider, app.nested_orientation, &nested_style,
                   app.nested_split_state.is_dragging(), app.nested_split_state.divider_focused);
}

fn render_divider(
    buf: &mut ratatui::buffer::Buffer,
    area: Rect,
    orientation: Orientation,
    style: &SplitPaneStyle,
    is_dragging: bool,
    is_focused: bool,
) {
    let divider_style = if is_dragging {
        style.divider_dragging_style
    } else if is_focused {
        style.divider_focused_style
    } else {
        style.divider_style
    };

    let divider_char = match orientation {
        Orientation::Horizontal => "│",
        Orientation::Vertical => "─",
    };

    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            buf.set_string(x, y, divider_char, divider_style);
        }
    }
}

fn render_help(f: &mut Frame, area: Rect, app: &App) {
    let focus_text = match app.focused_split {
        FocusedSplit::Main => "Main divider",
        FocusedSplit::Nested => "Nested divider",
    };

    let help_lines = vec![
        Line::from(vec![
            Span::styled("Focus: ", Style::default().fg(Color::Gray)),
            Span::styled(focus_text, Style::default().fg(Color::Green)),
            Span::raw(" | "),
            Span::styled("Main split: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}%", app.split_state.split_percent()),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" | "),
            Span::styled("Nested split: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}%", app.nested_split_state.split_percent()),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("Drag", Style::default().fg(Color::Yellow)),
            Span::raw(": Resize with mouse  "),
            Span::styled("Arrow keys", Style::default().fg(Color::Yellow)),
            Span::raw(": Resize focused  "),
            Span::styled("Home/End", Style::default().fg(Color::Yellow)),
            Span::raw(": Min/Max"),
        ]),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Switch focus  "),
            Span::styled("o", Style::default().fg(Color::Yellow)),
            Span::raw(": Toggle orientation  "),
            Span::styled("r", Style::default().fg(Color::Yellow)),
            Span::raw(": Reset to 50%  "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];
    let help = Paragraph::new(help_lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(help, area);
}

fn render_pane_content(name: &str, percent: u16, area: Rect, buf: &mut ratatui::buffer::Buffer) {
    let text = vec![
        Line::from(Span::styled(
            name,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Size: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}%", percent), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Width: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", area.width), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Height: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", area.height), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Drag the divider to resize",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "or use arrow keys",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )),
    ];
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    paragraph.render(area, buf);
}
