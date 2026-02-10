//! The `widgets` module provides reusable UI components.

use crate::{Frame, Rect};

pub mod text;
pub use text::Text;

/// The core trait for all UI components.
pub trait Widget {
    /// Draws the widget into the given area of the frame.
    fn render(self, area: Rect, frame: &mut Frame);
}
