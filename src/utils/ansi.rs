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
        // Introducer of a control sequence or control string, if any.
        // Both the 7-bit (ESC x) and 8-bit C1 forms are recognised.
        let introducer = match c {
            '\x1b' => match chars.peek() {
                Some(&next @ ('[' | ']' | 'P' | 'X' | '^' | '_')) => {
                    chars.next();
                    Some(next)
                }
                _ => None,
            },
            '\u{9b}' => Some('['),
            '\u{9d}' => Some(']'),
            '\u{90}' => Some('P'),
            '\u{98}' => Some('X'),
            '\u{9e}' => Some('^'),
            '\u{9f}' => Some('_'),
            _ => None,
        };

        match introducer {
            Some('[') => {
                // CSI: parameter bytes 0x30-0x3F, intermediate bytes
                // 0x20-0x2F, final byte 0x40-0x7E (ECMA-48 5.4).
                let mut params = String::new();
                while let Some(&next) = chars.peek() {
                    if ('\x30'..='\x3f').contains(&next) {
                        params.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let mut has_intermediates = false;
                while let Some(&next) = chars.peek() {
                    if ('\x20'..='\x2f').contains(&next) {
                        has_intermediates = true;
                        chars.next();
                    } else {
                        break;
                    }
                }

                // Only consume the final byte if it really is one; a
                // truncated/malformed sequence must not eat the next char.
                let final_byte = match chars.peek() {
                    Some(&next) if ('\x40'..='\x7e').contains(&next) => {
                        chars.next();
                        Some(next)
                    }
                    _ => None,
                };

                if final_byte == Some('m')
                    && !has_intermediates
                    && params.chars().all(|p| p.is_ascii_digit() || p == ';')
                {
                    // SGR (Select Graphic Rendition) sequence
                    if !current_text.is_empty() {
                        spans.push(Span::styled(
                            std::mem::take(&mut current_text),
                            current_style,
                        ));
                    }
                    current_style = parse_sgr_params(&params, current_style);
                }
                // Ignore all other (or malformed) control sequences
            }
            Some(_) => {
                // Control string (OSC, DCS, SOS, PM, APC): skip everything up
                // to the string terminator (BEL, ESC \\ or C1 ST). Another
                // ESC ends the string without being consumed.
                while let Some(&next) = chars.peek() {
                    match next {
                        '\x07' | '\u{9c}' => {
                            chars.next();
                            break;
                        }
                        '\x1b' => {
                            chars.next();
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                        _ => {
                            chars.next();
                        }
                    }
                }
            }
            None if c == '\x1b' => {
                // Other escape sequences: ESC, intermediates 0x20-0x2F, then
                // a final byte 0x30-0x7E (e.g. `ESC ( B`, `ESC 7`, `ESC c`).
                // A lone ESC is simply dropped.
                while let Some(&next) = chars.peek() {
                    if ('\x20'..='\x2f').contains(&next) {
                        chars.next();
                    } else {
                        break;
                    }
                }
                if chars
                    .peek()
                    .is_some_and(|next| ('\x30'..='\x7e').contains(next))
                {
                    chars.next();
                }
            }
            None => {
                // Drop other control characters (they could move the cursor
                // or otherwise corrupt the display), but keep tabs/newlines.
                if !c.is_control() || c == '\t' || c == '\n' {
                    current_text.push(c);
                }
            }
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
///
/// Parameters are parsed positionally: an empty parameter means 0, and an
/// invalid/out-of-range one becomes `None` so it never shifts the meaning
/// of the parameters after it.
fn parse_sgr_params(params: &str, mut style: Style) -> Style {
    if params.is_empty() {
        return Style::default();
    }

    let codes: Vec<Option<u16>> = params
        .split(';')
        .map(|s| {
            if s.is_empty() {
                Some(0)
            } else {
                s.parse::<u16>().ok()
            }
        })
        .collect();

    // Fetch a parameter as a u8 colour component / index
    let byte_at = |idx: usize| -> Option<u8> {
        codes
            .get(idx)
            .copied()
            .flatten()
            .and_then(|v| u8::try_from(v).ok())
    };

    let mut i = 0;
    while i < codes.len() {
        let Some(code) = codes[i] else {
            // Invalid parameter: skip it
            i += 1;
            continue;
        };
        match code {
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
                if let Some((color, consumed)) = parse_extended_color(&codes, i, byte_at) {
                    if let Some(color) = color {
                        style = style.fg(color);
                    }
                    i += consumed;
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
                if let Some((color, consumed)) = parse_extended_color(&codes, i, byte_at) {
                    if let Some(color) = color {
                        style = style.bg(color);
                    }
                    i += consumed;
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

/// Parse an extended colour (`38;5;n` / `38;2;r;g;b`, same for 48) starting
/// at `codes[i]`.
///
/// Returns the colour (`None` if a component is invalid) and the number of
/// extra parameters consumed, or `None` if the sub-mode is missing/unknown.
fn parse_extended_color(
    codes: &[Option<u16>],
    i: usize,
    byte_at: impl Fn(usize) -> Option<u8>,
) -> Option<(Option<Color>, usize)> {
    match codes.get(i + 1).copied().flatten() {
        // 256 color mode: 38;5;n
        Some(5) if i + 2 < codes.len() => Some((byte_at(i + 2).map(Color::Indexed), 2)),
        // RGB color mode: 38;2;r;g;b
        Some(2) if i + 4 < codes.len() => {
            let rgb = byte_at(i + 2)
                .zip(byte_at(i + 3))
                .zip(byte_at(i + 4))
                .map(|((r, g), b)| Color::Rgb(r, g, b));
            Some((rgb, 4))
        }
        _ => None,
    }
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
    use termimad::{MadSkin, crossterm::style::Color as TermColor};

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
    markdown
        .lines()
        .map(|l| Line::from(l.to_string()))
        .collect()
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

    fn plain(text: &str) -> String {
        parse_ansi_to_spans(text)
            .iter()
            .map(|s| s.content.to_string())
            .collect()
    }

    #[test]
    fn test_private_csi_is_stripped() {
        assert_eq!(plain("\x1b[?25lhi"), "hi");
        assert_eq!(plain("\x1b[>4;2mhi"), "hi");
    }

    #[test]
    fn test_csi_with_intermediates_is_stripped() {
        // DECSCUSR: CSI 2 SP q
        assert_eq!(plain("a\x1b[2 qb"), "ab");
    }

    #[test]
    fn test_osc_is_stripped() {
        assert_eq!(plain("\x1b]8;;http://x\x1b\\link\x1b]8;;\x1b\\"), "link");
        assert_eq!(plain("\x1b]0;title\x07text"), "text");
        assert_eq!(plain("\x1b]52;c;ZXZpbA==\x07ok"), "ok");
    }

    #[test]
    fn test_truncated_csi_does_not_eat_text() {
        // `\x1b[31` followed by a non-final byte: the next char survives
        assert_eq!(plain("\x1b[31中文"), "中文");
        assert_eq!(plain("\x1b[31\nX"), "\nX");
        assert_eq!(plain("abc\x1b[31"), "abc");
        // Note: in "\x1b[31Hello" the 'H' is a valid final byte (CUP), so
        // per ECMA-48 the sequence is complete and 'H' is consumed.
        assert_eq!(plain("\x1b[31Hello"), "ello");
    }

    #[test]
    fn test_lone_esc_and_controls_dropped() {
        assert_eq!(plain("a\x1bb"), "a");
        assert_eq!(plain("a\x1b"), "a");
        assert_eq!(plain("a\x1b(Bb"), "ab");
        assert_eq!(plain("a\x07b\rc\x08d"), "abcd");
        assert_eq!(plain("a\u{9b}2Jb"), "ab");
    }

    #[test]
    fn test_out_of_range_rgb_does_not_shift_params() {
        let spans = parse_ansi_to_spans("\x1b[38;2;300;10;20;1mX");
        assert!(spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(spans[0].style.fg, None);
    }

    #[test]
    fn test_invalid_param_does_not_shift() {
        let spans = parse_ansi_to_spans("\x1b[999;31mX");
        assert_eq!(spans[0].style.fg, Some(Color::Red));
        let spans = parse_ansi_to_spans("\x1b[;1mX");
        assert!(spans[0].style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_empty_string() {
        let spans = parse_ansi_to_spans("");
        assert_eq!(spans.len(), 1);
    }
}
