# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.1] - 2026-01-30

### Added
- `ScrollableContentState::push_line()` - Append a single line to the content
- `ScrollableContentState::clear()` - Clear all content and reset scroll position

## [0.4.0] - 2026-01-30

### Added
- **Interactive Components**
  - `ScrollableContent` - Scrollable text pane with focus support and keyboard/mouse navigation
    - `ScrollableContentState` for tracking scroll position, focus, and fullscreen state
    - `ScrollableContentStyle` with customizable colors and indicators (default, borderless presets)
    - `ScrollableContentAction` enum for handling scroll, page, and fullscreen events
    - Full keyboard support: Up/Down/j/k (scroll), PgUp/PgDown (page), Home/End (bounds), F10/Enter (fullscreen toggle)
    - Mouse scroll wheel support with configurable scroll area
    - Title and scroll position indicators

- **Display Components**
  - `AnimatedText` - Animated text labels with color effects
    - `AnimatedTextState` for frame-based animation with tick timing
    - `AnimatedTextStyle` with customizable colors and effects
    - `AnimatedTextEffect` enum: Pulse, Wave, Rainbow, GradientShift, Sparkle
    - `WaveDirection` enum for controlling wave animation direction
    - Color interpolation for smooth RGB transitions
    - Style presets: `default()`, `loading()`, `success()`, `warning()`, `error()`, `info()`, `rainbow()`
    - Builder methods: `.style()`, `.alignment()`, `.modifier()`

- **Utilities**
  - `MouseCaptureState` - Toggle mouse capture at runtime for "copy mode"
    - `enable_mouse_capture()`, `disable_mouse_capture()`, `toggle_mouse_capture()` functions
    - Allows native terminal text selection when capture is disabled
  - `ViewCopyMode` - Exits alternate screen for native text selection
    - `ViewCopyConfig` for customizable header, hints, and exit keys
    - `ViewCopyAction` enum for handling exit and toggle events
    - Line number toggle support
  - `ExitStrategy` - Application exit handling
    - `RestoreConsole` - Restores original terminal state
    - `PrintContent` - Prints content to stdout after exit
  - Clipboard utilities (requires `clipboard` feature)
    - `copy_to_clipboard()` - Copy text to system clipboard
    - `get_from_clipboard()` - Paste text from system clipboard
    - `ClipboardResult` enum for success/error/unavailable states

- **Examples**
  - `animated_text_demo` - Interactive demonstration of AnimatedText with all 5 effect modes, style cycling, and speed control
  - `copyable_pane_demo` - Demonstration of ScrollableContent with View/Copy mode toggle

- **Tests**
  - 15 unit tests for AnimatedText covering state management, tick behavior, color interpolation, effects, and rendering
  - 12 unit tests for ScrollableContent covering state, scrolling, keyboard handling, widget rendering, and styles

### Changed
- Moved component usage examples from README.md to examples/README.md for better organization

## [0.3.0] - 2026-01-21

### Added
- **Interactive Components**
  - `MenuBar` - Traditional desktop-style horizontal menu bar (File, Edit, View, Help)
    - `Menu` struct for top-level menu definitions with labels and items
    - `MenuBarItem` enum with `Action`, `Separator`, and `Submenu` variants
    - Builder pattern: `action()`, `separator()`, `submenu()`, `.shortcut()`, `.enabled()`
    - `MenuBarState` for tracking open state, active menu, highlight, scroll, and submenu state
    - `MenuBarAction` enum for handling MenuOpen, MenuClose, ItemSelect, SubmenuOpen, SubmenuClose, HighlightChange events
    - `MenuBarStyle` with customizable colors, sizing, and indicators (default, light, minimal presets)
    - Smart dropdown positioning with scrolling for long menus
    - Full keyboard support: Left/Right (menus), Up/Down (items), Enter/Space (select), Esc (close), Home/End
    - Full mouse support: click to open/select, hover to switch menus and highlight items

- **Display Components**
  - `MousePointer` - Visual indicator that displays at the current mouse cursor position
    - `MousePointerState` for tracking enabled status and position (disabled by default)
    - `MousePointerStyle` with customizable symbol, foreground, and background colors
    - Style presets: `default()` (█ Yellow), `crosshair()` (┼ Cyan), `arrow()` (▶ White), `dot()` (● Green), `plus()` (+ Magenta)
    - `custom(symbol, color)` constructor for user-defined styles
    - Builder methods: `.symbol()`, `.fg()`, `.bg()` for full customization
    - `render()` and `render_in_area()` methods for flexible positioning
    - Designed to render last (on top of other widgets) for overlay effect

- **Event Helpers**
  - `is_mouse_move()` - Detect mouse move events
  - `is_mouse_drag()` - Detect mouse drag events

- **Examples**
  - `menu_bar_demo` - Interactive demonstration of menu bar with File/Edit/View/Help menus, submenus, shortcuts, disabled items, and style cycling
  - `mouse_pointer_demo` - Interactive demonstration of mouse pointer with toggle, style cycling, and position display

- **Tests**
  - 28 unit tests for MenuBar covering item creation, state management, navigation, styles, keyboard handling, and mouse handling
  - 16 unit tests for MousePointer covering state management, style presets, rendering, and bounds checking
  - 2 unit tests for new mouse event helpers

