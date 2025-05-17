use crate::games::r#trait::Game;
use crate::games::utils::line_with_color;
use crossterm::event;
use crossterm::event::{
    KeyCode, KeyEvent,
};
use rand::Rng;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Terminal;
use std::time::{Duration, Instant};
use std::{io, thread};

enum GameState {
    Title,        // Game is waiting for you to press any key to start
    Waiting,      // Random delay is counting down (you should NOT press a key)
    TooSoon,      // You pressed a key during the wait period — try again
    Active,       // NOW! Press a key — the timer is running
    Success(u32), // You pressed in time — the number is your reaction time (ms)
    Stats(u32),   // The games is over — shows all times and the average
}

pub struct ReactionGame {
    state: GameState,            // Current state (Ready, Waiting, etc.)
    attempts: u32,               // Total number of attempts to make (default: 5)
    done: u32,                   // How many attempts have been completed
    reaction_history: Vec<u32>,  // Stores reaction times
    start_time: Option<Instant>, // When the Active phase started
    wait_until: Option<Instant>, // When the Waiting phase should end
    quit: bool,                  // Whether the user wants to quit or not
}

impl Game for ReactionGame {
    fn name(&self) -> &str {
        "⚡ Reaction Time"
    }

    fn description(&self) -> &str {
        "Test your visual reflexes"
    }

    fn handle_input(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.done = 0;
                self.reaction_history.clear();
                self.start_time = None;
                self.wait_until = None;

                match self.state {
                    GameState::Title => self.quit = true,
                    _ => self.state = GameState::Title,
                }
            }

            _ => match self.state {
                GameState::Title => {
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
                        let duration = Instant::now().duration_since(start).as_millis() as u32;
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
                        let avg = self.reaction_history.iter().sum::<u32>() / self.attempts;

                        self.state = GameState::Stats(avg);
                    }
                }
                GameState::Stats(_) => {
                    self.done = 0;
                    self.reaction_history.clear();
                    self.state = GameState::Title;
                }
            },
        }
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        while !self.quit {
            terminal
                .draw(|frame| {
                    let (color, lines) = match self.state {
                        GameState::Title => (
                            Color::Blue,
                            vec![
                                line_with_color("⚡", Color::White),
                                line_with_color(
                                    "When the red box turns green, press as quickly as you can",
                                    Color::White,
                                ),
                                line_with_color("Press any button to start", Color::White),
                            ],
                        ),
                        GameState::Waiting => (
                            Color::Red,
                            vec![line_with_color("Wait for green", Color::White)],
                        ),
                        GameState::TooSoon => (
                            Color::LightBlue,
                            vec![
                                line_with_color("Too soon!", Color::White),
                                line_with_color("Try again by pressing a button", Color::White),
                            ],
                        ),
                        GameState::Active => (
                            Color::Green,
                            vec![line_with_color("Press now!", Color::White)],
                        ),
                        GameState::Success(i) => (
                            Color::Cyan,
                            vec![
                                line_with_color(format!("{i} ms"), Color::White),
                                line_with_color("Keep going! Press to continue", Color::White),
                            ],
                        ),
                        GameState::Stats(avg) => (
                            Color::Cyan,
                            vec![
                                line_with_color("Average reaction time", Color::White),
                                line_with_color(format!("{avg} ms"), Color::White),
                            ],
                        ),
                    };

                    let size = frame.area();

                    // Background fill
                    let background = Block::default().style(Style::default().bg(color));
                    frame.render_widget(background, size);

                    // Layout to vertically center
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(40),
                            Constraint::Min(3),
                            Constraint::Percentage(40),
                        ])
                        .split(size);

                    let paragraph: Paragraph<'_> = Paragraph::new(lines)
                        .alignment(Alignment::Center)
                        .block(Block::default());

                    frame.render_widget(paragraph, chunks[1]);
                })
                .expect("Error while rendering game");

            if event::poll(Duration::from_millis(10))? {
                self.handle_events()?;
            }

            self.update();

            thread::sleep(Duration::from_millis(5));
        }

        self.quit = false; // Reset;
        Ok(())
    }
}

impl ReactionGame {
    pub fn new() -> Self {
        Self {
            state: GameState::Title,
            attempts: 5, // Add a better way to make defaults later
            done: 0,
            reaction_history: Vec::new(),
            start_time: None,
            wait_until: None,
            quit: false,
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
