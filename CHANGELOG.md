# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

- **Navigation Components**
  - `FileExplorer` - File system browser with selection, search, and hidden file toggle
  - `ListPicker` - Scrollable list selection component
  - `TreeView` - Hierarchical tree structure navigation
  - `Accordion` - Collapsible sections with single or multiple expansion modes
  - `Breadcrumb` - Hierarchical navigation path with ellipsis collapsing, keyboard/mouse support, and multiple style presets (default, slash, chevron, arrow)

- **Layout Components**
  - `TabView` - Tab bar with content switching, supports top/bottom/left/right positions, keyboard navigation (arrows, number keys, Home/End), and mouse click support

- **Viewer Components**
  - `LogViewer` - Scrollable log display with ANSI color support
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
  - `accordion_demo` - Accordion component with single and multiple modes
  - `tab_view_demo` - Tab view with position cycling and content switching
  - `breadcrumb_demo` - Breadcrumb navigation with multiple style presets and dynamic path manipulation
  - `display_demo` - Display components: Progress, StepDisplay, ParagraphExt
  - `navigation_demo` - Navigation components: ListPicker, TreeView
  - `explorer_log_demo` - Combined FileExplorer, LogViewer, and Toast (requires `filesystem` feature)

- **Tests**
  - Comprehensive unit tests for all 17 components
  - Tests for state management, rendering, styling, and edge cases

### Changed
- Renamed crate from `ratatui-extension` to `ratatui-interact`
