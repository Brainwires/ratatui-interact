//! Diff viewer widget
//!
//! A scrollable diff viewer with unified and side-by-side modes, syntax highlighting,
//! search functionality, and hunk navigation.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::components::{DiffViewer, DiffViewerState, DiffData};
//! use ratatui::layout::Rect;
//!
//! // Parse a unified diff
//! let diff_text = r#"
//! --- a/file.txt
//! +++ b/file.txt
//! @@ -1,3 +1,4 @@
//!  context line
//! -removed line
//! +added line
//! +another added line
//!  more context
//! "#;
//!
//! let diff = DiffData::from_unified_diff(diff_text);
//! let mut state = DiffViewerState::new(diff);
//!
//! // Create viewer
//! let viewer = DiffViewer::new(&state)
//!     .title("Changes");
//! ```

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Widget,
    },
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::log_viewer::SearchState;

// ============================================================================
// Enums
// ============================================================================

/// View mode for displaying diffs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffViewMode {
    /// Side-by-side view showing old and new files in parallel columns
    SideBySide,
    /// Unified view with + and - prefixes (default)
    #[default]
    Unified,
}

/// Type of diff line
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineType {
    /// Context line (unchanged)
    Context,
    /// Added line
    Addition,
    /// Removed line
    Deletion,
    /// Hunk header (@@ ... @@)
    HunkHeader,
}

/// Actions that can be triggered by diff viewer interactions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffViewerAction {
    /// Scroll to a specific line
    ScrollToLine(usize),
    /// Jump to a specific hunk by index
    JumpToHunk(usize),
    /// Toggle between side-by-side and unified modes
    ToggleViewMode,
}

// ============================================================================
// Data Structures
// ============================================================================

/// A single line in a diff
#[derive(Debug, Clone)]
pub struct DiffLine {
    /// Type of the line (context, addition, deletion, header)
    pub line_type: DiffLineType,
    /// Content of the line (without the +/- prefix)
    pub content: String,
    /// Line number in the old file (None for additions/headers)
    pub old_line_num: Option<usize>,
    /// Line number in the new file (None for deletions/headers)
    pub new_line_num: Option<usize>,
    /// Character ranges for inline changes (start, end) within the line
    pub inline_changes: Vec<(usize, usize)>,
}

impl DiffLine {
    /// Create a new diff line
    pub fn new(line_type: DiffLineType, content: String) -> Self {
        Self {
            line_type,
            content,
            old_line_num: None,
            new_line_num: None,
            inline_changes: Vec::new(),
        }
    }

    /// Create a context line
    pub fn context(content: String, old_num: usize, new_num: usize) -> Self {
        Self {
            line_type: DiffLineType::Context,
            content,
            old_line_num: Some(old_num),
            new_line_num: Some(new_num),
            inline_changes: Vec::new(),
        }
    }

    /// Create an addition line
    pub fn addition(content: String, new_num: usize) -> Self {
        Self {
            line_type: DiffLineType::Addition,
            content,
            old_line_num: None,
            new_line_num: Some(new_num),
            inline_changes: Vec::new(),
        }
    }

    /// Create a deletion line
    pub fn deletion(content: String, old_num: usize) -> Self {
        Self {
            line_type: DiffLineType::Deletion,
            content,
            old_line_num: Some(old_num),
            new_line_num: None,
            inline_changes: Vec::new(),
        }
    }

    /// Create a hunk header line
    pub fn hunk_header(content: String) -> Self {
        Self {
            line_type: DiffLineType::HunkHeader,
            content,
            old_line_num: None,
            new_line_num: None,
            inline_changes: Vec::new(),
        }
    }

    /// Set inline changes for highlighting
    pub fn with_inline_changes(mut self, changes: Vec<(usize, usize)>) -> Self {
        self.inline_changes = changes;
        self
    }
}

/// A hunk in a diff (a contiguous block of changes)
#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// The hunk header string (e.g., "@@ -1,3 +1,4 @@")
    pub header: String,
    /// Starting line number in the old file
    pub old_start: usize,
    /// Number of lines from the old file
    pub old_count: usize,
    /// Starting line number in the new file
    pub new_start: usize,
    /// Number of lines in the new file
    pub new_count: usize,
    /// Lines in this hunk
    pub lines: Vec<DiffLine>,
}

impl DiffHunk {
    /// Create a new hunk
    pub fn new(
        header: String,
        old_start: usize,
        old_count: usize,
        new_start: usize,
        new_count: usize,
    ) -> Self {
        Self {
            header,
            old_start,
            old_count,
            new_start,
            new_count,
            lines: Vec::new(),
        }
    }

    /// Add a line to the hunk
    pub fn add_line(&mut self, line: DiffLine) {
        self.lines.push(line);
    }

    /// Get the number of additions in this hunk
    pub fn addition_count(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Addition)
            .count()
    }

    /// Get the number of deletions in this hunk
    pub fn deletion_count(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Deletion)
            .count()
    }
}

/// Complete diff data for one or more files
#[derive(Debug, Clone, Default)]
pub struct DiffData {
    /// Path to the old file
    pub old_path: Option<String>,
    /// Path to the new file
    pub new_path: Option<String>,
    /// Hunks in the diff
    pub hunks: Vec<DiffHunk>,
}

impl DiffData {
    /// Create empty diff data
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create diff data with paths
    pub fn new(old_path: Option<String>, new_path: Option<String>) -> Self {
        Self {
            old_path,
            new_path,
            hunks: Vec::new(),
        }
    }

    /// Parse a unified diff text into DiffData
    pub fn from_unified_diff(text: &str) -> Self {
        let mut diff = DiffData::empty();
        let mut current_hunk: Option<DiffHunk> = None;
        let mut old_line_num: usize = 0;
        let mut new_line_num: usize = 0;
        // Lines still expected in the current hunk (from its header counts)
        let mut old_remaining: usize = 0;
        let mut new_remaining: usize = 0;

        for line in text.lines() {
            // File headers. Inside a hunk, "--- x" / "+++ x" are ordinary
            // deleted/added lines (e.g. a removed "-- comment"), so only treat
            // them as headers outside a hunk or once its line counts are used up.
            let in_hunk = current_hunk.is_some() && (old_remaining > 0 || new_remaining > 0);
            if !in_hunk {
                if let Some(path) = line.strip_prefix("--- ") {
                    diff.old_path = Some(path.trim_start_matches("a/").to_string());
                    continue;
                }
                if let Some(path) = line.strip_prefix("+++ ") {
                    diff.new_path = Some(path.trim_start_matches("b/").to_string());
                    continue;
                }
            }

            // Hunk header
            if line.starts_with("@@") {
                // Save previous hunk if any
                if let Some(hunk) = current_hunk.take() {
                    diff.hunks.push(hunk);
                }

                // Parse @@ -old_start,old_count +new_start,new_count @@
                if let Some((old_start, old_count, new_start, new_count)) = parse_hunk_header(line)
                {
                    current_hunk = Some(DiffHunk::new(
                        line.to_string(),
                        old_start,
                        old_count,
                        new_start,
                        new_count,
                    ));
                    old_line_num = old_start;
                    new_line_num = new_start;
                    old_remaining = old_count;
                    new_remaining = new_count;
                }
                continue;
            }

            // Diff content lines
            if let Some(hunk) = current_hunk.as_mut() {
                if let Some(content) = line.strip_prefix('+') {
                    // Addition
                    hunk.add_line(DiffLine::addition(content.to_string(), new_line_num));
                    new_line_num += 1;
                    new_remaining = new_remaining.saturating_sub(1);
                } else if let Some(content) = line.strip_prefix('-') {
                    // Deletion
                    hunk.add_line(DiffLine::deletion(content.to_string(), old_line_num));
                    old_line_num += 1;
                    old_remaining = old_remaining.saturating_sub(1);
                } else if let Some(content) = line.strip_prefix(' ') {
                    // Context
                    hunk.add_line(DiffLine::context(
                        content.to_string(),
                        old_line_num,
                        new_line_num,
                    ));
                    old_line_num += 1;
                    new_line_num += 1;
                    old_remaining = old_remaining.saturating_sub(1);
                    new_remaining = new_remaining.saturating_sub(1);
                } else if line.is_empty() || line == "\\ No newline at end of file" {
                    // Empty context line or no-newline marker
                    if line.is_empty() {
                        hunk.add_line(DiffLine::context(String::new(), old_line_num, new_line_num));
                        old_line_num += 1;
                        new_line_num += 1;
                        old_remaining = old_remaining.saturating_sub(1);
                        new_remaining = new_remaining.saturating_sub(1);
                    }
                }
            }
        }

        // Don't forget the last hunk
        if let Some(hunk) = current_hunk {
            diff.hunks.push(hunk);
        }

        diff
    }

