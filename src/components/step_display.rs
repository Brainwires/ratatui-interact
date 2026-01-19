//! Step display widget
//!
//! A multi-step progress display with expandable sub-steps and output areas.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{StepDisplay, StepDisplayState, Step, StepStatus};
//!
//! // Create steps
//! let steps = vec![
//!     Step::new("Build project")
//!         .with_sub_steps(vec!["Compile", "Link", "Package"]),
//!     Step::new("Run tests"),
//!     Step::new("Deploy"),
//! ];
//!
//! // Create state
//! let mut state = StepDisplayState::new(steps);
//!
//! // Update step status
//! state.start_step(0);
//! state.complete_step(0);
//! state.start_step(1);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::utils::display::{pad_to_width, truncate_to_width};

/// Status of a step or sub-step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepStatus {
    /// Not yet started
    #[default]
    Pending,
    /// Currently running
    Running,
    /// Successfully completed
    Completed,
    /// Failed with error
    Failed,
    /// Skipped
    Skipped,
}

impl StepStatus {
    /// Get the icon for this status
    pub fn icon(&self) -> &'static str {
        match self {
            StepStatus::Pending => "[ ]",
            StepStatus::Running => "[▶]",
            StepStatus::Completed => "[✓]",
            StepStatus::Failed => "[✗]",
            StepStatus::Skipped => "[↷]",
        }
    }

    /// Get the color for this status
    pub fn color(&self) -> Color {
        match self {
            StepStatus::Pending => Color::DarkGray,
            StepStatus::Running => Color::Yellow,
            StepStatus::Completed => Color::Green,
            StepStatus::Failed => Color::Red,
            StepStatus::Skipped => Color::DarkGray,
        }
    }

    /// Get the sub-step icon
    pub fn sub_icon(&self) -> &'static str {
        match self {
            StepStatus::Pending => "○",
            StepStatus::Running => "◐",
            StepStatus::Completed => "●",
            StepStatus::Failed => "✗",
            StepStatus::Skipped => "◌",
        }
    }
}

/// A sub-step within a step
#[derive(Debug, Clone)]
pub struct SubStep {
    /// Name of the sub-step
    pub name: String,
    /// Current status
    pub status: StepStatus,
}

impl SubStep {
    /// Create a new sub-step
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: StepStatus::Pending,
        }
    }
}

/// A step in the process
#[derive(Debug, Clone)]
pub struct Step {
    /// Name of the step
    pub name: String,
    /// Current status
    pub status: StepStatus,
    /// Sub-steps
    pub sub_steps: Vec<SubStep>,
    /// Whether the output is expanded
    pub expanded: bool,
    /// Output lines
    pub output: Vec<String>,
    /// Output scroll position
    pub scroll: u16,
}

impl Step {
    /// Create a new step
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: StepStatus::Pending,
            sub_steps: Vec::new(),
            expanded: false,
            output: Vec::new(),
            scroll: 0,
        }
    }

    /// Add sub-steps
    pub fn with_sub_steps(mut self, names: Vec<&str>) -> Self {
        self.sub_steps = names.into_iter().map(SubStep::new).collect();
        self
    }

    /// Add a line to output
    pub fn add_output(&mut self, line: impl Into<String>) {
        self.output.push(line.into());
        // Auto-scroll to bottom
        let visible_lines = 5;
        if self.output.len() > visible_lines {
            self.scroll = (self.output.len() - visible_lines) as u16;
        }
    }

    /// Clear output
    pub fn clear_output(&mut self) {
        self.output.clear();
        self.scroll = 0;
    }

    /// Get sub-step progress (completed, total)
    pub fn sub_step_progress(&self) -> (usize, usize) {
        let completed = self
            .sub_steps
            .iter()
            .filter(|s| s.status == StepStatus::Completed)
            .count();
        (completed, self.sub_steps.len())
    }
}

/// State for step display widget
#[derive(Debug, Clone)]
pub struct StepDisplayState {
    /// Steps
    pub steps: Vec<Step>,
    /// Currently focused step index
    pub focused_step: Option<usize>,
    /// Console scroll position
    pub scroll: u16,
}

impl StepDisplayState {
    /// Create a new step display state
    pub fn new(steps: Vec<Step>) -> Self {
        Self {
            steps,
            focused_step: None,
            scroll: 0,
        }
    }

