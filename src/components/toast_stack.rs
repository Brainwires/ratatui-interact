use std::collections::VecDeque;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Clear, Widget},
};

use super::toast::{Toast, ToastStyle};

/// Identifier for a toast in a stack.
pub type ToastId = u64;

/// How/when a toast should be dismissed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastDismissPolicy {
    /// Automatically dismiss after `duration_ms`.
    Auto { duration_ms: i64 },
    /// Persist until explicitly dismissed.
    Manual,
    /// Persist until explicitly dismissed, but also auto-dismiss after `duration_ms`.
    ManualOrTimeout { duration_ms: i64 },
}

impl ToastDismissPolicy {
    fn expires_at_ms(&self, now_ms: i64) -> Option<i64> {
        match *self {
            ToastDismissPolicy::Auto { duration_ms } => Some(now_ms + duration_ms),
            ToastDismissPolicy::Manual => None,
            ToastDismissPolicy::ManualOrTimeout { duration_ms } => Some(now_ms + duration_ms),
        }
    }
}

/// A single toast entry within a stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastItem {
    pub id: ToastId,
    pub message: String,
    pub style: ToastStyle,
    pub auto_style: bool,
    pub created_at_ms: i64,
    pub expires_at_ms: Option<i64>,
    pub dismiss_policy: ToastDismissPolicy,
}

/// Placement of a toast stack within a render area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPlacement {
    #[default]
    TopCenter,
    TopLeft,
    TopRight,
    BottomCenter,
    BottomLeft,
    BottomRight,
}

/// Order used when choosing which toasts are visible first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastOrder {
    /// Newest toasts are shown first.
    #[default]
    NewestFirst,
    /// Oldest toasts are shown first.
    OldestFirst,
}

/// Layout configuration for rendering a stack of toasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToastStackLayout {
    pub placement: ToastPlacement,
    /// Outer margin in cells: `(x, y)`.
    pub margin: (u16, u16),
    /// Vertical gap in cells between stacked toasts.
    pub gap_y: u16,
    pub max_width: u16,
    pub max_height: u16,
    pub max_toasts_visible: usize,
    pub order: ToastOrder,
    /// Default offset from the edge (top/bottom) used for placement.
    /// This mirrors the old single-toast behavior where the toast is slightly below the top.
    pub edge_offset_y: u16,
}

impl Default for ToastStackLayout {
    fn default() -> Self {
        Self {
            placement: ToastPlacement::TopCenter,
            margin: (1, 1),
            gap_y: 1,
            max_width: 80,
            max_height: 8,
            max_toasts_visible: 5,
            order: ToastOrder::NewestFirst,
            edge_offset_y: 3,
        }
    }
}

/// State container for multiple stacked toasts.
#[derive(Debug, Clone)]
pub struct ToastStackState {
    items: VecDeque<ToastItem>,
    next_id: ToastId,
    capacity: usize,
}

