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

use std::collections::{HashMap, HashSet};
#[cfg(feature = "filesystem")]
use std::path::Path;
use std::path::{Component, PathBuf};

use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::utils::display::{format_size, pad_to_width, truncate_to_width};

/// Type of file system entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    /// Regular file with extension and size
    File {
        /// File extension (without the leading dot), if any
        extension: Option<String>,
        /// File size in bytes
        size: u64,
    },
    /// Directory
    Directory,
    /// Parent directory (..)
    ParentDir,
    /// Symbolic link with target
    Symlink {
        /// Link target, if it could be read
        target: Option<PathBuf>,
    },
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
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
    ///
    /// Saturates at `u16::MAX`; rendering keeps the cursor visible even in
    /// directories with more entries than that.
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
    ///
    /// Relative paths and paths containing `.` or `..` components are made
    /// absolute (and canonicalized when the `filesystem` feature is enabled),
    /// so that the `..` entry and [`go_up`](Self::go_up) work as expected.
    pub fn new(start_dir: PathBuf) -> Self {
        Self {
            current_dir: normalize_dir(start_dir),
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
    ///
    /// On error the previous entries are left untouched. Entries whose
    /// metadata cannot be read are skipped rather than failing the listing.
    #[cfg(feature = "filesystem")]
    pub fn load_entries(&mut self) -> std::io::Result<()> {
        let entries = read_dir_entries(&self.current_dir, self.show_hidden)?;
        self.commit_entries(entries);
        Ok(())
    }

    /// Replace the entry list and reset cursor, scroll and filter.
    #[cfg(feature = "filesystem")]
    fn commit_entries(&mut self, entries: Vec<FileEntry>) {
        self.entries = entries;
        self.cursor_index = 0;
        self.scroll = 0;
        self.filtered_indices = None;
    }

    /// Navigate into a directory
    ///
    /// Errors are ignored and leave the state unchanged; use
    /// [`try_enter_directory`](Self::try_enter_directory) to observe them.
    pub fn enter_directory(&mut self, path: PathBuf) {
        let _ = self.try_enter_directory(path);
    }

    /// Navigate into a directory, reporting I/O errors
    ///
    /// With the `filesystem` feature the new directory is read first; if that
    /// fails, the error is returned and the state is left unchanged.
    pub fn try_enter_directory(&mut self, path: PathBuf) -> std::io::Result<()> {
        let path = normalize_dir(path);
        #[cfg(feature = "filesystem")]
        {
            let entries = read_dir_entries(&path, self.show_hidden)?;
            self.current_dir = path;
            self.commit_entries(entries);
        }
        #[cfg(not(feature = "filesystem"))]
        {
            self.current_dir = path;
        }
        Ok(())
    }

    /// Navigate up to parent directory
    ///
    /// Errors are ignored and leave the state unchanged; use
    /// [`try_go_up`](Self::try_go_up) to observe them.
    pub fn go_up(&mut self) {
        let _ = self.try_go_up();
    }

    /// Navigate up to parent directory, reporting I/O errors
    ///
    /// Does nothing at the filesystem root. On error the state is unchanged.
    pub fn try_go_up(&mut self) -> std::io::Result<()> {
        match self.current_dir.parent() {
            Some(parent) => self.try_enter_directory(parent.to_path_buf()),
            None => Ok(()),
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
    ///
    /// Errors are ignored and leave the state unchanged; use
    /// [`try_toggle_hidden`](Self::try_toggle_hidden) to observe them.
    pub fn toggle_hidden(&mut self) {
        let _ = self.try_toggle_hidden();
    }

    /// Toggle hidden files visibility, reporting I/O errors
    ///
    /// With the `filesystem` feature the directory is re-read; if that fails,
    /// the error is returned and the state is left unchanged.
    pub fn try_toggle_hidden(&mut self) -> std::io::Result<()> {
        #[cfg(feature = "filesystem")]
        {
            let entries = read_dir_entries(&self.current_dir, !self.show_hidden)?;
            self.show_hidden = !self.show_hidden;
            self.commit_entries(entries);
        }
        #[cfg(not(feature = "filesystem"))]
        {
            self.show_hidden = !self.show_hidden;
        }
        Ok(())
    }

    /// Enter search mode
    pub fn start_search(&mut self) {
        self.mode = FileExplorerMode::Search;
        self.search_query.clear();
    }

    /// Exit search mode
    ///
    /// The cursor stays on the entry it pointed at in the filtered list.
    pub fn cancel_search(&mut self) {
        self.mode = FileExplorerMode::Browse;
        self.search_query.clear();
        self.clear_filter();
    }

    /// Drop the filter, mapping the cursor back to an index into `entries`.
    fn clear_filter(&mut self) {
        if let Some(indices) = self.filtered_indices.take() {
            self.cursor_index = indices.get(self.cursor_index).copied().unwrap_or(0);
            self.scroll = self
                .scroll
                .min(self.cursor_index.min(u16::MAX as usize) as u16);
        }
    }

    /// Update search filter
    ///
    /// A non-empty query resets the cursor and scroll to the first match.
    pub fn update_filter(&mut self) {
        if self.search_query.is_empty() {
            self.clear_filter();
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
            self.scroll = 0;
        }
    }

    /// Ensure cursor is visible
    pub fn ensure_visible(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }

        let scroll = visible_scroll(self.scroll as usize, self.cursor_index, viewport_height);
        self.scroll = scroll.min(u16::MAX as usize) as u16;
    }
}

/// Scroll offset that keeps `cursor` within a viewport of `height` rows.
fn visible_scroll(scroll: usize, cursor: usize, height: usize) -> usize {
    if height == 0 {
        scroll
    } else if cursor < scroll {
        cursor
    } else if cursor >= scroll + height {
        cursor + 1 - height
    } else {
        scroll
    }
}

/// Make `dir` absolute and free of `.`/`..` components when needed.
///
/// Clean absolute paths are returned unchanged (symlinks are not resolved).
fn normalize_dir(dir: PathBuf) -> PathBuf {
    let clean = dir.is_absolute()
        && dir
            .components()
            .all(|c| !matches!(c, Component::CurDir | Component::ParentDir));
    if clean {
        return dir;
    }
    #[cfg(feature = "filesystem")]
    if let Ok(canonical) = std::fs::canonicalize(&dir) {
        return canonical;
    }
    std::path::absolute(&dir).unwrap_or(dir)
}

/// Read and sort the entries of `dir` (directories first, then files).
///
/// Entries whose metadata cannot be read are skipped.
#[cfg(feature = "filesystem")]
fn read_dir_entries(dir: &Path, show_hidden: bool) -> std::io::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();

    // Add parent directory if not at root
    if let Some(parent) = dir.parent() {
        entries.push(FileEntry::parent_dir(parent.to_path_buf()));
    }

    // Read directory entries
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        // Skip entries that cannot be read instead of aborting the listing
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files if not showing them
        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let Ok(metadata) = entry.metadata() else {
            continue;
        };
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
    dirs.sort_by_cached_key(|e| e.name.to_lowercase());
    files.sort_by_cached_key(|e| e.name.to_lowercase());

    entries.extend(dirs);
    entries.extend(files);

    Ok(entries)
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

impl From<&crate::theme::Theme> for FileExplorerStyle {
    fn from(theme: &crate::theme::Theme) -> Self {
        let p = &theme.palette;
        Self {
            border_style: Style::default().fg(p.border_accent),
            cursor_style: Style::default()
                .fg(p.highlight_fg)
                .bg(p.secondary)
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
    footer_builder: FooterBuilder,
}

impl<'a> FileExplorer<'a> {
    /// Create a new file explorer widget
    pub fn new(state: &'a FileExplorerState) -> Self {
        Self {
            state,
            style: FileExplorerStyle::default(),
            footer_builder: FooterBuilder::default(),
        }
    }

    /// Set the FooterBuilder for this FileExplorer to use during rendering
    pub fn footer_builder(mut self, footer_builder: FooterBuilder) -> Self {
        self.footer_builder = footer_builder;
        self
    }

    /// Set the style
    pub fn style(mut self, style: FileExplorerStyle) -> Self {
        self.style = style;
        self
    }

    /// Apply a theme to derive the style
    pub fn theme(self, theme: &crate::theme::Theme) -> Self {
        self.style(FileExplorerStyle::from(theme))
    }

    /// Build file list lines
    fn build_lines(&self, inner: Rect) -> Vec<Line<'static>> {
        let visible_height = inner.height as usize;
        // Keep the cursor visible even if `scroll` is stale or saturated
        let scroll = visible_scroll(
            self.state.scroll as usize,
            self.state.cursor_index,
            visible_height,
        );

        let entries_to_show: Vec<(usize, &FileEntry)> =
            if let Some(ref indices) = self.state.filtered_indices {
                indices
                    .iter()
                    .filter_map(|&i| self.state.entries.get(i).map(|e| (i, e)))
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
            // Width-aware truncation (also strips control characters)
            let display_name = truncate_to_width(&entry.name, name_width);

            lines.push(Line::from(vec![
                Span::styled(cursor.to_string(), style),
                Span::styled(" ", style),
                Span::styled(checkbox.to_string(), style),
                Span::styled(" ", style),
                Span::styled(icon.to_string(), style),
                Span::styled(" ", style),
                Span::styled(pad_to_width(&display_name, name_width), name_style),
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
        let footer = self.footer_builder.build(self.state.mode);
        let footer_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray));
        let footer_para = Paragraph::new(footer)
            .block(footer_block)
            .alignment(Alignment::Center);
        footer_para.render(chunks[1], buf);
    }
}

/// Builder for the key-binding hints shown in the file explorer footer
#[derive(Clone)]
pub struct FooterBuilder {
    keybinds: HashMap<FileExplorerMode, Vec<(String, String)>>,
    /// Maximum number of key bindings per footer line
    pub binds_per_line: usize,
}

impl FooterBuilder {
    /// Construct a FooterBuilder with no preset bindings
    pub fn new() -> Self {
        Self {
            binds_per_line: 5,
            keybinds: HashMap::from([
                (FileExplorerMode::Browse, Vec::new()),
                (FileExplorerMode::Search, Vec::new()),
            ]),
        }
    }

    /// Build and return the footer
    pub fn build(&self, mode: FileExplorerMode) -> Vec<Line<'static>> {
        let spans_per_line = 2 * self.binds_per_line;
        let mut lines = Vec::new();
        let mut spans = Vec::with_capacity(spans_per_line);

        if let Some(binds) = self.keybinds.get(&mode) {
            for (key, action) in binds {
                if spans.len() == spans_per_line {
                    lines.push(Line::from(spans));
                    spans = Vec::with_capacity(self.binds_per_line);
                }
                spans.push(Span::styled(key.clone(), Style::default().fg(Color::Green)));
                spans.push(Span::raw(format!("{action} ")));
            }
        }

        lines.push(Line::from(spans));
        lines
    }

    /// Add a keybind to the footer
    pub fn with_keybind(mut self, mode: FileExplorerMode, key: String, action: String) -> Self {
        self.keybinds.entry(mode).or_default().push((key, action));
        self
    }
}

impl Default for FooterBuilder {
    fn default() -> Self {
        let browse_binds = vec![
            ("↑↓".to_string(), ":Move".to_string()),
            ("Enter".to_string(), ":Open".to_string()),
            ("Space".to_string(), ":Select".to_string()),
            ("/".to_string(), ":Search".to_string()),
            (".".to_string(), ":Hidden".to_string()),
            ("a".to_string(), ":All".to_string()),
            ("n".to_string(), ":None".to_string()),
            ("Esc".to_string(), ":Close".to_string()),
        ];

        let search_binds = vec![
            ("Enter".to_string(), ":Confirm".to_string()),
            ("Esc".to_string(), ":Cancel".to_string()),
        ];

        Self {
            binds_per_line: 5,
            keybinds: HashMap::from([
                (FileExplorerMode::Browse, browse_binds),
                (FileExplorerMode::Search, search_binds),
            ]),
        }
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
    fn test_file_entry_parent_dir() {
        let entry = FileEntry::parent_dir(PathBuf::from("/home"));
        assert_eq!(entry.name, "..");
        assert!(entry.is_dir());
        assert!(!entry.is_selectable());
        assert_eq!(entry.entry_type, EntryType::ParentDir);
    }

    #[test]
    fn test_file_entry_symlink() {
        let entry = FileEntry::new(
            "link",
            PathBuf::from("/home/user/link"),
            EntryType::Symlink {
                target: Some(PathBuf::from("/target")),
            },
        );
        assert!(!entry.is_dir());
        assert!(!entry.is_selectable()); // Symlinks are not selectable
    }

    #[test]
    fn test_state_new() {
        // An absolute path that exists on every OS (`/tmp` is relative on Windows).
        let dir = std::env::temp_dir();
        let state = FileExplorerState::new(dir.clone());
        assert_eq!(state.current_dir, dir);
        assert!(state.entries.is_empty());
        assert_eq!(state.cursor_index, 0);
        assert!(!state.show_hidden);
        assert_eq!(state.mode, FileExplorerMode::Browse);
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
    fn test_cursor_up_at_top() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        state.entries = vec![FileEntry::new(
            "file.txt",
            PathBuf::from("/tmp/file.txt"),
            EntryType::File {
                extension: Some("txt".into()),
                size: 100,
            },
        )];
        state.cursor_index = 0;
        state.cursor_up();
        assert_eq!(state.cursor_index, 0); // Should not go negative
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
    fn test_select_all() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        state.entries = vec![
            FileEntry::new("dir", PathBuf::from("/tmp/dir"), EntryType::Directory),
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

        state.select_all();
        // Only files should be selected, not directories
        assert_eq!(state.selected_files.len(), 2);
    }

    #[test]
    fn test_select_none() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        state.entries = vec![FileEntry::new(
            "file.txt",
            PathBuf::from("/tmp/file.txt"),
            EntryType::File {
                extension: Some("txt".into()),
                size: 100,
            },
        )];

        state.toggle_selection();
        assert_eq!(state.selected_files.len(), 1);
        state.select_none();
        assert!(state.selected_files.is_empty());
    }

    #[test]
    fn test_toggle_hidden() {
        let mut state = FileExplorerState::new(std::env::temp_dir());
        assert!(!state.show_hidden);
        state.toggle_hidden();
        assert!(state.show_hidden);
        state.toggle_hidden();
        assert!(!state.show_hidden);
    }

    #[test]
    fn test_search_mode() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        assert_eq!(state.mode, FileExplorerMode::Browse);

        state.start_search();
        assert_eq!(state.mode, FileExplorerMode::Search);
        assert!(state.search_query.is_empty());

        state.cancel_search();
        assert_eq!(state.mode, FileExplorerMode::Browse);
        assert!(state.filtered_indices.is_none());
    }

    #[test]
    fn test_update_filter() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        state.entries = vec![
            FileEntry::new(
                "test.rs",
                PathBuf::from("/tmp/test.rs"),
                EntryType::File {
                    extension: Some("rs".into()),
                    size: 100,
                },
            ),
            FileEntry::new(
                "main.rs",
                PathBuf::from("/tmp/main.rs"),
                EntryType::File {
                    extension: Some("rs".into()),
                    size: 200,
                },
            ),
            FileEntry::new(
                "other.txt",
                PathBuf::from("/tmp/other.txt"),
                EntryType::File {
                    extension: Some("txt".into()),
                    size: 300,
                },
            ),
        ];

        state.search_query = "test".into();
        state.update_filter();

        assert!(state.filtered_indices.is_some());
        assert_eq!(state.filtered_indices.as_ref().unwrap().len(), 1);
        assert_eq!(state.visible_count(), 1);
    }

    #[test]
    fn test_update_filter_empty_clears() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        state.entries = vec![FileEntry::new(
            "file.txt",
            PathBuf::from("/tmp/file.txt"),
            EntryType::File {
                extension: Some("txt".into()),
                size: 100,
            },
        )];

        state.search_query = "file".into();
        state.update_filter();
        assert!(state.filtered_indices.is_some());

        state.search_query = "".into();
        state.update_filter();
        assert!(state.filtered_indices.is_none());
    }

    #[test]
    fn test_current_entry() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        state.entries = vec![
            FileEntry::new(
                "first.txt",
                PathBuf::from("/tmp/first.txt"),
                EntryType::File {
                    extension: Some("txt".into()),
                    size: 100,
                },
            ),
            FileEntry::new(
                "second.txt",
                PathBuf::from("/tmp/second.txt"),
                EntryType::File {
                    extension: Some("txt".into()),
                    size: 200,
                },
            ),
        ];

        assert_eq!(state.current_entry().unwrap().name, "first.txt");
        state.cursor_down();
        assert_eq!(state.current_entry().unwrap().name, "second.txt");
    }

    #[test]
    fn test_ensure_visible() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        state.entries = (0..20)
            .map(|i| {
                FileEntry::new(
                    format!("file{}.txt", i),
                    PathBuf::from(format!("/tmp/file{}.txt", i)),
                    EntryType::File {
                        extension: Some("txt".into()),
                        size: 100,
                    },
                )
            })
            .collect();

        state.cursor_index = 15;
        state.ensure_visible(10);
        assert!(state.scroll >= 6); // 15 - 10 + 1 = 6
    }

    #[test]
    fn test_ensure_visible_zero_viewport() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        state.cursor_index = 5;
        state.scroll = 3;
        state.ensure_visible(0);
        assert_eq!(state.scroll, 3); // Should not change
    }

    #[test]
    fn test_style_color_for_extension() {
        let style = FileExplorerStyle::default();
        assert_eq!(style.color_for_extension(Some("rs")), Color::Yellow);
        assert_eq!(style.color_for_extension(Some("json")), Color::Green);
        assert_eq!(style.color_for_extension(Some("unknown")), Color::Gray);
        assert_eq!(style.color_for_extension(None), Color::Gray);
    }

    #[test]
    fn test_style_color_for_various_extensions() {
        let style = FileExplorerStyle::default();
        assert_eq!(style.color_for_extension(Some("toml")), Color::Green);
        assert_eq!(style.color_for_extension(Some("yaml")), Color::Green);
        assert_eq!(style.color_for_extension(Some("md")), Color::White);
        assert_eq!(style.color_for_extension(Some("py")), Color::Cyan);
        assert_eq!(style.color_for_extension(Some("js")), Color::Magenta);
        assert_eq!(style.color_for_extension(Some("sh")), Color::Red);
    }

    #[test]
    fn test_file_explorer_render() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        state.entries = vec![
            FileEntry::new("dir", PathBuf::from("/tmp/dir"), EntryType::Directory),
            FileEntry::new(
                "file.txt",
                PathBuf::from("/tmp/file.txt"),
                EntryType::File {
                    extension: Some("txt".into()),
                    size: 1024,
                },
            ),
        ];

        let explorer = FileExplorer::new(&state);
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 20));
        explorer.render(Rect::new(0, 0, 60, 20), &mut buf);
        // Should not panic
    }

    #[test]
    fn test_file_explorer_footer_default_num_lines() {
        let footer_builder = FooterBuilder::default();
        let lines = footer_builder.build(FileExplorerMode::Browse);
        assert_eq!(lines.len(), 2);
    }

    fn file(name: &str) -> FileEntry {
        FileEntry::new(
            name,
            PathBuf::from(format!("/tmp/{name}")),
            EntryType::File {
                extension: None,
                size: 1,
            },
        )
    }

    #[test]
    fn test_render_non_ascii_name_no_panic() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        state.entries.push(FileEntry::new(
            "ééééééééé.txt",
            PathBuf::from("/tmp/x"),
            EntryType::File {
                extension: Some("txt".into()),
                size: 1,
            },
        ));
        state.entries.push(file("日本語日本語日本語.txt"));
        state
            .entries
            .push(file("bad\u{FFFD}\u{1b}name-that-is-long"));
        // inner width 30 -> name_width 8: used to slice inside a 2-byte char
        let mut buf = Buffer::empty(Rect::new(0, 0, 32, 10));
        FileExplorer::new(&state).render(Rect::new(0, 0, 32, 10), &mut buf);
    }

    #[test]
    fn test_update_filter_resets_scroll() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        for i in 0..30 {
            state.entries.push(file(&format!("f{i:02}")));
        }
        state.cursor_index = 25;
        state.ensure_visible(5);
        assert!(state.scroll > 0);
        state.search_query = "f01".into();
        state.update_filter();
        assert_eq!(state.cursor_index, 0);
        assert_eq!(state.scroll, 0);
    }

    #[test]
    fn test_cancel_search_keeps_cursor_on_entry() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        for n in ["a", "b", "c", "target"] {
            state.entries.push(file(n));
        }
        state.search_query = "target".into();
        state.update_filter();
        assert_eq!(state.current_entry().unwrap().name, "target");
        state.cancel_search();
        assert_eq!(state.current_entry().unwrap().name, "target");

        // Clearing the query via update_filter maps the cursor back too
        state.search_query = "target".into();
        state.update_filter();
        state.search_query.clear();
        state.update_filter();
        assert_eq!(state.current_entry().unwrap().name, "target");
    }

    #[test]
    fn test_new_relative_dir_is_absolute() {
        let state = FileExplorerState::new(PathBuf::from("."));
        assert!(state.current_dir.is_absolute());
        assert!(
            state
                .current_dir
                .components()
                .all(|c| !matches!(c, Component::CurDir | Component::ParentDir))
        );
        // The ".." entry would point somewhere real
        assert!(
            state
                .current_dir
                .parent()
                .is_some_and(|p| !p.as_os_str().is_empty())
        );
    }

    #[test]
    fn test_scroll_saturates_and_cursor_still_rendered() {
        let mut state = FileExplorerState::new(PathBuf::from("/tmp"));
        state.entries = (0..70_000).map(|i| file(&format!("n{i}"))).collect();
        state.cursor_index = 69_999;
        state.ensure_visible(5);
        assert_eq!(state.scroll, u16::MAX);
        let explorer = FileExplorer::new(&state);
        let lines = explorer.build_lines(Rect::new(0, 0, 60, 5));
        let text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.to_string()))
            .collect();
        assert!(text.contains("n69999"));
    }

    #[cfg(feature = "filesystem")]
    #[test]
    fn test_enter_missing_directory_keeps_state() {
        let mut state = FileExplorerState::new(std::env::temp_dir());
        state.load_entries().unwrap();
        let before_dir = state.current_dir.clone();
        let before_len = state.entries.len();
        let missing = std::env::temp_dir().join("ratatui-interact-definitely-missing-dir");
        assert!(state.try_enter_directory(missing.clone()).is_err());
        state.enter_directory(missing);
        assert_eq!(state.current_dir, before_dir);
        assert_eq!(state.entries.len(), before_len);
    }

    #[cfg(feature = "filesystem")]
    #[test]
    fn test_go_up_from_relative_start() {
        let mut state = FileExplorerState::new(PathBuf::from("."));
        state.load_entries().unwrap();
        let start = state.current_dir.clone();
        state.try_go_up().unwrap();
        assert_eq!(Some(state.current_dir.as_path()), start.parent());
    }
}
