# Briks 🧱

**Briks** is a minimalist, testable TUI (Text User Interface) framework for Rust, inspired by the **Model-View-Update (MVU)** architecture (Elm Architecture).

It is designed for developers who want to build robust, flicker-free terminal applications with 100% testable logic.

## ✨ Features

- **MVU Architecture**: Clean separation of state (Model), logic (Update), and presentation (View).
- **Flicker-Free Diff-Rendering**: A smart rendering engine that only updates terminal cells that have actually changed.
- **RAII Terminal Management**: Automatic handling of Raw Mode and cursor visibility. No more broken terminal states on crash!
- **TrueColor & Styling**: Full 24-bit RGB support, ANSI 256 colors, and text modifiers (Bold, Italic, etc.).
- **Built for Testing**: Hardware-abstracted design using Dependency Injection, allowing you to unit test your entire UI loop without a real terminal.
- **Zero Dependencies**: Built from scratch using only `std` and `libc`.

## 🚀 Quick Start

Add `briks` to your `Cargo.toml` (once published) and implement the `Application` trait:

```rust
use briks::{Application, Color, Command, Event, Frame, KeyCode, Modifier, Style, run};

struct Counter {
    value: i32,
}

enum Action {
    Increment,
    Decrement,
    Quit,
}

impl Application for Counter {
    type Action = Action;

    fn on_event(&self, event: Event) -> Option<Self::Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('+') => Some(Action::Increment),
                KeyCode::Char('-') => Some(Action::Decrement),
                KeyCode::Char('q') => Some(Action::Quit),
                _ => None,
            },
            _ => None,
        }
    }

    fn update(&mut self, action: Self::Action) -> Command {
        match action {
            Action::Increment => self.value += 1,
            Action::Decrement => self.value -= 1,
            Action::Quit => return Command::Quit,
        }
        Command::None
    }

    fn draw(&self, frame: &mut Frame) {
        // Use scoped styling for the header
        frame.with_style(Style::new().fg(Color::Cyan).modifier(Modifier::BOLD), |f| {
            f.write_str(0, 0, &format!("Count: {}", self.value));
        });
        
        frame.write_str(0, 1, "Press + to inc, - to dec, q to quit");
    }
}
```


## 🛠️ Current Status

Briks is currently in active development. We are currently working on:
- [x] Hardware Abstraction Layer (HAL)
- [x] Event Parsing (ANSI & UTF-8)
- [x] Diff-Rendering Engine
- [x] Stateful Styling API
- [ ] Layout Engine (Flexbox-inspired)
- [ ] Widget Library

## 📜 License

Licensed under the [MIT License](LICENSE).
