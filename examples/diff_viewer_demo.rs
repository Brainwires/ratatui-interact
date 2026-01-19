//! Diff Viewer Demo
//!
//! Interactive demo showing diff viewer features:
//! - Unified and side-by-side view modes
//! - Keyboard navigation (scroll, hunk jumping, change navigation)
//! - Mouse scrolling support
//! - Search functionality
//! - Multiple sample diffs
//!
//! Run with: cargo run --example diff_viewer_demo

use std::io;
use std::time::Duration;

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
    widgets::{Block, Borders, Paragraph, Tabs},
};

use ratatui_interact::components::{
    DiffData, DiffViewer, DiffViewerState, DiffViewerStyle, handle_diff_viewer_key,
    handle_diff_viewer_mouse,
};
use ratatui_interact::events::is_close_key;

/// Sample diffs for demonstration
const BASIC_DIFF: &str = r#"--- a/greeting.txt
+++ b/greeting.txt
@@ -1,4 +1,5 @@
 Hello World
-This is old text
+This is new text
+And an additional line
 Goodbye
 The End
"#;

const CODE_DIFF: &str = r#"--- a/src/main.rs
+++ b/src/main.rs
@@ -1,10 +1,12 @@
 fn main() {
-    println!("Hello, world!");
+    println!("Hello, Rust!");
+
+    // New feature: process arguments
     let args: Vec<String> = std::env::args().collect();
-    if args.len() > 1 {
-        println!("Args: {:?}", args);
+    for arg in &args[1..] {
+        println!("Processing: {}", arg);
     }
 }

@@ -15,6 +17,10 @@ fn helper_function() {
     let x = 42;
     let y = x * 2;
-    println!("Result: {}", y);
+    let result = calculate(x, y);
+    println!("Result: {}", result);
+}
+
+fn calculate(a: i32, b: i32) -> i32 {
+    a + b
 }
"#;

const LARGE_DIFF: &str = r#"--- a/config.json
+++ b/config.json
@@ -1,20 +1,25 @@
 {
   "name": "my-app",
-  "version": "1.0.0",
+  "version": "2.0.0",
   "description": "A sample application",
+  "author": "Developer",
+  "license": "MIT",
   "main": "index.js",
   "scripts": {
     "start": "node index.js",
-    "test": "jest"
+    "test": "jest --coverage",
+    "lint": "eslint src/",
+    "build": "tsc"
   },
   "dependencies": {
-    "express": "^4.17.1",
-    "lodash": "^4.17.21"
+    "express": "^4.18.2",
+    "lodash": "^4.17.21",
+    "axios": "^1.4.0"
   },
   "devDependencies": {
-    "jest": "^27.0.0"
+    "jest": "^29.0.0",
+    "typescript": "^5.0.0",
+    "eslint": "^8.40.0"
   }
 }
"#;

/// Application state
struct App {
    /// Diff viewer states for each tab
    diff_states: Vec<DiffViewerState>,
    /// Currently selected tab
    selected_tab: usize,
    /// Tab names
    tab_names: Vec<&'static str>,
    /// Should quit
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let diffs = vec![
            DiffData::from_unified_diff(BASIC_DIFF),
            DiffData::from_unified_diff(CODE_DIFF),
            DiffData::from_unified_diff(LARGE_DIFF),
        ];

        let diff_states = diffs.into_iter().map(DiffViewerState::new).collect();

        Self {
            diff_states,
            selected_tab: 0,
            tab_names: vec!["Basic", "Code", "Config"],
            should_quit: false,
        }
    }

    fn current_state(&self) -> &DiffViewerState {
        &self.diff_states[self.selected_tab]
    }

    fn current_state_mut(&mut self) -> &mut DiffViewerState {
        &mut self.diff_states[self.selected_tab]
    }

    fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % self.tab_names.len();
    }

    fn prev_tab(&mut self) {
        if self.selected_tab == 0 {
            self.selected_tab = self.tab_names.len() - 1;
        } else {
            self.selected_tab -= 1;
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

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if is_close_key(&key) || key.code == KeyCode::Char('q') {
                        app.should_quit = true;
                    } else if key.code == KeyCode::Tab {
                        app.next_tab();
                    } else if key.code == KeyCode::BackTab {
                        app.prev_tab();
                    } else {
                        // Pass to diff viewer
                        handle_diff_viewer_key(app.current_state_mut(), &key);
                    }
                }
                Event::Mouse(mouse) => {
                    handle_diff_viewer_mouse(app.current_state_mut(), &mouse);
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
    let area = f.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title and tabs
            Constraint::Min(1),    // Diff viewer
            Constraint::Length(3), // Help
        ])
        .split(area);

    // Title and tabs
    render_header(f, app, chunks[0]);

    // Diff viewer
    let state = app.current_state();
    let title = format!(
        "{} - {}",
        app.tab_names[app.selected_tab],
        state.diff.old_path.as_deref().unwrap_or("unknown")
    );

    // Update visible dimensions in state
    let inner_height = chunks[1].height.saturating_sub(4) as usize; // Account for borders and status
    let inner_width = chunks[1].width.saturating_sub(2) as usize;
    app.current_state_mut().visible_height = inner_height;
    app.current_state_mut().visible_width = inner_width;

    let style = DiffViewerStyle::default();
    let viewer = DiffViewer::new(app.current_state())
        .title(&title)
        .style(style)
        .show_stats(true);

    f.render_widget(viewer, chunks[1]);

    // Help footer
    render_help(f, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Diff Viewer Demo ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Tabs
    let tabs: Vec<Line> = app
        .tab_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == app.selected_tab {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(*name, style))
        })
        .collect();

    let tabs_widget = Tabs::new(tabs)
        .select(app.selected_tab)
        .divider(" | ")
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    f.render_widget(tabs_widget, inner);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(vec![
            Span::styled("Tab/Shift+Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": switch diff  "),
            Span::styled("j/k/↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(": scroll  "),
            Span::styled("]/[", Style::default().fg(Color::Yellow)),
            Span::raw(": next/prev hunk  "),
            Span::styled("n/N", Style::default().fg(Color::Yellow)),
            Span::raw(": next/prev change  "),
        ]),
        Line::from(vec![
            Span::styled("v", Style::default().fg(Color::Yellow)),
            Span::raw(": toggle view mode  "),
            Span::styled("g/G", Style::default().fg(Color::Yellow)),
            Span::raw(": top/bottom  "),
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::raw(": search  "),
            Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": quit"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::TOP))
        .style(Style::default().fg(Color::Gray));

    f.render_widget(help, area);
}
