//! The `frame` module provides a high-level drawing API.
//!
//! A [`Frame`] wraps a [`Buffer`] and provides methods
//! for drawing text, shapes, and widgets without having to manipulate
//! individual cells manually.

use crate::buffer::Buffer;

/// A high-level handle for drawing to a buffer.
pub struct Frame<'a> {
    buffer: &'a mut Buffer,
}

impl<'a> Frame<'a> {
    /// Creates a new frame wrapping the given buffer.
    pub fn new(buffer: &'a mut Buffer) -> Self {
        Self { buffer }
    }

    /// Returns the width of the frame.
    pub fn width(&self) -> u16 {
        self.buffer.width
    }

    /// Returns the height of the frame.
    pub fn height(&self) -> u16 {
        self.buffer.height
    }

    /// Writes a string to the buffer starting at the given coordinates.
    ///
    /// Text that exceeds the buffer width will be clipped.
    pub fn write_str(&mut self, x: u16, y: u16, text: &str) {
        for (i, c) in text.chars().enumerate() {
            self.buffer.set(x + (i as u16), y, c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;

    #[test]
    fn test_frame_write_str() {
        let mut buffer = Buffer::new(10, 1);
        let mut frame = Frame::new(&mut buffer);

        frame.write_str(2, 0, "Hello");

        assert_eq!(buffer.get(1, 0).symbol, ' ');
        assert_eq!(buffer.get(2, 0).symbol, 'H');
        assert_eq!(buffer.get(6, 0).symbol, 'o');
        assert_eq!(buffer.get(7, 0).symbol, ' ');
    }

    #[test]
    fn test_frame_write_str_clipping() {
        let mut buffer = Buffer::new(5, 1);
        let mut frame = Frame::new(&mut buffer);

        // "Hello World" is 11 chars, buffer is 5.
        // Starting at 2, it should only write "Hel"
        frame.write_str(2, 0, "Hello World");

        assert_eq!(buffer.get(1, 0).symbol, ' ');
        assert_eq!(buffer.get(2, 0).symbol, 'H');
        assert_eq!(buffer.get(4, 0).symbol, 'l');
    }
}