    /// Get total number of additions across all hunks
    pub fn total_additions(&self) -> usize {
        self.hunks.iter().map(|h| h.addition_count()).sum()
    }

    /// Get total number of deletions across all hunks
    pub fn total_deletions(&self) -> usize {
        self.hunks.iter().map(|h| h.deletion_count()).sum()
    }

    /// Get all lines flattened (for display purposes)
    pub fn all_lines(&self) -> Vec<&DiffLine> {
        let mut lines = Vec::new();
        for hunk in &self.hunks {
            for line in &hunk.lines {
                lines.push(line);
            }
        }
        lines
    }

    /// Check if the diff is empty
    pub fn is_empty(&self) -> bool {
        self.hunks.is_empty()
    }
}

/// Parse a hunk header line like "@@ -1,3 +1,4 @@" or "@@ -1 +1 @@"
fn parse_hunk_header(line: &str) -> Option<(usize, usize, usize, usize)> {
    // Remove @@ markers and any trailing context
    let content = line.trim_start_matches("@@ ").trim_end();
    let end_marker_pos = content.find(" @@")?;
    let ranges = &content[..end_marker_pos];

    let mut parts = ranges.split_whitespace();
    let old_range = parts.next()?.strip_prefix('-')?;
    let new_range = parts.next()?.strip_prefix('+')?;

    let (old_start, old_count) = parse_range(old_range);
    let (new_start, new_count) = parse_range(new_range);

    Some((old_start, old_count, new_start, new_count))
}

/// Parse a range like "1,3" or "1" into (start, count)
fn parse_range(range: &str) -> (usize, usize) {
    if let Some((start, count)) = range.split_once(',') {
        (start.parse().unwrap_or(1), count.parse().unwrap_or(1))
    } else {
        (range.parse().unwrap_or(1), 1)
    }
}

/// Pair deletion/addition line indices for side-by-side display.
///
/// Context lines appear on both sides; runs of deletions and additions are
/// paired up row by row.
fn pair_line_indices(lines: &[DiffLine]) -> Vec<(Option<usize>, Option<usize>)> {
    let mut pairs = Vec::new();
    let mut deletions: Vec<usize> = Vec::new();
    let mut additions: Vec<usize> = Vec::new();

    let flush = |pairs: &mut Vec<_>, deletions: &mut Vec<usize>, additions: &mut Vec<usize>| {
        let max_len = deletions.len().max(additions.len());
        for i in 0..max_len {
            pairs.push((deletions.get(i).copied(), additions.get(i).copied()));
        }
        deletions.clear();
        additions.clear();
    };

    for (idx, line) in lines.iter().enumerate() {
        match line.line_type {
            DiffLineType::Context => {
                // Flush any pending deletions/additions
                flush(&mut pairs, &mut deletions, &mut additions);
                pairs.push((Some(idx), Some(idx)));
            }
            DiffLineType::Deletion => deletions.push(idx),
            DiffLineType::Addition => additions.push(idx),
            DiffLineType::HunkHeader => {
                // Shouldn't happen here
            }
        }
    }

    // Flush remaining
    flush(&mut pairs, &mut deletions, &mut additions);

    pairs
}

/// One displayed row of the diff in a given view mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DisplayRow {
    /// Index of the hunk this row belongs to
    hunk: usize,
    /// What the row shows
    kind: RowKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowKind {
    /// The hunk header
    Header,
    /// Indices into the hunk's lines shown on this row (left/old, right/new).
    /// In unified mode only the first is used.
    Lines(Option<usize>, Option<usize>),
}

impl RowKind {
    fn contains_line(&self, idx: usize) -> bool {
        matches!(self, RowKind::Lines(a, b) if *a == Some(idx) || *b == Some(idx))
    }

    fn first_line(&self) -> Option<usize> {
        match self {
            RowKind::Header => None,
            RowKind::Lines(a, b) => a.or(*b),
        }
    }
}

// ============================================================================
// State
// ============================================================================

/// State for the diff viewer widget
#[derive(Debug, Clone)]
pub struct DiffViewerState {
    /// The diff data to display
    pub diff: DiffData,
    /// Current view mode
    pub view_mode: DiffViewMode,
    /// Vertical scroll position
    pub scroll_y: usize,
    /// Horizontal scroll position
    pub scroll_x: usize,
    /// Visible viewport height in rows.
    ///
    /// [`DiffViewer`] renders from a shared reference and cannot update this,
    /// so the caller must keep it in sync (for example with
    /// [`DiffViewerState::set_viewport`] each frame). Paging and
    /// [`DiffViewerState::go_to_bottom`] depend on it; while it is 0 they
    /// treat the page as one row.
    pub visible_height: usize,
    /// Visible viewport width in columns (see `visible_height`).
    pub visible_width: usize,
    /// Currently selected hunk index (for navigation)
    pub selected_hunk: Option<usize>,
    /// Whether to show line numbers
    pub show_line_numbers: bool,
    /// Search state
    pub search: SearchState,
}

impl DiffViewerState {
    /// Create a new diff viewer state with diff data
    pub fn new(diff: DiffData) -> Self {
        let selected_hunk = if diff.hunks.is_empty() { None } else { Some(0) };
        Self {
            diff,
            view_mode: DiffViewMode::default(),
            scroll_y: 0,
            scroll_x: 0,
            visible_height: 0,
            visible_width: 0,
            selected_hunk,
            show_line_numbers: true,
            search: SearchState::default(),
        }
    }

    /// Create a state from unified diff text
    pub fn from_unified_diff(text: &str) -> Self {
        let diff = DiffData::from_unified_diff(text);
        Self::new(diff)
    }

    /// Create an empty diff viewer state
    pub fn empty() -> Self {
        Self::new(DiffData::empty())
    }

    /// Set the diff data
    pub fn set_diff(&mut self, diff: DiffData) {
        self.diff = diff;
        self.scroll_y = 0;
        self.scroll_x = 0;
        self.selected_hunk = if self.diff.hunks.is_empty() {
            None
        } else {
            Some(0)
        };
        self.search.matches.clear();
    }

    /// Rows displayed in the given view mode.
    fn rows_for(&self, mode: DiffViewMode) -> Vec<DisplayRow> {
        let mut rows = Vec::new();
        for (hunk_idx, hunk) in self.diff.hunks.iter().enumerate() {
            rows.push(DisplayRow {
                hunk: hunk_idx,
                kind: RowKind::Header,
            });
            match mode {
                DiffViewMode::Unified => {
                    rows.extend((0..hunk.lines.len()).map(|i| DisplayRow {
                        hunk: hunk_idx,
                        kind: RowKind::Lines(Some(i), None),
                    }));
                }
                DiffViewMode::SideBySide => {
                    rows.extend(
                        pair_line_indices(&hunk.lines)
                            .into_iter()
                            .map(|(old, new)| DisplayRow {
                                hunk: hunk_idx,
                                kind: RowKind::Lines(old, new),
                            }),
                    );
                }
            }
        }
        rows
    }

    /// Rows displayed in the current view mode.
    fn rows(&self) -> Vec<DisplayRow> {
        self.rows_for(self.view_mode)
    }

    /// Diff lines shown on a row
    fn row_lines(&self, row: &DisplayRow) -> impl Iterator<Item = &DiffLine> {
        let lines = &self.diff.hunks[row.hunk].lines;
        let (a, b) = match row.kind {
            RowKind::Header => (None, None),
            RowKind::Lines(a, b) => (a, if b == a { None } else { b }),
        };
        a.into_iter().chain(b).filter_map(move |i| lines.get(i))
    }

    /// Whether a row shows an addition or deletion
    fn row_is_change(&self, row: &DisplayRow) -> bool {
        self.row_lines(row)
            .any(|l| l.line_type == DiffLineType::Addition || l.line_type == DiffLineType::Deletion)
    }

