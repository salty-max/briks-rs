//! A simple widget that displays a string of text.

use crate::{Frame, Rect, Style, widgets::Widget};

/// A simple widget that displays a string of text.
pub struct Text {
    text: String,
    style: Style,
    wrap: bool,
}

impl Text {
    /// Creates a new text widget.
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
            wrap: false,
        }
    }

    /// Sets the style of the text.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets whether the text should wrap when it reaches the edge of the area.
    ///
    /// If true, text will wrap to the next line. If false (default), text will be clipped.
    pub fn wrap(mut self, wrapped: bool) -> Self {
        self.wrap = wrapped;
        self
    }
}

impl Widget for Text {
    fn render(self, area: Rect, frame: &mut Frame) {
        frame.render_area(area, |f| {
            f.with_style(self.style, |f| {
                if self.wrap {
                    let mut wx: u16 = 0;
                    let mut wy: u16 = 0;

                    for line in self.text.lines() {
                        for w in line.split_whitespace() {
                            if wx + w.len() as u16 > f.width() {
                                wx = 0;
                                wy += 1;
                            }
                            if wy >= f.height() {
                                break;
                            }

                            f.write_str(wx, wy, w);
                            wx += w.len() as u16 + 1;
                        }

                        // End of paragraph: force new line
                        wx = 0;
                        wy += 1;
                        if wy >= f.height() {
                            break;
                        }
                    }
                } else {
                    f.write_str(0, 0, &self.text);
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Buffer, Color};

    #[test]
    fn test_text_render() {
        let mut buffer = Buffer::new(10, 1);
        let mut frame = Frame::new(&mut buffer, Rect::new(0, 0, 10, 1));
        let text = Text::new("Hello");

        text.render(Rect::new(0, 0, 10, 1), &mut frame);

        assert_eq!(buffer.get(0, 0).symbol, 'H');
        assert_eq!(buffer.get(4, 0).symbol, 'o');
    }

    #[test]
    fn test_text_styled_render() {
        let mut buffer = Buffer::new(10, 1);
        let mut frame = Frame::new(&mut buffer, Rect::new(0, 0, 10, 1));
        let style = Style::new().fg(Color::Red);
        let text = Text::new("A").style(style);

        text.render(Rect::new(0, 0, 10, 1), &mut frame);

        assert_eq!(buffer.get(0, 0).symbol, 'A');
        assert_eq!(buffer.get(0, 0).style.foreground, Some(Color::Red));
    }

    #[test]
    fn test_text_wrap() {
        let mut buffer = Buffer::new(5, 3);
        let mut frame = Frame::new(&mut buffer, Rect::new(0, 0, 5, 3));
        let text = Text::new("Hello World").wrap(true);

        text.render(Rect::new(0, 0, 5, 3), &mut frame);

        // "Hello" (5 chars) fits on line 0
        assert_eq!(buffer.get(0, 0).symbol, 'H');
        assert_eq!(buffer.get(4, 0).symbol, 'o');

        // "World" (5 chars) wraps to line 1
        assert_eq!(buffer.get(0, 1).symbol, 'W');
        assert_eq!(buffer.get(4, 1).symbol, 'd');
    }
}
