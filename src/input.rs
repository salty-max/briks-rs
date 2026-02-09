//! The `input` module handles the parsing of raw byte streams into semantic events.
//!
//! It provides:
//! * [`Event`]: A high-level enum representing things that happen (Keys, Resizes).
//! * [`Input`]: The main entry point to read from the terminal and get events.
//! * [`Parser`]: A state machine that decodes ANSI escape sequences and UTF-8 characters.

use std::collections::VecDeque;
use std::fmt;

use crate::terminal::Terminal;

/// Represents a distinct event occurring in the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A keyboard press (char, control key, function key).
    Key(KeyEvent),
    /// A terminal resize event (cols, rows).
    Resize(u16, u16),
}

/// Represents a specific key press, including modifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    /// The specific key that was pressed.
    pub code: KeyCode,
    /// Any modifiers held down (Shift, Ctrl, Alt).
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    /// Creates a new `KeyEvent` with no modifiers.
    pub fn new(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::empty(),
        }
    }

    /// Creates a new `KeyEvent` with specific modifiers.
    pub fn with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
}

/// Represents the key identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyCode {
    /// A standard character key (e.g., 'a', '1', '?').
    Char(char),
    Enter,
    Backspace,
    Esc,
    Left,
    Right,
    Up,
    Down,
    Tab,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    /// Function keys (F1-F12).
    F(u8),
    Null,
}

/// A bitflag struct representing Shift, Ctrl, and Alt modifiers.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct KeyModifiers(u8);

impl KeyModifiers {
    pub const SHIFT: Self = Self(0b0000_0001);
    pub const CTRL: Self = Self(0b0000_0010);
    pub const ALT: Self = Self(0b0000_0100);

    /// Returns an empty set of modifiers.
    pub fn empty() -> Self {
        Self(0)
    }

    /// Checks if a specific modifier is set.
    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Inserts a modifier into the set.
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

impl fmt::Debug for KeyModifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = Vec::new();
        if self.contains(Self::SHIFT) {
            list.push("SHIFT");
        }
        if self.contains(Self::CTRL) {
            list.push("CTRL");
        }
        if self.contains(Self::ALT) {
            list.push("ALT");
        }
        write!(f, "KeyModifiers({:?})", list)
    }
}

/// Allows combining modifiers using the `|` operator.
impl std::ops::BitOr for KeyModifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// The main input handler.
///
/// Reads raw bytes from the [`Terminal`] and uses the [`Parser`] to produce [`Event`]s.
pub struct Input {
    parser: Parser,
}