impl Default for ToastStackState {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastStackState {
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
            next_id: 1,
            capacity: 5,
        }
    }

    /// Set maximum number of toasts retained in the stack.
    ///
    /// When pushing beyond capacity, the oldest toasts are evicted.
    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity.max(1);
        while self.items.len() > self.capacity {
            self.items.pop_front();
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> impl Iterator<Item = &ToastItem> {
        self.items.iter()
    }

    pub fn items_mut(&mut self) -> impl Iterator<Item = &mut ToastItem> {
        self.items.iter_mut()
    }

    pub fn push_auto(&mut self, message: impl Into<String>, duration_ms: i64) -> ToastId {
        self.push_with_policy(message, ToastDismissPolicy::Auto { duration_ms })
    }

    pub fn push_manual(&mut self, message: impl Into<String>) -> ToastId {
        self.push_with_policy(message, ToastDismissPolicy::Manual)
    }

    pub fn push_manual_or_timeout(
        &mut self,
        message: impl Into<String>,
        duration_ms: i64,
    ) -> ToastId {
        self.push_with_policy(message, ToastDismissPolicy::ManualOrTimeout { duration_ms })
    }

    pub fn push_with_policy(
        &mut self,
        message: impl Into<String>,
        dismiss_policy: ToastDismissPolicy,
    ) -> ToastId {
        let now = current_time_ms();
        let msg = message.into();
        let id = self.alloc_id();

        let item = ToastItem {
            id,
            style: ToastStyle::Info,
            auto_style: true,
            created_at_ms: now,
            expires_at_ms: dismiss_policy.expires_at_ms(now),
            dismiss_policy,
            message: msg,
        };

        self.items.push_back(item);
        self.enforce_capacity();
        id
    }

    /// Push a pre-built item. If `item.id == 0`, an id will be allocated.
    pub fn push(&mut self, mut item: ToastItem) -> ToastId {
        let now = current_time_ms();
        if item.id == 0 {
            item.id = self.alloc_id();
        }
        if item.created_at_ms == 0 {
            item.created_at_ms = now;
        }
        if item.expires_at_ms.is_none() {
            item.expires_at_ms = item.dismiss_policy.expires_at_ms(item.created_at_ms);
        }

        let id = item.id;
        self.items.push_back(item);
        self.enforce_capacity();
        id
    }

    pub fn dismiss(&mut self, id: ToastId) -> bool {
        if let Some(pos) = self.items.iter().position(|t| t.id == id) {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }

    /// Dismiss the most recently added toast.
    pub fn dismiss_top(&mut self) -> bool {
        self.items.pop_back().is_some()
    }

    pub fn dismiss_all(&mut self) {
        self.items.clear();
    }

    /// Remove expired toasts.
    pub fn clear_expired(&mut self) {
        let now = current_time_ms();
        self.clear_expired_at(now);
    }

    pub fn clear_expired_at(&mut self, now_ms: i64) {
        self.items
            .retain(|t| t.expires_at_ms.map(|e| now_ms < e).unwrap_or(true));
    }

    fn alloc_id(&mut self) -> ToastId {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    fn enforce_capacity(&mut self) {
        while self.items.len() > self.capacity {
            self.items.pop_front();
        }
    }
}

/// A stacked toast renderer. Also provides hit-testing based on the same layout.
#[derive(Debug, Clone)]
pub struct ToastStack<'a> {
    pub state: &'a ToastStackState,
    pub layout: ToastStackLayout,
}

impl<'a> ToastStack<'a> {
    pub fn new(state: &'a ToastStackState) -> Self {
        Self {
            state,
            layout: ToastStackLayout::default(),
        }
    }

    pub fn layout(mut self, layout: ToastStackLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Compute rectangles for visible toasts using the current layout.
    pub fn compute_rects(&self, area: Rect) -> Vec<(ToastId, Rect)> {
        compute_toast_rects(area, self.state, self.layout)
    }

    /// Hit-test a cell position within the given area.
    pub fn hit_test(&self, area: Rect, x: u16, y: u16) -> Option<ToastId> {
        self.compute_rects(area)
            .into_iter()
            .find(|(_, r)| {
                x >= r.x
                    && x < r.x.saturating_add(r.width)
                    && y >= r.y
                    && y < r.y.saturating_add(r.height)
            })
            .map(|(id, _)| id)
    }

    pub fn render_with_clear(self, area: Rect, buf: &mut Buffer) {
        for (_, rect) in self.compute_rects(area) {
            Clear.render(rect, buf);
        }
        self.render(area, buf);
    }
}

impl Widget for ToastStack<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rects = self.compute_rects(area);
        for (id, rect) in rects {
            if let Some(item) = self.state.items.iter().find(|t| t.id == id) {
                let mut toast = Toast::new(&item.message)
                    .max_width(self.layout.max_width)
                    .max_height(self.layout.max_height)
                    .top_offset(0);

                toast = if item.auto_style {
                    toast.auto_style()
                } else {
                    toast.style(item.style)
                };

                toast.render(rect, buf);
            }
        }
    }
}