    /// Get total row count for scrolling (depends on the view mode)
    fn total_lines(&self) -> usize {
        match self.view_mode {
            DiffViewMode::Unified => self
                .diff
                .hunks
                .iter()
                .map(|h| h.lines.len() + 1)
                .sum::<usize>(), // +1 for hunk header
            DiffViewMode::SideBySide => self.rows().len(),
        }
    }

    /// Page size used for paging, at least one row.
    fn page_size(&self) -> usize {
        self.visible_height.max(1)
    }

    /// Set `visible_height` / `visible_width` from the area the
    /// [`DiffViewer`] will be rendered into (the whole widget area, including
    /// borders, status bar and search bar).
    ///
    /// Call this before handling input or rendering so that paging and
    /// [`DiffViewerState::go_to_bottom`] use the real viewport size.
    pub fn set_viewport(&mut self, area: Rect) {
        let inner = content_inner_area(area, self.search.active);
        self.visible_height = inner.height as usize;
        self.visible_width = inner.width as usize;
    }

    // Navigation methods

    /// Scroll up by one line
    pub fn scroll_up(&mut self) {
        self.scroll_y = self.scroll_y.saturating_sub(1);
    }

    /// Scroll down by one line
    pub fn scroll_down(&mut self) {
        let total = self.total_lines();
        if self.scroll_y + 1 < total {
            self.scroll_y += 1;
        }
    }

    /// Scroll left
    pub fn scroll_left(&mut self) {
        self.scroll_x = self.scroll_x.saturating_sub(4);
    }

    /// Scroll right
    pub fn scroll_right(&mut self) {
        self.scroll_x += 4;
    }

    /// Scroll up by one page
    ///
    /// Uses `visible_height`, which the caller must set (see
    /// [`DiffViewerState::set_viewport`]).
    pub fn page_up(&mut self) {
        self.scroll_y = self.scroll_y.saturating_sub(self.page_size());
    }

    /// Scroll down by one page
    ///
    /// Uses `visible_height`, which the caller must set (see
    /// [`DiffViewerState::set_viewport`]).
    pub fn page_down(&mut self) {
        let total = self.total_lines();
        let max_scroll = total.saturating_sub(self.page_size());
        self.scroll_y = (self.scroll_y + self.page_size()).min(max_scroll);
    }

    /// Go to top
    pub fn go_to_top(&mut self) {
        self.scroll_y = 0;
        self.selected_hunk = if self.diff.hunks.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    /// Go to bottom
    ///
    /// Uses `visible_height`, which the caller must set (see
    /// [`DiffViewerState::set_viewport`]); while it is unknown (0) this
    /// scrolls to the last row.
    pub fn go_to_bottom(&mut self) {
        let total = self.total_lines();
        self.scroll_y = total.saturating_sub(self.page_size());
        self.selected_hunk = if self.diff.hunks.is_empty() {
            None
        } else {
            Some(self.diff.hunks.len() - 1)
        };
    }

    /// Go to a specific line (0-indexed)
    pub fn go_to_line(&mut self, line: usize) {
        let total = self.total_lines();
        self.scroll_y = line.min(total.saturating_sub(1));
    }

    // Hunk navigation

    /// Get the row index where a hunk starts (in the current view mode)
    fn hunk_start_line(&self, hunk_index: usize) -> usize {
        let rows = self.rows();
        rows.iter()
            .position(|r| r.hunk == hunk_index && r.kind == RowKind::Header)
            .unwrap_or(rows.len())
    }

    /// Scroll targets for change rows: one row above each change, so the
    /// change appears with a line of context.
    fn change_scroll_targets(&self) -> Vec<usize> {
        self.rows()
            .iter()
            .enumerate()
            .filter(|(_, r)| self.row_is_change(r))
            .map(|(i, _)| i.saturating_sub(1))
            .collect()
    }

    /// Jump to the next hunk
    pub fn next_hunk(&mut self) {
        if self.diff.hunks.is_empty() {
            return;
        }
        let current = self.selected_hunk.unwrap_or(0);
        let next = (current + 1).min(self.diff.hunks.len() - 1);
        self.selected_hunk = Some(next);
        self.scroll_y = self.hunk_start_line(next);
    }

    /// Jump to the previous hunk
    pub fn prev_hunk(&mut self) {
        if self.diff.hunks.is_empty() {
            return;
        }
        let current = self.selected_hunk.unwrap_or(0);
        let prev = current.saturating_sub(1);
        self.selected_hunk = Some(prev);
        self.scroll_y = self.hunk_start_line(prev);
    }

    /// Jump to a specific hunk by index
    pub fn jump_to_hunk(&mut self, index: usize) {
        if index < self.diff.hunks.len() {
            self.selected_hunk = Some(index);
            self.scroll_y = self.hunk_start_line(index);
        }
    }

    /// Navigate to next change (addition or deletion)
    pub fn next_change(&mut self) {
        let targets = self.change_scroll_targets();
        if let Some(&next) = targets.iter().find(|&&t| t > self.scroll_y) {
            self.scroll_y = next;
        } else {
            // Wrap around to the first change (or the top)
            self.scroll_y = targets.first().copied().unwrap_or(0);
        }
    }

    /// Navigate to previous change (addition or deletion)
    pub fn prev_change(&mut self) {
        if self.scroll_y == 0 {
            // Start from end
            self.go_to_bottom();
        }

        let line_idx = self.scroll_y.saturating_sub(1);
        let targets = self.change_scroll_targets();

        // Find the closest change before current position
        if let Some(&prev) = targets.iter().rev().find(|&&t| t <= line_idx) {
            self.scroll_y = prev;
        } else if let Some(&last) = targets.last() {
            // Wrap to last change
            self.scroll_y = last;
        }
    }

    // View mode

    /// Toggle between side-by-side and unified view modes
    pub fn toggle_view_mode(&mut self) {
        let mode = match self.view_mode {
            DiffViewMode::SideBySide => DiffViewMode::Unified,
            DiffViewMode::Unified => DiffViewMode::SideBySide,
        };
        self.set_view_mode(mode);
    }

    /// Set the view mode
    ///
    /// The scroll position and search matches are mapped to the rows of the
    /// new mode, since side-by-side shows fewer rows than unified.
    pub fn set_view_mode(&mut self, mode: DiffViewMode) {
        if mode == self.view_mode {
            return;
        }
        let old_rows = self.rows();
        let anchor = old_rows.get(self.scroll_y).copied();
        let current_match_row = self
            .search
            .matches
            .get(self.search.current_match)
            .and_then(|&i| old_rows.get(i).copied());

        self.view_mode = mode;
        let new_rows = self.rows();
        let map_row = |row: DisplayRow| -> Option<usize> {
            new_rows.iter().position(|r| {
                r.hunk == row.hunk
                    && match row.kind.first_line() {
                        None => r.kind == RowKind::Header,
                        Some(line) => r.kind.contains_line(line),
                    }
            })
        };

        if let Some(row) = anchor {
            if let Some(new_idx) = map_row(row) {
                self.scroll_y = new_idx;
            }
        }
        self.scroll_y = self.scroll_y.min(new_rows.len().saturating_sub(1));

        if !self.search.query.is_empty() {
            self.recompute_search_matches();
            if let Some(target) = current_match_row.and_then(map_row) {
                if let Some(pos) = self.search.matches.iter().position(|&m| m == target) {
                    self.search.current_match = pos;
                }
            }
        }
    }

    // Search methods

    /// Start search mode
    pub fn start_search(&mut self) {
        self.search.active = true;
        self.search.query.clear();
        self.search.matches.clear();
        self.search.current_match = 0;
    }

    /// Cancel search mode
    pub fn cancel_search(&mut self) {
        self.search.active = false;
    }

    /// Recompute `search.matches` (row indices in the current view mode)
    /// without moving the scroll position.
    fn recompute_search_matches(&mut self) {
        self.search.matches.clear();
        self.search.current_match = 0;

        if self.search.query.is_empty() {
            return;
        }

        let query = self.search.query.to_lowercase();
        let matches: Vec<usize> = self
            .rows()
            .iter()
            .enumerate()
            .filter(|(_, row)| match row.kind {
                RowKind::Header => self.diff.hunks[row.hunk]
                    .header
                    .to_lowercase()
                    .contains(&query),
                RowKind::Lines(..) => self
                    .row_lines(row)
                    .any(|l| l.content.to_lowercase().contains(&query)),
            })
            .map(|(i, _)| i)
            .collect();
        self.search.matches = matches;
    }

    /// Update search with current query
    ///
    /// Matches are row indices in the current view mode.
    pub fn update_search(&mut self) {
        self.recompute_search_matches();

        // Jump to first match if any
        if !self.search.matches.is_empty() {
            self.scroll_y = self.search.matches[0];
        }
    }

    /// Go to next search match
    pub fn next_match(&mut self) {
        if self.search.matches.is_empty() {
            return;
        }
        self.search.current_match = (self.search.current_match + 1) % self.search.matches.len();
        self.scroll_y = self.search.matches[self.search.current_match];
    }

    /// Go to previous search match
    pub fn prev_match(&mut self) {
        if self.search.matches.is_empty() {
            return;
        }
        if self.search.current_match == 0 {
            self.search.current_match = self.search.matches.len() - 1;
        } else {
            self.search.current_match -= 1;
        }
        self.scroll_y = self.search.matches[self.search.current_match];
    }
}

// ============================================================================
// Style
// ============================================================================

/// Style configuration for diff viewer
#[derive(Debug, Clone)]
pub struct DiffViewerStyle {
    /// Border style
    pub border_style: Style,
    /// Line number style
    pub line_number_style: Style,
    /// Context line style (unchanged lines)
    pub context_style: Style,
    /// Addition text style
    pub addition_style: Style,
    /// Addition background color
    pub addition_bg: Color,
    /// Deletion text style
    pub deletion_style: Style,
    /// Deletion background color
    pub deletion_bg: Color,
    /// Inline addition highlight style (for character-level diffs)
    pub inline_addition_style: Style,
    /// Inline deletion highlight style (for character-level diffs)
    pub inline_deletion_style: Style,
    /// Hunk header style
    pub hunk_header_style: Style,
    /// Search match highlight style
    pub match_style: Style,
    /// Current search match highlight style
    pub current_match_style: Style,
    /// Gutter separator character
    pub gutter_separator: &'static str,
    /// Side-by-side mode separator character
    pub side_separator: &'static str,
}

impl Default for DiffViewerStyle {
    fn default() -> Self {
        Self {
            border_style: Style::default().fg(Color::Cyan),
            line_number_style: Style::default().fg(Color::DarkGray),
            context_style: Style::default().fg(Color::White),
            addition_style: Style::default().fg(Color::Green),
            addition_bg: Color::Rgb(0, 40, 0),
            deletion_style: Style::default().fg(Color::Red),
            deletion_bg: Color::Rgb(40, 0, 0),
            inline_addition_style: Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
            inline_deletion_style: Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
            hunk_header_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            match_style: Style::default()
                .bg(Color::Rgb(60, 60, 30))
                .fg(Color::Yellow),
            current_match_style: Style::default().bg(Color::Yellow).fg(Color::Black),
            gutter_separator: "│",
            side_separator: "│",
        }
    }
}

impl From<&crate::theme::Theme> for DiffViewerStyle {
    fn from(theme: &crate::theme::Theme) -> Self {
        let p = &theme.palette;
        Self {
            border_style: Style::default().fg(p.border_accent),
            line_number_style: Style::default().fg(p.text_disabled),
            context_style: Style::default().fg(p.text),
            addition_style: Style::default().fg(p.diff_add_fg),
            addition_bg: p.diff_add_bg,
            deletion_style: Style::default().fg(p.diff_del_fg),
            deletion_bg: p.diff_del_bg,
            inline_addition_style: Style::default()
                .fg(p.highlight_fg)
                .bg(p.diff_add_fg)
                .add_modifier(Modifier::BOLD),
            inline_deletion_style: Style::default()
                .fg(p.highlight_fg)
                .bg(p.diff_del_fg)
                .add_modifier(Modifier::BOLD),
            hunk_header_style: Style::default()
                .fg(p.secondary)
                .add_modifier(Modifier::BOLD),
            match_style: Style::default().bg(Color::Rgb(60, 60, 30)).fg(p.primary),
            current_match_style: Style::default().bg(p.highlight_bg).fg(p.highlight_fg),
            gutter_separator: "│",
            side_separator: "│",
        }
    }
}

impl DiffViewerStyle {
    /// Create a style with high contrast colors
    pub fn high_contrast() -> Self {
        Self {
            addition_style: Style::default().fg(Color::LightGreen),
            addition_bg: Color::Rgb(0, 60, 0),
            deletion_style: Style::default().fg(Color::LightRed),
            deletion_bg: Color::Rgb(60, 0, 0),
            ..Default::default()
        }
    }

