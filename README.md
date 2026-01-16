# ratatui-interact

[![Crates.io](https://img.shields.io/crates/v/ratatui-interact.svg)](https://crates.io/crates/ratatui-interact)
[![Documentation](https://docs.rs/ratatui-interact/badge.svg)](https://docs.rs/ratatui-interact)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Interactive TUI components for [ratatui](https://github.com/ratatui/ratatui) with **focus management** and **mouse support**.

Ratatui doesn't include built-in focus navigation or mouse click handling. This library fills that gap with ready-to-use interactive widgets and a flexible composition system.

## Features

- **Focus Management** - Tab/Shift+Tab navigation with `FocusManager<T>`
- **Mouse Click Support** - Click regions with hit-testing via `ClickRegion` and `ClickRegionRegistry`
- **Interactive Widgets** - CheckBox, Input, Button, PopupDialog
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

| Component | Description |
|-----------|-------------|
| **CheckBox** | Toggleable checkbox with multiple symbol styles (ASCII, Unicode, checkmark) |
| **Input** | Text input with cursor, insertion, deletion, and navigation |
| **Button** | Multiple variants: SingleLine, Block, Toggle, Icon+Text |
| **PopupDialog** | Container for modal dialogs with focus management |

## Examples

Run the examples to see components in action:

```bash
cargo run --example checkbox_demo
cargo run --example input_demo
cargo run --example button_demo
cargo run --example dialog_demo
```

## Comparison with Alternatives

| Feature | ratatui-interact | rat-focus | tui-input |
|---------|------------------|-----------|-----------|
| Focus management | ✅ Generic `FocusManager<T>` | ✅ `FocusFlag` based | ❌ |
| Mouse click regions | ✅ `ClickRegion` with hit-testing | ✅ Area-based | ❌ |
| Ready-to-use widgets | ✅ CheckBox, Input, Button, Dialog | ❌ | ✅ Input only |
| Composition traits | ✅ Focusable, Clickable, Container | ❌ | ❌ |

## License

MIT
