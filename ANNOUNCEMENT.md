🚀 **Announcing ratatui-interact v0.2.0 — Interactive TUI Components for Ratatui**

Ratatui is amazing for building terminal UIs, but it doesn't include focus navigation or mouse click handling out of the box. **ratatui-interact** fills that gap with a complete toolkit of ready-to-use interactive widgets.

**Core Features:**
• `FocusManager<T>` — Generic focus state with Tab/Shift+Tab navigation
• `ClickRegionRegistry` — Mouse click regions with hit-testing
• Composition traits (`Focusable`, `Clickable`, `Container`) for building custom components

**20+ Components Included:**
• **Interactive:** CheckBox, Input, TextArea, Button, Select (dropdown), ContextMenu (with submenus!), PopupDialog, HotkeyDialog
• **Display:** Progress, Spinner (12 animation styles), MarqueeText, Toast notifications
• **Navigation:** ListPicker, TreeView, FileExplorer, Accordion, Breadcrumb
• **Layout:** TabView (top/bottom/left/right positions), SplitPane (draggable resizing)
• **Viewers:** LogViewer, DiffViewer (unified + side-by-side modes), StepDisplay
• **Utilities:** ANSI parser, unicode-aware text helpers

**30,000+ lines of code**, comprehensive tests, and 18 runnable examples.

```toml
[dependencies]
ratatui-interact = "0.2"
```

📦 **crates.io:** https://crates.io/crates/ratatui-interact
📚 **docs.rs:** https://docs.rs/ratatui-interact
🔗 **GitHub:** https://github.com/Brainwires/ratatui-interact
