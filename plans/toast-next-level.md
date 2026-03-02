# Plan: Next-level Toasts (stacked + close via keyboard/mouse)

## Goal
Enhance the existing `Toast` component so the library can display **multiple toasts simultaneously** (stacked) and support **persistent/dismissable toasts** that require explicit closing via **keyboard** and/or **mouse click**.

This plan is written for the current codebase at `src/components/toast.rs` where `ToastState` currently supports **one** message with an expiry timestamp.

## Non-goals (for first iteration)
- Animations (slide/fade)
- Rich content (buttons, links)
- Toast priority / reordering rules beyond FIFO/LIFO
- Cross-terminal mouse protocol setup (assumed handled by consumer / app layer)

## Codebase reconnaissance (what exists today)
- `src/components/toast.rs`
  - `ToastState { message: Option<String>, expires_at: Option<i64> }`
  - `ToastState::show(message, duration_ms)` replaces existing toast.
  - `Toast` widget renders a centered popup with `render_with_clear`.
- `src/components/mod.rs` re-exports `Toast`, `ToastState`, `ToastStyle`.
- Event handling infrastructure exists elsewhere (e.g. `src/events`) but toast currently has **no** event integration.

Implication: We need a new state model (queue/stack of toasts) and a rendering strategy for multiple toasts. Dismiss interactions require either:
1) library-level event handler utilities (optional), or
2) exposing hitboxes/ids so the app can route events.

## Proposed design (API + behavior)
### 1) New data model
Introduce a `ToastItem` and a multi-toast state container.

**Types**
- `ToastId` (u64 or usize) unique id for referencing/dismissing
- `ToastItem` fields:
  - `id: ToastId`
  - `message: String`
  - `style: ToastStyle` or `{ auto_style: bool, explicit_style: Option<ToastStyle> }`
  - `created_at_ms: i64`
  - `expires_at_ms: Option<i64>` (None means persistent)
  - `dismiss_policy: ToastDismissPolicy` (see below)

**Dismiss policy**
- `enum ToastDismissPolicy {
    Auto { duration_ms: i64 },
    Manual, // requires explicit close
    ManualOrTimeout { duration_ms: i64 },
  }`

### 2) Multi-toast state
Add a new state type; keep old `ToastState` for backwards compatibility or deprecate.

Option A (recommended):
- Add `ToastStackState` (or `ToastsState`) alongside existing `ToastState`.
- Keep `ToastState` working (single toast) as a thin wrapper over `ToastStackState` with capacity=1.

`ToastStackState` responsibilities:
- store `VecDeque<ToastItem>`
- generate ids
- provide:
  - `push_auto(message, duration_ms) -> ToastId`
  - `push_manual(message) -> ToastId`
  - `push(item) -> ToastId`
  - `dismiss(id) -> bool`
  - `dismiss_top() -> bool` (for keyboard close)
  - `clear_expired(now)`
  - `items()` iterator for rendering
  - `len()`, `is_empty()`
  - `set_capacity(n)` and eviction policy (drop oldest, drop newest, etc.)

### 3) Rendering multiple toasts
Create a renderer that can compute a stack of rectangles and draw them.

Add:
- `ToastPlacement`:
  - top-right, top-center, top-left, bottom-right, bottom-center, bottom-left
  - default: top-center (matches current centered look)
- `ToastStackLayout` config:
  - `placement: ToastPlacement`
  - `margin: (u16, u16)`
  - `gap_y: u16` (vertical spacing)
  - `max_width`, `max_height` per toast
  - `max_toasts_visible: usize` (render cap)

Implementation strategy:
- Reuse `Toast::calculate_area()` logic but allow passing a target anchor point.
- Compute each toast’s height based on wrapping, then stack along Y direction.
- For top placements, stack downward; for bottom placements, stack upward.
- Render order:
  - Decide z-order (newest on top vs newest at bottom). Recommended: newest on top for top placement; newest on bottom for bottom placement.

New widget:
- `ToastStack<'a>` that takes `&'a ToastStackState` and a layout config.
- It renders all visible toasts with `Clear` behind each.

### 4) Dismiss by keyboard
Library should expose easy dismissal for “close toast” keybindings.

Add methods:
- `ToastStackState::dismiss_top()`
- `ToastStackState::dismiss_all()`

App integration is typically:
- on key press (Esc / Ctrl+w / etc) call `dismiss_top()`.

### 5) Dismiss by mouse click
In terminal UI, mouse events provide a cell coordinate (x,y). We need hit-testing.

