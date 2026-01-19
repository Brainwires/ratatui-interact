# ratatui-interact

[![Crates.io](https://img.shields.io/crates/v/ratatui-interact.svg)](https://crates.io/crates/ratatui-interact)
[![Documentation](https://docs.rs/ratatui-interact/badge.svg)](https://docs.rs/ratatui-interact)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Interactive TUI components for [ratatui](https://github.com/ratatui/ratatui) with **focus management** and **mouse support**.

Ratatui doesn't include built-in focus navigation or mouse click handling. This library fills that gap with ready-to-use interactive widgets and a flexible composition system.

## Features

- **Focus Management** - Tab/Shift+Tab navigation with `FocusManager<T>`
- **Mouse Click Support** - Click regions with hit-testing via `ClickRegion` and `ClickRegionRegistry`
- **Interactive Widgets** - CheckBox, Input, Button, Select, PopupDialog
- **Display Widgets** - ParagraphExt, Toast, Progress, MarqueeText
- **Navigation Widgets** - ListPicker, TreeView, FileExplorer, Accordion
- **Layout Widgets** - TabView
- **Viewer Widgets** - LogViewer, StepDisplay
- **Utilities** - ANSI parsing, display helpers
- **Composition Traits** - `Focusable`, `Clickable`, `Container` for building custom components

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ratatui-interact = "0.1"
```

Or from git:

```toml
[dependencies]
ratatui-interact = { git = "https://github.com/Brainwires/ratatui-interact.git" }
```

## Quick Start

```rust
use ratatui_interact::prelude::*;

// Define focusable elements
#[derive(Clone, Eq, PartialEq, Hash)]
enum Element {
    NameInput,
    EnableCheckbox,
    SubmitButton,
}

// Set up focus manager
let mut focus = FocusManager::new();
focus.register(Element::NameInput);
focus.register(Element::EnableCheckbox);
focus.register(Element::SubmitButton);

// Create component states
let mut input_state = InputState::new("Hello");
let mut checkbox_state = CheckBoxState::new(false);
let button_state = ButtonState::enabled();

// Handle keyboard events
match event {
    Event::Key(key) if is_tab(key) => focus.next(),
    Event::Key(key) if is_shift_tab(key) => focus.prev(),
    _ => {}
}

// Render with focus awareness
let input = Input::new(&input_state)
    .label("Name")
    .focused(focus.is_focused(&Element::NameInput));
let click_region = input.render_stateful(frame, area);

// Handle mouse clicks
if let Some(element) = click_region.contains(mouse_x, mouse_y) {
    focus.focus(&element);
}
```

## Components

### Interactive Components

| Component | Description |
|-----------|-------------|
| **CheckBox** | Toggleable checkbox with multiple symbol styles (ASCII, Unicode, checkmark) |
| **Input** | Text input with cursor, insertion, deletion, and navigation |
| **Button** | Multiple variants: SingleLine, Block, Toggle, Icon+Text |
| **Select** | Dropdown select box with popup options, keyboard/mouse navigation |
| **PopupDialog** | Container for modal dialogs with focus management |

### Display Components

| Component | Description |
|-----------|-------------|
| **ParagraphExt** | Extended paragraph with word-wrapping and scrolling |
| **Toast** | Transient notification popup with auto-expiration and style variants |
| **Progress** | Progress bar with label, percentage, and step counter |
| **MarqueeText** | Scrolling text for long content in limited space (continuous, bounce, static modes) |

### Navigation Components

| Component | Description |
|-----------|-------------|
| **ListPicker** | Scrollable list with selection cursor for picking items |
| **TreeView** | Collapsible tree view with selection and customizable rendering |
| **FileExplorer** | File browser with multi-select, search, and hidden file toggle |
| **Accordion** | Collapsible sections with single or multiple expansion modes |
| **Breadcrumb** | Hierarchical navigation path with ellipsis collapsing and keyboard/mouse support |

### Layout Components

| Component | Description |
|-----------|-------------|
| **TabView** | Tab bar with content switching, supports top/bottom/left/right positions |

### Viewer Components

| Component | Description |
|-----------|-------------|
| **LogViewer** | Scrollable log viewer with line numbers, search, and log-level coloring |
| **StepDisplay** | Multi-step progress display with sub-steps and output areas |

## Utilities

### ANSI Parser

Parse ANSI escape codes to ratatui styles:

```rust
use ratatui_interact::utils::ansi::parse_ansi_to_spans;

let text = "\x1b[31mRed\x1b[0m Normal";
let spans = parse_ansi_to_spans(text);
```

Supports: SGR codes (bold, italic, colors), 256-color mode, RGB mode.

### Display Utilities

```rust
use ratatui_interact::utils::display::{
    truncate_to_width, pad_to_width, clean_for_display, format_size
};

// Unicode-aware truncation with ellipsis
let truncated = truncate_to_width("Hello World", 8); // "Hello..."

// Unicode-aware padding
let padded = pad_to_width("Hi", 10); // "Hi        "

// Clean text for display (strips ANSI, handles \r)
let clean = clean_for_display("\x1b[31mText\x1b[0m");

// Human-readable file sizes
let size = format_size(1536); // "1.5 KB"
```

## Component Examples

### Progress Bar

```rust
use ratatui_interact::components::{Progress, ProgressStyle};

// From ratio (0.0 to 1.0)
let progress = Progress::new(0.75)
    .label("Downloading")
    .show_percentage(true);

// From step counts
let progress = Progress::from_steps(3, 10)
    .label("Processing")
    .show_steps(true);

// Different styles
let success = Progress::new(1.0).style(ProgressStyle::success());
let warning = Progress::new(0.9).style(ProgressStyle::warning());
```

### Marquee Text

```rust
use ratatui_interact::components::{MarqueeText, MarqueeState, MarqueeStyle, MarqueeMode};

// Create state (call tick() each frame to animate)
let mut state = MarqueeState::new();

// Continuous scrolling (loops around)
let marquee = MarqueeText::new("This is a long message that scrolls continuously", &mut state)
    .style(MarqueeStyle::default().mode(MarqueeMode::Continuous));

// Bounce mode (scrolls back and forth) - great for file paths
let mut state = MarqueeState::new();
let marquee = MarqueeText::new("/home/user/very/long/path/to/file.rs", &mut state)
    .style(MarqueeStyle::file_path());

// Static mode (truncate with ellipsis)
let mut state = MarqueeState::new();
let marquee = MarqueeText::new("Long text truncated with ellipsis", &mut state)
    .style(MarqueeStyle::default().mode(MarqueeMode::Static));

// In your event loop, advance the animation
state.tick(text_width, viewport_width, &style);
```

Marquee modes:
- `Continuous` - Text loops with a separator (default: "   ")
- `Bounce` - Text scrolls to end, pauses, then scrolls back
- `Static` - No animation, just truncate with ellipsis

Style presets:
- `MarqueeStyle::file_path()` - Cyan, bounce mode, longer pause
- `MarqueeStyle::status()` - Yellow bold, continuous
- `MarqueeStyle::title()` - Bold, bounce mode, long pause

### Select (Dropdown)

```rust
use ratatui_interact::components::{Select, SelectState, SelectStyle, handle_select_key, handle_select_mouse};

let options = vec!["Red", "Green", "Blue", "Yellow"];
let mut state = SelectState::new(options.len());

// Pre-select an option
let mut state = SelectState::with_selected(options.len(), 1); // "Green"

// Render the select box
let select = Select::new(&options, &state)
    .label("Color")
    .placeholder("Choose a color...");
let click_region = select.render_stateful(frame, area);

// Render dropdown when open (must be rendered last to appear on top)
let mut dropdown_regions = Vec::new();
if state.is_open {
    dropdown_regions = select.render_dropdown(frame, area, screen_area);
}

// Handle keyboard (Enter/Space to open, Up/Down to navigate, Enter to select, Esc to close)
if let Some(action) = handle_select_key(&key_event, &mut state) {
    match action {
        SelectAction::Select(idx) => println!("Selected: {}", options[idx]),
        _ => {}
    }
}

// Handle mouse clicks
handle_select_mouse(&mouse_event, &mut state, area, &dropdown_regions);
```

Style presets:
- `SelectStyle::default()` - Yellow highlight, checkmark indicator
- `SelectStyle::minimal()` - Subtle yellow text highlight
- `SelectStyle::arrow()` - Arrow indicator (`→`)
- `SelectStyle::bracket()` - Bracket indicator (`[x]`)

### List Picker

```rust
use ratatui_interact::components::{ListPicker, ListPickerState};
use ratatui::text::Line;

let items = vec!["Option A", "Option B", "Option C"];
let mut state = ListPickerState::new(items.len());

// Navigate
state.select_next();
state.select_prev();

// Custom rendering
let picker = ListPicker::new(&items, &state)
    .title("Select Option")
    .render_item(|item, _idx, selected| {
        vec![Line::from(item.to_string())]
    });
```

### Tree View

```rust
use ratatui_interact::components::{TreeView, TreeViewState, TreeNode};

#[derive(Clone, Debug)]
struct Task { name: String, done: bool }

let nodes = vec![
    TreeNode::new("1", Task { name: "Build".into(), done: false })
        .with_children(vec![
            TreeNode::new("1.1", Task { name: "Compile".into(), done: true }),
            TreeNode::new("1.2", Task { name: "Link".into(), done: false }),
        ]),
];

let mut state = TreeViewState::new();
state.toggle_collapsed("1"); // Collapse/expand

let tree = TreeView::new(&nodes, &state)
    .render_item(|node, selected| {
        format!("[{}] {}", if node.data.done { "x" } else { " " }, node.data.name)
    });
```

### Accordion

```rust
use ratatui_interact::components::{Accordion, AccordionState, AccordionMode};

// Single mode: only one section expanded at a time (FAQ-style)
let mut state = AccordionState::new(items.len())
    .with_mode(AccordionMode::Single);

// Multiple mode: any number can be expanded (settings-style)
let mut state = AccordionState::new(items.len())
    .with_mode(AccordionMode::Multiple)
    .with_expanded(vec!["section1".into()]);

// Toggle, expand, collapse
state.toggle("faq1");
state.expand("faq2");
state.collapse("faq1");

// Create accordion with custom renderers
let accordion = Accordion::new(&items, &state)
    .id_fn(|item, _| item.id.clone())
    .render_header(|item, _idx, is_focused| {
        Line::raw(item.title.clone())
    })
    .render_content(|item, _idx, area, buf| {
        let paragraph = Paragraph::new(item.content.as_str());
        paragraph.render(area, buf);
    });
```

### Breadcrumb

```rust
use ratatui_interact::components::{
    Breadcrumb, BreadcrumbItem, BreadcrumbState, BreadcrumbStyle,
    handle_breadcrumb_key, handle_breadcrumb_mouse,
};

// Create breadcrumb items with optional icons
let items = vec![
    BreadcrumbItem::new("home", "Home").icon("🏠"),
    BreadcrumbItem::new("users", "Users"),
    BreadcrumbItem::new("profile", "Profile Settings"),
];

// Create state
let mut state = BreadcrumbState::new(items);
state.focused = true;

// Create breadcrumb with default style (uses " > " separator)
let breadcrumb = Breadcrumb::new(&state);
let click_regions = breadcrumb.render_stateful(area, buf);

// Different style presets:
// - BreadcrumbStyle::slash()   - " / " (Unix path style)
// - BreadcrumbStyle::chevron() - " › " (Unicode chevron)
// - BreadcrumbStyle::arrow()   - " → " (Unicode arrow)
// - BreadcrumbStyle::minimal() - Subdued colors

let breadcrumb = Breadcrumb::new(&state)
    .style(BreadcrumbStyle::chevron());

// Handle keyboard (arrows navigate, Enter activates, e expands ellipsis)
if let Some(action) = handle_breadcrumb_key(&key_event, &mut state) {
    match action {
        BreadcrumbAction::Navigate(id) => println!("Navigate to: {}", id),
        BreadcrumbAction::ExpandEllipsis => println!("Ellipsis toggled"),
    }
}

// Handle mouse clicks
handle_breadcrumb_mouse(&mouse_event, &mut state, &click_regions);

// Dynamic path manipulation
state.push(BreadcrumbItem::new("new_item", "New Item"));
state.pop();
state.clear();
```

Ellipsis collapsing: Long paths automatically collapse with `...` (configurable threshold).
Example: `Home > ... > Settings > Profile` when showing 7+ items.

### Tab View

```rust
use ratatui_interact::components::{
    Tab, TabView, TabViewState, TabViewStyle, TabPosition,
    handle_tab_view_key, handle_tab_view_mouse,
};
use ratatui_interact::traits::ClickRegionRegistry;

// Create tabs with optional icons and badges
let tabs = vec![
    Tab::new("General").icon("⚙"),
    Tab::new("Network").icon("🌐").badge("3"),
    Tab::new("Security").icon("🔒"),
];

// Create state
let mut state = TabViewState::new(tabs.len());

// Create style (tabs on left side)
let style = TabViewStyle::left().tab_width(18);

// Create tab view with content renderer
let tab_view = TabView::new(&tabs, &state)
    .style(style)
    .content(|idx, area, buf| {
        let text = match idx {
            0 => "General settings content",
            1 => "Network configuration content",
            _ => "Security options content",
        };
        Paragraph::new(text).render(area, buf);
    });

// Render and register click regions
let mut registry: ClickRegionRegistry<TabViewAction> = ClickRegionRegistry::new();
tab_view.render_with_registry(area, buf, &mut registry);

// Handle keyboard (arrows navigate, Enter focuses content, Esc focuses tabs, 1-9 direct select)
handle_tab_view_key(&mut state, &key_event, style.position);

// Handle mouse clicks
handle_tab_view_mouse(&mut state, &registry, &mouse_event);
```

Style presets:
- `TabViewStyle::top()` - Horizontal tabs above content (default)
- `TabViewStyle::bottom()` - Horizontal tabs below content
- `TabViewStyle::left()` - Vertical tabs on left side
- `TabViewStyle::right()` - Vertical tabs on right side
- `TabViewStyle::minimal()` - No borders, simple dividers

### Log Viewer

```rust
use ratatui_interact::components::{LogViewer, LogViewerState};

let logs = vec![
    "[INFO] Application started".to_string(),
    "[ERROR] Connection failed".to_string(),
];

let mut state = LogViewerState::new(logs);

// Search
state.search("ERROR");
state.next_match();

// Scroll
state.scroll_down(5);
state.scroll_right(10);

let viewer = LogViewer::new(&state)
    .title("Application Log")
    .show_line_numbers(true);
```

### Step Display

```rust
use ratatui_interact::components::{Step, StepDisplayState, StepDisplay, StepStatus};

let steps = vec![
    Step::new("Initialize").with_sub_steps(vec!["Load config", "Connect DB"]),
    Step::new("Process data"),
    Step::new("Finalize"),
];

let mut state = StepDisplayState::new(steps);

// Update progress
state.start_step(0);
state.start_sub_step(0, 0);
state.complete_sub_step(0, 0);
state.add_output(0, "Config loaded successfully");
state.complete_step(0);

let display = StepDisplay::new(&state);
```

### File Explorer

```rust
use ratatui_interact::components::{FileExplorerState, FileExplorer};
use std::path::PathBuf;

let mut state = FileExplorerState::new(PathBuf::from("/home/user"));

// Navigate
state.cursor_down();
state.cursor_up();
state.toggle_selection(); // Multi-select
state.toggle_hidden(); // Show/hide hidden files

// Enter search mode
state.start_search();
state.search_push('r'); // Filter by 'r'

let explorer = FileExplorer::new(&state)
    .title("Select Files")
    .show_hidden(true);
```

## Toast Notifications

Toast notifications provide transient feedback to users:

```rust
use ratatui_interact::components::{Toast, ToastState, ToastStyle};

struct App {
    toast_state: ToastState,
}

// Show a toast for 3 seconds
app.toast_state.show("File saved successfully!", 3000);

// In your render function:
fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    // Draw your main content first...

    // Then draw toast on top if visible
    if let Some(message) = app.toast_state.get_message() {
        Toast::new(message)
            .style(ToastStyle::Success)
            .render_with_clear(area, frame.buffer_mut());
    }
}

// In your event loop, periodically clear expired toasts
app.toast_state.clear_if_expired();
```

Toast styles are auto-detected from message content, or can be set explicitly:
- `ToastStyle::Info` (cyan) - default
- `ToastStyle::Success` (green) - messages containing "success", "saved", "done"
- `ToastStyle::Warning` (yellow) - messages containing "warning", "warn"
- `ToastStyle::Error` (red) - messages containing "error", "fail"

## Mouse Click Handling for Buttons

Buttons support mouse clicks through click regions. Use `render_with_registry()` for the simplest pattern:

```rust
use ratatui_interact::components::{Button, ButtonState};
use ratatui_interact::traits::ClickRegionRegistry;

struct App {
    click_regions: ClickRegionRegistry<usize>,
    // ... other fields
}

// In your render function:
fn render(app: &mut App, frame: &mut Frame) {
    // Clear at start of each frame
    app.click_regions.clear();

    let state = ButtonState::enabled();
    let button = Button::new("OK", &state);

    // Render and register in one call
    button.render_with_registry(area, frame.buffer_mut(), &mut app.click_regions, 0);
}

// In your event handler:
fn handle_mouse(app: &App, mouse: MouseEvent) {
    if is_left_click(&mouse) {
        if let Some(&idx) = app.click_regions.handle_click(mouse.column, mouse.row) {
            // Button at index `idx` was clicked
        }
    }
}
```

For more control, use the two-step pattern with `render_stateful()`:

```rust
let region = button.render_stateful(area, buf);
registry.register(region.area, my_custom_action);
```

## Examples

Run the examples to see components in action:

```bash
# Interactive Components
cargo run --example checkbox_demo    # Checkbox with multiple styles
cargo run --example input_demo       # Text input with cursor
cargo run --example button_demo      # Button variants and styles
cargo run --example select_demo      # Dropdown select boxes
cargo run --example dialog_demo      # Modal dialogs

# Display & Viewer Components
cargo run --example marquee_demo     # Scrolling text animation
cargo run --example display_demo     # Progress, StepDisplay, ParagraphExt

# Navigation Components
cargo run --example accordion_demo   # Collapsible sections
cargo run --example tab_view_demo    # Tab bar with positions
cargo run --example breadcrumb_demo  # Hierarchical path navigation
cargo run --example navigation_demo  # ListPicker and TreeView

# Combined Demo (requires filesystem feature)
cargo run --example explorer_log_demo --features filesystem  # FileExplorer + LogViewer + Toast
```

## Comparison with Alternatives

| Feature | ratatui-interact | rat-focus | tui-input |
|---------|------------------|-----------|-----------|
| Focus management | ✅ Generic `FocusManager<T>` | ✅ `FocusFlag` based | ❌ |
| Mouse click regions | ✅ `ClickRegion` with hit-testing | ✅ Area-based | ❌ |
| Ready-to-use widgets | ✅ Many (see above) | ❌ | ✅ Input only |
| Composition traits | ✅ Focusable, Clickable, Container | ❌ | ❌ |

## License

MIT
