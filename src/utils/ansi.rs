//! ANSI Parser and Markdown Rendering
//!
//! Parses ANSI escape codes and renders markdown to ratatui styles.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::utils::ansi::{parse_ansi_to_spans, render_markdown_to_lines};
//!
//! // Parse ANSI escape codes to styled spans
//! let text = "\x1b[31mRed text\x1b[0m Normal text";
//! let spans = parse_ansi_to_spans(text);
//!
//! // Render markdown to lines (requires termimad feature)
//! // let lines = render_markdown_to_lines("# Header\n**Bold** text");
//! ```

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Parse ANSI escape codes and convert to ratatui Style.
///
/// Returns a vector of styled Spans that can be used with ratatui widgets.
///
/// # Supported ANSI codes
///
/// - Reset (0)
/// - Bold (1), Dim (2), Italic (3), Underline (4)
/// - Reversed (7), Strikethrough (9)
/// - Foreground colors (30-37, 90-97)
/// - Background colors (40-47, 100-107)
/// - 256 color mode (38;5;n, 48;5;n)
/// - RGB color mode (38;2;r;g;b, 48;2;r;g;b)
///
/// # Example
///
/// ```rust
/// use ratatui_interact::utils::ansi::parse_ansi_to_spans;
///
/// let spans = parse_ansi_to_spans("\x1b[1;32mBold green\x1b[0m normal");
/// assert_eq!(spans.len(), 2);
/// ```
pub fn parse_ansi_to_spans(text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut current_style = Style::default();
    let mut current_text = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Start of ANSI escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['

                // Flush current text with current style
                if !current_text.is_empty() {
                    spans.push(Span::styled(
                        std::mem::take(&mut current_text),
                        current_style,
                    ));
                }

                // Parse the escape sequence parameters
                let mut params = String::new();
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() || next == ';' {
                        params.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                // Get the command character
                let cmd = chars.next();

                if cmd == Some('m') {
                    // SGR (Select Graphic Rendition) sequence
                    current_style = parse_sgr_params(&params, current_style);
                }
                // Ignore other escape sequences
            } else {
                current_text.push(c);
            }
        } else {
            current_text.push(c);
        }
    }

    // Flush remaining text
    if !current_text.is_empty() {
        spans.push(Span::styled(current_text, current_style));
    }

    if spans.is_empty() {
        spans.push(Span::raw(""));
    }

    spans
}

/// Parse SGR (Select Graphic Rendition) parameters and update style
fn parse_sgr_params(params: &str, mut style: Style) -> Style {
    if params.is_empty() || params == "0" {
        return Style::default();
    }

    let codes: Vec<u8> = params
        .split(';')
        .filter_map(|s| s.parse::<u8>().ok())
        .collect();

    let mut i = 0;
    while i < codes.len() {
        match codes[i] {
            0 => style = Style::default(),
            1 => style = style.add_modifier(Modifier::BOLD),
            2 => style = style.add_modifier(Modifier::DIM),
            3 => style = style.add_modifier(Modifier::ITALIC),
            4 => style = style.add_modifier(Modifier::UNDERLINED),
            7 => style = style.add_modifier(Modifier::REVERSED),
            9 => style = style.add_modifier(Modifier::CROSSED_OUT),
            22 => style = style.remove_modifier(Modifier::BOLD | Modifier::DIM),
            23 => style = style.remove_modifier(Modifier::ITALIC),
            24 => style = style.remove_modifier(Modifier::UNDERLINED),
            27 => style = style.remove_modifier(Modifier::REVERSED),
            29 => style = style.remove_modifier(Modifier::CROSSED_OUT),
            // Foreground colors (30-37)
            30 => style = style.fg(Color::Black),
            31 => style = style.fg(Color::Red),
            32 => style = style.fg(Color::Green),
            33 => style = style.fg(Color::Yellow),
            34 => style = style.fg(Color::Blue),
            35 => style = style.fg(Color::Magenta),
            36 => style = style.fg(Color::Cyan),
            37 => style = style.fg(Color::White),
            38 => {
                // Extended foreground color
                if i + 2 < codes.len() && codes[i + 1] == 5 {
                    // 256 color mode: 38;5;n
                    style = style.fg(Color::Indexed(codes[i + 2]));
                    i += 2;
                } else if i + 4 < codes.len() && codes[i + 1] == 2 {
                    // RGB color mode: 38;2;r;g;b
                    style = style.fg(Color::Rgb(codes[i + 2], codes[i + 3], codes[i + 4]));
                    i += 4;
                }
            }
            39 => style = style.fg(Color::Reset),
            // Background colors (40-47)
            40 => style = style.bg(Color::Black),
            41 => style = style.bg(Color::Red),
            42 => style = style.bg(Color::Green),
            43 => style = style.bg(Color::Yellow),
            44 => style = style.bg(Color::Blue),
            45 => style = style.bg(Color::Magenta),
            46 => style = style.bg(Color::Cyan),
            47 => style = style.bg(Color::White),
            48 => {
                // Extended background color
                if i + 2 < codes.len() && codes[i + 1] == 5 {
                    // 256 color mode: 48;5;n
                    style = style.bg(Color::Indexed(codes[i + 2]));
                    i += 2;
                } else if i + 4 < codes.len() && codes[i + 1] == 2 {
                    // RGB color mode: 48;2;r;g;b
                    style = style.bg(Color::Rgb(codes[i + 2], codes[i + 3], codes[i + 4]));
                    i += 4;
                }
            }
            49 => style = style.bg(Color::Reset),
            // Bright foreground colors (90-97)
            90 => style = style.fg(Color::DarkGray),
            91 => style = style.fg(Color::LightRed),
            92 => style = style.fg(Color::LightGreen),
            93 => style = style.fg(Color::LightYellow),
            94 => style = style.fg(Color::LightBlue),
            95 => style = style.fg(Color::LightMagenta),
            96 => style = style.fg(Color::LightCyan),
            97 => style = style.fg(Color::White),
            // Bright background colors (100-107)
            100 => style = style.bg(Color::DarkGray),
            101 => style = style.bg(Color::LightRed),
            102 => style = style.bg(Color::LightGreen),
            103 => style = style.bg(Color::LightYellow),
            104 => style = style.bg(Color::LightBlue),
            105 => style = style.bg(Color::LightMagenta),
            106 => style = style.bg(Color::LightCyan),
            107 => style = style.bg(Color::White),
            _ => {} // Ignore unknown codes
        }
        i += 1;
    }

    style
}

