use briks::{
    Application, Color, Command, Constraint, Direction, Event, Frame, KeyCode, Layout, Modifier,
    Style, Widget, run, widgets::Text,
};

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
        let [top, _, bottom] = Layout::new(
            Direction::Vertical,
            vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill,
            ],
        )
        .split_to(frame.area());

        Text::new(format!("Count: {}", self.value))
            .style(Style::new().modifier(Modifier::BOLD))
            .render(top, frame);
        Text::new("Press +/-, q to quit.")
            .style(Style::new().fg(Color::Rgb(128, 128, 128)))
            .render(bottom, frame);
    }
}

fn main() -> std::io::Result<()> {
    let counter = Counter { value: 0 };

    run(counter)
}
