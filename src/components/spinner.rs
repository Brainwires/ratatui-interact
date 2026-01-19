//! Spinner widget for loading/processing indicators
//!
//! An animated spinner with multiple styles and optional label support.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{Spinner, SpinnerState, SpinnerStyle, SpinnerFrames};
//! use ratatui::layout::Rect;
//! use ratatui::buffer::Buffer;
//! use ratatui::widgets::Widget;
//!
//! // Create state and advance each frame
//! let mut state = SpinnerState::new();
//!
//! // Simple spinner
//! let spinner = Spinner::new(&state);
//!
//! // With label
//! let spinner = Spinner::new(&state)
//!     .label("Loading...");
//!
//! // Different spinner styles
//! let spinner = Spinner::new(&state)
//!     .frames(SpinnerFrames::Braille)
//!     .label("Processing");
//!
//! // In your event loop, advance the animation
//! state.tick();
//! ```

use std::time::{Duration, Instant};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_width::UnicodeWidthStr;

/// Predefined spinner frame sets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpinnerFrames {
    /// Classic dots: ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
    #[default]
    Dots,
    /// Braille pattern: ⣾ ⣽ ⣻ ⢿ ⡿ ⣟ ⣯ ⣷
    Braille,
    /// Line spinner: | / - \
    Line,
    /// Circle: ◐ ◓ ◑ ◒
    Circle,
    /// Box: ▖ ▘ ▝ ▗
    Box,
    /// Arrow: ← ↖ ↑ ↗ → ↘ ↓ ↙
    Arrow,
    /// Bounce: ⠁ ⠂ ⠄ ⠂
    Bounce,
    /// Grow: ▁ ▃ ▄ ▅ ▆ ▇ █ ▇ ▆ ▅ ▄ ▃
    Grow,
    /// Clock: 🕐 🕑 🕒 🕓 🕔 🕕 🕖 🕗 🕘 🕙 🕚 🕛
    Clock,
    /// Moon: 🌑 🌒 🌓 🌔 🌕 🌖 🌗 🌘
    Moon,
    /// Simple ASCII: . o O @ *
    Ascii,
    /// Toggle: ⊶ ⊷
    Toggle,
}

impl SpinnerFrames {
    /// Get the frames for this spinner style
    pub fn frames(&self) -> &'static [&'static str] {
        match self {
            SpinnerFrames::Dots => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            SpinnerFrames::Braille => &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
            SpinnerFrames::Line => &["|", "/", "-", "\\"],
            SpinnerFrames::Circle => &["◐", "◓", "◑", "◒"],
            SpinnerFrames::Box => &["▖", "▘", "▝", "▗"],
            SpinnerFrames::Arrow => &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
            SpinnerFrames::Bounce => &["⠁", "⠂", "⠄", "⠂"],
            SpinnerFrames::Grow => &["▁", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃"],
            SpinnerFrames::Clock => &["🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚", "🕛"],
            SpinnerFrames::Moon => &["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"],
            SpinnerFrames::Ascii => &[".", "o", "O", "@", "*"],
            SpinnerFrames::Toggle => &["⊶", "⊷"],
        }
    }

    /// Get the recommended interval for this spinner style (in milliseconds)
    pub fn interval_ms(&self) -> u64 {
        match self {
            SpinnerFrames::Dots => 80,
            SpinnerFrames::Braille => 80,
            SpinnerFrames::Line => 100,
            SpinnerFrames::Circle => 100,
            SpinnerFrames::Box => 100,
            SpinnerFrames::Arrow => 100,
            SpinnerFrames::Bounce => 120,
            SpinnerFrames::Grow => 80,
            SpinnerFrames::Clock => 100,
            SpinnerFrames::Moon => 150,
            SpinnerFrames::Ascii => 150,
            SpinnerFrames::Toggle => 200,
        }
    }
}

/// State for the spinner animation
#[derive(Debug, Clone)]
pub struct SpinnerState {
    /// Current frame index
    pub frame: usize,
    /// Last tick time
    last_tick: Option<Instant>,
    /// Frame interval
    interval: Duration,
    /// Whether the spinner is active
    pub active: bool,
}