### Changed
- Updated Rust edition from 2021 to 2024
- Updated minimum Rust version to 1.85
- Updated ratatui from 0.29 to 0.30
- Updated crossterm from 0.28 to 0.29
- Updated regex from 1.10 to 1.12
- Updated termimad from 0.30 to 0.34

## [0.2.0] - 2026-01-19

### Added
- **Interactive Components**
  - `ContextMenu` - Right-click popup menu with actions, separators, shortcuts, icons, disabled items, and nested submenus
    - `ContextMenuItem` enum with `Action`, `Separator`, and `Submenu` variants
    - Builder pattern: `action()`, `separator()`, `submenu()`, `.icon()`, `.shortcut()`, `.enabled()`
    - `ContextMenuState` for tracking open state, position, highlight, scroll, and submenu state
    - `ContextMenuAction` enum for handling Open, Close, Select, SubmenuOpen, SubmenuClose, HighlightChange events
    - `ContextMenuStyle` with customizable colors, sizing, and indicators (default, light, minimal presets)
    - Smart positioning that adjusts to stay within screen bounds
    - Full keyboard support: Up/Down, Enter/Space, Right (submenu), Left/Esc (close), Home/End
    - Full mouse support: click to select, hover to highlight, click outside to close

- **Examples**
  - `context_menu_demo` - Interactive demonstration of context menu with file list, actions, disabled items, and submenus

- **Tests**
  - 47 comprehensive unit tests for ContextMenu covering:
    - Item creation and builder patterns
    - State management (open, close, navigation, submenu)
    - Style configuration and presets
    - Keyboard event handling
    - Mouse event handling
    - Edge cases (disabled items, empty menus, bounds)

## [0.1.0] - 2026-01-19

### Added
- **Interactive Components**
  - `Button` - Clickable button with focus support and click region registry
  - `CheckBox` - Toggle checkbox with label
  - `Input` - Text input field with cursor management
  - `Select` - Dropdown select box with popup options, keyboard navigation, and mouse support
  - `PopupDialog` - Modal dialog component (in `container` module)

- **Display Components**
  - `Toast` - Notification widget with auto-dismiss and multiple severity levels
  - `ParagraphExt` - Extended paragraph with click region registry support
  - `Progress` - Progress bar indicator
  - `MarqueeText` - Scrolling text widget with continuous, bounce, and static modes
  - `Spinner` - Animated loading indicator with 12 frame styles (dots, braille, line, circle, arrow, clock, moon, etc.)

- **Navigation Components**
  - `FileExplorer` - File system browser with selection, search, and hidden file toggle
  - `ListPicker` - Scrollable list selection component
  - `TreeView` - Hierarchical tree structure navigation
  - `Accordion` - Collapsible sections with single or multiple expansion modes
  - `Breadcrumb` - Hierarchical navigation path with ellipsis collapsing, keyboard/mouse support, and multiple style presets (default, slash, chevron, arrow)

- **Layout Components**
  - `TabView` - Tab bar with content switching, supports top/bottom/left/right positions, keyboard navigation (arrows, number keys, Home/End), and mouse click support
  - `SplitPane` - Resizable split pane with drag-to-resize divider, supports horizontal (left/right) and vertical (top/bottom) orientations, keyboard resize with arrow keys, and nested split panes

- **Viewer Components**
  - `LogViewer` - Scrollable log display with ANSI color support
  - `DiffViewer` - Diff viewer with unified and side-by-side modes, hunk/change navigation, search, and syntax highlighting for additions/deletions
  - `StepDisplay` - Step-by-step progress display

- **Utilities**
  - `ansi` - ANSI escape code parser for colored text rendering
  - `display` - Display utility functions

- **Core Features**
  - `FocusManager` - Generic focus state management across components
  - `ClickRegionRegistry` - Mouse click region tracking and hit detection
  - Event handlers for keyboard and mouse input

- **Examples**
  - `button_demo` - Button component demonstration
  - `checkbox_demo` - Checkbox component demonstration
  - `dialog_demo` - Dialog component demonstration
  - `input_demo` - Input component demonstration
  - `select_demo` - Select dropdown component with multiple instances
  - `marquee_demo` - Marquee text scrolling demonstration
  - `spinner_demo` - Animated spinner with 12 different frame styles
  - `diff_viewer_demo` - Diff viewer with unified/side-by-side modes and navigation
  - `accordion_demo` - Accordion component with single and multiple modes
  - `tab_view_demo` - Tab view with position cycling and content switching
  - `split_pane_demo` - Split pane with nested panes, drag-to-resize, and orientation toggle
  - `breadcrumb_demo` - Breadcrumb navigation with multiple style presets and dynamic path manipulation
  - `display_demo` - Display components: Progress, StepDisplay, ParagraphExt
  - `navigation_demo` - Navigation components: ListPicker, TreeView
  - `explorer_log_demo` - Combined FileExplorer, LogViewer, and Toast (requires `filesystem` feature)

- **Tests**
  - Comprehensive unit tests for all 19 components
  - Tests for state management, rendering, styling, and edge cases

### Changed
- Renamed crate from `ratatui-extension` to `ratatui-interact`
