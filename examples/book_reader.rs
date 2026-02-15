use phosphor::{
    Application, Color, Command, Constraint, Direction, Event, Frame, KeyCode, Layout, Modifier,
    MouseEvent, MouseKind, Style, run,
    widgets::{Block, Borders, Scrollable, Text},
};

struct BookReader {
    scroll_y: u16,
    content: String,
}

impl BookReader {
    fn new() -> Self {
        let content = r#"
Barenziah was born in 2E 893 to the Lord and Lady of Mournhold, in the land of Morrowind. She was a dark elf, or Dunmer, as they call themselves. She was a beautiful child, with skin like obsidian and eyes like rubies.

Her early years were happy enough, though she was a willful child, prone to tantrums and mischief. Her nursemaid, a stout Nord woman named Katisha, often despaired of her charge. "She's a wild one, that Barenziah," Katisha would say, shaking her head. "Mark my words, she'll come to no good."

But Barenziah's parents doted on her. They gave her everything she asked for, and more. She had the finest clothes, the most expensive toys, and the best tutors in the land. She learned to read and write at an early age, and she showed a talent for magic.

When Barenziah was five years old, the Empire of Cyrodiil invaded Morrowind. The Emperor, Tiber Septim, demanded that the Dunmer submit to his rule. The Lord of Mournhold refused, and war broke out.

The war lasted for many years. Barenziah's father was killed in battle, and her mother died of grief shortly after. Barenziah was left an orphan, the sole heir to the throne of Mournhold.

The Empire eventually conquered Morrowind, and Tiber Septim appointed a governor to rule the province. The governor, a man named Symmachus, took Barenziah under his wing. He raised her as his own daughter, and he taught her the ways of the Empire.

Barenziah grew up to be a beautiful and intelligent young woman. She was popular with the people of Mournhold, and she was respected by the Imperial officials. But she never forgot her heritage. She knew that she was the rightful ruler of Mournhold, and she dreamed of one day reclaiming her throne.

One day, a young man named Straw came to Mournhold. Straw was a thief, a rogue, and a charmer. He was also a member of the Thieves Guild. Barenziah was immediately smitten with him.

Straw told Barenziah stories of his adventures, of the places he had been and the things he had seen. He told her of the great cities of the Empire, of the strange creatures that lived in the wilderness, and of the treasures that were hidden in ancient ruins.

Barenziah was captivated. She had never left Mournhold, and she longed to see the world. She begged Straw to take her with him when he left.

Straw agreed, and the two of them ran away together. They traveled across Morrowind, stealing from the rich and giving to the poor. They had many adventures, and they fell deeply in love.

But their happiness was not to last. Symmachus sent his guards to find Barenziah, and they eventually caught up with the couple. Straw was arrested and thrown in prison, and Barenziah was returned to Mournhold.

Symmachus was furious with Barenziah. He told her that she had disgraced her family and her people. He confined her to the palace, and he forbade her from ever seeing Straw again.

Barenziah was heartbroken. She cried for days, and she refused to eat or sleep. But eventually, she realized that she had to be strong. She had a duty to her people, and she could not let her personal feelings get in the way of her responsibilities.

She devoted herself to her studies, and she learned everything she could about politics and diplomacy. She became a wise and just ruler, and she eventually won the respect of Symmachus and the Emperor.

But she never forgot Straw. She kept a secret diary, in which she wrote about her love for him. And every night, before she went to sleep, she would look out her window and whisper his name.

(To be continued...)
"#
        .trim()
        .replace("\n", " \n "); // Add spaces for wrapping simulation if we had it

        Self {
            scroll_y: 0,
            content: content.to_string(),
        }
    }
}

enum Action {
    ScrollUp,
    ScrollDown,
    Quit,
}

impl Application for BookReader {
    type Action = Action;

    fn on_event(&self, event: Event) -> Option<Self::Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Up | KeyCode::Char('k') => Some(Action::ScrollUp),
                KeyCode::Down | KeyCode::Char('j') => Some(Action::ScrollDown),
                _ => None,
            },
            Event::Mouse(MouseEvent { kind, .. }) => match kind {
                MouseKind::ScrollUp => Some(Action::ScrollUp),
                MouseKind::ScrollDown => Some(Action::ScrollDown),
                _ => None,
            },
            _ => None,
        }
    }

    fn update(&mut self, action: Self::Action) -> Command {
        match action {
            Action::Quit => return Command::Quit,
            Action::ScrollUp => {
                self.scroll_y = self.scroll_y.saturating_sub(1);
            }
            Action::ScrollDown => {
                // Simple bound check (approximate lines)
                if self.scroll_y < 50 {
                    self.scroll_y += 1;
                }
            }
        }
        Command::None
    }

    fn draw(&self, frame: &mut Frame) {
        let [header, body, footer] = Layout::new(
            Direction::Vertical,
            vec![
                Constraint::Length(4),
                Constraint::Fill,
                Constraint::Length(1),
            ],
        )
        .split_to(frame.area());

        // --- Header ---
        let header_block = Block::new()
            .borders(Borders::ALL)
            .padding_x(1)
            .style(Style::new().fg(Color::Yellow));

        let header_inner = header_block.inner(header);
        frame.render_widget(header_block, header);

        frame.render_area(header_inner, |f| {
            f.write_str_with_style(
                0,
                0,
                "The Real Barenziah, Part 1",
                Style::new().modifier(Modifier::BOLD),
            );
            f.write_str(0, 1, "by Plitinius Mero");
        });

        // --- Body (Scrollable) ---
        let body_block = Block::new().borders(Borders::NONE); // Just for padding if needed
        let body_inner = body_block.inner(body);

        // We render the text into a virtual area that is as wide as the screen
        // but much taller (to hold the whole book).
        let text_widget = Text::new(&self.content).wrap(true);

        let scrollable = Scrollable::new(text_widget)
            .virtual_size(body_inner.width, 100) // Arbitrary height for now
            .scroll(0, self.scroll_y);

        frame.render_widget(scrollable, body_inner);

        // --- Footer ---
        frame.render_widget(
            Text::new("Use Up/Down or Mouse Wheel to scroll. Q to quit.")
                .style(Style::new().fg(Color::Rgb(100, 100, 100))),
            footer,
        );
    }
}

fn main() -> std::io::Result<()> {
    run(BookReader::new())
}