fn compute_toast_rects(
    area: Rect,
    state: &ToastStackState,
    layout: ToastStackLayout,
) -> Vec<(ToastId, Rect)> {
    if area.width == 0 || area.height == 0 {
        return vec![];
    }

    // Choose visible items order.
    let mut ids: Vec<ToastId> = match layout.order {
        ToastOrder::NewestFirst => state.items.iter().rev().map(|t| t.id).collect(),
        ToastOrder::OldestFirst => state.items.iter().map(|t| t.id).collect(),
    };
    if ids.len() > layout.max_toasts_visible {
        ids.truncate(layout.max_toasts_visible);
    }

    // We need sizes to layout; compute from message length similarly to Toast::calculate_area,
    // but without anchoring to center/top_offset.
    let max_content_width = (area.width as usize)
        .saturating_sub(8)
        .min(layout.max_width as usize);

    let mut sizes: Vec<(ToastId, u16, u16)> = Vec::with_capacity(ids.len());
    for id in &ids {
        let item = match state.items.iter().find(|t| t.id == *id) {
            Some(it) => it,
            None => continue,
        };

        let content_width = item.message.len() + 4;
        let toast_width = (content_width.min(max_content_width).max(20)) as u16;

        let inner_width = toast_width.saturating_sub(2) as usize;
        let lines_needed = (item.message.len() + inner_width - 1) / inner_width.max(1);
        let toast_height = (lines_needed as u16 + 2).min(layout.max_height);

        sizes.push((*id, toast_width, toast_height));
    }

    let (mx, my) = layout.margin;

    // Starting y depends on placement.
    let is_top = matches!(
        layout.placement,
        ToastPlacement::TopCenter | ToastPlacement::TopLeft | ToastPlacement::TopRight
    );

    let mut rects: Vec<(ToastId, Rect)> = Vec::with_capacity(sizes.len());

    if is_top {
        let mut y = area
            .y
            .saturating_add(my)
            .saturating_add(layout.edge_offset_y);
        for (id, w, h) in sizes {
            if y.saturating_add(h) > area.y.saturating_add(area.height) {
                break;
            }
            let x = compute_x(area, w, mx, layout.placement);
            rects.push((id, Rect::new(x, y, w, h)));
            y = y.saturating_add(h).saturating_add(layout.gap_y);
        }
    } else {
        let mut y_bottom = area.y.saturating_add(area.height).saturating_sub(my);
        for (id, w, h) in sizes {
            if y_bottom < area.y.saturating_add(h) {
                break;
            }
            let y = y_bottom.saturating_sub(h);
            let x = compute_x(area, w, mx, layout.placement);
            rects.push((id, Rect::new(x, y, w, h)));
            y_bottom = y.saturating_sub(layout.gap_y);
        }
    }

    rects
}

fn compute_x(area: Rect, toast_width: u16, margin_x: u16, placement: ToastPlacement) -> u16 {
    match placement {
        ToastPlacement::TopLeft | ToastPlacement::BottomLeft => area.x.saturating_add(margin_x),
        ToastPlacement::TopRight | ToastPlacement::BottomRight => area
            .x
            .saturating_add(area.width)
            .saturating_sub(margin_x)
            .saturating_sub(toast_width),
        ToastPlacement::TopCenter | ToastPlacement::BottomCenter => {
            area.x + (area.width.saturating_sub(toast_width)) / 2
        }
    }
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_push_and_dismiss() {
        let mut s = ToastStackState::new();
        s.set_capacity(10);

        let a = s.push_manual("A");
        let b = s.push_auto("B", 100_000);
        assert_eq!(s.len(), 2);

        assert!(s.dismiss(a));
        assert_eq!(s.len(), 1);
        assert!(!s.dismiss(a));

        assert!(s.dismiss(b));
        assert!(s.is_empty());
    }

    #[test]
    fn state_clear_expired_at() {
        let mut s = ToastStackState::new();
        s.set_capacity(10);

        let now = 1_000;
        let id = s.push_with_policy("Auto", ToastDismissPolicy::Auto { duration_ms: 10 });
        // Patch created/expires to be deterministic for test
        if let Some(item) = s.items_mut().find(|t| t.id == id) {
            item.created_at_ms = now;
            item.expires_at_ms = Some(now + 10);
        }

        s.clear_expired_at(now + 9);
        assert_eq!(s.len(), 1);
        s.clear_expired_at(now + 10);
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn layout_hit_test() {
        let mut s = ToastStackState::new();
        s.set_capacity(10);
        let id = s.push_manual("Hello world");

        let stack = ToastStack::new(&s);
        let area = Rect::new(0, 0, 100, 40);
        let rects = stack.compute_rects(area);
        assert_eq!(rects.len(), 1);
        let r = rects[0].1;
        assert_eq!(rects[0].0, id);

        assert_eq!(stack.hit_test(area, r.x, r.y), Some(id));
        assert_eq!(
            stack.hit_test(area, r.x + r.width - 1, r.y + r.height - 1),
            Some(id)
        );
        assert_eq!(stack.hit_test(area, r.x.saturating_sub(1), r.y), None);
    }
}