    /// Create a monochrome style
    pub fn monochrome() -> Self {
        Self {
            addition_style: Style::default().add_modifier(Modifier::BOLD),
            addition_bg: Color::Reset,
            deletion_style: Style::default().add_modifier(Modifier::DIM),
            deletion_bg: Color::Reset,
            inline_addition_style: Style::default()
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            inline_deletion_style: Style::default()
                .add_modifier(Modifier::DIM | Modifier::CROSSED_OUT),
            ..Default::default()
        }
    }
}

// ============================================================================
// Widget
// ============================================================================

/// Diff viewer widget
pub struct DiffViewer<'a> {
    state: &'a DiffViewerState,
    style: DiffViewerStyle,
    title: Option<&'a str>,
    show_stats: bool,
}

impl<'a> DiffViewer<'a> {
    /// Create a new diff viewer
    pub fn new(state: &'a DiffViewerState) -> Self {
        Self {
            state,
            style: DiffViewerStyle::default(),
            title: None,
            show_stats: true,
        }
    }

    /// Set the title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Set the style
    pub fn style(mut self, style: DiffViewerStyle) -> Self {
        self.style = style;
        self
    }

    /// Apply a theme to derive the style
    pub fn theme(self, theme: &crate::theme::Theme) -> Self {
        self.style(DiffViewerStyle::from(theme))
    }

    /// Enable or disable line numbers
    pub fn show_line_numbers(self, _show: bool) -> Self {
        // Line numbers are controlled by state, not the widget
        // This is a no-op provided for API consistency
        self
    }

    /// Enable or disable stats display
    pub fn show_stats(mut self, show: bool) -> Self {
        self.show_stats = show;
        self
    }

    /// Calculate line number width
    fn line_number_width(&self) -> usize {
        if !self.state.show_line_numbers {
            return 0;
        }
        // Calculate max line number across all hunks
        let max_line = self
            .state
            .diff
            .hunks
            .iter()
            .map(|h| h.old_start + h.old_count.max(h.new_count))
            .max()
            .unwrap_or(1);
        max_line.to_string().len().max(3)
    }

    /// Search match flags `(is_match, is_current_match)` for a display row
    fn match_flags(&self, row: usize) -> (bool, bool) {
        let matches = &self.state.search.matches;
        (
            matches.contains(&row),
            matches.get(self.state.search.current_match) == Some(&row),
        )
    }

    /// Build lines for unified view
    fn build_unified_lines(&self, inner: Rect) -> Vec<Line<'static>> {
        let visible_height = inner.height as usize;
        let line_num_width = self.line_number_width();
        let visible_width = if self.state.show_line_numbers {
            inner.width.saturating_sub((line_num_width * 2 + 4) as u16) as usize
        } else {
            inner.width.saturating_sub(2) as usize // Just prefix space
        };

        let mut lines = Vec::new();
        let mut current_line = 0;
        let start_line = self.state.scroll_y;
        let end_line = start_line + visible_height;

        for hunk in &self.state.diff.hunks {
            // Hunk header
            if current_line >= start_line && current_line < end_line {
                let is_match = self.state.search.matches.contains(&current_line);
                let is_current_match = self
                    .state
                    .search
                    .matches
                    .get(self.state.search.current_match)
                    == Some(&current_line);

                let header_style = if is_current_match {
                    self.style.current_match_style
                } else if is_match {
                    self.style.match_style
                } else {
                    self.style.hunk_header_style
                };

                let header_content: String = hunk
                    .header
                    .chars()
                    .skip(self.state.scroll_x)
                    .take(inner.width as usize)
                    .collect();
                lines.push(Line::from(Span::styled(header_content, header_style)));
            }
            current_line += 1;

            // Hunk lines
            for line in &hunk.lines {
                if current_line >= start_line && current_line < end_line {
                    let is_match = self.state.search.matches.contains(&current_line);
                    let is_current_match = self
                        .state
                        .search
                        .matches
                        .get(self.state.search.current_match)
                        == Some(&current_line);

                    lines.push(self.build_unified_line(
                        line,
                        line_num_width,
                        visible_width,
                        is_match,
                        is_current_match,
                    ));
                }
                current_line += 1;

                if current_line >= end_line {
                    break;
                }
            }

            if current_line >= end_line {
                break;
            }
        }

