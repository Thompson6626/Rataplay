use crate::games::Game;
use crate::games::utils::line_with_color;
use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::{Frame, Terminal};
use std::cmp::PartialEq;
use std::io;
use std::io::Stdout;
use std::time::{Duration, Instant};

/// Represents the different states the game can be in during its execution.
#[derive(Debug, PartialEq, Eq)]
enum GameState {
    /// The initial title screen displayed before the game starts.
    Title,
    /// The state in which the number to be remembered is shown to the player.
    Showing,
    /// The state in which the game is waiting for the player's input.
    Waiting,
    /// The state entered when the player's input matches the shown number.
    Success,
    /// The state entered when the game ends due to failure.
    End,
}

/// Represents a single session of the number memory game.
///
/// In this game, a number is briefly shown to the player, who must then recall and input it.
/// As the player progresses, the difficulty increases by showing longer numbers.
pub struct NumberMemory {
    /// The current state of the game.
    state: GameState,
    /// The number currently shown to the player that must be remembered.
    number: Option<String>,
    /// The player's input in response to the shown number.
    answer: Option<String>,
    /// The current level of difficulty (increases as the player succeeds).
    level: u32,
    /// Indicates whether the player has chosen to quit the game.
    quit: bool,
    /// The timestamp marking when the number started being shown.
    show_start: Option<Instant>,
    /// The duration for which the number is shown before disappearing.
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
        let mut pressed = false;
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => match self.state {
                GameState::Title => self.quit_game(),
                _ => {
                    self.reset_game();
                    pressed = true;
                },
            },
            _ => {}
        }
        if pressed {
            return;
        }
        match self.state {
            GameState::Title => self.show_number(),
            GameState::Showing => {
                // No input is handled during the showing state
            }
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
                self.handle_events()?;
            } else {
                self.check_to_change_waiting();
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
            showing_duration: Duration::from_millis(1700),
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
        let full_area = frame.area();

        // Step 1: Full cyan background
        let bg_block = Block::default().style(Style::default().bg(Color::Cyan));
        frame.render_widget(bg_block, full_area);

        // Step 2: Vertical layout with top padding, content, bottom padding
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Length(5),
                Constraint::Percentage(30),
            ])
            .split(full_area);

        // Step 3: Sub-layout for content (status + gauge)
        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // top spacing
                Constraint::Length(1), // status message
                Constraint::Length(1), // space
                Constraint::Length(3), // gauge
                Constraint::Length(1), // bottom spacing
            ])
            .split(vertical_layout[1]);

        // Step 4: Render centered status message
        let status_message = self.number.as_deref().unwrap_or("No number available");

        let message_paragraph = Paragraph::new(Line::from(Span::styled(
            status_message,
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center);

        frame.render_widget(message_paragraph, content_layout[1]);

        // Step 5: Calculate remaining progress (reversed)
        let percent_remaining = self
            .show_start
            .map(|start| {
                let elapsed = start.elapsed().as_secs_f64();
                let total = self.showing_duration.as_secs_f64();
                ((total - elapsed) / total).clamp(0.0, 1.0)
            })
            .unwrap_or(1.0); // 100% if timer hasn't started

        // Step 6: Horizontal layout to center the gauge
        let gauge_row = content_layout[3];
        let gauge_horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(gauge_row);

        // Step 7: Render gauge
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .percent((percent_remaining * 100.0) as u16);

        frame.render_widget(gauge, gauge_horizontal_layout[1]);
    }

    fn render_waiting_screen(&self, frame: &mut Frame) {
        let size = frame.area();

        let bg_block = Block::default().style(Style::default().bg(Color::Cyan));
        frame.render_widget(bg_block, size); // this paints the entire terminal background

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40), // top padding
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Percentage(40), // bottom padding
            ])
            .split(size);

        // Text color style (black text on cyan)
        let text_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        let texts = [
            "What was the number?",
            "Press enter to submit",
            self.answer.as_deref().unwrap_or(""), // Option<String> -> &str
        ];

        for (i, text) in texts.iter().enumerate() {
            let paragraph = Paragraph::new(Line::from(Span::styled(*text, text_style)))
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, vertical_chunks[i + 1]); // skip top padding
        }
    }

    fn render_success_screen(&self, frame: &mut Frame) {
        let size = frame.area();

        // Step 1: Fill the whole background with cyan
        let bg_block = Block::default().style(Style::default().bg(Color::Cyan));
        frame.render_widget(bg_block, size);

        // Step 2: Vertically center 6 lines (3 labels + 3 values)
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30), // top padding
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Percentage(30), // bottom padding
            ])
            .split(size);

        // Style for both labels and values
        let text_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        // Step 3: Render all centered text lines
        let lines = [
            "Number",
            self.number.as_deref().unwrap_or(""),
            "Your Answer",
            self.answer.as_deref().unwrap_or(""),
            "Level",
            &self.level.to_string(),
        ];

        for (i, text) in lines.iter().enumerate() {
            let paragraph = Paragraph::new(Line::from(Span::styled(*text, text_style)))
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, vertical_chunks[i + 1]); // +1 to skip top padding
        }
    }

    fn render_end_screen(&self, frame: &mut Frame) {
        let size = frame.area(); // you used `area()` before but it's usually `size()`

        // Step 1: Fill entire background with cyan
        let bg_block = Block::default().style(Style::default().bg(Color::Cyan));
        frame.render_widget(bg_block, size);

        // Step 2: Vertically center 6 lines (3 labels + 3 values)
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Percentage(30),
            ])
            .split(size);

        // Base style for labels and values
        let base_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        // Crossed-out style for the answer
        let crossed_style = base_style.add_modifier(Modifier::CROSSED_OUT);

        // Step 3: Render text with appropriate styles
        let texts = [
            ("Number", base_style),
            (self.number.as_deref().unwrap_or(""), base_style),
            ("Your Answer", base_style),
            (self.answer.as_deref().unwrap_or(""), crossed_style), // crossed out!
            ("Level", base_style),
            (&self.level.to_string(), base_style),
        ];

        for (i, (text, style)) in texts.iter().enumerate() {
            let paragraph = Paragraph::new(Line::from(Span::styled(*text, *style)))
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, vertical_chunks[i + 1]); // skip top padding
        }
    }

    fn show_number(&mut self) {
        self.state = GameState::Showing;
        self.show_start = Some(Instant::now());
        self.number = Some(self.generate_random_number());
        self.answer = Some(String::new());    
    }

    fn check_to_change_waiting(&mut self) {
        if let Some(start_show) = self.show_start {
            if start_show.elapsed() >= self.showing_duration {
                self.state = GameState::Waiting;
            }
        }
    }

    fn generate_random_number(&self) -> String {
        let mut rng = rand::rng();
        (0..self.level)
            .map(|_| rng.random_range(0..10).to_string())
            .collect()
    }

    fn init_game(&mut self) {
        self.answer = Some(String::new());
    }

    fn reset_common(&mut self) {
        self.state = GameState::Title;
        self.answer = None;
        self.number = None;
        self.show_start = None;
        self.level = 1;
    }

    fn quit_game(&mut self) {
        self.quit = true;
        self.reset_common();
    }

    fn reset_game(&mut self) {
        self.reset_common();
        self.answer = Some(String::new());
    }

}
