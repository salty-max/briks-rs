use briks::{Application, Command, Event, Frame, KeyCode, run};

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

    fn update(&mut self, msg: Self::Action) -> Command {
        match msg {
            Action::Increment => self.value += 1,
            Action::Decrement => self.value -= 1,
            Action::Quit => return Command::Quit,
        }
        Command::None
    }

    fn draw(&self, frame: &mut Frame) {
        frame.write_str(
            0,
            0,
            format!("Count: {}\r\nPress +/-, q to quit", self.value).as_str(),
        );
    }
}

fn main() -> std::io::Result<()> {
    let counter = Counter { value: 0 };

    run(counter)
}
