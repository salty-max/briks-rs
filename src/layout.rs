//! The `layout` module provides tools for dividing terminal space.
//!
//! The core type is [`Rect`], which represents a rectangular area on the screen.
//! The [`Layout`] engine can split a [`Rect`] into multiple sub-rectangles based on [`Constraint`]s.

/// The direction in which a rectangle is split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Split horizontally (side-by-side).
    Horizontal,
    /// Split vertically (top-to-bottom).
    Vertical,
}

/// Constraints used to define the size of a layout segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constraint {
    /// A fixed percentage of the available space (0-100).
    Percentage(u16),
    /// A fixed number of cells.
    Length(u16),
}

/// A rectangular area on the screen.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// The horizontal coordinate of the top-left corner.
    pub x: u16,
    /// The vertical coordinate of the top-left corner.
    pub y: u16,
    /// The width of the rectangle in columns.
    pub width: u16,
    /// The height of the rectangle in rows.
    pub height: u16,
}

impl Rect {
    /// Creates a new rectangle.
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the total number of cells in the rectangle.
    pub fn area(&self) -> u16 {
        self.width * self.height
    }

    /// Returns the x-coordinate of the left edge.
    pub fn left(&self) -> u16 {
        self.x
    }

    /// Returns the x-coordinate of the right edge.
    pub fn right(&self) -> u16 {
        self.x + self.width
    }

    /// Returns the y-coordinate of the top edge.
    pub fn top(&self) -> u16 {
        self.y
    }

    /// Returns the y-coordinate of the bottom edge.
    pub fn bottom(&self) -> u16 {
        self.y + self.height
    }
}

/// A layout engine that divides a rectangle into sub-rectangles based on constraints.
pub struct Layout {
    /// The direction of the split.
    pub direction: Direction,
    /// The constraints for each segment.
    pub constraints: Vec<Constraint>,
}

impl Layout {
    /// Creates a new layout.
    pub fn new(direction: Direction, constraints: Vec<Constraint>) -> Self {
        Self {
            direction,
            constraints,
        }
    }

    /// Splits the given rectangle into sub-rectangles.
    ///
    /// The number of returned rectangles matches the number of constraints.
    pub fn split(&self, rect: Rect) -> Vec<Rect> {
        let mut rects = Vec::new();
        let total_primary = match &self.direction {
            Direction::Horizontal => rect.width,
            Direction::Vertical => rect.height,
        };

        let start_x = rect.x;
        let start_y = rect.y;
        let mut offset = 0;

        for c in &self.constraints {
            let size = match c {
                Constraint::Length(l) => *l,
                Constraint::Percentage(p) => (p * total_primary) / 100,
            };

            let sub_rect = match &self.direction {
                Direction::Horizontal => Rect::new(start_x + offset, start_y, size, rect.height),
                Direction::Vertical => Rect::new(start_x, start_y + offset, rect.width, size),
            };

            rects.push(sub_rect);
            offset += size;
        }

        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_calculations() {
        let rect = Rect::new(10, 10, 20, 5);
        assert_eq!(rect.area(), 100);
        assert_eq!(rect.left(), 10);
        assert_eq!(rect.right(), 30);
        assert_eq!(rect.top(), 10);
        assert_eq!(rect.bottom(), 15);
    }

    #[test]
    fn test_layout_split_vertical() {
        let layout = Layout::new(
            Direction::Vertical,
            vec![Constraint::Length(2), Constraint::Percentage(50)],
        );
        let rect = Rect::new(0, 0, 10, 10);
        let rects = layout.split(rect);

        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0], Rect::new(0, 0, 10, 2));
        assert_eq!(rects[1], Rect::new(0, 2, 10, 5));
    }
}
