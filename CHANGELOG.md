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
  - `PopupDialog` - Modal dialog component (in `container` module)

- **Display Components**
  - `Toast` - Notification widget with auto-dismiss and multiple severity levels
  - `ParagraphExt` - Extended paragraph with click region registry support
  - `Progress` - Progress bar indicator

- **Navigation Components**
  - `FileExplorer` - File system browser with selection, search, and hidden file toggle
  - `ListPicker` - Scrollable list selection component
  - `TreeView` - Hierarchical tree structure navigation

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

### Changed
- Renamed crate from `ratatui-extension` to `ratatui-interact`
