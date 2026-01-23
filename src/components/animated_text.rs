//! AnimatedText widget for animated text labels with color effects
//!
//! Provides animated text display with multiple effect modes:
//! - Pulse: Entire text oscillates between two colors
//! - Wave: A highlighted portion travels back and forth across the text
//! - Rainbow: Colors cycle through a spectrum across the text
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{AnimatedText, AnimatedTextState, AnimatedTextStyle, AnimatedTextEffect};
//! use ratatui::layout::Rect;
//! use ratatui::buffer::Buffer;
//! use ratatui::widgets::Widget;
//! use ratatui::style::Color;
//!
//! // Create state and advance each frame
//! let mut state = AnimatedTextState::new();
//!
//! // Pulse effect - entire text oscillates between colors
//! let style = AnimatedTextStyle::pulse(Color::Cyan, Color::Blue);
//! let text = AnimatedText::new("Loading...", &state).style(style);
//!
//! // Wave effect - highlight travels across the text
//! let style = AnimatedTextStyle::wave(Color::White, Color::Yellow);
//! let text = AnimatedText::new("Processing data", &state).style(style);
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

/// Animation effect types for animated text
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimatedTextEffect {
    /// Entire text pulses/oscillates between two colors
    #[default]
    Pulse,
    /// A highlighted portion travels back and forth (wave effect)
    Wave,
    /// Colors cycle through a rainbow spectrum across the text
    Rainbow,
    /// Gradient that shifts over time
    GradientShift,
    /// Text appears to sparkle with random highlights
    Sparkle,
}

/// Direction for wave animation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WaveDirection {
    /// Wave travels left to right
    #[default]
    Forward,
    /// Wave travels right to left
    Backward,
}

/// State for animated text
#[derive(Debug, Clone)]
pub struct AnimatedTextState {
    /// Current animation frame (0-255 for smooth transitions)
    pub frame: u8,
    /// Wave position (for wave effect)
    pub wave_position: usize,
    /// Wave direction (for bounce behavior)
    pub wave_direction: WaveDirection,
    /// Last tick time
    last_tick: Option<Instant>,
    /// Frame interval
    interval: Duration,
    /// Whether the animation is active
    pub active: bool,
    /// Random seed for sparkle effect
    sparkle_seed: u64,
}

impl Default for AnimatedTextState {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimatedTextState {
    /// Create a new animated text state
    pub fn new() -> Self {
        Self {
            frame: 0,
            wave_position: 0,
            wave_direction: WaveDirection::Forward,
            last_tick: None,
            interval: Duration::from_millis(50),
            active: true,
            sparkle_seed: 0,
        }
    }

    /// Create state with a specific interval
    pub fn with_interval(interval_ms: u64) -> Self {
        Self {
            interval: Duration::from_millis(interval_ms),
            ..Self::new()
        }
    }

    /// Set the animation interval
    pub fn set_interval(&mut self, interval_ms: u64) {
        self.interval = Duration::from_millis(interval_ms);
    }

    /// Advance the animation by one tick
    ///
    /// Returns true if the frame changed
    pub fn tick(&mut self) -> bool {
        self.tick_with_text_width(20) // Default width assumption
    }