        lines
    }

    /// Build a single unified diff line
    fn build_unified_line(
        &self,
        line: &DiffLine,
        line_num_width: usize,
        visible_width: usize,
        is_match: bool,
        is_current_match: bool,
    ) -> Line<'static> {
        let mut spans = Vec::new();

        // Line numbers
        if self.state.show_line_numbers {
            let old_num = line
                .old_line_num
                .map(|n| format!("{:>width$}", n, width = line_num_width))
                .unwrap_or_else(|| " ".repeat(line_num_width));
            let new_num = line
                .new_line_num
                .map(|n| format!("{:>width$}", n, width = line_num_width))
                .unwrap_or_else(|| " ".repeat(line_num_width));

            spans.push(Span::styled(old_num, self.style.line_number_style));
            spans.push(Span::styled(" ", self.style.line_number_style));
            spans.push(Span::styled(new_num, self.style.line_number_style));
            spans.push(Span::styled(
                format!(" {} ", self.style.gutter_separator),
                self.style.line_number_style,
            ));
        }

        // Prefix and content
        let (prefix, content_style, bg_style) = match line.line_type {
            DiffLineType::Context => (" ", self.style.context_style, Style::default()),
            DiffLineType::Addition => (
                "+",
                self.style.addition_style,
                Style::default().bg(self.style.addition_bg),
            ),
            DiffLineType::Deletion => (
                "-",
                self.style.deletion_style,
                Style::default().bg(self.style.deletion_bg),
            ),
            DiffLineType::HunkHeader => ("@", self.style.hunk_header_style, Style::default()),
        };

        // Apply search highlighting
        let final_style = if is_current_match {
            self.style.current_match_style
        } else if is_match {
            self.style.match_style
        } else {
            content_style.patch(bg_style)
        };

        spans.push(Span::styled(prefix.to_string(), final_style));

        // Content with horizontal scroll
        let content: String = line
            .content
            .chars()
            .skip(self.state.scroll_x)
            .take(visible_width)
            .collect();

        spans.push(Span::styled(content, final_style));

        Line::from(spans)
    }

    /// Build lines for side-by-side view
    fn build_side_by_side_lines(&self, inner: Rect) -> Vec<Line<'static>> {
        let visible_height = inner.height as usize;
        let half_width = (inner.width.saturating_sub(1) / 2) as usize; // -1 for separator
        let line_num_width = self.line_number_width();
        let content_width = if self.state.show_line_numbers {
            half_width.saturating_sub(line_num_width + 3) // line num + prefix + separator
        } else {
            half_width.saturating_sub(2) // Just prefix
        };

        let mut lines = Vec::new();
        let mut current_line = 0;
        let start_line = self.state.scroll_y;
        let end_line = start_line + visible_height;

        for hunk in &self.state.diff.hunks {
            // Hunk header (spans both sides)
            if current_line >= start_line && current_line < end_line {
                let (is_match, is_current_match) = self.match_flags(current_line);
                let header_style = if is_current_match {
                    self.style.current_match_style
                } else if is_match {
                    self.style.match_style
                } else {
                    self.style.hunk_header_style
                };
                let header_content: String = hunk
                    .header
                    .chars()
                    .skip(self.state.scroll_x)
                    .take(inner.width as usize)
                    .collect();
                lines.push(Line::from(Span::styled(header_content, header_style)));
            }
            current_line += 1;

            // Process lines in pairs for side-by-side
            let paired_lines = self.pair_lines_for_side_by_side(&hunk.lines);

            for (old_line, new_line) in paired_lines {
                if current_line >= start_line && current_line < end_line {
                    let (is_match, is_current_match) = self.match_flags(current_line);
                    lines.push(self.build_side_by_side_line(
                        old_line,
                        new_line,
                        line_num_width,
                        content_width,
                        half_width,
                        (is_match, is_current_match),
                    ));
                }
                current_line += 1;

                if current_line >= end_line {
                    break;
                }
            }

            if current_line >= end_line {
                break;
            }
        }

        lines
    }

    /// Pair deletion/addition lines for side-by-side display
    fn pair_lines_for_side_by_side<'b>(
        &self,
        lines: &'b [DiffLine],
    ) -> Vec<(Option<&'b DiffLine>, Option<&'b DiffLine>)> {
        pair_line_indices(lines)
            .into_iter()
            .map(|(old, new)| (old.map(|i| &lines[i]), new.map(|i| &lines[i])))
            .collect()
    }

    /// Build a side-by-side line
    fn build_side_by_side_line(
        &self,
        old_line: Option<&DiffLine>,
        new_line: Option<&DiffLine>,
        line_num_width: usize,
        content_width: usize,
        half_width: usize,
        match_flags: (bool, bool),
    ) -> Line<'static> {
        let mut spans = Vec::new();

        // Left side (old)
        spans.extend(self.build_half_line(
            old_line,
            line_num_width,
            content_width,
            true,
            match_flags,
        ));

        // Pad to half width (by display width, so wide characters line up)
        let left_len: usize = spans.iter().map(|s| s.content.width()).sum();
        if left_len < half_width {
            spans.push(Span::raw(" ".repeat(half_width - left_len)));
        }

        // Separator
        spans.push(Span::styled(
            self.style.side_separator,
            Style::default().fg(Color::DarkGray),
        ));

        // Right side (new)
        spans.extend(self.build_half_line(
            new_line,
            line_num_width,
            content_width,
            false,
            match_flags,
        ));

        Line::from(spans)
    }

    /// Build one half of a side-by-side line
    fn build_half_line(
        &self,
        line: Option<&DiffLine>,
        line_num_width: usize,
        content_width: usize,
        is_old: bool,
        (is_match, is_current_match): (bool, bool),
    ) -> Vec<Span<'static>> {
        let mut spans = Vec::new();

        match line {
            Some(l) => {
                // Line number
                if self.state.show_line_numbers {
                    let num = if is_old {
                        l.old_line_num
                    } else {
                        l.new_line_num
                    };
                    let num_str = num
                        .map(|n| format!("{:>width$}", n, width = line_num_width))
                        .unwrap_or_else(|| " ".repeat(line_num_width));
                    spans.push(Span::styled(num_str, self.style.line_number_style));
                    spans.push(Span::raw(" "));
                }

                // Determine style based on line type and side
                let (prefix, style, bg) = match l.line_type {
                    DiffLineType::Context => (" ", self.style.context_style, Style::default()),
                    DiffLineType::Addition => (
                        "+",
                        self.style.addition_style,
                        Style::default().bg(self.style.addition_bg),
                    ),
                    DiffLineType::Deletion => (
                        "-",
                        self.style.deletion_style,
                        Style::default().bg(self.style.deletion_bg),
                    ),
                    DiffLineType::HunkHeader => {
                        ("@", self.style.hunk_header_style, Style::default())
                    }
                };

                // Apply search highlighting
                let final_style = if is_current_match {
                    self.style.current_match_style
                } else if is_match {
                    self.style.match_style
                } else {
                    style.patch(bg)
                };

                spans.push(Span::styled(prefix.to_string(), final_style));

                // Content with scroll, truncated by display width
                let mut used = 0;
                let content: String = l
                    .content
                    .chars()
                    .skip(self.state.scroll_x)
                    .take_while(|c| {
                        used += c.width().unwrap_or(0);
                        used <= content_width
                    })
                    .collect();
                spans.push(Span::styled(content, final_style));
            }
            None => {
                // Empty half
                if self.state.show_line_numbers {
                    spans.push(Span::raw(" ".repeat(line_num_width + 1)));
                }
                spans.push(Span::raw(" ".repeat(content_width + 1)));
            }
        }

        spans
    }
}

