//! Toast Stack Demo
//!
//! Interactive demo showing stacked toast notification features:
//! - Auto-dismiss and manual (persistent) toasts
//! - Mouse click to dismiss individual toasts
//! - Keyboard dismiss (Esc = top, d = all)
//! - Configurable placement and stacking order
//!
//! Run with: cargo run --example toast_stack_demo

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use ratatui_interact::components::{
    ToastOrder, ToastPlacement, ToastStack, ToastStackLayout, ToastStackState, ToastStyle,
};
use ratatui_interact::events::is_close_key;

const PLACEMENTS: &[ToastPlacement] = &[
    ToastPlacement::TopCenter,
    ToastPlacement::TopRight,
    ToastPlacement::BottomRight,
    ToastPlacement::BottomCenter,
    ToastPlacement::BottomLeft,
    ToastPlacement::TopLeft,
];

fn placement_name(p: ToastPlacement) -> &'static str {
    match p {
        ToastPlacement::TopCenter => "TopCenter",
        ToastPlacement::TopRight => "TopRight",
        ToastPlacement::TopLeft => "TopLeft",
        ToastPlacement::BottomCenter => "BottomCenter",
        ToastPlacement::BottomRight => "BottomRight",
        ToastPlacement::BottomLeft => "BottomLeft",
    }
}

struct App {
    toast_state: ToastStackState,
    placement_idx: usize,
    order: ToastOrder,
    auto_count: u32,
    manual_count: u32,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            toast_state: ToastStackState::new(),
            placement_idx: 0,
            order: ToastOrder::NewestFirst,
            auto_count: 0,
            manual_count: 0,
            should_quit: false,
        }
    }

    fn placement(&self) -> ToastPlacement {
        PLACEMENTS[self.placement_idx]
    }

    fn cycle_placement(&mut self) {
        self.placement_idx = (self.placement_idx + 1) % PLACEMENTS.len();
    }

    fn toggle_order(&mut self) {
        self.order = match self.order {
            ToastOrder::NewestFirst => ToastOrder::OldestFirst,
            ToastOrder::OldestFirst => ToastOrder::NewestFirst,
        };
    }

    fn push_auto(&mut self, style: ToastStyle) {
        self.auto_count += 1;
        let label = match style {
            ToastStyle::Info => "Info",
            ToastStyle::Success => "Success",
            ToastStyle::Warning => "Warning",
            ToastStyle::Error => "Error",
        };
        let msg = format!("{} toast #{}", label, self.auto_count);
        let id = self.toast_state.push_auto(&msg, 3000);
        // Override auto_style so we use the explicit style
        if let Some(item) = self.toast_state.items_mut().find(|t| t.id == id) {
            item.style = style;
            item.auto_style = false;
        }
    }

    fn push_manual(&mut self) {
        self.manual_count += 1;
        let msg = format!(
            "Persistent toast #{} (click or Esc to dismiss)",
            self.manual_count
        );
        let id = self.toast_state.push_manual(&msg);
        if let Some(item) = self.toast_state.items_mut().find(|t| t.id == id) {
            item.style = ToastStyle::Warning;
            item.auto_style = false;
        }
    }

    fn layout(&self) -> ToastStackLayout {
        ToastStackLayout {
            placement: self.placement(),
            order: self.order,
            ..Default::default()
        }
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

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.code == KeyCode::Char('q') {
                        app.should_quit = true;
                    } else if is_close_key(&key) {
                        // Esc dismisses top toast, or quits if none
                        if !app.toast_state.dismiss_top() {
                            app.should_quit = true;
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('1') => app.push_auto(ToastStyle::Info),
                            KeyCode::Char('2') => app.push_auto(ToastStyle::Success),
                            KeyCode::Char('3') => app.push_auto(ToastStyle::Warning),
                            KeyCode::Char('4') => app.push_auto(ToastStyle::Error),
                            KeyCode::Char('m') => app.push_manual(),
                            KeyCode::Char('d') => app.toast_state.dismiss_all(),
                            KeyCode::Char('p') => app.cycle_placement(),
                            KeyCode::Char('o') => app.toggle_order(),
                            _ => {}
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    if matches!(
                        mouse.kind,
                        MouseEventKind::Down(crossterm::event::MouseButton::Left)
                    ) {
                        let area = terminal.get_frame().area();
                        let stack = ToastStack::new(&app.toast_state).layout(app.layout());
                        if let Some(id) = stack.hit_test(area, mouse.column, mouse.row) {
                            app.toast_state.dismiss(id);
                        }
                    }
                }
                _ => {}
            }
        }

        // Expire old toasts
        app.toast_state.clear_expired();

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
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Content area
            Constraint::Length(5), // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Toast Stack Demo")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Info panel
    let order_name = match app.order {
        ToastOrder::NewestFirst => "NewestFirst",
        ToastOrder::OldestFirst => "OldestFirst",
    };
    let info_lines = vec![
        Line::from(vec![
            Span::styled("Placement: ", Style::default().fg(Color::Gray)),
            Span::styled(
                placement_name(app.placement()),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled("Order: ", Style::default().fg(Color::Gray)),
            Span::styled(
                order_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled("Active: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", app.toast_state.len()),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Push toasts and watch them stack! Click or press Esc to dismiss.",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let info = Paragraph::new(info_lines);
    f.render_widget(info, chunks[1]);

    // Help
    let help_lines = vec![
        Line::from(vec![
            Span::styled("1", Style::default().fg(Color::Cyan)),
            Span::raw(": Info  "),
            Span::styled("2", Style::default().fg(Color::Green)),
            Span::raw(": Success  "),
            Span::styled("3", Style::default().fg(Color::Yellow)),
            Span::raw(": Warning  "),
            Span::styled("4", Style::default().fg(Color::Red)),
            Span::raw(": Error  "),
            Span::styled("m", Style::default().fg(Color::Magenta)),
            Span::raw(": Manual"),
        ]),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Dismiss top  "),
            Span::styled("d", Style::default().fg(Color::Yellow)),
            Span::raw(": Dismiss all  "),
            Span::styled("p", Style::default().fg(Color::Yellow)),
            Span::raw(": Cycle placement  "),
            Span::styled("o", Style::default().fg(Color::Yellow)),
            Span::raw(": Toggle order  "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];
    let help = Paragraph::new(help_lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);

    // Render toasts on top
    app.toast_state.clear_expired();
    if !app.toast_state.is_empty() {
        ToastStack::new(&app.toast_state)
            .layout(app.layout())
            .render_with_clear(area, f.buffer_mut());
    }
}