    /// Advance the animation with known text width (for wave calculations)
    ///
    /// Returns true if the frame changed
    pub fn tick_with_text_width(&mut self, text_width: usize) -> bool {
        if !self.active {
            return false;
        }

        let now = Instant::now();

        match self.last_tick {
            Some(last) if now.duration_since(last) >= self.interval => {
                // Advance frame (wraps at 256)
                self.frame = self.frame.wrapping_add(4);

                // Update wave position
                let max_pos = text_width.saturating_sub(1);
                match self.wave_direction {
                    WaveDirection::Forward => {
                        if self.wave_position >= max_pos {
                            self.wave_direction = WaveDirection::Backward;
                        } else {
                            self.wave_position += 1;
                        }
                    }
                    WaveDirection::Backward => {
                        if self.wave_position == 0 {
                            self.wave_direction = WaveDirection::Forward;
                        } else {
                            self.wave_position -= 1;
                        }
                    }
                }

                // Update sparkle seed
                self.sparkle_seed = self.sparkle_seed.wrapping_add(1);

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

    /// Reset the animation to initial state
    pub fn reset(&mut self) {
        self.frame = 0;
        self.wave_position = 0;
        self.wave_direction = WaveDirection::Forward;
        self.last_tick = None;
        self.sparkle_seed = 0;
    }

    /// Start the animation
    pub fn start(&mut self) {
        self.active = true;
    }

    /// Stop the animation
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Check if the animation is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get the current interpolation factor (0.0 to 1.0)
    pub fn interpolation_factor(&self) -> f32 {
        // Use a sine wave for smooth oscillation
        let radians = (self.frame as f32 / 255.0) * std::f32::consts::PI * 2.0;
        (radians.sin() + 1.0) / 2.0
    }
}

/// Style configuration for animated text
#[derive(Debug, Clone)]
pub struct AnimatedTextStyle {
    /// Animation effect type
    pub effect: AnimatedTextEffect,
    /// Primary/base color
    pub primary_color: Color,
    /// Secondary color (for pulse/wave effects)
    pub secondary_color: Color,
    /// Text modifiers (bold, italic, etc.)
    pub modifiers: Modifier,
    /// Width of the wave highlight (in characters)
    pub wave_width: usize,
    /// Background color (optional)
    pub background: Option<Color>,
    /// Rainbow colors for rainbow effect
    pub rainbow_colors: Vec<Color>,
}

impl Default for AnimatedTextStyle {
    fn default() -> Self {
        Self {
            effect: AnimatedTextEffect::Pulse,
            primary_color: Color::White,
            secondary_color: Color::Cyan,
            modifiers: Modifier::empty(),
            wave_width: 3,
            background: None,
            rainbow_colors: vec![
                Color::Red,
                Color::Yellow,
                Color::Green,
                Color::Cyan,
                Color::Blue,
                Color::Magenta,
            ],
        }
    }
}

impl AnimatedTextStyle {
    /// Create a new style with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a pulse effect style
    pub fn pulse(primary: Color, secondary: Color) -> Self {
        Self {
            effect: AnimatedTextEffect::Pulse,
            primary_color: primary,
            secondary_color: secondary,
            ..Default::default()
        }
    }

    /// Create a wave effect style
    pub fn wave(base: Color, highlight: Color) -> Self {
        Self {
            effect: AnimatedTextEffect::Wave,
            primary_color: base,
            secondary_color: highlight,
            wave_width: 3,
            ..Default::default()
        }
    }

    /// Create a rainbow effect style
    pub fn rainbow() -> Self {
        Self {
            effect: AnimatedTextEffect::Rainbow,
            ..Default::default()
        }
    }

    /// Create a gradient shift effect style
    pub fn gradient_shift(start: Color, end: Color) -> Self {
        Self {
            effect: AnimatedTextEffect::GradientShift,
            primary_color: start,
            secondary_color: end,
            ..Default::default()
        }
    }

    /// Create a sparkle effect style
    pub fn sparkle(base: Color, sparkle: Color) -> Self {
        Self {
            effect: AnimatedTextEffect::Sparkle,
            primary_color: base,
            secondary_color: sparkle,
            ..Default::default()
        }
    }

    /// Set the animation effect
    pub fn effect(mut self, effect: AnimatedTextEffect) -> Self {
        self.effect = effect;
        self
    }

    /// Set the primary color
    pub fn primary_color(mut self, color: Color) -> Self {
        self.primary_color = color;
        self
    }

    /// Set the secondary color
    pub fn secondary_color(mut self, color: Color) -> Self {
        self.secondary_color = color;
        self
    }

    /// Set text modifiers
    pub fn modifiers(mut self, modifiers: Modifier) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Add bold modifier
    pub fn bold(mut self) -> Self {
        self.modifiers = self.modifiers | Modifier::BOLD;
        self
    }

    /// Add italic modifier
    pub fn italic(mut self) -> Self {
        self.modifiers = self.modifiers | Modifier::ITALIC;
        self
    }

    /// Set the wave width
    pub fn wave_width(mut self, width: usize) -> Self {
        self.wave_width = width.max(1);
        self
    }

    /// Set the background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Set custom rainbow colors
    pub fn rainbow_colors(mut self, colors: Vec<Color>) -> Self {
        if !colors.is_empty() {
            self.rainbow_colors = colors;
        }
        self
    }

    // Preset styles

    /// Success style (green pulse)
    pub fn success() -> Self {
        Self::pulse(Color::Green, Color::LightGreen)
            .bold()
    }

    /// Warning style (yellow pulse)
    pub fn warning() -> Self {
        Self::pulse(Color::Yellow, Color::LightYellow)
            .bold()
    }

    /// Error style (red pulse)
    pub fn error() -> Self {
        Self::pulse(Color::Red, Color::LightRed)
            .bold()
    }

    /// Info style (blue wave)
    pub fn info() -> Self {
        Self::wave(Color::Blue, Color::Cyan)
    }

    /// Loading style (cyan wave)
    pub fn loading() -> Self {
        Self::wave(Color::DarkGray, Color::Cyan)
            .wave_width(5)
    }

    /// Highlight style (yellow sparkle)
    pub fn highlight() -> Self {
        Self::sparkle(Color::White, Color::Yellow)
    }
}

/// An animated text widget with color effects
///
/// Displays text with animated color transitions, including:
/// - Pulse: Color oscillates between two values
/// - Wave: A highlight travels back and forth
/// - Rainbow: Colors cycle across the text
#[derive(Debug, Clone)]
pub struct AnimatedText<'a> {
    /// The text to display
    text: &'a str,
    /// Reference to the animation state
    state: &'a AnimatedTextState,
    /// Style configuration
    style: AnimatedTextStyle,
}

impl<'a> AnimatedText<'a> {
    /// Create a new animated text widget
    pub fn new(text: &'a str, state: &'a AnimatedTextState) -> Self {
        Self {
            text,
            state,
            style: AnimatedTextStyle::default(),
        }
    }

    /// Set the style
    pub fn style(mut self, style: AnimatedTextStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the effect directly
    pub fn effect(mut self, effect: AnimatedTextEffect) -> Self {
        self.style.effect = effect;
        self
    }

    /// Set colors directly (primary and secondary)
    pub fn colors(mut self, primary: Color, secondary: Color) -> Self {
        self.style.primary_color = primary;
        self.style.secondary_color = secondary;
        self
    }

    /// Get the display width of the text
    pub fn display_width(&self) -> usize {
        self.text.width()
    }

    /// Interpolate between two colors based on factor (0.0 to 1.0)
    fn interpolate_color(c1: Color, c2: Color, factor: f32) -> Color {
        match (c1, c2) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                let r = (r1 as f32 + (r2 as f32 - r1 as f32) * factor) as u8;
                let g = (g1 as f32 + (g2 as f32 - g1 as f32) * factor) as u8;
                let b = (b1 as f32 + (b2 as f32 - b1 as f32) * factor) as u8;
                Color::Rgb(r, g, b)
            }
            _ => {
                // For non-RGB colors, just switch at midpoint
                if factor < 0.5 { c1 } else { c2 }
            }
        }
    }

    /// Get color for pulse effect
    fn pulse_color(&self) -> Color {
        let factor = self.state.interpolation_factor();
        Self::interpolate_color(self.style.primary_color, self.style.secondary_color, factor)
    }

    /// Get color for a specific character position in wave effect
    fn wave_color(&self, char_index: usize) -> Color {
        let wave_center = self.state.wave_position;
        let half_width = self.style.wave_width / 2;
        let start = wave_center.saturating_sub(half_width);
        let end = wave_center + half_width + 1;

        if char_index >= start && char_index < end {
            // Calculate intensity based on distance from center
            let distance = if char_index >= wave_center {
                char_index - wave_center
            } else {
                wave_center - char_index
            };
            let max_distance = half_width.max(1);
            let intensity = 1.0 - (distance as f32 / max_distance as f32);
            Self::interpolate_color(
                self.style.primary_color,
                self.style.secondary_color,
                intensity,
            )
        } else {
            self.style.primary_color
        }
    }

    /// Get color for a specific character position in rainbow effect
    fn rainbow_color(&self, char_index: usize) -> Color {
        let colors = &self.style.rainbow_colors;
        if colors.is_empty() {
            return self.style.primary_color;
        }

        // Offset the rainbow based on frame for animation
        let offset = (self.state.frame as usize) / 16;
        let color_index = (char_index + offset) % colors.len();
        colors[color_index]
    }

    /// Get color for gradient shift effect
    fn gradient_color(&self, char_index: usize, text_width: usize) -> Color {
        if text_width == 0 {
            return self.style.primary_color;
        }

        // Calculate position in gradient (0.0 to 1.0)
        let base_position = char_index as f32 / text_width.max(1) as f32;

        // Shift the gradient based on frame
        let shift = self.state.frame as f32 / 255.0;
        let position = (base_position + shift) % 1.0;

        Self::interpolate_color(self.style.primary_color, self.style.secondary_color, position)
    }

    /// Check if a character should sparkle
    fn should_sparkle(&self, char_index: usize) -> bool {
        // Simple pseudo-random based on position and seed
        let hash = char_index.wrapping_mul(31).wrapping_add(self.state.sparkle_seed as usize);
        hash % 8 == 0 // ~12.5% chance
    }

    /// Get color for sparkle effect
    fn sparkle_color(&self, char_index: usize) -> Color {
        if self.should_sparkle(char_index) {
            self.style.secondary_color
        } else {
            self.style.primary_color
        }
    }
}

impl Widget for AnimatedText<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let text_width = self.text.width();
        let mut x = area.x;
        let y = area.y;