impl Default for SpinnerState {
    fn default() -> Self {
        Self::new()
    }
}

impl SpinnerState {
    /// Create a new spinner state
    pub fn new() -> Self {
        Self {
            frame: 0,
            last_tick: None,
            interval: Duration::from_millis(80),
            active: true,
        }
    }

    /// Create a new spinner state with a specific interval
    pub fn with_interval(interval_ms: u64) -> Self {
        Self {
            frame: 0,
            last_tick: None,
            interval: Duration::from_millis(interval_ms),
            active: true,
        }
    }

    /// Create a new spinner state configured for specific frames
    pub fn for_frames(frames: SpinnerFrames) -> Self {
        Self::with_interval(frames.interval_ms())
    }

    /// Set the frame interval
    pub fn set_interval(&mut self, interval_ms: u64) {
        self.interval = Duration::from_millis(interval_ms);
    }

    /// Advance to the next frame if enough time has passed
    ///
    /// Returns true if the frame changed
    pub fn tick(&mut self) -> bool {
        self.tick_with_frames(10) // Default frame count
    }

    /// Advance to the next frame with a specific frame count
    ///
    /// Returns true if the frame changed
    pub fn tick_with_frames(&mut self, frame_count: usize) -> bool {
        if !self.active || frame_count == 0 {
            return false;
        }

        let now = Instant::now();

        match self.last_tick {
            Some(last) if now.duration_since(last) >= self.interval => {
                self.frame = (self.frame + 1) % frame_count;
                self.last_tick = Some(now);
                true
            }
            None => {
                self.last_tick = Some(now);
                false
            }
            _ => false,
        }
    }

    /// Force advance to the next frame
    pub fn next_frame(&mut self, frame_count: usize) {
        if frame_count > 0 {
            self.frame = (self.frame + 1) % frame_count;
        }
    }

    /// Reset to the first frame
    pub fn reset(&mut self) {
        self.frame = 0;
        self.last_tick = None;
    }

    /// Start the spinner
    pub fn start(&mut self) {
        self.active = true;
    }

    /// Stop the spinner
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Check if the spinner is active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Label position relative to the spinner
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelPosition {
    /// Label appears before (left of) the spinner
    Before,
    /// Label appears after (right of) the spinner
    #[default]
    After,
}

/// Style configuration for spinners
#[derive(Debug, Clone)]
pub struct SpinnerStyle {
    /// Spinner frames to use
    pub frames: SpinnerFrames,
    /// Style for the spinner character
    pub spinner_style: Style,
    /// Style for the label text
    pub label_style: Style,
    /// Position of the label
    pub label_position: LabelPosition,
    /// Separator between spinner and label
    pub separator: &'static str,
}

impl Default for SpinnerStyle {
    fn default() -> Self {
        Self {
            frames: SpinnerFrames::Dots,
            spinner_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            label_style: Style::default().fg(Color::White),
            label_position: LabelPosition::After,
            separator: " ",
        }
    }
}

impl SpinnerStyle {
    /// Create a new spinner style with specific frames
    pub fn new(frames: SpinnerFrames) -> Self {
        Self {
            frames,
            ..Default::default()
        }
    }

    /// Set the spinner frames
    pub fn frames(mut self, frames: SpinnerFrames) -> Self {
        self.frames = frames;
        self
    }

    /// Set the spinner color
    pub fn color(mut self, color: Color) -> Self {
        self.spinner_style = self.spinner_style.fg(color);
        self
    }

    /// Set the label style
    pub fn label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }

    /// Set the label position
    pub fn label_position(mut self, position: LabelPosition) -> Self {
        self.label_position = position;
        self
    }

    /// Set the separator between spinner and label
    pub fn separator(mut self, separator: &'static str) -> Self {
        self.separator = separator;
        self
    }

    /// Success style (green spinner)
    pub fn success() -> Self {
        Self {
            spinner_style: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            ..Default::default()
        }
    }

    /// Warning style (yellow spinner)
    pub fn warning() -> Self {
        Self {
            spinner_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            ..Default::default()
        }
    }

