# ratatui-extension

Reusable TUI components extending [ratatui](https://github.com/ratatui/ratatui) with focus management and mouse support.

## Features

- **Focus Management**: Tab navigation between components
- **Click Regions**: Mouse click support with hit-testing
- **Composition**: Container-based component hierarchies for dialogs

## Components

- `CheckBox` - Toggleable checkbox with customizable symbols
- `Input` - Text input field with cursor and editing support
- `Button` - Buttons with multiple display variants
- `PopupDialog` - Container for popup dialogs with focus management

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
ratatui-extension = { git = "https://github.com/Brainwires/ratatui-extension.git" }
```

Or as a submodule:

```toml
[dependencies]
ratatui-extension = { path = "path/to/ratatui-extension" }
```

## Quick Start

```rust
use ratatui_extension::prelude::*;

// Create component state
let mut checkbox_state = CheckBoxState::new(false);
let mut input_state = InputState::new("Hello");
let button_state = ButtonState::enabled();

// Use in your render function
fn render(frame: &mut Frame, area: Rect) {
    let checkbox = CheckBox::new("Enable", &checkbox_state);
    let input = Input::new(&input_state).label("Name");
    let button = Button::new("Submit", &button_state);

    // Render and get click regions
    let cb_region = checkbox.render_stateful(area, frame.buffer_mut());
    let input_region = input.render_stateful(frame, input_area);
    let btn_region = button.render_stateful(button_area, frame.buffer_mut());
}
```

## License

MIT
