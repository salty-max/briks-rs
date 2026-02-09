//! **Briks** is a minimalist TUI (Text User Interface) framework.
//!
//! It provides a runtime loop, input handling, and a raw terminal abstraction.
//! Users implement the [`Application`] trait to define their logic.

use std::io;
use std::thread;
use std::time::Duration;

use crate::{
    input::{Event, Input},
    terminal::Terminal,
};

pub mod input;
#[macro_use]
pub mod logger;
pub mod terminal;

/// Commands returned by the application to control the runtime flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Continue running the application loop.
    None,
    /// Stop the application and exit.
    Quit,
}

/// The core trait for a Briks application.
///
/// This follows a simplified Model-View-Update (MVU) pattern:
/// 1. **Draw**: The state is rendered to a string.
/// 2. **Event**: Input is converted into an internal `Action`.
/// 3. **Update**: The `Action` modifies the state and returns a `Command`.
pub trait Application {
    /// The message type used to update the application state.
    /// This allows decoupling raw input events from business logic.
    type Action;

    /// Called once before the event loop starts.
    fn init(&self) -> Command {
        Command::None
    }

    /// Maps a raw terminal [`Event`] to an application-specific [`Self::Action`].
    ///
    /// Return `None` to ignore the event.
    fn on_event(&self, _event: Event) -> Option<Self::Action> {
        None
    }

    /// Updates the application state based on an action.
    ///
    /// Returns a [`Command`] to control the runtime (e.g., to quit).
    fn update(&mut self, msg: Self::Action) -> Command;

    /// Renders the current application state as a string.
    fn draw(&self) -> String;
}

/// Entry point to run a Briks application.
///
/// This initializes the terminal in Raw Mode, sets up input capturing,
/// and enters the main event loop.
pub fn run<App: Application>(app: App) -> io::Result<()> {
    let terminal = Terminal::new()?;
    let input = Input::new();
    run_app(app, terminal, input)
}

/// The internal event loop.
fn run_app<App: Application>(mut app: App, terminal: Terminal, mut input: Input) -> io::Result<()> {
    // Check if the app wants to exit immediately
    if let Command::Quit = app.init() {
        return Ok(());
    }

    loop {
        // --- 1. Render Phase ---
        let view = app.draw();

        // Clear screen (\x1b[2J) and move cursor home (\x1b[H)
        // TODO: Double buffering
        terminal.write(b"\x1b[2J\x1b[H")?;
        terminal.write(view.as_bytes())?;

        // --- 2. Input Phase ---
        let events = input.read(&terminal);
        for event in events {
            // Map raw event -> App Action
            if let Some(msg) = app.on_event(event) {
                // Update State
                match app.update(msg) {
                    Command::Quit => return Ok(()),
                    Command::None => {}
                }
            }
        }

        // --- 3. Idle Phase ---
        // Simple frame limiter (approx 60 FPS) to reduce CPU usage.
        thread::sleep(Duration::from_millis(16));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{Event, KeyCode, KeyEvent};
    // Note: We use the mock system to simulate input without a real terminal
    use crate::terminal::mocks::MockSystem;

    struct TestApp;

    impl Application for TestApp {
        type Action = ();

        fn on_event(&self, event: Event) -> Option<Self::Action> {
            // Quit if 'q' is pressed
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) = event
            {
                Some(())
            } else {
                None
            }
        }

        fn update(&mut self, _msg: Self::Action) -> Command {
            Command::Quit
        }

        fn draw(&self) -> String {
            "Test".to_string()
        }
    }

    #[test]
    fn test_run_loop_quits() {
        // Arrange
        let mock = MockSystem::new();
        mock.push_input(b"q"); // Inject 'q' into the mock input buffer

        // Inject the mock system into the Terminal
        let terminal = Terminal::new_with_system(Box::new(mock)).unwrap();
        let input = Input::new();
        let app = TestApp;

        // Act
        // This runs the loop. It should read 'q', call on_event,
        // receive (), call update, receive Command::Quit, and return Ok.
        let res = run_app(app, terminal, input);

        // Assert
        assert!(res.is_ok());
    }
}