/// Split the widget area into content, status bar and optional search bar.
fn layout_chunks(area: Rect, search_active: bool) -> std::rc::Rc<[Rect]> {
    // Layout: content + status bar + optional search bar
    let constraints = if search_active {
        vec![
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ]
    } else {
        vec![Constraint::Min(1), Constraint::Length(1)]
    };

    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area)
}

/// Inner (inside the border) content area for a given widget area.
fn content_inner_area(area: Rect, search_active: bool) -> Rect {
    Block::default()
        .borders(Borders::ALL)
        .inner(layout_chunks(area, search_active)[0])
}

impl Widget for DiffViewer<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = layout_chunks(area, self.state.search.active);

        // Build title with stats
        let title_text = if let Some(t) = self.title {
            if self.show_stats {
                let additions = self.state.diff.total_additions();
                let deletions = self.state.diff.total_deletions();
                format!(" {} (+{} -{}) ", t, additions, deletions)
            } else {
                format!(" {} ", t)
            }
        } else if self.show_stats {
            let additions = self.state.diff.total_additions();
            let deletions = self.state.diff.total_deletions();
            format!(" +{} -{} ", additions, deletions)
        } else {
            String::new()
        };

        let block = Block::default()
            .title(title_text)
            .borders(Borders::ALL)
            .border_style(self.style.border_style);

        let inner = block.inner(chunks[0]);
        block.render(chunks[0], buf);

        // Content
        let lines = match self.state.view_mode {
            DiffViewMode::Unified => self.build_unified_lines(inner),
            DiffViewMode::SideBySide => self.build_side_by_side_lines(inner),
        };

        let para = Paragraph::new(lines);
        para.render(inner, buf);

        // Scrollbar
        let total_lines = self.state.total_lines();
        if total_lines > inner.height as usize {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state =
                ScrollbarState::new(total_lines).position(self.state.scroll_y);
            scrollbar.render(inner, buf, &mut scrollbar_state);
        }

        // Status bar
        render_diff_status_bar(self.state, &self.style, chunks[1], buf);

        // Search bar
        if self.state.search.active && chunks.len() > 2 {
            render_diff_search_bar(self.state, chunks[2], buf);
        }
    }
}

/// Render the status bar
fn render_diff_status_bar(
    state: &DiffViewerState,
    _style: &DiffViewerStyle,
    area: Rect,
    buf: &mut Buffer,
) {
    let total_lines = state.total_lines();
    let current_line = state.scroll_y + 1;
    let percent = if total_lines > 0 {
        (current_line as f64 / total_lines as f64 * 100.0) as u16
    } else {
        0
    };

    let mode_str = match state.view_mode {
        DiffViewMode::Unified => "Unified",
        DiffViewMode::SideBySide => "Side-by-Side",
    };

    let hunk_info = if let Some(hunk_idx) = state.selected_hunk {
        format!(" | Hunk {}/{}", hunk_idx + 1, state.diff.hunks.len())
    } else {
        String::new()
    };

    let h_scroll_info = if state.scroll_x > 0 {
        format!(" | Col: {}", state.scroll_x + 1)
    } else {
        String::new()
    };

    let search_info = if !state.search.matches.is_empty() {
        format!(
            " | Match {}/{}",
            state.search.current_match + 1,
            state.search.matches.len()
        )
    } else if !state.search.query.is_empty() && state.search.matches.is_empty() {
        " | No matches".to_string()
    } else {
        String::new()
    };

    let status = Line::from(vec![
        Span::styled(" j/k", Style::default().fg(Color::Yellow)),
        Span::raw(": scroll "),
        Span::styled("]/[", Style::default().fg(Color::Yellow)),
        Span::raw(": hunk "),
        Span::styled("n/N", Style::default().fg(Color::Yellow)),
        Span::raw(": change "),
        Span::styled("v", Style::default().fg(Color::Yellow)),
        Span::raw(": mode "),
        Span::styled("/", Style::default().fg(Color::Yellow)),
        Span::raw(": search | "),
        Span::raw(format!(
            "{} | Line {}/{} ({}%){}{}{}",
            mode_str, current_line, total_lines, percent, hunk_info, h_scroll_info, search_info
        )),
    ]);

    let para = Paragraph::new(status).style(Style::default().bg(Color::DarkGray));
    para.render(area, buf);
}

/// Render the search bar
fn render_diff_search_bar(state: &DiffViewerState, area: Rect, buf: &mut Buffer) {
    let search_line = Line::from(vec![
        Span::styled(" Search: ", Style::default().fg(Color::Yellow)),
        Span::raw(state.search.query.clone()),
        Span::styled("▌", Style::default().fg(Color::White)),
    ]);

    let para = Paragraph::new(search_line).style(Style::default().bg(Color::Rgb(40, 40, 60)));
    para.render(area, buf);
}

// ============================================================================
// Event Handlers
// ============================================================================

/// Handle keyboard input for diff viewer
///
/// Returns true if the key was handled
pub fn handle_diff_viewer_key(state: &mut DiffViewerState, key: &KeyEvent) -> bool {
    // Search mode handling
    if state.search.active {
        match key.code {
            KeyCode::Esc => {
                state.cancel_search();
                return true;
            }
            KeyCode::Enter => {
                state.search.active = false;
                return true;
            }
            KeyCode::Backspace => {
                state.search.query.pop();
                state.update_search();
                return true;
            }
            KeyCode::Char(c) => {
                state.search.query.push(c);
                state.update_search();
                return true;
            }
            _ => return false,
        }
    }

    match key.code {
        // Vertical scroll
        KeyCode::Char('j') | KeyCode::Down => {
            state.scroll_down();
            true
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.scroll_up();
            true
        }

        // Horizontal scroll
        KeyCode::Char('h') | KeyCode::Left => {
            state.scroll_left();
            true
        }
        KeyCode::Char('l') | KeyCode::Right => {
            state.scroll_right();
            true
        }

        // Page navigation
        KeyCode::PageDown => {
            state.page_down();
            true
        }
        KeyCode::PageUp => {
            state.page_up();
            true
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.page_down();
            true
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.page_up();
            true
        }

        // Top/Bottom
        KeyCode::Char('g') => {
            state.go_to_top();
            true
        }
        KeyCode::Char('G') => {
            state.go_to_bottom();
            true
        }
        KeyCode::Home => {
            state.go_to_top();
            true
        }
        KeyCode::End => {
            state.go_to_bottom();
            true
        }

        // Hunk navigation
        KeyCode::Char(']') => {
            state.next_hunk();
            true
        }
        KeyCode::Char('[') => {
            state.prev_hunk();
            true
        }

        // Change navigation
        KeyCode::Char('n') => {
            if state.search.matches.is_empty() {
                state.next_change();
            } else {
                state.next_match();
            }
            true
        }
        KeyCode::Char('N') => {
            if state.search.matches.is_empty() {
                state.prev_change();
            } else {
                state.prev_match();
            }
            true
        }

        // View mode toggle
        KeyCode::Char('v') | KeyCode::Char('m') => {
            state.toggle_view_mode();
            true
        }

        // Search
        KeyCode::Char('/') => {
            state.start_search();
            true
        }

        _ => false,
    }
}

