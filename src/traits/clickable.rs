//! Clickable trait for mouse interaction
//!
//! Provides click region management for components that respond to mouse clicks.
//! Click regions are registered during rendering and checked during event handling.
//!
//! # Example
//!
//! ```rust
//! use ratatui_interact::traits::{ClickRegion, ClickRegionRegistry};
//! use ratatui::layout::Rect;
//!
//! // Create a registry for tracking click regions
//! let mut registry: ClickRegionRegistry<&str> = ClickRegionRegistry::new();
//!
//! // Register click regions during render
//! registry.register(Rect::new(0, 0, 10, 1), "button1");
//! registry.register(Rect::new(15, 0, 10, 1), "button2");
//!
//! // Check for clicks during event handling
//! if let Some(action) = registry.handle_click(5, 0) {
//!     assert_eq!(*action, "button1");
//! }
//! ```

use ratatui::layout::Rect;

/// A registered click region that responds to mouse clicks.
///
/// Associates a rectangular area with user-defined data that is returned
/// when a click occurs within the region.
#[derive(Debug, Clone)]
pub struct ClickRegion<T: Clone> {
    /// The area that responds to clicks.
    pub area: Rect,
    /// User-defined data associated with this region.
    pub data: T,
}

impl<T: Clone> ClickRegion<T> {
    /// Create a new click region.
    ///
    /// # Arguments
    ///
    /// * `area` - The rectangular area that responds to clicks
    /// * `data` - Data to return when this region is clicked
    pub fn new(area: Rect, data: T) -> Self {
        Self { area, data }
    }

    /// Check if a point is within this region.
    ///
    /// # Arguments
    ///
    /// * `col` - The column (x) position
    /// * `row` - The row (y) position
    ///
    /// # Returns
    ///
    /// `true` if the point is within the region's bounds.
    pub fn contains(&self, col: u16, row: u16) -> bool {
        col >= self.area.x
            && col < self.area.x + self.area.width
            && row >= self.area.y
            && row < self.area.y + self.area.height
    }
}

/// Trait for components that respond to mouse clicks.
///
/// Implement this trait to make a component clickable with automatic
/// hit-testing based on registered click regions.
pub trait Clickable {
    /// The type of action that a click produces.
    type ClickAction: Clone;

    /// Returns all click regions for this component.
    ///
    /// Called after rendering to get the active regions.
    fn click_regions(&self) -> &[ClickRegion<Self::ClickAction>];

    /// Handle a click at the given position.
    ///
    /// Returns `Some(action)` if the click was within a region,
    /// `None` otherwise.
    ///
    /// Default implementation checks all regions and returns the first match.
    fn handle_click(&self, col: u16, row: u16) -> Option<Self::ClickAction> {
        self.click_regions()
            .iter()
            .find(|r| r.contains(col, row))
            .map(|r| r.data.clone())
    }
}

/// Registry for managing click regions during render.
///
/// Use this to track clickable areas that are populated during rendering
/// and checked during event handling.
///
/// # Example
///
/// ```rust
/// use ratatui_interact::traits::ClickRegionRegistry;
/// use ratatui::layout::Rect;
///
/// #[derive(Clone, PartialEq, Debug)]
/// enum ButtonId { Save, Cancel }
///
/// let mut registry: ClickRegionRegistry<ButtonId> = ClickRegionRegistry::new();
///
/// // Clear before each render
/// registry.clear();
///
/// // Register regions during render
/// registry.register(Rect::new(0, 0, 8, 1), ButtonId::Save);
/// registry.register(Rect::new(10, 0, 8, 1), ButtonId::Cancel);
///
/// // Check clicks during event handling
/// assert_eq!(registry.handle_click(4, 0), Some(&ButtonId::Save));
/// assert_eq!(registry.handle_click(14, 0), Some(&ButtonId::Cancel));
/// assert_eq!(registry.handle_click(9, 0), None); // Gap between buttons
/// ```
#[derive(Debug, Clone)]
pub struct ClickRegionRegistry<T: Clone> {
    regions: Vec<ClickRegion<T>>,
}

