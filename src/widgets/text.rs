//! A simple widget that displays a string of text.

use crate::{Frame, Rect, Style, widgets::Widget};

/// A simple widget that displays a string of text.
pub struct Text {
    text: String,
    style: Style,
}

impl Text {
    /// Creates a new text widget.
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
        }
    }

    /// Sets the style of the text.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for Text {
    fn render(self, area: Rect, frame: &mut Frame) {
        frame.render_area(area, |f| {
            f.with_style(self.style, |f2| {
                f2.write_str(0, 0, &self.text);
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
}
