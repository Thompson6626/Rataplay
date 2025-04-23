use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::Modifier;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use std::ops::Div;
use std::time::{Duration, Instant};
enum GameState {
    Ready,         // Game is waiting for you to press any key to start
    Waiting,       // Random delay is counting down (you should NOT press a key)
    TooSoon,       // You pressed a key during the wait period — try again
    Active,        // NOW! Press a key — the timer is running
    Success(u128), // You pressed in time — the number is your reaction time (ms)
    Stats(u128),   // The game is over — shows all times and the average
}

pub struct ReactionGame {
    state: GameState,            // Current state (Ready, Waiting, etc.)
    attempts: u32,               // Total number of attempts to make (default: 5)
    done: u32,                   // How many attempts have been completed
    reaction_history: Vec<u128>, // Stores reaction times
    start_time: Option<Instant>, // When the Active phase started
    wait_until: Option<Instant>, // When the Waiting phase should end
}

impl ReactionGame {
    pub fn new() -> Self {
        ReactionGame {
            state: GameState::Ready,
            attempts: 5, // Add a better way to make defaults later
            done: 0,
            reaction_history: Vec::new(),
            start_time: None,
            wait_until: None,
        }
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.state = GameState::Ready,

            _ => match self.state {
                GameState::Ready => {
                    self.start_waiting();
                }
                GameState::Waiting => {
                    self.state = GameState::TooSoon;
                    self.wait_until = None;
                }
                GameState::TooSoon => {
                    self.start_waiting();
                }
                GameState::Active => {
                    if let Some(start) = self.start_time {
                        let duration = Instant::now().duration_since(start).as_millis();
                        self.reaction_history.push(duration);
                        self.done += 1;
                        self.state = GameState::Success(duration);
                    }
                }
                GameState::Success(_) => {
                    // Attempts left
                    if self.done < self.attempts {
                        self.start_waiting();
                    } else {
                        //
                        let avg = self
                            .reaction_history
                            .iter()
                            .sum::<u128>()
                            .div(self.attempts as u128);
                        self.state = GameState::Stats(avg);
                    }
                }
                GameState::Stats(_) => {
                    self.done = 0;
                    self.reaction_history.clear();
                    self.state = GameState::Ready;
                }
            },
        }
    }

    pub fn update(&mut self) {
        if let GameState::Waiting = self.state {
            if let Some(when) = self.wait_until {
                if Instant::now() >= when {
                    self.state = GameState::Active;
                    self.start_time = Some(Instant::now());
                    self.wait_until = None;
                }
            }
        }
    }

    fn start_waiting(&mut self) {
        self.state = GameState::Waiting;
        let mut rng = rand::rng();
        let millis = rng.random_range(2000..4000);
        self.wait_until = Some(Instant::now() + Duration::from_millis(millis));
        self.start_time = None;
    }
}

pub fn ui(f: &mut Frame, app: &ReactionGame) {
    pub fn make_line<T: Into<String>>(text: T) -> Line<'static> {
        Line::from(Span::styled(
            text.into(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
    }

    // Only bg color and lines of text will change through screens
    let (color, lines) = match &app.state {
        GameState::Ready => (
            Color::Blue,
            vec![
                make_line("When the red box turns green, click as quickly as you can"),
                make_line("Click anywhere to start"),
            ],
        ),
        GameState::Waiting => (Color::Red, vec![make_line("Wait for green")]),
        GameState::TooSoon => (
            Color::LightBlue,
            vec![make_line("Too Soon!"), make_line("Click to try again")],
        ),
        GameState::Active => (Color::Green, vec![make_line("Click now!")]),
        GameState::Success(i) => (
            Color::Cyan,
            vec![
                make_line(format!("{i} ms")),
                make_line("Click to keep going"),
            ],
        ),
        GameState::Stats(avg) => (
            Color::Cyan,
            vec![make_line("Reaction time"), make_line(format!("{avg} ms"))],
        ),
    };

    let size = f.area();

    // Background fill
    let background = Block::default().style(Style::default().bg(color));
    f.render_widget(background, size);

    // Layout to vertically center
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Min(3),
            Constraint::Percentage(40),
        ])
        .split(size);

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default());

    f.render_widget(paragraph, chunks[1]);
}
