use std::io;

use crate::{
    input::{Event, Input},
    terminal::Terminal,
};

pub mod input;
#[macro_use]
pub mod logger;
pub mod terminal;

pub enum Command {
    None,
    Quit,
}

pub trait Application {
    type Action;

    fn init(&self) -> Command {
        Command::None
    }
    fn on_event(&self, _event: Event) -> Option<Self::Action> {
        None
    }
    fn update(&mut self, msg: Self::Action) -> Command;
    fn draw(&self) -> String;
}

pub fn run<App: Application>(mut app: App) -> io::Result<()> {
    let terminal = Terminal::new()?;
    let mut input = Input::new();

    if let Command::Quit = app.init() {
        return Ok(());
    }

    loop {
        // Render
        let view = app.draw();
        // Clear screen (ESC [ 2 J) and move cursor (ESC [ H)
        terminal.write(b"\x1b[2J\x1b[H")?;
        terminal.write(view.as_bytes())?;

        // Process events
        let events = input.read(&terminal);
        for event in events {
            if let Some(msg) = app.on_event(event) {
                match app.update(msg) {
                    Command::Quit => return Ok(()),
                    Command::None => {}
                }
            }
        }

        // Simple frame limiter
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
