//! File explorer widget
//!
//! A file browser with directory navigation, file type icons, and multi-selection.
//!
//! # Example
//!
//! ```rust,ignore
//! use ratatui_interact::components::{FileExplorer, FileExplorerState, FileEntry, EntryType};
//! use std::path::PathBuf;
//!
//! // Create state
//! let mut state = FileExplorerState::new(PathBuf::from("/home/user"));
//!
//! // Load entries (typically done in your app)
//! state.load_entries().unwrap();
//!
//! // Create explorer
//! let explorer = FileExplorer::new(&state)
//!     .title_format(|path| format!("Browse: {}", path.display()));
//! ```

use std::collections::HashSet;
use std::path::PathBuf;

use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::utils::display::format_size;

/// Type of file system entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    /// Regular file with extension and size
    File {
        extension: Option<String>,
        size: u64,
    },
    /// Directory
    Directory,
    /// Parent directory (..)
    ParentDir,
    /// Symbolic link with target
    Symlink { target: Option<PathBuf> },
}

/// A file system entry
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Display name
    pub name: String,
    /// Full path
    pub path: PathBuf,
    /// Entry type
    pub entry_type: EntryType,
}

impl FileEntry {
    /// Create a new file entry
    pub fn new(name: impl Into<String>, path: PathBuf, entry_type: EntryType) -> Self {
        Self {
            name: name.into(),
            path,
            entry_type,
        }
    }

    /// Create a parent directory entry
    pub fn parent_dir(parent_path: PathBuf) -> Self {
        Self {
            name: "..".into(),
            path: parent_path,
            entry_type: EntryType::ParentDir,
        }
    }

    /// Check if this is a directory (including parent dir)
    pub fn is_dir(&self) -> bool {
        matches!(self.entry_type, EntryType::Directory | EntryType::ParentDir)
    }

    /// Check if this is selectable (files only, not directories)
    pub fn is_selectable(&self) -> bool {
        matches!(self.entry_type, EntryType::File { .. })
    }
}

/// Mode for the file explorer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileExplorerMode {
    /// Normal browsing mode
    #[default]
    Browse,
    /// Search/filter mode
    Search,
}

/// State for the file explorer widget
#[derive(Debug, Clone)]
pub struct FileExplorerState {
    /// Current directory
    pub current_dir: PathBuf,
    /// List of entries in current directory
    pub entries: Vec<FileEntry>,
    /// Current cursor position
    pub cursor_index: usize,
    /// Scroll offset
    pub scroll: u16,
    /// Selected files (for multi-select)
    pub selected_files: HashSet<PathBuf>,
    /// Whether to show hidden files
    pub show_hidden: bool,
    /// Current mode
    pub mode: FileExplorerMode,
    /// Search/filter query
    pub search_query: String,
    /// Filtered entry indices (None = show all)
    pub filtered_indices: Option<Vec<usize>>,
}

impl FileExplorerState {
    /// Create a new file explorer state
    pub fn new(start_dir: PathBuf) -> Self {
        Self {
            current_dir: start_dir,
            entries: Vec::new(),
            cursor_index: 0,
            scroll: 0,
            selected_files: HashSet::new(),
            show_hidden: false,
            mode: FileExplorerMode::Browse,
            search_query: String::new(),
            filtered_indices: None,
        }
    }

    /// Load entries from the current directory
    #[cfg(feature = "filesystem")]
    pub fn load_entries(&mut self) -> std::io::Result<()> {
        self.entries.clear();
        self.cursor_index = 0;
        self.scroll = 0;
        self.filtered_indices = None;

        // Add parent directory if not at root
        if let Some(parent) = self.current_dir.parent() {
            self.entries
                .push(FileEntry::parent_dir(parent.to_path_buf()));
        }

        // Read directory entries
        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in std::fs::read_dir(&self.current_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files if not showing them
            if !self.show_hidden && name.starts_with('.') {
                continue;
            }

            let metadata = entry.metadata()?;
            let entry_type = if metadata.is_dir() {
                EntryType::Directory
            } else if metadata.is_symlink() {
                EntryType::Symlink {
                    target: std::fs::read_link(&path).ok(),
                }
            } else {
                EntryType::File {
                    extension: path.extension().map(|e| e.to_string_lossy().to_string()),
                    size: metadata.len(),
                }
            };

            let file_entry = FileEntry::new(name, path, entry_type);
            if file_entry.is_dir() {
                dirs.push(file_entry);
            } else {
                files.push(file_entry);
            }
        }

        // Sort: directories first (alphabetically), then files (alphabetically)
        dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        self.entries.extend(dirs);
        self.entries.extend(files);

        Ok(())
    }