impl Input {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    /// Reads available bytes from the terminal and returns a vector of parsed events.
    ///
    /// This method is non-blocking if the underlying terminal read is non-blocking,
    /// or blocking otherwise (standard `read` behavior).
    pub fn read(&mut self, term: &Terminal) -> Vec<Event> {
        let mut buf = [0u8; 1024];
        match term.read(&mut buf) {
            Ok(n) if n > 0 => self.parser.parse(&buf[..n]),
            _ => Vec::new(),
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal state machine for parsing byte streams into Events.
pub struct Parser {
    buffer: VecDeque<u8>,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
        }
    }

    /// Parses a slice of bytes and appends them to the internal buffer,
    /// returning any complete events found.
    pub fn parse(&mut self, bytes: &[u8]) -> Vec<Event> {
        self.buffer.extend(bytes);
        let mut events: Vec<Event> = Vec::new();

        loop {
            if self.buffer.is_empty() {
                break;
            }

            // Look at the first byte without removing it yet
            match self.buffer[0] {
                b'\r' => {
                    events.push(Event::Key(KeyEvent::new(KeyCode::Enter)));
                    self.buffer.pop_front();
                }
                b'\x1b' => {
                    // Start of an Escape Sequence
                    if self.buffer.len() == 1 {
                        break; // Incomplete, wait for more data
                    }

                    // Check for CSI (Control Sequence Introducer) `\x1b[`
                    if self.buffer.len() >= 3 && self.buffer[1] == b'[' {
                        match self.buffer[2] {
                            b'A' => {
                                events.push(Event::Key(KeyEvent::new(KeyCode::Up)));
                                self.consume(3);
                            }
                            _ => {
                                // Unknown CSI sequence, consume ESC to prevent stuck loop
                                events.push(Event::Key(KeyEvent::new(KeyCode::Esc)));
                                self.buffer.pop_front();
                            }
                        }
                    } else {
                        // Just a raw Esc key
                        events.push(Event::Key(KeyEvent::new(KeyCode::Esc)));
                        self.buffer.pop_front();
                    }
                }
                b => {
                    // Regular UTF-8 Character parsing
                    let width = utf8_char_width(b);

                    if width == 0 {
                        // Invalid byte, consume it to skip
                        self.buffer.pop_front();
                    } else if self.buffer.len() >= width {
                        // Extract the valid UTF-8 slice
                        let bytes: Vec<u8> = self.buffer.range(0..width).copied().collect();
                        if let Ok(s) = std::str::from_utf8(&bytes)
                            && let Some(c) = s.chars().next()
                        {
                            events.push(Event::Key(KeyEvent::new(KeyCode::Char(c))));
                        }
                        self.consume(width);
                    } else {
                        // Incomplete UTF-8 char, wait for more data
                        break;
                    }
                }
            }
        }

        events
    }

    /// Helper to remove `n` bytes from the front of the queue
    fn consume(&mut self, n: usize) {
        for _ in 0..n {
            self.buffer.pop_front();
        }
    }
}

/// Helper to determine the byte width of a UTF-8 character based on the first byte.
fn utf8_char_width(first_byte: u8) -> usize {
    if first_byte & 0b10000000 == 0 {
        1
    } else if first_byte & 0b11100000 == 0b11000000 {
        2
    } else if first_byte & 0b11110000 == 0b11100000 {
        3
    } else if first_byte & 0b11111000 == 0b11110000 {
        4
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_char() {
        let mut parser = Parser::new();
        let events = parser.parse(b"a");
        assert_eq!(events, vec![Event::Key(KeyEvent::new(KeyCode::Char('a')))]);
    }

    #[test]
    fn test_parse_enter() {
        let mut parser = Parser::new();
        let events = parser.parse(b"\r");
        assert_eq!(events, vec![Event::Key(KeyEvent::new(KeyCode::Enter))]);
    }

    #[test]
    fn test_parse_arrow() {
        let mut parser = Parser::new();
        let events = parser.parse(b"\x1b[A");
        assert_eq!(events, vec![Event::Key(KeyEvent::new(KeyCode::Up))]);
    }

    #[test]
    fn test_parse_multiple() {
        let mut parser = Parser::new();
        let events = parser.parse(b"a\rb");
        assert_eq!(
            events,
            vec![
                Event::Key(KeyEvent::new(KeyCode::Char('a'))),
                Event::Key(KeyEvent::new(KeyCode::Enter)),
                Event::Key(KeyEvent::new(KeyCode::Char('b'))),
            ]
        );
    }

    #[test]
    fn test_parse_utf8() {
        let mut parser = Parser::new();
        // 'é' is 0xC3 0xA9 in UTF-8
        let events = parser.parse(&[0xc3, 0xa9]);
        assert_eq!(events, vec![Event::Key(KeyEvent::new(KeyCode::Char('é')))]);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::terminal::{Terminal, mocks::MockSystem}; // FIX: Reusing the existing MockSystem

    #[test]
    fn test_input_read() {
        // Arrange
        let mock = MockSystem::new();
        // Pre-fill the mock's input buffer with data "a"
        mock.push_input(b"a");

        let term = Terminal::new_with_system(Box::new(mock)).unwrap();
        let mut input = Input::new();

        // Act
        let events = input.read(&term);

        // Assert
        assert_eq!(events, vec![Event::Key(KeyEvent::new(KeyCode::Char('a')))]);
    }
}