        // Build base style with modifiers
        let base_style = Style::default()
            .add_modifier(self.style.modifiers);

        let base_style = if let Some(bg) = self.style.background {
            base_style.bg(bg)
        } else {
            base_style
        };

        // Render each character with its color
        let mut char_index = 0;
        for ch in self.text.chars() {
            let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);

            if x >= area.x + area.width {
                break;
            }

            // Get color based on effect type
            let fg_color = match self.style.effect {
                AnimatedTextEffect::Pulse => self.pulse_color(),
                AnimatedTextEffect::Wave => self.wave_color(char_index),
                AnimatedTextEffect::Rainbow => self.rainbow_color(char_index),
                AnimatedTextEffect::GradientShift => self.gradient_color(char_index, text_width),
                AnimatedTextEffect::Sparkle => self.sparkle_color(char_index),
            };

            let style = base_style.fg(fg_color);

            // Only render if it fits
            if x as usize + ch_width <= (area.x + area.width) as usize {
                buf.set_string(x, y, ch.to_string(), style);
                x += ch_width as u16;
            }

            char_index += 1;
        }

        // Clear rest of the area if text is shorter
        while x < area.x + area.width {
            buf.set_string(x, y, " ", base_style);
            x += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animated_text_state_new() {
        let state = AnimatedTextState::new();
        assert_eq!(state.frame, 0);
        assert_eq!(state.wave_position, 0);
        assert!(state.active);
    }

    #[test]
    fn test_animated_text_state_with_interval() {
        let state = AnimatedTextState::with_interval(100);
        assert_eq!(state.interval, Duration::from_millis(100));
    }

    #[test]
    fn test_animated_text_state_reset() {
        let mut state = AnimatedTextState::new();
        state.frame = 128;
        state.wave_position = 10;
        state.wave_direction = WaveDirection::Backward;

        state.reset();

        assert_eq!(state.frame, 0);
        assert_eq!(state.wave_position, 0);
        assert_eq!(state.wave_direction, WaveDirection::Forward);
    }

    #[test]
    fn test_animated_text_state_start_stop() {
        let mut state = AnimatedTextState::new();
        assert!(state.is_active());

        state.stop();
        assert!(!state.is_active());

        state.start();
        assert!(state.is_active());
    }

    #[test]
    fn test_animated_text_state_interpolation() {
        let mut state = AnimatedTextState::new();

        // At frame 0, should be near 0.5 (sin(0) = 0, (0+1)/2 = 0.5)
        let factor = state.interpolation_factor();
        assert!((factor - 0.5).abs() < 0.1);

        // At frame 64 (quarter turn), should be near 1.0
        state.frame = 64;
        let factor = state.interpolation_factor();
        assert!(factor > 0.8);

        // At frame 192 (three-quarter turn), should be near 0.0
        state.frame = 192;
        let factor = state.interpolation_factor();
        assert!(factor < 0.2);
    }

    #[test]
    fn test_animated_text_style_presets() {
        let pulse = AnimatedTextStyle::pulse(Color::Red, Color::Blue);
        assert_eq!(pulse.effect, AnimatedTextEffect::Pulse);
        assert_eq!(pulse.primary_color, Color::Red);
        assert_eq!(pulse.secondary_color, Color::Blue);

        let wave = AnimatedTextStyle::wave(Color::White, Color::Yellow);
        assert_eq!(wave.effect, AnimatedTextEffect::Wave);

        let rainbow = AnimatedTextStyle::rainbow();
        assert_eq!(rainbow.effect, AnimatedTextEffect::Rainbow);
    }

    #[test]
    fn test_animated_text_style_builder() {
        let style = AnimatedTextStyle::new()
            .effect(AnimatedTextEffect::Wave)
            .primary_color(Color::Green)
            .secondary_color(Color::Cyan)
            .wave_width(5)
            .bold();

        assert_eq!(style.effect, AnimatedTextEffect::Wave);
        assert_eq!(style.primary_color, Color::Green);
        assert_eq!(style.secondary_color, Color::Cyan);
        assert_eq!(style.wave_width, 5);
        assert!(style.modifiers.contains(Modifier::BOLD));
    }

    #[test]
    fn test_animated_text_style_presets_themed() {
        let success = AnimatedTextStyle::success();
        assert_eq!(success.primary_color, Color::Green);

        let warning = AnimatedTextStyle::warning();
        assert_eq!(warning.primary_color, Color::Yellow);

        let error = AnimatedTextStyle::error();
        assert_eq!(error.primary_color, Color::Red);

        let info = AnimatedTextStyle::info();
        assert_eq!(info.effect, AnimatedTextEffect::Wave);
    }

    #[test]
    fn test_animated_text_display_width() {
        let state = AnimatedTextState::new();
        let text = AnimatedText::new("Hello", &state);
        assert_eq!(text.display_width(), 5);

        let text = AnimatedText::new("Hello World", &state);
        assert_eq!(text.display_width(), 11);
    }

    #[test]
    fn test_animated_text_render() {
        let state = AnimatedTextState::new();
        let text = AnimatedText::new("Test", &state);

        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));
        text.render(Rect::new(0, 0, 10, 1), &mut buf);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_animated_text_render_wave() {
        let mut state = AnimatedTextState::new();
        state.wave_position = 2;

        let style = AnimatedTextStyle::wave(Color::White, Color::Yellow);
        let text = AnimatedText::new("Hello", &state).style(style);

        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));
        text.render(Rect::new(0, 0, 10, 1), &mut buf);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_animated_text_render_rainbow() {
        let state = AnimatedTextState::new();
        let style = AnimatedTextStyle::rainbow();
        let text = AnimatedText::new("Rainbow!", &state).style(style);

        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));
        text.render(Rect::new(0, 0, 10, 1), &mut buf);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_animated_text_render_empty_area() {
        let state = AnimatedTextState::new();
        let text = AnimatedText::new("Test", &state);

        let mut buf = Buffer::empty(Rect::new(0, 0, 0, 0));
        text.render(Rect::new(0, 0, 0, 0), &mut buf);
        // Should not panic on empty area
    }

    #[test]
    fn test_interpolate_color_rgb() {
        // Test RGB interpolation
        let c1 = Color::Rgb(0, 0, 0);
        let c2 = Color::Rgb(255, 255, 255);

        let result = AnimatedText::interpolate_color(c1, c2, 0.5);
        if let Color::Rgb(r, g, b) = result {
            assert!((r as i16 - 127).abs() <= 1);
            assert!((g as i16 - 127).abs() <= 1);
            assert!((b as i16 - 127).abs() <= 1);
        } else {
            panic!("Expected RGB color");
        }
    }

    #[test]
    fn test_interpolate_color_non_rgb() {
        // Non-RGB colors should switch at midpoint
        let c1 = Color::Red;
        let c2 = Color::Blue;

        assert_eq!(AnimatedText::interpolate_color(c1, c2, 0.3), Color::Red);
        assert_eq!(AnimatedText::interpolate_color(c1, c2, 0.7), Color::Blue);
    }

    #[test]
    fn test_wave_direction_changes() {
        let mut state = AnimatedTextState::new();
        state.interval = Duration::from_millis(0); // Immediate ticks
        state.last_tick = Some(Instant::now() - Duration::from_secs(1));

        // Move forward
        let text_width = 10;
        for _ in 0..15 {
            state.tick_with_text_width(text_width);
        }

        // Should have hit the end and reversed
        assert_eq!(state.wave_direction, WaveDirection::Backward);
    }
}