    /// Navigate into a directory
    pub fn enter_directory(&mut self, path: PathBuf) {
        self.current_dir = path;
        #[cfg(feature = "filesystem")]
        let _ = self.load_entries();
    }

    /// Navigate up to parent directory
    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            #[cfg(feature = "filesystem")]
            let _ = self.load_entries();
        }
    }

    /// Move cursor up
    pub fn cursor_up(&mut self) {
        let count = self.visible_count();
        if count > 0 && self.cursor_index > 0 {
            self.cursor_index -= 1;
        }
    }

    /// Move cursor down
    pub fn cursor_down(&mut self) {
        let count = self.visible_count();
        if count > 0 && self.cursor_index + 1 < count {
            self.cursor_index += 1;
        }
    }

    /// Get the number of visible entries
    pub fn visible_count(&self) -> usize {
        self.filtered_indices
            .as_ref()
            .map(|i| i.len())
            .unwrap_or(self.entries.len())
    }

    /// Get the currently selected entry
    pub fn current_entry(&self) -> Option<&FileEntry> {
        if let Some(ref indices) = self.filtered_indices {
            indices
                .get(self.cursor_index)
                .and_then(|&i| self.entries.get(i))
        } else {
            self.entries.get(self.cursor_index)
        }
    }

    /// Toggle selection of current file
    pub fn toggle_selection(&mut self) {
        if let Some(entry) = self.current_entry()
            && entry.is_selectable()
        {
            let path = entry.path.clone();
            if self.selected_files.contains(&path) {
                self.selected_files.remove(&path);
            } else {
                self.selected_files.insert(path);
            }
        }
    }

    /// Select all files
    pub fn select_all(&mut self) {
        for entry in &self.entries {
            if entry.is_selectable() {
                self.selected_files.insert(entry.path.clone());
            }
        }
    }

    /// Clear all selections
    pub fn select_none(&mut self) {
        self.selected_files.clear();
    }

    /// Toggle hidden files visibility
    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        #[cfg(feature = "filesystem")]
        let _ = self.load_entries();
    }

    /// Enter search mode
    pub fn start_search(&mut self) {
        self.mode = FileExplorerMode::Search;
        self.search_query.clear();
    }

    /// Exit search mode
    pub fn cancel_search(&mut self) {
        self.mode = FileExplorerMode::Browse;
        self.search_query.clear();
        self.filtered_indices = None;
    }

    /// Update search filter
    pub fn update_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = None;
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_indices = Some(
                self.entries
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.name.to_lowercase().contains(&query))
                    .map(|(i, _)| i)
                    .collect(),
            );
            self.cursor_index = 0;
        }
    }

    /// Ensure cursor is visible
    pub fn ensure_visible(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }

        if self.cursor_index < self.scroll as usize {
            self.scroll = self.cursor_index as u16;
        } else if self.cursor_index >= self.scroll as usize + viewport_height {
            self.scroll = (self.cursor_index - viewport_height + 1) as u16;
        }
    }
}

/// Style configuration for file explorer
#[derive(Debug, Clone)]
pub struct FileExplorerStyle {
    /// Border style
    pub border_style: Style,
    /// Style for selected (cursor) item
    pub cursor_style: Style,
    /// Style for directory names
    pub dir_style: Style,
    /// Style for file names (by extension)
    pub file_colors: Vec<(Vec<&'static str>, Color)>,
    /// Default file color
    pub default_file_color: Color,
    /// Style for file sizes
    pub size_style: Style,
    /// Checkbox checked
    pub checkbox_checked: &'static str,
    /// Checkbox unchecked
    pub checkbox_unchecked: &'static str,
    /// Directory icon
    pub dir_icon: &'static str,
    /// Parent directory icon
    pub parent_icon: &'static str,
    /// Symlink icon
    pub symlink_icon: &'static str,
}

impl Default for FileExplorerStyle {
    fn default() -> Self {
        Self {
            border_style: Style::default().fg(Color::Cyan),
            cursor_style: Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            dir_style: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            file_colors: vec![
                (vec!["rs"], Color::Yellow),
                (vec!["toml", "json", "yaml", "yml"], Color::Green),
                (vec!["md", "txt", "rst"], Color::White),
                (vec!["py"], Color::Cyan),
                (vec!["js", "ts", "tsx", "jsx"], Color::Magenta),
                (vec!["sh", "bash", "zsh"], Color::Red),
            ],
            default_file_color: Color::Gray,
            size_style: Style::default().fg(Color::DarkGray),
            checkbox_checked: "[x]",
            checkbox_unchecked: "[ ]",
            dir_icon: "[DIR]",
            parent_icon: " .. ",
            symlink_icon: "[LNK]",
        }
    }
}

impl FileExplorerStyle {
    /// Get color for a file extension
    pub fn color_for_extension(&self, ext: Option<&str>) -> Color {
        if let Some(ext) = ext {
            for (extensions, color) in &self.file_colors {
                if extensions.contains(&ext) {
                    return *color;
                }
            }
        }
        self.default_file_color
    }
}

/// File explorer widget
pub struct FileExplorer<'a> {
    state: &'a FileExplorerState,
    style: FileExplorerStyle,
}