    /// Get total progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        if self.steps.is_empty() {
            return 0.0;
        }
        let completed = self
            .steps
            .iter()
            .filter(|s| s.status == StepStatus::Completed)
            .count();
        completed as f64 / self.steps.len() as f64
    }

    /// Get current step index (first non-completed)
    pub fn current_step(&self) -> usize {
        self.steps
            .iter()
            .position(|s| s.status != StepStatus::Completed && s.status != StepStatus::Skipped)
            .unwrap_or(self.steps.len())
    }

    /// Start a step
    pub fn start_step(&mut self, index: usize) {
        if let Some(step) = self.steps.get_mut(index) {
            step.status = StepStatus::Running;
            step.expanded = true;
        }
    }

    /// Complete a step
    pub fn complete_step(&mut self, index: usize) {
        if let Some(step) = self.steps.get_mut(index) {
            step.status = StepStatus::Completed;
        }
    }

    /// Fail a step
    pub fn fail_step(&mut self, index: usize) {
        if let Some(step) = self.steps.get_mut(index) {
            step.status = StepStatus::Failed;
        }
    }

    /// Skip a step
    pub fn skip_step(&mut self, index: usize) {
        if let Some(step) = self.steps.get_mut(index) {
            step.status = StepStatus::Skipped;
        }
    }

    /// Start a sub-step
    pub fn start_sub_step(&mut self, step_index: usize, sub_index: usize) {
        if let Some(step) = self.steps.get_mut(step_index) {
            if let Some(sub) = step.sub_steps.get_mut(sub_index) {
                sub.status = StepStatus::Running;
            }
        }
    }

    /// Complete a sub-step
    pub fn complete_sub_step(&mut self, step_index: usize, sub_index: usize) {
        if let Some(step) = self.steps.get_mut(step_index) {
            if let Some(sub) = step.sub_steps.get_mut(sub_index) {
                sub.status = StepStatus::Completed;
            }
        }
    }

    /// Add output to a step
    pub fn add_output(&mut self, step_index: usize, line: impl Into<String>) {
        if let Some(step) = self.steps.get_mut(step_index) {
            step.add_output(line);
        }
    }

    /// Toggle expansion of a step
    pub fn toggle_expanded(&mut self, index: usize) {
        if let Some(step) = self.steps.get_mut(index) {
            step.expanded = !step.expanded;
        }
    }

    /// Scroll output for a step
    pub fn scroll_output(&mut self, index: usize, delta: i32) {
        if let Some(step) = self.steps.get_mut(index) {
            let max_scroll = step.output.len().saturating_sub(5) as i32;
            let new_scroll = (step.scroll as i32 + delta).clamp(0, max_scroll);
            step.scroll = new_scroll as u16;
        }
    }
}

/// Style for step display
#[derive(Debug, Clone)]
pub struct StepDisplayStyle {
    /// Output box border color when focused
    pub focused_border: Color,
    /// Output box border color when not focused
    pub unfocused_border: Color,
    /// Maximum visible output lines
    pub max_output_lines: usize,
}

impl Default for StepDisplayStyle {
    fn default() -> Self {
        Self {
            focused_border: Color::Cyan,
            unfocused_border: Color::DarkGray,
            max_output_lines: 5,
        }
    }
}

/// Step display widget
pub struct StepDisplay<'a> {
    state: &'a StepDisplayState,
    style: StepDisplayStyle,
}

impl<'a> StepDisplay<'a> {
    /// Create a new step display
    pub fn new(state: &'a StepDisplayState) -> Self {
        Self {
            state,
            style: StepDisplayStyle::default(),
        }
    }

    /// Set the style
    pub fn style(mut self, style: StepDisplayStyle) -> Self {
        self.style = style;
        self
    }

    /// Build content lines
    fn build_lines(&self, area: Rect) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let full_width = area.width as usize;