    /// Error style (red spinner)
    pub fn error() -> Self {
        Self {
            spinner_style: Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
            ..Default::default()
        }
    }

    /// Info style (blue spinner)
    pub fn info() -> Self {
        Self {
            spinner_style: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            ..Default::default()
        }
    }

    /// Minimal style (dimmed)
    pub fn minimal() -> Self {
        Self {
            spinner_style: Style::default().fg(Color::DarkGray),
            label_style: Style::default().fg(Color::DarkGray),
            ..Default::default()
        }
    }
}

/// A spinner widget for loading/processing indicators
///
/// Displays an animated spinner character with an optional label.
#[derive(Debug, Clone)]
pub struct Spinner<'a> {
    /// Reference to the spinner state
    state: &'a SpinnerState,
    /// Optional label text
    label: Option<&'a str>,
    /// Style configuration
    style: SpinnerStyle,
}

impl<'a> Spinner<'a> {
    /// Create a new spinner with the given state
    pub fn new(state: &'a SpinnerState) -> Self {
        Self {
            state,
            label: None,
            style: SpinnerStyle::default(),
        }
    }

    /// Set the label text
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set the spinner frames
    pub fn frames(mut self, frames: SpinnerFrames) -> Self {
        self.style.frames = frames;
        self
    }

    /// Set the style
    pub fn style(mut self, style: SpinnerStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the spinner color
    pub fn color(mut self, color: Color) -> Self {
        self.style.spinner_style = self.style.spinner_style.fg(color);
        self
    }

    /// Set the label position
    pub fn label_position(mut self, position: LabelPosition) -> Self {
        self.style.label_position = position;
        self
    }

    /// Get the current frame character
    fn current_frame(&self) -> &'static str {
        let frames = self.style.frames.frames();
        let idx = self.state.frame % frames.len();
        frames[idx]
    }

    /// Calculate the display width of the spinner (including label)
    pub fn display_width(&self) -> usize {
        let frame_width = self.current_frame().width();
        match self.label {
            Some(label) => frame_width + self.style.separator.width() + label.width(),
            None => frame_width,
        }
    }
}

