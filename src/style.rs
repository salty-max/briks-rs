//! The `style` module provides types for customizing the appearance of text.
//!
//! It supports ANSI colors and text modifiers like Bold, Italic, and Underline.

/// Represents a color in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    pub fn from_hex(hex: &str) -> Option<Self> {
        let s = hex.strip_prefix("#").unwrap_or(hex);
        if s.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;

        Some(Color::Rgb(r, g, b))
    }
}

/// A bitflag representing text modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifier(u16);

impl Modifier {
    pub const BOLD: Self = Self(0b0000_0001);
    pub const ITALIC: Self = Self(0b0000_0010);
    pub const UNDERLINE: Self = Self(0b0000_0100);
    pub const REVERSED: Self = Self(0b0000_1000);
    pub const DIM: Self = Self(0b0001_0000);

    pub fn empty() -> Self {
        Self(0)
    }

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

impl std::ops::BitOr for Modifier {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Represents the visual style of a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub modifiers: Modifier,
}

impl Style {
    /// Creates a new, default style (no colors, no modifiers).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the foreground color.
    pub fn fg(mut self, color: Color) -> Self {
        self.foreground = Some(color);
        self
    }

    /// Sets the background color.
    pub fn bg(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Adds a modifier.
    pub fn modifier(mut self, modifier: Modifier) -> Self {
        self.modifiers.insert(modifier);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_builder() {
        let style = Style::new()
            .fg(Color::Red)
            .bg(Color::Blue)
            .modifier(Modifier::BOLD | Modifier::ITALIC);

        assert_eq!(style.foreground, Some(Color::Red));
        assert_eq!(style.background, Some(Color::Blue));
        assert!(style.modifiers.contains(Modifier::BOLD));
        assert!(style.modifiers.contains(Modifier::ITALIC));
        assert!(!style.modifiers.contains(Modifier::UNDERLINE));
    }

    #[test]
    fn test_color_from_hex() {
        assert_eq!(Color::from_hex("#FF5733"), Some(Color::Rgb(255, 87, 51)));
        assert_eq!(Color::from_hex("000000"), Some(Color::Rgb(0, 0, 0)));
        assert_eq!(Color::from_hex("FFFFFF"), Some(Color::Rgb(255, 255, 255)));
        assert_eq!(Color::from_hex("#123"), None);
        assert_eq!(Color::from_hex("invalid"), None);
    }
}