        for (idx, step) in self.state.steps.iter().enumerate() {
            // Step header
            let icon_color = step.status.color();
            let step_style = match step.status {
                StepStatus::Running => Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                StepStatus::Failed => Style::default().fg(Color::Red),
                StepStatus::Completed => Style::default().fg(Color::Green),
                _ => Style::default().fg(Color::White),
            };

            let header_suffix = if !step.sub_steps.is_empty() {
                let (completed, total) = step.sub_step_progress();
                format!(" ({}/{})", completed, total)
            } else {
                String::new()
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{} ", step.status.icon()),
                    Style::default().fg(icon_color),
                ),
                Span::styled(format!("Step {}: ", idx + 1), step_style),
                Span::styled(step.name.clone(), step_style),
                Span::styled(header_suffix, Style::default().fg(Color::DarkGray)),
            ]));

            // Sub-steps (if running or expanded)
            if !step.sub_steps.is_empty() && (step.expanded || step.status == StepStatus::Running) {
                for sub in &step.sub_steps {
                    let sub_color = sub.status.color();
                    let sub_style = match sub.status {
                        StepStatus::Running => Style::default().fg(Color::Yellow),
                        StepStatus::Completed => Style::default().fg(Color::Green),
                        StepStatus::Failed => Style::default().fg(Color::Red),
                        StepStatus::Skipped => Style::default().fg(Color::DarkGray),
                        _ => Style::default().fg(Color::White),
                    };

                    lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(
                            format!("{} ", sub.status.sub_icon()),
                            Style::default().fg(sub_color),
                        ),
                        Span::styled(sub.name.clone(), sub_style),
                    ]));
                }
            }

            // Output frame (if expanded and has output)
            if step.expanded && !step.output.is_empty() {
                let is_focused = self.state.focused_step == Some(idx);
                let border_color = if is_focused {
                    self.style.focused_border
                } else {
                    self.style.unfocused_border
                };

                let border_width = full_width.saturating_sub(6);
                let content_width = full_width.saturating_sub(8);

                // Top border
                lines.push(Line::from(Span::styled(
                    format!("  ┌{:─<width$}┐  ", " Output ", width = border_width),
                    Style::default().fg(border_color),
                )));

                // Output content
                let visible_lines = self.style.max_output_lines;
                let scroll = step.scroll as usize;
                let total = step.output.len();

                for i in 0..visible_lines {
                    let line_idx = scroll + i;
                    let content = if line_idx < total {
                        truncate_to_width(&step.output[line_idx], content_width)
                    } else {
                        String::new()
                    };

                    let padded = pad_to_width(&content, content_width);
                    lines.push(Line::from(vec![
                        Span::styled("  │ ", Style::default().fg(border_color)),
                        Span::styled(padded, Style::default().fg(Color::Gray)),
                        Span::styled(" │  ", Style::default().fg(border_color)),
                    ]));
                }

                // Bottom border with scroll info
                let scroll_info = if total > visible_lines {
                    format!(" [{}/{} lines] ", scroll + visible_lines.min(total), total)
                } else {
                    String::new()
                };

                lines.push(Line::from(Span::styled(
                    format!("  └{:─<width$}┘  ", scroll_info, width = border_width),
                    Style::default().fg(border_color),
                )));

                // Empty line after output
                lines.push(Line::from(""));
            }
        }

        lines
    }
}

impl Widget for StepDisplay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = self.build_lines(area);
        let para = Paragraph::new(lines).scroll((self.state.scroll, 0));
        para.render(area, buf);
    }
}

/// Calculate total height needed for step display
pub fn calculate_height(state: &StepDisplayState, style: &StepDisplayStyle) -> u16 {
    let mut height = 0u16;

    for step in &state.steps {
        height += 1; // Step header

        // Sub-steps
        if !step.sub_steps.is_empty() && (step.expanded || step.status == StepStatus::Running) {
            height += step.sub_steps.len() as u16;
        }

        // Output frame
        if step.expanded && !step.output.is_empty() {
            height += 2; // borders
            height += style.max_output_lines as u16;
            height += 1; // empty line after
        }
    }

    height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_status() {
        assert_eq!(StepStatus::Pending.icon(), "[ ]");
        assert_eq!(StepStatus::Running.icon(), "[▶]");
        assert_eq!(StepStatus::Completed.icon(), "[✓]");
    }

    #[test]
    fn test_step_progress() {
        let step = Step::new("Test").with_sub_steps(vec!["A", "B", "C"]);
        let (completed, total) = step.sub_step_progress();
        assert_eq!(completed, 0);
        assert_eq!(total, 3);
    }

    #[test]
    fn test_state_progress() {
        let steps = vec![
            Step::new("Step 1"),
            Step::new("Step 2"),
            Step::new("Step 3"),
            Step::new("Step 4"),
        ];
        let mut state = StepDisplayState::new(steps);

        assert_eq!(state.progress(), 0.0);

        state.complete_step(0);
        assert!((state.progress() - 0.25).abs() < 0.01);

        state.complete_step(1);
        assert!((state.progress() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_state_operations() {
        let steps = vec![Step::new("Test")];
        let mut state = StepDisplayState::new(steps);

        state.start_step(0);
        assert_eq!(state.steps[0].status, StepStatus::Running);
        assert!(state.steps[0].expanded);

        state.add_output(0, "Line 1");
        assert_eq!(state.steps[0].output.len(), 1);

        state.complete_step(0);
        assert_eq!(state.steps[0].status, StepStatus::Completed);
    }
}
