//! View/Copy mode utilities
//!
//! Provides functionality for entering a "View/Copy mode" that exits the alternate
//! screen and prints content to the terminal's main buffer, allowing native text
//! selection and copying.
//!
//! # Example
//!
//! ```rust,ignore
//! use ratatui_interact::utils::{ViewCopyMode, ViewCopyConfig};
//!
//! // Create config
//! let config = ViewCopyConfig::default()
//!     .with_header("My Content")
//!     .show_hints(true);
//!
//! // Enter view/copy mode
//! let mut mode = ViewCopyMode::enter(&mut terminal, &config)?;
//!
//! // Print content
//! mode.print_lines(&content_lines)?;
//!
//! // Wait for exit (handles 'c', 'q', Esc)
//! // Returns true if user pressed 'n' (toggle line numbers)
//! loop {
//!     match mode.wait_for_input()? {
//!         ViewCopyAction::Exit => break,
//!         ViewCopyAction::ToggleLineNumbers => {
//!             // Regenerate content and reprint
//!             mode.clear()?;
//!             mode.print_lines(&new_content)?;
//!         }
//!         ViewCopyAction::None => {}
//!     }
//! }
//!
//! // Exit view/copy mode (re-enters alternate screen)
//! mode.exit(&mut terminal)?;
//! ```

use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode, DisableLineWrap, EnableLineWrap,
    },
};

/// Action returned from waiting for input in View/Copy mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewCopyAction {
    /// User wants to exit view/copy mode
    Exit,
    /// User wants to toggle line numbers
    ToggleLineNumbers,
    /// No action (continue waiting)
    None,
}

/// Configuration for View/Copy mode
#[derive(Debug, Clone)]
pub struct ViewCopyConfig {
    /// Header text to show at the top
    pub header: Option<String>,
    /// Whether to show keyboard hints
    pub show_hints: bool,
    /// Exit keys (default: 'c', 'q', Esc)
    pub exit_keys: Vec<KeyCode>,
    /// Toggle line numbers key (default: 'n')
    pub toggle_key: KeyCode,
}

impl Default for ViewCopyConfig {
    fn default() -> Self {
        Self {
            header: None,
            show_hints: true,
            exit_keys: vec![KeyCode::Char('c'), KeyCode::Char('q'), KeyCode::Esc],
            toggle_key: KeyCode::Char('n'),
        }
    }
}

impl ViewCopyConfig {
    /// Set the header text
    pub fn with_header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Set whether to show keyboard hints
    pub fn show_hints(mut self, show: bool) -> Self {
        self.show_hints = show;
        self
    }

    /// Set custom exit keys
    pub fn exit_keys(mut self, keys: Vec<KeyCode>) -> Self {
        self.exit_keys = keys;
        self
    }

    /// Set the toggle line numbers key
    pub fn toggle_key(mut self, key: KeyCode) -> Self {
        self.toggle_key = key;
        self
    }
}

/// Handle for View/Copy mode
///
/// Created by `ViewCopyMode::enter()`, must call `exit()` when done.
pub struct ViewCopyMode {
    config: ViewCopyConfig,
}

impl ViewCopyMode {
    /// Enter View/Copy mode
    ///
    /// This will:
    /// 1. Leave the alternate screen
    /// 2. Disable mouse capture
    /// 3. Clear the screen and scrollback buffer
    /// 4. Disable raw mode (so println works normally)
    pub fn enter<W: Write>(stdout: &mut W) -> io::Result<Self> {
        Self::enter_with_config(stdout, ViewCopyConfig::default())
    }

    /// Enter View/Copy mode with custom configuration
    pub fn enter_with_config<W: Write>(
        stdout: &mut W,
        config: ViewCopyConfig,
    ) -> io::Result<Self> {
        use crossterm::event::DisableMouseCapture;

        // Leave alternate screen and disable mouse capture
        execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

        // Disable raw mode so println works
        disable_raw_mode()?;

        // Clear screen and scrollback buffer
        execute!(
            stdout,
            Clear(ClearType::Purge),
            Clear(ClearType::All),
            MoveTo(0, 0),
            DisableLineWrap
        )?;
        stdout.flush()?;

        Ok(Self { config })
    }

    /// Clear the screen (for reprinting content)
    pub fn clear(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            Clear(ClearType::Purge),
            Clear(ClearType::All),
            MoveTo(0, 0)
        )?;
        stdout.flush()?;
        Ok(())
    }

    /// Print lines to stdout with optional header and hints
    pub fn print_lines(&self, lines: &[String]) -> io::Result<()> {
        if self.config.show_hints {
            if let Some(header) = &self.config.header {
                println!("=== {} ===", header);
            } else {
                println!("=== View/Copy Mode ===");
            }
            println!("Press 'c', 'q', or Esc to exit | 'n' to toggle line numbers");
            println!("{}", "─".repeat(60));
            println!();
        }

        for line in lines {
            println!("{}", line);
        }

        if self.config.show_hints {
            println!();
            println!("{}", "─".repeat(60));
            println!("Press 'c', 'q', or Esc to exit | 'n' to toggle line numbers");
        }

        io::stdout().flush()?;
        Ok(())
    }

    /// Print raw lines without any formatting
    pub fn print_raw(&self, lines: &[String]) -> io::Result<()> {
        for line in lines {
            println!("{}", line);
        }
        io::stdout().flush()?;
        Ok(())
    }

    /// Wait for user input and return the action
    ///
    /// Note: This temporarily enables raw mode to catch keypresses,
    /// then disables it again so subsequent prints work.
    pub fn wait_for_input(&self) -> io::Result<ViewCopyAction> {
        // Enable raw mode to catch keys
        enable_raw_mode()?;

        let action = loop {
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.config.exit_keys.contains(&key.code) {
                        break ViewCopyAction::Exit;
                    } else if key.code == self.config.toggle_key {
                        break ViewCopyAction::ToggleLineNumbers;
                    }
                }
            }
        };

        // Disable raw mode for any subsequent prints
        disable_raw_mode()?;

        Ok(action)
    }

    /// Exit View/Copy mode and return to the TUI
    ///
    /// This will:
    /// 1. Re-enable raw mode
    /// 2. Re-enter the alternate screen
    /// 3. Re-enable mouse capture
    /// 4. Clear the terminal to force a full redraw
    pub fn exit<B>(self, terminal: &mut ratatui::Terminal<B>) -> io::Result<()>
    where
        B: ratatui::backend::Backend,
        io::Error: From<B::Error>,
    {
        use crossterm::event::EnableMouseCapture;

        let mut stdout = io::stdout();

        // Re-enable raw mode
        enable_raw_mode()?;

        // Re-enter alternate screen and enable mouse capture
        execute!(stdout, EnableLineWrap, EnterAlternateScreen, EnableMouseCapture)?;

        // Clear terminal to force full redraw
        terminal.clear()?;

        Ok(())
    }
}

/// Clear the main screen buffer before entering alternate screen
///
/// Call this at app startup to ensure View/Copy mode has a clean buffer.
/// This prevents old terminal content from appearing when leaving alternate screen.
pub fn clear_main_screen() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        Clear(ClearType::Purge),
        Clear(ClearType::All),
        MoveTo(0, 0)
    )?;
    stdout.flush()?;
    Ok(())
}
