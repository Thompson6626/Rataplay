use crate::games::Game;
use crate::games::utils::line_with_color;
use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::{Frame, Terminal};
use std::cmp::PartialEq;
use std::io;
use std::io::Stdout;
use std::time::{Duration, Instant};

/// Represents the different game states.
#[derive(Debug, PartialEq, Eq)]
enum GameState {
    Title,   // Initial screen
    Showing, // Number shown to player
    Waiting, // Waiting for player input
    Success, //
    End,     // Game over (similar to success)
}
/// Represents a memory task in the game.
pub struct NumberMemory {
    state: GameState,
    number: Option<String>, // The number to remember
    answer: Option<String>, // The player's response
    level: u32,
    quit: bool, // Whether the player has quit
    show_start: Option<Instant>,
    showing_duration: Duration,
}

impl Game for NumberMemory {
    fn name(&self) -> &str {
        "🧠🔢 Number Memory"
    }

    fn description(&self) -> &str {
        "Remember the longest number you can"
    }

    fn handle_input(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => match self.state {
                GameState::Title => self.quit_game(),
                _ => self.reset_game(),
            },
            _ => {}
        }
        match self.state {
            GameState::Title => self.show_number(),
            GameState::Showing => {}
            GameState::Waiting => match key_event.code {
                KeyCode::Enter => {
                    // Get input from the input bar
                    let equal = self
                        .answer
                        .as_ref()
                        .zip(self.number.as_ref())
                        .map(|(s1, s2)| s1 == s2)
                        .unwrap_or(false);

                    if equal {
                        self.level += 1;
                        self.state = GameState::Success;
                    } else {
                        self.state = GameState::End;
                    }
                }
                KeyCode::Backspace => {
                    match &mut self.answer {
                        Some(ans) => {
                            ans.pop();
                            ()
                        } // Pop from the string and do nothing with the result
                        None => (), // Do nothing if there is no answer
                    }
                }
                KeyCode::Char(c) => match &mut self.answer {
                    Some(ans) => {
                        if c.is_numeric() {
                            ans.push(c);
                        }
                    }
                    None => (),
                },
                _ => {}
            },
            GameState::Success => self.show_number(),
            GameState::End => self.reset_game(),
        }
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        self.init_game();

        while !self.quit {
            terminal
                .draw(|frame| match self.state {
                    GameState::Title => self.render_title_screen(frame),
                    GameState::Showing => self.render_showing_screen(frame),
                    GameState::Waiting => self.render_waiting_screen(frame),
                    GameState::Success => self.render_success_screen(frame),
                    GameState::End => self.render_end_screen(frame),
                })
                .expect("Failed to render game");

            if self.state != GameState::Showing {
                self.handle_events()?
            }
        }

        self.quit_game();
        Ok(())
    }
}

impl NumberMemory {
    pub fn new() -> Self {
        Self {
            state: GameState::Title,
            number: None,
            answer: None,
            level: 1,
            quit: false,
            show_start: None,
            showing_duration: Duration::from_millis(4500),
        }
    }

    fn render_title_screen(&self, frame: &mut Frame) {
        let lines = vec![
            line_with_color("Number Memory", Color::White)
                .style(Style::default().add_modifier(Modifier::BOLD)),
            line_with_color(
                "The average person can remember 7 numbers at once.Can you do more?",
                Color::White,
            ),
        ];

        let size = frame.area();

        // Background fill
        let background = Block::default().style(Style::default().bg(Color::Cyan));
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

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(Block::default());

        frame.render_widget(paragraph, chunks[1]);
    }

    fn render_showing_screen(&self, frame: &mut Frame) {
        let size = frame.area();

        let outer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30), // top padding
                Constraint::Length(5),      // content height
                Constraint::Percentage(30), // bottom padding
            ])
            .split(size);

        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3), // space for gauge
                Constraint::Length(1),
            ])
            .split(outer_chunks[1]);

        // "Processing..." text
        let text = Paragraph::new(
            self.number
                .as_ref()
                .map(|n| n.as_str())
                .unwrap_or("No number available"),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().fg(Color::White)),
        ); // adjust text color

        frame.render_widget(text, inner_chunks[0]);

        // Compute reversed progress
        let percent = self
            .show_start
            .map(|start| {
                let elapsed = start.elapsed().as_secs_f64();
                let total = self.showing_duration.as_secs_f64();
                ((total - elapsed) / total).clamp(0.0, 1.0) // reversed
            })
            .unwrap_or(1.0); // full if not started

        // Centered gauge, reversed direction
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .title("") // Adjust title if needed
                    .style(Style::default().bg(Color::Cyan)),
            ) // Background color of the block
            .gauge_style(
                Style::default()
                    .fg(Color::White) // Foreground color (the bar itself)
                    .bg(Color::Cyan),
            ) // Background color (behind the bar)
            .percent((percent * 100.0) as u16);

        frame.render_widget(gauge, inner_chunks[1]);
    }

    fn render_waiting_screen(&self, frame: &mut Frame) {
        // render
    }
    fn render_success_screen(&self, frame: &mut Frame) {}
    fn render_end_screen(&self, frame: &mut Frame) {}

    fn show_number(&mut self) {
        self.state = GameState::Showing;
        self.show_start = Some(Instant::now());
        self.number = Some(self.generate_random_number());
        self.answer.as_mut().expect("Should not be None").clear();
    }

    fn generate_random_number(&self) -> String {
        let mut rng = rand::rng(); // Use the thread-local RNG
        let mut digits = Vec::with_capacity(self.level as usize); // Pre-allocate capacity for digits

        for _ in 0..self.level {
            let digit = rng.random_range(0..10); // Generate a number between 0 and 9
            digits.push(char::from_digit(digit, 10).unwrap()); // Push the digit as a character
        }

        digits.into_iter().collect()
    }

    fn init_game(&mut self) {
        self.answer = Some(String::new());
    }

    fn quit_game(&mut self) {
        self.quit = true;
        self.state = GameState::Title;
        self.answer = None;
        self.number = None;
        self.level = 1;
    }

    fn reset_game(&mut self) {
        self.state = GameState::Title;
        self.answer.as_mut().expect("Should be Some").clear();
        self.number = None;
        self.level = 1;
    }
}
