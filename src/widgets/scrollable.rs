//! A widget that allows scrolling its content.

use crate::{Buffer, Frame, Rect, Widget};

/// A wrapper widget that renders its child into a virtual buffer and displays a viewport.
pub struct Scrollable<W> {
    content: W,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl<W> Scrollable<W> {
    /// Creates a new scrollable wrapper around the given widget.
    ///
    /// By default, the virtual size is 100x100 and the scroll offset is (0,0).
    pub fn new(content: W) -> Self {
        Self {
            content,
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        }
    }

    /// Sets the scroll offset.
    pub fn scroll(mut self, x: u16, y: u16) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Sets the virtual size of the content.
    pub fn virtual_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

impl<W: Widget> Widget for Scrollable<W> {
    fn render(self, area: Rect, frame: &mut Frame) {
        // 1. Create temp buffer
        let mut buffer = Buffer::new(self.width, self.height);
        let tmp_area = Rect::new(0, 0, self.width, self.height);
        let mut tmp_frame = Frame::new(&mut buffer, tmp_area);

        // 2. Render content into temp buffer
        self.content.render(tmp_area, &mut tmp_frame);

        // 3. Copy slice to main frame
        let copy_width = std::cmp::min(area.width, self.width.saturating_sub(self.x));
        let copy_height = std::cmp::min(area.height, self.height.saturating_sub(self.y));

        frame.buffer_mut().copy_from(
            &buffer,
            Rect::new(self.x, self.y, copy_width, copy_height),
            area.x,
            area.y,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::Text;

    #[test]
    fn test_scrollable_render() {
        let mut buffer = Buffer::new(5, 1);
        let mut frame = Frame::new(&mut buffer, Rect::new(0, 0, 5, 1));

        // Text is "Hello World" (11 chars)
        // Virtual size is 20x1
        // Scroll is 6 (should start at 'W')
        let text = Text::new("Hello World");
        let scrollable = Scrollable::new(text).virtual_size(20, 1).scroll(6, 0);

        scrollable.render(Rect::new(0, 0, 5, 1), &mut frame);

        assert_eq!(buffer.get(0, 0).symbol, 'W');
        assert_eq!(buffer.get(1, 0).symbol, 'o');
        assert_eq!(buffer.get(4, 0).symbol, 'd');
    }
}