impl<'a> FileExplorer<'a> {
    /// Create a new file explorer widget
    pub fn new(state: &'a FileExplorerState) -> Self {
        Self {
            state,
            style: FileExplorerStyle::default(),
        }
    }

    /// Set the style
    pub fn style(mut self, style: FileExplorerStyle) -> Self {
        self.style = style;
        self
    }

    /// Build file list lines
    fn build_lines(&self, inner: Rect) -> Vec<Line<'static>> {
        let visible_height = inner.height as usize;
        let scroll = self.state.scroll as usize;

        let entries_to_show: Vec<(usize, &FileEntry)> =
            if let Some(ref indices) = self.state.filtered_indices {
                indices
                    .iter()
                    .map(|&i| (i, &self.state.entries[i]))
                    .collect()
            } else {
                self.state.entries.iter().enumerate().collect()
            };

        let mut lines = Vec::new();

        for (display_idx, (_entry_idx, entry)) in entries_to_show
            .iter()
            .enumerate()
            .skip(scroll)
            .take(visible_height)
        {
            let is_cursor = display_idx == self.state.cursor_index;
            let is_checked = self.state.selected_files.contains(&entry.path);

            let style = if is_cursor {
                self.style.cursor_style
            } else {
                Style::default()
            };

            let cursor = if is_cursor { ">" } else { " " };
            let checkbox = match &entry.entry_type {
                EntryType::File { .. } => {
                    if is_checked {
                        self.style.checkbox_checked
                    } else {
                        self.style.checkbox_unchecked
                    }
                }
                _ => "   ",
            };

            let (icon, name_style) = match &entry.entry_type {
                EntryType::Directory => (
                    self.style.dir_icon,
                    if is_cursor {
                        self.style.cursor_style
                    } else {
                        self.style.dir_style
                    },
                ),
                EntryType::ParentDir => (
                    self.style.parent_icon,
                    if is_cursor {
                        self.style.cursor_style
                    } else {
                        self.style.dir_style
                    },
                ),
                EntryType::File { extension, .. } => {
                    let color = self.style.color_for_extension(extension.as_deref());
                    (
                        "     ",
                        if is_cursor {
                            self.style.cursor_style
                        } else {
                            Style::default().fg(color)
                        },
                    )
                }
                EntryType::Symlink { .. } => (
                    self.style.symlink_icon,
                    if is_cursor {
                        self.style.cursor_style
                    } else {
                        Style::default().fg(Color::Magenta)
                    },
                ),
            };

            let size_str = match &entry.entry_type {
                EntryType::File { size, .. } => format_size(*size),
                _ => String::new(),
            };

            // Calculate name width
            let name_width = inner.width.saturating_sub(22) as usize;
            let display_name = if entry.name.len() > name_width {
                format!("{}...", &entry.name[..name_width.saturating_sub(3)])
            } else {
                entry.name.clone()
            };

            lines.push(Line::from(vec![
                Span::styled(cursor.to_string(), style),
                Span::styled(" ", style),
                Span::styled(checkbox.to_string(), style),
                Span::styled(" ", style),
                Span::styled(icon.to_string(), style),
                Span::styled(" ", style),
                Span::styled(
                    format!("{:<width$}", display_name, width = name_width),
                    name_style,
                ),
                Span::styled(
                    format!("{:>10}", size_str),
                    if is_cursor {
                        self.style.cursor_style
                    } else {
                        self.style.size_style
                    },
                ),
            ]));
        }

        lines
    }
}

