//! Progress bar widget
//!
//! A styled progress bar with label and step counter support.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{Progress, ProgressStyle};
//! use ratatui::layout::Rect;
//! use ratatui::buffer::Buffer;
//! use ratatui::widgets::Widget;
//!
//! // Simple progress bar
//! let progress = Progress::new(0.75)
//!     .label("Processing");
//!
//! // With step counter
//! let progress = Progress::new(0.5)
//!     .label("Building")
//!     .steps(5, 10);
//!
//! // Custom style
//! let progress = Progress::new(0.25)
//!     .style(ProgressStyle::warning());
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, Widget},
};

/// Style configuration for progress bars
#[derive(Debug, Clone)]
pub struct ProgressStyle {
    /// Foreground color of the filled portion
    pub filled_color: Color,
    /// Background color of the unfilled portion
    pub unfilled_color: Color,
    /// Style for the label text
    pub label_style: Style,
    /// Whether to show borders
    pub bordered: bool,
}

impl Default for ProgressStyle {
    fn default() -> Self {
        Self {
            filled_color: Color::Green,
            unfilled_color: Color::DarkGray,
            label_style: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            bordered: true,
        }
    }
}

impl ProgressStyle {
    /// Create a new progress style with custom colors
    pub fn new(filled: Color, unfilled: Color) -> Self {
        Self {
            filled_color: filled,
            unfilled_color: unfilled,
            ..Default::default()
        }
    }

    /// Success style (green)
    pub fn success() -> Self {
        Self::default()
    }

    /// Warning style (yellow)
    pub fn warning() -> Self {
        Self {
            filled_color: Color::Yellow,
            ..Default::default()
        }
    }

    /// Error style (red)
    pub fn error() -> Self {
        Self {
            filled_color: Color::Red,
            ..Default::default()
        }
    }

    /// Info style (cyan)
    pub fn info() -> Self {
        Self {
            filled_color: Color::Cyan,
            ..Default::default()
        }
    }

    /// Set whether to show borders
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }
}

/// A progress bar widget with label and step counter support.
///
/// The progress value should be between 0.0 and 1.0.
#[derive(Debug, Clone)]
pub struct Progress<'a> {
    /// Progress value (0.0 to 1.0)
    ratio: f64,
    /// Optional label text
    label: Option<&'a str>,
    /// Optional step counter (current, total)
    steps: Option<(usize, usize)>,
    /// Style configuration
    style: ProgressStyle,
}

impl<'a> Progress<'a> {
    /// Create a new progress bar with the given ratio (0.0 to 1.0)
    pub fn new(ratio: f64) -> Self {
        Self {
            ratio: ratio.clamp(0.0, 1.0),
            label: None,
            steps: None,
            style: ProgressStyle::default(),
        }
    }

    /// Create a progress bar from current/total values
    pub fn from_steps(current: usize, total: usize) -> Self {
        let ratio = if total > 0 {
            current as f64 / total as f64
        } else {
            0.0
        };
        Self::new(ratio).steps(current, total)
    }

    /// Set the label text
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set the step counter (current step, total steps)
    pub fn steps(mut self, current: usize, total: usize) -> Self {
        self.steps = Some((current, total));
        self
    }

    /// Set the style
    pub fn style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    /// Build the label string
    fn build_label(&self) -> String {
        let percent = (self.ratio * 100.0) as u16;

        match (&self.label, &self.steps) {
            (Some(label), Some((current, total))) => {
                format!("{} - {}/{} steps ({}%)", label, current, total, percent)
            }
            (Some(label), None) => {
                format!("{} ({}%)", label, percent)
            }
            (None, Some((current, total))) => {
                format!("{}/{} ({}%)", current, total, percent)
            }
            (None, None) => {
                format!("{}%", percent)
            }
        }
    }
}

impl Widget for Progress<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let label = self.build_label();
        let label_span = Span::styled(label, self.style.label_style);

        let mut gauge = Gauge::default()
            .gauge_style(
                Style::default()
                    .fg(self.style.filled_color)
                    .bg(self.style.unfilled_color),
            )
            .percent((self.ratio * 100.0) as u16)
            .label(label_span);

        if self.style.bordered {
            gauge = gauge.block(Block::default().borders(Borders::ALL));
        }

        gauge.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_new() {
        let p = Progress::new(0.5);
        assert!((p.ratio - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_progress_clamp() {
        let p = Progress::new(1.5);
        assert!((p.ratio - 1.0).abs() < 0.001);

        let p = Progress::new(-0.5);
        assert!((p.ratio - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_progress_from_steps() {
        let p = Progress::from_steps(5, 10);
        assert!((p.ratio - 0.5).abs() < 0.001);
        assert_eq!(p.steps, Some((5, 10)));
    }

    #[test]
    fn test_progress_label() {
        let p = Progress::new(0.75).label("Building");
        assert_eq!(p.build_label(), "Building (75%)");
    }

    #[test]
    fn test_progress_label_with_steps() {
        let p = Progress::new(0.5).label("Processing").steps(5, 10);
        assert_eq!(p.build_label(), "Processing - 5/10 steps (50%)");
    }

    #[test]
    fn test_progress_render() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 3));
        let progress = Progress::new(0.5).label("Test");
        progress.render(Rect::new(0, 0, 40, 3), &mut buf);
        // Just verify it doesn't panic
    }
}