Add hit-test support:
- When rendering, compute toast rectangles and associate them with `ToastId`.
- Expose a pure function:
  - `ToastStack::layout(area, state) -> Vec<(ToastId, Rect)>`
  - or `ToastStackState::hit_test(area, config, x, y) -> Option<ToastId>`

Recommended approach:
- Put layout computation in `ToastStackLayoutEngine` (module-private) used by both render and hit-test.
- Provide public helper:
  - `ToastStackLayout::hit_test(area, state, pos) -> Option<ToastId>`

App usage:
- On `MouseEvent::Down(MouseButton::Left, x, y, ..)`:
  - if `Some(id)` => `state.dismiss(id)`.

### 6) Backwards compatibility & migration
- Keep `Toast` widget unchanged (still renders one toast).
- Keep `ToastState` but consider adding:
  - `ToastState` deprecated note in docs OR keep as single-toast convenience.
- Add new exports in `src/components/mod.rs`:
  - `ToastId`, `ToastItem`, `ToastStack`, `ToastStackState`, `ToastPlacement`, `ToastStackLayout`.

### 7) Examples
Add a new example under `examples/`:
- `examples/toast_stack.rs`
  - periodically push auto toasts
  - create manual toasts on keypress
  - close top toast via a key (e.g. `Esc`)
  - enable mouse and close by clicking a toast

If the project already has an app skeleton in examples, integrate there.

### 8) Tests
Add unit tests in `src/components/toast_stack.rs` (or in `toast.rs` if kept together):
- state:
  - `push_auto` adds items
  - `clear_expired` removes only those expired
  - `manual` items never expire
  - `dismiss(id)` removes correct one
  - `capacity` evicts as expected
- layout:
  - produces non-overlapping rects
  - respects placement, gap, margins
  - hit-test returns correct id

### 9) Implementation steps (detailed execution sequence)
1. **Refactor time source**
   - Extract `current_time_ms()` into a shared helper (module-private) so both old/new state can use it.

2. **Introduce new module/file**
   - Create `src/components/toast_stack.rs` (or extend `toast.rs` if you prefer single file).
   - Define `ToastId`, `ToastDismissPolicy`, `ToastItem`, `ToastStackState`.

3. **Implement state management**
   - ID generator: `next_id: u64` incrementing.
   - `push_*` methods set `expires_at_ms` appropriately.
   - `clear_expired()` uses `now` (passed in or computed internally).
   - Add capacity support with a default (e.g. 5) and eviction policy.

4. **Implement layout engine**
   - Create a function `compute_toast_rects(area, items, layout) -> Vec<(ToastId, Rect)>`.
   - For each toast, compute width/height similarly to existing `Toast::calculate_area`.
   - Place according to `ToastPlacement`.
   - Apply stacking and stop at `max_toasts_visible`.

5. **Implement rendering widget**
   - `struct ToastStack<'a> { state: &'a ToastStackState, layout: ToastStackLayout, ... }`
   - Render: iterate rects and render each toast with `Toast::new(&item.message)` + style configuration.
   - Ensure `Clear` is rendered under each.

6. **Mouse hit-test API**
   - Provide `ToastStack::hit_test(area, x, y) -> Option<ToastId>` or equivalent.
   - Ensure it shares the same rect computation as renderer.

7. **Keyboard dismissal helpers**
   - Add `dismiss_top` and docs recommending typical keybindings.

8. **Re-exports**
   - Update `src/components/mod.rs` and `src/lib.rs` exports.

9. **Docs**
   - Add rustdoc examples for:
     - stacked auto toasts
     - manual toasts + dismiss
     - mouse click dismissal via hit-test

10. **Examples**
   - Add `examples/toast_stack.rs` demonstrating stacking + manual close.

11. **Finalize tests**
   - Add focused tests for layout and state.
   - Run `cargo test`.

## Acceptance criteria
- Can display N toasts simultaneously, stacked with configurable placement.
- Auto toasts expire independently.
- Manual toasts remain until dismissed.
- API supports closing top toast via keyboard.
- API supports hit-testing and dismissing by mouse click.
- Existing single-toast API continues to work (or is clearly deprecated with migration path).

## Risks / open questions
- Mouse event types depend on the consumer’s event backend (crossterm/termion). We should keep library API backend-agnostic (take x/y coordinates only).
- Exact stacking order preferences may vary; provide a simple `order: NewestFirst|OldestFirst` option.
- Terminal size constraints: when toasts exceed available height, we should truncate the rendered list (`max_toasts_visible`) and/or skip those that don’t fit.