/// Handle mouse input for diff viewer
///
/// Returns an action if one was triggered
pub fn handle_diff_viewer_mouse(
    state: &mut DiffViewerState,
    mouse: &MouseEvent,
) -> Option<DiffViewerAction> {
    match mouse.kind {
        MouseEventKind::ScrollDown => {
            state.scroll_down();
            state.scroll_down();
            state.scroll_down();
            None
        }
        MouseEventKind::ScrollUp => {
            state.scroll_up();
            state.scroll_up();
            state.scroll_up();
            None
        }
        _ => None,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_DIFF: &str = r#"--- a/file.txt
+++ b/file.txt
@@ -1,5 +1,6 @@
 context line 1
-removed line
+added line
+another added line
 context line 2
 context line 3
"#;

    fn buf_text(buf: &Buffer) -> String {
        let a = buf.area;
        let mut s = String::new();
        for y in a.y..a.y + a.height {
            for x in a.x..a.x + a.width {
                s.push_str(buf[(x, y)].symbol());
            }
            s.push('\n');
        }
        s
    }

    #[test]
    fn test_parse_keeps_deleted_line_starting_with_dashes() {
        // A deleted SQL comment "-- old comment" looks like a "--- " header
        let d = "--- a/x.sql\n+++ b/x.sql\n@@ -1,2 +1,1 @@\n--- old comment\n select 1;\n";
        let data = DiffData::from_unified_diff(d);
        assert_eq!(data.old_path, Some("x.sql".to_string()));
        assert_eq!(data.total_deletions(), 1);
        assert_eq!(data.hunks[0].lines[0].content, "-- old comment");

        // Same for an added line "++ x"
        let d = "@@ -1,1 +1,2 @@\n+++ added\n same\n";
        let data = DiffData::from_unified_diff(d);
        assert_eq!(data.new_path, None);
        assert_eq!(data.total_additions(), 1);
    }

    #[test]
    fn test_parse_multi_file_headers_after_hunk() {
        let d = "--- a/one\n+++ b/one\n@@ -1,1 +1,1 @@\n-a\n+b\n--- a/two\n+++ b/two\n@@ -1,1 +1,1 @@\n-c\n+d\n";
        let data = DiffData::from_unified_diff(d);
        assert_eq!(data.hunks.len(), 2);
        assert_eq!(data.total_deletions(), 2);
        assert_eq!(data.total_additions(), 2);
        assert_eq!(data.new_path, Some("two".to_string()));
    }

    #[test]
    fn test_go_to_bottom_without_visible_height_stays_in_range() {
        let d = "@@ -1,2 +1,2 @@\n-a\n+b\n c\n";
        let mut st = DiffViewerState::from_unified_diff(d);
        st.go_to_bottom();
        // header + 3 lines
        assert_eq!(st.scroll_y, 3);
    }

    #[test]
    fn test_paging_without_visible_height_moves() {
        let mut st = DiffViewerState::from_unified_diff(SAMPLE_DIFF);
        st.page_down();
        assert_eq!(st.scroll_y, 1);
        st.page_up();
        assert_eq!(st.scroll_y, 0);
    }

    #[test]
    fn test_set_viewport_matches_render() {
        let mut st = DiffViewerState::from_unified_diff(SAMPLE_DIFF);
        // 10 rows: 1 status bar + 2 borders -> 7 content rows
        st.set_viewport(Rect::new(0, 0, 40, 10));
        assert_eq!(st.visible_height, 7);
        assert_eq!(st.visible_width, 38);
    }

    #[test]
    fn test_side_by_side_go_to_bottom_shows_content() {
        let d = "@@ -1,3 +1,3 @@\n-a\n-b\n-c\n+A\n+B\n+C\n";
        let mut st = DiffViewerState::from_unified_diff(d);
        st.set_view_mode(DiffViewMode::SideBySide);
        st.visible_height = 2;
        // Unified has 7 rows, side-by-side only 4 (header + 3 pairs)
        st.go_to_bottom();
        assert_eq!(st.scroll_y, 2);
        let area = Rect::new(0, 0, 40, 5); // border(2)+status(1) => inner height 2
        let mut buf = Buffer::empty(area);
        DiffViewer::new(&st).render(area, &mut buf);
        let t = buf_text(&buf);
        assert!(
            t.contains('C'),
            "side-by-side bottom shows no content:\n{t}"
        );
    }

    #[test]
    fn test_side_by_side_search_and_navigation() {
        let d = "@@ -1,3 +1,3 @@\n-a\n-b\n-c\n+A\n+B\n+needle\n";
        let mut st = DiffViewerState::from_unified_diff(d);
        st.set_view_mode(DiffViewMode::SideBySide);
        st.search.query = "needle".into();
        st.update_search();
        // Row 3 in side-by-side (header, a|A, b|B, c|needle)
        assert_eq!(st.search.matches, vec![3]);

        // Switching modes maps matches and the scroll position
        st.set_view_mode(DiffViewMode::Unified);
        assert_eq!(st.search.matches, vec![6]);
        st.set_view_mode(DiffViewMode::SideBySide);
        assert_eq!(st.search.matches, vec![3]);

        // Change navigation stays within side-by-side rows
        st.go_to_top();
        st.next_change();
        assert_eq!(st.scroll_y, 1);
        st.next_change();
        assert_eq!(st.scroll_y, 2);
        st.next_change();
        assert_eq!(st.scroll_y, 0); // wrapped
    }

    #[test]
    fn test_side_by_side_search_highlight() {
        let d = "@@ -1,1 +1,1 @@\n-old\n+needle\n";
        let mut st = DiffViewerState::from_unified_diff(d);
        st.set_view_mode(DiffViewMode::SideBySide);
        st.search.query = "needle".into();
        st.update_search();
        st.scroll_y = 0;
        let area = Rect::new(0, 0, 40, 6);
        let mut buf = Buffer::empty(area);
        let style = DiffViewerStyle::default();
        DiffViewer::new(&st)
            .style(style.clone())
            .render(area, &mut buf);
        // Row 1 of the content (y = 2) is the matched pair
        let highlighted =
            (0..area.width).any(|x| buf[(x, 2)].bg == style.current_match_style.bg.unwrap());
        assert!(highlighted, "no search highlight in side-by-side");
    }

    #[test]
    fn test_side_by_side_padding_uses_display_width() {
        // The separator column must be the same for a row with wide (CJK)
        // characters as for an ASCII-only row.
        let separator_col = |diff: &str| {
            let mut st = DiffViewerState::from_unified_diff(diff);
            st.set_view_mode(DiffViewMode::SideBySide);
            let area = Rect::new(0, 0, 41, 5);
            let mut buf = Buffer::empty(area);
            DiffViewer::new(&st).render(area, &mut buf);
            // Content row 1 is y = 2; skip the outer border columns
            (1..area.width - 1).find(|&x| buf[(x, 2)].symbol() == "│")
        };
        let ascii = separator_col("@@ -1,1 +1,1 @@\n-ab\n+ab\n");
        let wide = separator_col("@@ -1,1 +1,1 @@\n-你好\n+ab\n");
        assert!(ascii.is_some());
        assert_eq!(wide, ascii);
    }

    #[test]
    fn test_parse_unified_diff_basic() {
        let diff = DiffData::from_unified_diff(SAMPLE_DIFF);

        assert_eq!(diff.old_path, Some("file.txt".to_string()));
        assert_eq!(diff.new_path, Some("file.txt".to_string()));
        assert_eq!(diff.hunks.len(), 1);

        let hunk = &diff.hunks[0];
        assert_eq!(hunk.old_start, 1);
        assert_eq!(hunk.old_count, 5);
        assert_eq!(hunk.new_start, 1);
        assert_eq!(hunk.new_count, 6);
    }

    #[test]
    fn test_parse_unified_diff_lines() {
        let diff = DiffData::from_unified_diff(SAMPLE_DIFF);
        let hunk = &diff.hunks[0];

        // 6 lines: context, deletion, addition, addition, context, context
        assert_eq!(hunk.lines.len(), 6);
        assert_eq!(hunk.lines[0].line_type, DiffLineType::Context);
        assert_eq!(hunk.lines[1].line_type, DiffLineType::Deletion);
        assert_eq!(hunk.lines[2].line_type, DiffLineType::Addition);
        assert_eq!(hunk.lines[3].line_type, DiffLineType::Addition);
        assert_eq!(hunk.lines[4].line_type, DiffLineType::Context);
        assert_eq!(hunk.lines[5].line_type, DiffLineType::Context);
    }

    #[test]
    fn test_parse_unified_diff_line_numbers() {
        let diff = DiffData::from_unified_diff(SAMPLE_DIFF);
        let hunk = &diff.hunks[0];

        // Context line 1
        assert_eq!(hunk.lines[0].old_line_num, Some(1));
        assert_eq!(hunk.lines[0].new_line_num, Some(1));

        // Deletion (removed line)
        assert_eq!(hunk.lines[1].old_line_num, Some(2));
        assert_eq!(hunk.lines[1].new_line_num, None);

        // Addition (added line)
        assert_eq!(hunk.lines[2].old_line_num, None);
        assert_eq!(hunk.lines[2].new_line_num, Some(2));
    }

    #[test]
    fn test_diff_statistics() {
        let diff = DiffData::from_unified_diff(SAMPLE_DIFF);

        assert_eq!(diff.total_additions(), 2);
        assert_eq!(diff.total_deletions(), 1);
    }

    #[test]
    fn test_state_new() {
        let diff = DiffData::from_unified_diff(SAMPLE_DIFF);
        let state = DiffViewerState::new(diff);

        assert_eq!(state.scroll_y, 0);
        assert_eq!(state.scroll_x, 0);
        assert_eq!(state.view_mode, DiffViewMode::Unified);
        assert_eq!(state.selected_hunk, Some(0));
    }

    #[test]
    fn test_state_from_unified_diff() {
        let state = DiffViewerState::from_unified_diff(SAMPLE_DIFF);

        assert!(!state.diff.hunks.is_empty());
        assert_eq!(state.diff.total_additions(), 2);
    }

    #[test]
    fn test_state_scroll() {
        let mut state = DiffViewerState::from_unified_diff(SAMPLE_DIFF);

        assert_eq!(state.scroll_y, 0);
        state.scroll_down();
        assert_eq!(state.scroll_y, 1);
        state.scroll_up();
        assert_eq!(state.scroll_y, 0);
        state.scroll_up(); // Should not go negative
        assert_eq!(state.scroll_y, 0);
    }

    #[test]
    fn test_horizontal_scroll() {
        let mut state = DiffViewerState::from_unified_diff(SAMPLE_DIFF);

        state.scroll_right();
        assert_eq!(state.scroll_x, 4);
        state.scroll_right();
        assert_eq!(state.scroll_x, 8);
        state.scroll_left();
        assert_eq!(state.scroll_x, 4);
        state.scroll_left();
        assert_eq!(state.scroll_x, 0);
        state.scroll_left(); // Should not go negative
        assert_eq!(state.scroll_x, 0);
    }

    #[test]
    fn test_page_navigation() {
        let mut state = DiffViewerState::from_unified_diff(SAMPLE_DIFF);
        state.visible_height = 2;

        state.page_down();
        assert_eq!(state.scroll_y, 2);
        state.page_up();
        assert_eq!(state.scroll_y, 0);
    }

    #[test]
    fn test_go_to_top_bottom() {
        let mut state = DiffViewerState::from_unified_diff(SAMPLE_DIFF);
        state.visible_height = 2;

        state.go_to_bottom();
        assert!(state.scroll_y > 0);

        state.go_to_top();
        assert_eq!(state.scroll_y, 0);
    }

    #[test]
    fn test_view_mode_toggle() {
        let mut state = DiffViewerState::from_unified_diff(SAMPLE_DIFF);

        assert_eq!(state.view_mode, DiffViewMode::Unified);
        state.toggle_view_mode();
        assert_eq!(state.view_mode, DiffViewMode::SideBySide);
        state.toggle_view_mode();
        assert_eq!(state.view_mode, DiffViewMode::Unified);
    }

    #[test]
    fn test_hunk_navigation() {
        let multi_hunk_diff = r#"--- a/file.txt
+++ b/file.txt
@@ -1,3 +1,3 @@
 line 1
-old line 2
+new line 2
 line 3
@@ -10,3 +10,3 @@
 line 10
-old line 11
+new line 11
 line 12
"#;
        let mut state = DiffViewerState::from_unified_diff(multi_hunk_diff);

        assert_eq!(state.selected_hunk, Some(0));
        state.next_hunk();
        assert_eq!(state.selected_hunk, Some(1));
        state.next_hunk(); // Should stay at last
        assert_eq!(state.selected_hunk, Some(1));
        state.prev_hunk();
        assert_eq!(state.selected_hunk, Some(0));
        state.prev_hunk(); // Should stay at first
        assert_eq!(state.selected_hunk, Some(0));
    }

    #[test]
    fn test_search() {
        let mut state = DiffViewerState::from_unified_diff(SAMPLE_DIFF);

        state.start_search();
        state.search.query = "added".to_string();
        state.update_search();

        assert!(!state.search.matches.is_empty());
    }

    #[test]
    fn test_search_next_prev() {
        let mut state = DiffViewerState::from_unified_diff(SAMPLE_DIFF);

        state.search.query = "line".to_string();
        state.update_search();

        let initial_match = state.search.current_match;
        state.next_match();
        assert_ne!(state.search.current_match, initial_match);
        state.prev_match();
        assert_eq!(state.search.current_match, initial_match);
    }

    #[test]
    fn test_empty_state() {
        let state = DiffViewerState::empty();
        assert!(state.diff.hunks.is_empty());
        assert_eq!(state.selected_hunk, None);
    }

    #[test]
    fn test_parse_hunk_header() {
        let result = parse_hunk_header("@@ -1,3 +1,4 @@");
        assert_eq!(result, Some((1, 3, 1, 4)));

        let result = parse_hunk_header("@@ -10 +20,5 @@");
        assert_eq!(result, Some((10, 1, 20, 5)));

        let result = parse_hunk_header("@@ -1,2 +3,4 @@ function name");
        assert_eq!(result, Some((1, 2, 3, 4)));
    }

    #[test]
    fn test_diff_line_constructors() {
        let context = DiffLine::context("test".to_string(), 1, 2);
        assert_eq!(context.line_type, DiffLineType::Context);
        assert_eq!(context.old_line_num, Some(1));
        assert_eq!(context.new_line_num, Some(2));

        let addition = DiffLine::addition("new".to_string(), 5);
        assert_eq!(addition.line_type, DiffLineType::Addition);
        assert_eq!(addition.new_line_num, Some(5));
        assert_eq!(addition.old_line_num, None);

        let deletion = DiffLine::deletion("old".to_string(), 3);
        assert_eq!(deletion.line_type, DiffLineType::Deletion);
        assert_eq!(deletion.old_line_num, Some(3));
        assert_eq!(deletion.new_line_num, None);
    }

    #[test]
    fn test_diff_hunk_counts() {
        let mut hunk = DiffHunk::new("@@ -1,3 +1,4 @@".to_string(), 1, 3, 1, 4);
        hunk.add_line(DiffLine::context("ctx".to_string(), 1, 1));
        hunk.add_line(DiffLine::deletion("del".to_string(), 2));
        hunk.add_line(DiffLine::addition("add1".to_string(), 2));
        hunk.add_line(DiffLine::addition("add2".to_string(), 3));

        assert_eq!(hunk.addition_count(), 2);
        assert_eq!(hunk.deletion_count(), 1);
    }

    #[test]
    fn test_style_default() {
        let style = DiffViewerStyle::default();
        assert_eq!(style.gutter_separator, "│");
        assert_eq!(style.side_separator, "│");
    }

    #[test]
    fn test_key_handler_scroll() {
        let mut state = DiffViewerState::from_unified_diff(SAMPLE_DIFF);

        let key_j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(handle_diff_viewer_key(&mut state, &key_j));
        assert_eq!(state.scroll_y, 1);

        let key_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert!(handle_diff_viewer_key(&mut state, &key_k));
        assert_eq!(state.scroll_y, 0);
    }

    #[test]
    fn test_key_handler_view_mode() {
        let mut state = DiffViewerState::from_unified_diff(SAMPLE_DIFF);

        let key_v = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE);
        assert!(handle_diff_viewer_key(&mut state, &key_v));
        assert_eq!(state.view_mode, DiffViewMode::SideBySide);
    }

    #[test]
    fn test_key_handler_search() {
        let mut state = DiffViewerState::from_unified_diff(SAMPLE_DIFF);

        let key_slash = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        assert!(handle_diff_viewer_key(&mut state, &key_slash));
        assert!(state.search.active);

        let key_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(handle_diff_viewer_key(&mut state, &key_a));
        assert_eq!(state.search.query, "a");

        let key_esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(handle_diff_viewer_key(&mut state, &key_esc));
        assert!(!state.search.active);
    }

    #[test]
    fn test_render_does_not_panic() {
        let state = DiffViewerState::from_unified_diff(SAMPLE_DIFF);
        let viewer = DiffViewer::new(&state).title("Test Diff");

        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 20));
        viewer.render(Rect::new(0, 0, 80, 20), &mut buf);
    }

    #[test]
    fn test_render_side_by_side_does_not_panic() {
        let mut state = DiffViewerState::from_unified_diff(SAMPLE_DIFF);
        state.view_mode = DiffViewMode::SideBySide;
        let viewer = DiffViewer::new(&state);

        let mut buf = Buffer::empty(Rect::new(0, 0, 120, 20));
        viewer.render(Rect::new(0, 0, 120, 20), &mut buf);
    }
}