impl Widget for FileExplorer<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // File list
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Title with path and selection count
        let selected_count = self.state.selected_files.len();
        let title = if selected_count > 0 {
            format!(
                " {} ({} selected) ",
                self.state.current_dir.display(),
                selected_count
            )
        } else {
            format!(" {} ", self.state.current_dir.display())
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.style.border_style)
            .title(title);

        let inner = block.inner(chunks[0]);
        block.render(chunks[0], buf);

        // File list
        let lines = self.build_lines(inner);
        let paragraph = Paragraph::new(lines);
        paragraph.render(inner, buf);

        // Footer
        let footer = build_footer(self.state.mode);
        let footer_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray));
        let footer_para = Paragraph::new(footer)
            .block(footer_block)
            .alignment(Alignment::Center);
        footer_para.render(chunks[1], buf);
    }
}

/// Build footer lines based on current mode
fn build_footer(mode: FileExplorerMode) -> Vec<Line<'static>> {
    match mode {
        FileExplorerMode::Browse => vec![
            Line::from(vec![
                Span::styled("↑↓", Style::default().fg(Color::Green)),
                Span::raw(":Move "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(":Open "),
                Span::styled("Space", Style::default().fg(Color::Green)),
                Span::raw(":Select "),
                Span::styled("/", Style::default().fg(Color::Green)),
                Span::raw(":Search "),
                Span::styled(".", Style::default().fg(Color::Green)),
                Span::raw(":Hidden"),
            ]),
            Line::from(vec![
                Span::styled("a", Style::default().fg(Color::Green)),
                Span::raw(":All "),
                Span::styled("n", Style::default().fg(Color::Green)),
                Span::raw(":None "),
                Span::styled("Esc", Style::default().fg(Color::Green)),
                Span::raw(":Close"),
            ]),
        ],
        FileExplorerMode::Search => vec![Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(":Confirm "),
            Span::styled("Esc", Style::default().fg(Color::Green)),
            Span::raw(":Cancel"),
        ])],
    }
}

/// Draw a search bar overlay
pub fn draw_search_bar(f: &mut Frame, query: &str, area: Rect) {
    let search_text = Line::from(vec![
        Span::styled(
            "Search: ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(query.to_string(), Style::default().fg(Color::White)),
        Span::styled(
            "_",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(vec![search_text]).block(block);

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_entry() {
        let entry = FileEntry::new(
            "test.rs",
            PathBuf::from("/home/user/test.rs"),
            EntryType::File {
                extension: Some("rs".into()),
                size: 1024,
            },
        );
        assert!(!entry.is_dir());
        assert!(entry.is_selectable());

        let dir = FileEntry::new("src", PathBuf::from("/home/user/src"), EntryType::Directory);
        assert!(dir.is_dir());
        assert!(!dir.is_selectable());
    }

    #[test]
    fn test_state_navigation() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        state.entries = vec![
            FileEntry::parent_dir(PathBuf::from("/")),
            FileEntry::new(
                "file1.txt",
                PathBuf::from("/tmp/file1.txt"),
                EntryType::File {
                    extension: Some("txt".into()),
                    size: 100,
                },
            ),
            FileEntry::new(
                "file2.txt",
                PathBuf::from("/tmp/file2.txt"),
                EntryType::File {
                    extension: Some("txt".into()),
                    size: 200,
                },
            ),
        ];

        assert_eq!(state.cursor_index, 0);
        state.cursor_down();
        assert_eq!(state.cursor_index, 1);
        state.cursor_down();
        assert_eq!(state.cursor_index, 2);
        state.cursor_down(); // Should not go past end
        assert_eq!(state.cursor_index, 2);
        state.cursor_up();
        assert_eq!(state.cursor_index, 1);
    }

    #[test]
    fn test_selection() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        state.entries = vec![FileEntry::new(
            "file.txt",
            PathBuf::from("/tmp/file.txt"),
            EntryType::File {
                extension: Some("txt".into()),
                size: 100,
            },
        )];

        assert!(state.selected_files.is_empty());
        state.toggle_selection();
        assert_eq!(state.selected_files.len(), 1);
        state.toggle_selection();
        assert!(state.selected_files.is_empty());
    }

    #[test]
    fn test_style_color_for_extension() {
        let style = FileExplorerStyle::default();
        assert_eq!(style.color_for_extension(Some("rs")), Color::Yellow);
        assert_eq!(style.color_for_extension(Some("json")), Color::Green);
        assert_eq!(style.color_for_extension(Some("unknown")), Color::Gray);
        assert_eq!(style.color_for_extension(None), Color::Gray);
    }
}