/// Convert markdown text to styled Lines for ratatui.
///
/// This function requires the `termimad` crate to be available.
/// It renders markdown to terminal text with ANSI codes, then converts
/// those codes to ratatui styles.
///
/// # Example
///
/// ```rust,ignore
/// use ratatui_interact::utils::ansi::render_markdown_to_lines;
///
/// let lines = render_markdown_to_lines("# Header\n**Bold** and *italic*");
/// ```
#[cfg(feature = "markdown")]
pub fn render_markdown_to_lines(markdown: &str) -> Vec<Line<'static>> {
    use termimad::{crossterm::style::Color as TermColor, MadSkin};

    let mut skin = MadSkin::default();

    // Configure colors for terminal markdown
    skin.headers[0].set_fg(TermColor::Cyan);
    skin.headers[1].set_fg(TermColor::Blue);
    skin.bold.set_fg(TermColor::Yellow);
    skin.italic.set_fg(TermColor::Magenta);
    skin.inline_code.set_fg(TermColor::Green);
    skin.code_block.set_fg(TermColor::Green);

    // Render markdown to terminal text (contains ANSI codes)
    let rendered = skin.text(markdown, None);
    let rendered_str = rendered.to_string();

    // Convert to ratatui Lines, parsing ANSI codes to styles
    let mut lines = Vec::new();
    for line in rendered_str.lines() {
        let spans = parse_ansi_to_spans(line);
        lines.push(Line::from(spans));
    }

    // If no lines were generated (empty markdown), add at least one empty line
    if lines.is_empty() {
        lines.push(Line::from(""));
    }

    lines
}

/// Stub for render_markdown_to_lines when markdown feature is disabled
#[cfg(not(feature = "markdown"))]
pub fn render_markdown_to_lines(markdown: &str) -> Vec<Line<'static>> {
    // Without termimad, just return plain text lines
    markdown.lines().map(|l| Line::from(l.to_string())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plain_text() {
        let spans = parse_ansi_to_spans("Hello world");
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn test_parse_red_text() {
        let spans = parse_ansi_to_spans("\x1b[31mRed\x1b[0m");
        assert_eq!(spans.len(), 1); // Just "Red" with style
        assert_eq!(spans[0].content, "Red");
    }

    #[test]
    fn test_parse_bold() {
        let spans = parse_ansi_to_spans("\x1b[1mBold\x1b[0m normal");
        assert_eq!(spans.len(), 2); // "Bold" + " normal"
    }

    #[test]
    fn test_parse_256_color() {
        let spans = parse_ansi_to_spans("\x1b[38;5;196mRed 256\x1b[0m");
        assert_eq!(spans.len(), 1); // Just "Red 256" with style
        assert_eq!(spans[0].content, "Red 256");
    }

    #[test]
    fn test_parse_rgb_color() {
        let spans = parse_ansi_to_spans("\x1b[38;2;255;0;0mRGB Red\x1b[0m");
        assert_eq!(spans.len(), 1); // Just "RGB Red" with style
        assert_eq!(spans[0].content, "RGB Red");
    }

    #[test]
    fn test_empty_string() {
        let spans = parse_ansi_to_spans("");
        assert_eq!(spans.len(), 1);
    }
}