impl Widget for Spinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let frame = self.current_frame();
        let mut x = area.x;
        let y = area.y;

        match (self.label, self.style.label_position) {
            (Some(label), LabelPosition::Before) => {
                // Label first, then separator, then spinner
                let label_width = label.width() as u16;
                if x + label_width <= area.x + area.width {
                    buf.set_string(x, y, label, self.style.label_style);
                    x += label_width;
                }

                let sep_width = self.style.separator.width() as u16;
                if x + sep_width <= area.x + area.width {
                    buf.set_string(x, y, self.style.separator, Style::default());
                    x += sep_width;
                }

                let frame_width = frame.width() as u16;
                if x + frame_width <= area.x + area.width {
                    buf.set_string(x, y, frame, self.style.spinner_style);
                }
            }
            (Some(label), LabelPosition::After) => {
                // Spinner first, then separator, then label
                let frame_width = frame.width() as u16;
                if x + frame_width <= area.x + area.width {
                    buf.set_string(x, y, frame, self.style.spinner_style);
                    x += frame_width;
                }

                let sep_width = self.style.separator.width() as u16;
                if x + sep_width <= area.x + area.width {
                    buf.set_string(x, y, self.style.separator, Style::default());
                    x += sep_width;
                }

                let label_width = label.width() as u16;
                if x + label_width <= area.x + area.width {
                    buf.set_string(x, y, label, self.style.label_style);
                }
            }
            (None, _) => {
                // Just the spinner
                buf.set_string(x, y, frame, self.style.spinner_style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_state_new() {
        let state = SpinnerState::new();
        assert_eq!(state.frame, 0);
        assert!(state.active);
    }

    #[test]
    fn test_spinner_state_for_frames() {
        let state = SpinnerState::for_frames(SpinnerFrames::Braille);
        assert_eq!(state.interval, Duration::from_millis(80));
    }

    #[test]
    fn test_spinner_state_next_frame() {
        let mut state = SpinnerState::new();
        assert_eq!(state.frame, 0);

        state.next_frame(5);
        assert_eq!(state.frame, 1);

        state.next_frame(5);
        assert_eq!(state.frame, 2);

        // Wrap around
        state.frame = 4;
        state.next_frame(5);
        assert_eq!(state.frame, 0);
    }

    #[test]
    fn test_spinner_state_reset() {
        let mut state = SpinnerState::new();
        state.frame = 5;
        state.reset();
        assert_eq!(state.frame, 0);
    }

    #[test]
    fn test_spinner_state_start_stop() {
        let mut state = SpinnerState::new();
        assert!(state.is_active());

        state.stop();
        assert!(!state.is_active());

        state.start();
        assert!(state.is_active());
    }

    #[test]
    fn test_spinner_frames() {
        assert_eq!(SpinnerFrames::Dots.frames().len(), 10);
        assert_eq!(SpinnerFrames::Braille.frames().len(), 8);
        assert_eq!(SpinnerFrames::Line.frames().len(), 4);
        assert_eq!(SpinnerFrames::Circle.frames().len(), 4);
        assert_eq!(SpinnerFrames::Arrow.frames().len(), 8);
        assert_eq!(SpinnerFrames::Clock.frames().len(), 12);
        assert_eq!(SpinnerFrames::Moon.frames().len(), 8);
    }

    #[test]
    fn test_spinner_frames_interval() {
        assert_eq!(SpinnerFrames::Dots.interval_ms(), 80);
        assert_eq!(SpinnerFrames::Line.interval_ms(), 100);
        assert_eq!(SpinnerFrames::Moon.interval_ms(), 150);
    }

    #[test]
    fn test_spinner_style_presets() {
        let success = SpinnerStyle::success();
        assert_eq!(success.spinner_style.fg, Some(Color::Green));

        let warning = SpinnerStyle::warning();
        assert_eq!(warning.spinner_style.fg, Some(Color::Yellow));

        let error = SpinnerStyle::error();
        assert_eq!(error.spinner_style.fg, Some(Color::Red));

        let info = SpinnerStyle::info();
        assert_eq!(info.spinner_style.fg, Some(Color::Blue));
    }

    #[test]
    fn test_spinner_display_width() {
        let state = SpinnerState::new();

        let spinner = Spinner::new(&state);
        assert!(spinner.display_width() > 0);

        let spinner_with_label = Spinner::new(&state).label("Loading");
        assert!(spinner_with_label.display_width() > spinner.display_width());
    }

    #[test]
    fn test_spinner_current_frame() {
        let mut state = SpinnerState::new();
        let spinner = Spinner::new(&state).frames(SpinnerFrames::Line);

        // Frame 0 should be "|"
        assert_eq!(spinner.current_frame(), "|");

        state.frame = 1;
        let spinner = Spinner::new(&state).frames(SpinnerFrames::Line);
        assert_eq!(spinner.current_frame(), "/");

        state.frame = 2;
        let spinner = Spinner::new(&state).frames(SpinnerFrames::Line);
        assert_eq!(spinner.current_frame(), "-");

        state.frame = 3;
        let spinner = Spinner::new(&state).frames(SpinnerFrames::Line);
        assert_eq!(spinner.current_frame(), "\\");
    }

    #[test]
    fn test_spinner_render() {
        let state = SpinnerState::new();
        let spinner = Spinner::new(&state).label("Loading...");

        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
        spinner.render(Rect::new(0, 0, 20, 1), &mut buf);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_spinner_render_label_before() {
        let state = SpinnerState::new();
        let spinner = Spinner::new(&state)
            .label("Status:")
            .label_position(LabelPosition::Before);

        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
        spinner.render(Rect::new(0, 0, 20, 1), &mut buf);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_spinner_render_empty_area() {
        let state = SpinnerState::new();
        let spinner = Spinner::new(&state);

        let mut buf = Buffer::empty(Rect::new(0, 0, 0, 0));
        spinner.render(Rect::new(0, 0, 0, 0), &mut buf);
        // Should not panic on empty area
    }
}
