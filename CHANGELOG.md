# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