impl<T: Clone> Default for ClickRegionRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> ClickRegionRegistry<T> {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    /// Create a new registry with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            regions: Vec::with_capacity(capacity),
        }
    }

    /// Clear all registered regions.
    ///
    /// Call this at the start of each render to reset the regions.
    pub fn clear(&mut self) {
        self.regions.clear();
    }

    /// Register a new click region.
    ///
    /// # Arguments
    ///
    /// * `area` - The rectangular area that responds to clicks
    /// * `data` - Data to return when this region is clicked
    pub fn register(&mut self, area: Rect, data: T) {
        self.regions.push(ClickRegion::new(area, data));
    }

    /// Handle a click at the given position.
    ///
    /// Returns a reference to the data if the click was within a region,
    /// `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `col` - The column (x) position
    /// * `row` - The row (y) position
    pub fn handle_click(&self, col: u16, row: u16) -> Option<&T> {
        self.regions
            .iter()
            .find(|r| r.contains(col, row))
            .map(|r| &r.data)
    }

    /// Get all registered regions.
    pub fn regions(&self) -> &[ClickRegion<T>] {
        &self.regions
    }

    /// Check if any regions are registered.
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    /// Get the number of registered regions.
    pub fn len(&self) -> usize {
        self.regions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_click_region_contains() {
        let region = ClickRegion::new(Rect::new(10, 5, 20, 3), "test");

        // Inside
        assert!(region.contains(10, 5)); // Top-left corner
        assert!(region.contains(29, 7)); // Bottom-right corner (exclusive bounds)
        assert!(region.contains(20, 6)); // Middle

        // Outside
        assert!(!region.contains(9, 5)); // Left of region
        assert!(!region.contains(30, 5)); // Right of region
        assert!(!region.contains(10, 4)); // Above region
        assert!(!region.contains(10, 8)); // Below region
    }

    #[test]
    fn test_click_region_zero_size() {
        let region = ClickRegion::new(Rect::new(5, 5, 0, 0), "test");
        assert!(!region.contains(5, 5));
    }

    #[test]
    fn test_registry_basic_operations() {
        let mut registry: ClickRegionRegistry<&str> = ClickRegionRegistry::new();

        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register(Rect::new(0, 0, 10, 1), "first");
        registry.register(Rect::new(15, 0, 10, 1), "second");

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 2);

        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_handle_click() {
        let mut registry: ClickRegionRegistry<i32> = ClickRegionRegistry::new();

        registry.register(Rect::new(0, 0, 10, 1), 1);
        registry.register(Rect::new(15, 0, 10, 1), 2);
        registry.register(Rect::new(0, 2, 25, 2), 3);

        // Click on first region
        assert_eq!(registry.handle_click(5, 0), Some(&1));

        // Click on second region
        assert_eq!(registry.handle_click(20, 0), Some(&2));

        // Click on third region
        assert_eq!(registry.handle_click(12, 3), Some(&3));

        // Click in gap
        assert_eq!(registry.handle_click(12, 0), None);

        // Click outside all regions
        assert_eq!(registry.handle_click(100, 100), None);
    }

    #[test]
    fn test_registry_overlapping_regions() {
        let mut registry: ClickRegionRegistry<&str> = ClickRegionRegistry::new();

        // Overlapping regions - first registered wins
        registry.register(Rect::new(0, 0, 20, 2), "back");
        registry.register(Rect::new(5, 0, 10, 1), "front");

        // Click on overlapping area returns first registered
        assert_eq!(registry.handle_click(7, 0), Some(&"back"));

        // Click on non-overlapping part of back region
        assert_eq!(registry.handle_click(2, 1), Some(&"back"));
    }

    #[test]
    fn test_clickable_trait() {
        #[derive(Clone, PartialEq, Debug)]
        enum Action {
            Click,
        }

        struct ClickableWidget {
            regions: Vec<ClickRegion<Action>>,
        }

        impl Clickable for ClickableWidget {
            type ClickAction = Action;

            fn click_regions(&self) -> &[ClickRegion<Self::ClickAction>] {
                &self.regions
            }
        }

        let widget = ClickableWidget {
            regions: vec![ClickRegion::new(Rect::new(0, 0, 10, 1), Action::Click)],
        };

        assert_eq!(widget.handle_click(5, 0), Some(Action::Click));
        assert_eq!(widget.handle_click(15, 0), None);
    }
}
